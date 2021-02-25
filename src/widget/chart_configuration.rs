#![allow(clippy::single_match)]
#![allow(irrefutable_let_patterns)]

use std::hash::{Hash, Hasher};

use crossterm::terminal;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Layout, Rect};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, StatefulWidget, Widget};

use super::chart::prices_kagi::{self, ReversalOption};
use super::{block, CachableWidget, CacheState};
use crate::common::{ChartType, TimeFrame};
use crate::draw::{add_padding, PaddingDirection};
use crate::theme::style;
use crate::THEME;

#[derive(Default, Debug, Clone)]
pub struct ChartConfigurationState {
    pub input: Input,
    pub selection: Option<Selection>,
    pub error_message: Option<String>,
    pub kagi_options: KagiOptions,
    pub cache_state: CacheState,
}

impl ChartConfigurationState {
    pub fn add_char(&mut self, c: char) {
        let input_field = match self.selection {
            Some(Selection::KagiReversalValue) => &mut self.input.kagi_reversal_value,
            _ => return,
        };

        // Width of our text input box
        if input_field.len() == 20 {
            return;
        }

        input_field.push(c);
    }

    pub fn del_char(&mut self) {
        let input_field = match self.selection {
            Some(Selection::KagiReversalValue) => &mut self.input.kagi_reversal_value,
            _ => return,
        };

        input_field.pop();
    }

    pub fn tab(&mut self) {
        let tab_field = match self.selection {
            Some(Selection::KagiReversalType) => &mut self.input.kagi_reversal_type,
            _ => return,
        };

        let mod_value = match self.selection {
            Some(Selection::KagiReversalType) => 2,
            _ => 1,
        };

        *tab_field = (*tab_field + 1) % mod_value;
    }

    pub fn enter(&mut self) {
        self.error_message.take();

        // Validate Kagi reversal option
        let new_kagi_reversal_option = {
            let input_value = &self.input.kagi_reversal_value;

            let value = match input_value.parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    self.error_message = Some("Reversal Value must be a valid number".to_string());
                    return;
                }
            };

            Some(match self.input.kagi_reversal_type {
                0 => ReversalOption::Pct(value),
                1 => ReversalOption::Amount(value),
                _ => unreachable!(),
            })
        };

        // Everything validated, save the form values to our state
        self.kagi_options.reversal_option = new_kagi_reversal_option;
    }

    pub fn selection_up(&mut self) {
        let new_selection = match self.selection {
            None => Selection::KagiReversalValue,
            Some(Selection::KagiReversalType) => Selection::KagiReversalValue,
            Some(Selection::KagiReversalValue) => Selection::KagiReversalType,
        };

        self.selection = Some(new_selection);
    }

    pub fn selection_down(&mut self) {
        let new_selection = match self.selection {
            None => Selection::KagiReversalType,
            Some(Selection::KagiReversalType) => Selection::KagiReversalValue,
            Some(Selection::KagiReversalValue) => Selection::KagiReversalType,
        };

        self.selection = Some(new_selection);
    }

    pub fn reset_with_defaults(&mut self, time_frame: TimeFrame) {
        self.input = Default::default();
        self.error_message.take();

        let default_reversal_amount = match time_frame {
            TimeFrame::Day1 => 0.01,
            _ => 0.04,
        };
        let (reversal_type, reversal_amount) = self
            .kagi_options
            .reversal_option
            .map(|o| match o {
                ReversalOption::Pct(amount) => (0, amount),
                ReversalOption::Amount(amount) => (1, amount),
            })
            .unwrap_or((0, default_reversal_amount));

        self.selection = Some(Selection::KagiReversalType);
        self.input.kagi_reversal_value = format!("{:.2}", reversal_amount);
        self.input.kagi_reversal_type = reversal_type;
    }
}

impl Hash for ChartConfigurationState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.input.hash(state);
        self.selection.hash(state);
        self.error_message.hash(state);
        self.kagi_options.hash(state);
    }
}

#[derive(Debug, Default, Clone, Hash)]
pub struct Input {
    pub kagi_reversal_type: usize,
    pub kagi_reversal_value: String,
}

#[derive(Default, Debug, Clone, Copy, Hash)]
pub struct KagiOptions {
    pub reversal_option: Option<prices_kagi::ReversalOption>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum Selection {
    KagiReversalType,
    KagiReversalValue,
}

pub struct ChartConfigurationWidget {
    pub chart_type: ChartType,
    pub time_frame: TimeFrame,
}

impl StatefulWidget for ChartConfigurationWidget {
    type State = ChartConfigurationState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_cached(area, buf, state);
    }
}

impl CachableWidget<ChartConfigurationState> for ChartConfigurationWidget {
    fn cache_state_mut(state: &mut ChartConfigurationState) -> &mut CacheState {
        &mut state.cache_state
    }

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut ChartConfigurationState) {
        block::new(" Configuration ").render(area, buf);
        area = add_padding(area, 1, PaddingDirection::All);
        area = add_padding(area, 1, PaddingDirection::Left);
        area = add_padding(area, 1, PaddingDirection::Right);

        // layout[0] - Info / Error message
        // layout[1] - Kagi options
        let mut layout = Layout::default()
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        layout[0] = add_padding(layout[0], 1, PaddingDirection::Top);
        layout[0] = add_padding(layout[0], 1, PaddingDirection::Bottom);

        let info_error = if let Some(msg) = state.error_message.as_ref() {
            vec![Spans::from(Span::styled(msg, style().fg(THEME.loss())))]
        } else {
            vec![
                Spans::from(Span::styled(
                    "<Up / Down>: move up / down",
                    style().fg(THEME.text_normal()),
                )),
                Spans::from(Span::styled(
                    "<Tab>: toggle option",
                    style().fg(THEME.text_normal()),
                )),
                Spans::from(Span::styled(
                    "<Enter>: submit changes",
                    style().fg(THEME.text_normal()),
                )),
            ]
        };

        Paragraph::new(info_error)
            .style(style().fg(THEME.text_normal()))
            .render(layout[0], buf);

        match self.chart_type {
            ChartType::Kagi => render_kagi_options(layout[1], buf, state),
            _ => {}
        }
    }
}

fn render_kagi_options(mut area: Rect, buf: &mut Buffer, state: &mut ChartConfigurationState) {
    Block::default()
        .style(style())
        .title(vec![Span::styled(
            "Kagi Options ",
            style().fg(THEME.text_normal()),
        )])
        .borders(Borders::TOP)
        .border_style(style().fg(THEME.border_secondary()))
        .render(area, buf);

    area = add_padding(area, 1, PaddingDirection::Top);

    // layout[0] - Left column
    // layout[1] - Divider
    // layout[2] - Right Column
    let layout = Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(16),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let left_column = vec![
        Spans::default(),
        Spans::from(vec![
            Span::styled(
                if state.selection == Some(Selection::KagiReversalType) {
                    "> "
                } else {
                    "  "
                },
                style().fg(THEME.text_primary()),
            ),
            Span::styled("Reversal Type", style().fg(THEME.text_normal())),
        ]),
        Spans::default(),
        Spans::from(vec![
            Span::styled(
                if state.selection == Some(Selection::KagiReversalValue) {
                    "> "
                } else {
                    "  "
                },
                style().fg(THEME.text_primary()),
            ),
            Span::styled("Reversal Value", style().fg(THEME.text_normal())),
        ]),
    ];

    let right_column = vec![
        Spans::default(),
        Spans::from(vec![
            Span::styled(
                "Pct",
                style().fg(THEME.text_normal()).bg(
                    match (state.selection, state.input.kagi_reversal_type) {
                        (Some(Selection::KagiReversalType), 0) => THEME.highlight_focused(),
                        (Some(Selection::KagiReversalType), _) => THEME.highlight_unfocused(),
                        (_, 0) => THEME.highlight_unfocused(),
                        (_, _) => THEME.background(),
                    },
                ),
            ),
            Span::styled(" | ", style().fg(THEME.text_normal())),
            Span::styled(
                "Amount",
                style().fg(THEME.text_normal()).bg(
                    match (state.selection, state.input.kagi_reversal_type) {
                        (Some(Selection::KagiReversalType), 1) => THEME.highlight_focused(),
                        (Some(Selection::KagiReversalType), _) => THEME.highlight_unfocused(),
                        (_, 1) => THEME.highlight_unfocused(),
                        (_, _) => THEME.background(),
                    },
                ),
            ),
        ]),
        Spans::default(),
        Spans::from(vec![Span::styled(
            format!("{: <22}", &state.input.kagi_reversal_value),
            style()
                .fg(if state.selection == Some(Selection::KagiReversalValue) {
                    THEME.text_secondary()
                } else {
                    THEME.text_normal()
                })
                .bg(if state.selection == Some(Selection::KagiReversalValue) {
                    THEME.highlight_unfocused()
                } else {
                    THEME.background()
                }),
        )]),
    ];

    Paragraph::new(left_column)
        .style(style().fg(THEME.text_normal()))
        .render(layout[0], buf);

    Paragraph::new(right_column)
        .style(style().fg(THEME.text_normal()))
        .render(layout[2], buf);

    // Set "cursor" color
    if matches!(state.selection, Some(Selection::KagiReversalValue)) {
        let size = terminal::size().unwrap_or((0, 0));

        let x = layout[2].left() as usize + state.input.kagi_reversal_value.len().min(20);
        let y = layout[2].top() as usize + 3;
        let idx = y * size.0 as usize + x;

        if let Some(cell) = buf.content.get_mut(idx) {
            cell.bg = THEME.text_secondary();
        }
    }
}
