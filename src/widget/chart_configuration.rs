use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use crossterm::terminal;
use serde::Deserialize;
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

    fn get_tab_artifacts(&mut self) -> Option<(&mut usize, usize)> {
        let tab_field = match self.selection {
            Some(Selection::KagiReversalType) => &mut self.input.kagi_reversal_type,
            Some(Selection::KagiPriceType) => &mut self.input.kagi_price_type,
            _ => return None,
        };

        let mod_value = match self.selection {
            Some(Selection::KagiReversalType) => 2,
            Some(Selection::KagiPriceType) => 2,
            _ => 1,
        };
        Some((tab_field, mod_value))
    }

    pub fn tab(&mut self) {
        if let Some((tab_field, mod_value)) = self.get_tab_artifacts() {
            *tab_field = (*tab_field + 1) % mod_value;
        }
    }

    pub fn back_tab(&mut self) {
        if let Some((tab_field, mod_value)) = self.get_tab_artifacts() {
            *tab_field = (*tab_field + mod_value - 1) % mod_value;
        }
    }

    pub fn enter(&mut self, time_frame: TimeFrame) {
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

            match self.input.kagi_reversal_type {
                0 => ReversalOption::Pct(value),
                1 => ReversalOption::Amount(value),
                _ => unreachable!(),
            }
        };

        let new_kagi_price_option = Some(match self.input.kagi_price_type {
            0 => prices_kagi::PriceOption::Close,
            1 => prices_kagi::PriceOption::HighLow,
            _ => unreachable!(),
        });

        // Everything validated, save the form values to our state
        match &mut self.kagi_options.reversal_option {
            reversal_options @ None => {
                let mut options_by_timeframe = BTreeMap::new();
                for iter_time_frame in TimeFrame::ALL.iter() {
                    let default_reversal_amount = match iter_time_frame {
                        TimeFrame::Day1 => 0.01,
                        _ => 0.04,
                    };

                    // If this is the time frame we are submitting for, store that value,
                    // otherwise use the default still
                    if *iter_time_frame == time_frame {
                        options_by_timeframe.insert(*iter_time_frame, new_kagi_reversal_option);
                    } else {
                        options_by_timeframe.insert(
                            *iter_time_frame,
                            ReversalOption::Pct(default_reversal_amount),
                        );
                    }
                }

                *reversal_options = Some(KagiReversalOption::ByTimeFrame(options_by_timeframe));
            }
            reversal_options @ Some(KagiReversalOption::Single(_)) => {
                // Always succeeds since we already pattern matched it
                if let KagiReversalOption::Single(config_option) = reversal_options.clone().unwrap()
                {
                    let mut options_by_timeframe = BTreeMap::new();
                    for iter_time_frame in TimeFrame::ALL.iter() {
                        // If this is the time frame we are submitting for, store that value,
                        // otherwise use the single value defined from the config
                        if *iter_time_frame == time_frame {
                            options_by_timeframe.insert(*iter_time_frame, new_kagi_reversal_option);
                        } else {
                            options_by_timeframe.insert(*iter_time_frame, config_option);
                        }
                    }

                    *reversal_options = Some(KagiReversalOption::ByTimeFrame(options_by_timeframe));
                }
            }
            Some(KagiReversalOption::ByTimeFrame(options_by_timeframe)) => {
                options_by_timeframe.insert(time_frame, new_kagi_reversal_option);
            }
        }

        self.kagi_options.price_option = new_kagi_price_option;
    }

    pub fn selection_up(&mut self) {
        let new_selection = match self.selection {
            None => Selection::KagiReversalValue,
            Some(Selection::KagiReversalValue) => Selection::KagiReversalType,
            Some(Selection::KagiReversalType) => Selection::KagiPriceType,
            Some(Selection::KagiPriceType) => Selection::KagiReversalValue,
        };

        self.selection = Some(new_selection);
    }

    pub fn selection_down(&mut self) {
        let new_selection = match self.selection {
            None => Selection::KagiPriceType,
            Some(Selection::KagiPriceType) => Selection::KagiReversalType,
            Some(Selection::KagiReversalType) => Selection::KagiReversalValue,
            Some(Selection::KagiReversalValue) => Selection::KagiPriceType,
        };

        self.selection = Some(new_selection);
    }

    pub fn reset_form(&mut self, time_frame: TimeFrame) {
        self.input = Default::default();
        self.error_message.take();

        let default_reversal_amount = match time_frame {
            TimeFrame::Day1 => 0.01,
            _ => 0.04,
        };

        let (reversal_type, reversal_amount) = self
            .kagi_options
            .reversal_option
            .as_ref()
            .map(|o| {
                let option = match o {
                    KagiReversalOption::Single(option) => *option,
                    KagiReversalOption::ByTimeFrame(options_by_timeframe) => options_by_timeframe
                        .get(&time_frame)
                        .copied()
                        .unwrap_or(ReversalOption::Pct(default_reversal_amount)),
                };

                match option {
                    ReversalOption::Pct(amount) => (0, amount),
                    ReversalOption::Amount(amount) => (1, amount),
                }
            })
            .unwrap_or((0, default_reversal_amount));

        let price_type = self
            .kagi_options
            .price_option
            .map(|p| match p {
                prices_kagi::PriceOption::Close => 0,
                prices_kagi::PriceOption::HighLow => 1,
            })
            .unwrap_or(0);

        self.selection = Some(Selection::KagiPriceType);
        self.input.kagi_reversal_value = format!("{:.2}", reversal_amount);
        self.input.kagi_reversal_type = reversal_type;
        self.input.kagi_price_type = price_type;
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
    pub kagi_price_type: usize,
}

#[derive(Default, Debug, Clone, Deserialize, Hash)]
pub struct KagiOptions {
    #[serde(rename = "reversal")]
    pub reversal_option: Option<KagiReversalOption>,
    #[serde(rename = "price")]
    pub price_option: Option<prices_kagi::PriceOption>,
}

#[derive(Debug, Clone, Deserialize, Hash)]
#[serde(untagged)]
pub enum KagiReversalOption {
    Single(prices_kagi::ReversalOption),
    ByTimeFrame(BTreeMap<TimeFrame, prices_kagi::ReversalOption>),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum Selection {
    KagiPriceType,
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
                    "  <Up / Down>: move up / down",
                    style().fg(THEME.text_normal()),
                )),
                Spans::from(Span::styled(
                    "  <Tab>: toggle option",
                    style().fg(THEME.text_normal()),
                )),
                Spans::from(Span::styled(
                    "  <Enter>: submit changes",
                    style().fg(THEME.text_normal()),
                )),
            ]
        };

        Paragraph::new(info_error)
            .style(style().fg(THEME.text_normal()))
            .render(layout[0], buf);

        match self.chart_type {
            ChartType::Line => {}
            ChartType::Candlestick => {}
            ChartType::Kagi => render_kagi_options(layout[1], buf, state),
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
                if state.selection == Some(Selection::KagiPriceType) {
                    "> "
                } else {
                    "  "
                },
                style().fg(THEME.text_primary()),
            ),
            Span::styled("Price Type", style().fg(THEME.text_normal())),
        ]),
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
                "Close",
                style().fg(THEME.text_normal()).bg(
                    match (state.selection, state.input.kagi_price_type) {
                        (Some(Selection::KagiPriceType), 0) => THEME.highlight_focused(),
                        (_, 0) => THEME.highlight_unfocused(),
                        (_, _) => THEME.background(),
                    },
                ),
            ),
            Span::styled(" | ", style().fg(THEME.text_normal())),
            Span::styled(
                "High / Low",
                style().fg(THEME.text_normal()).bg(
                    match (state.selection, state.input.kagi_price_type) {
                        (Some(Selection::KagiPriceType), 1) => THEME.highlight_focused(),
                        (_, 1) => THEME.highlight_unfocused(),
                        (_, _) => THEME.background(),
                    },
                ),
            ),
        ]),
        Spans::default(),
        Spans::from(vec![
            Span::styled(
                "Pct",
                style().fg(THEME.text_normal()).bg(
                    match (state.selection, state.input.kagi_reversal_type) {
                        (Some(Selection::KagiReversalType), 0) => THEME.highlight_focused(),
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
        let y = layout[2].top() as usize + 5;
        let idx = y * size.0 as usize + x;

        if let Some(cell) = buf.content.get_mut(idx) {
            cell.bg = THEME.text_secondary();
        }
    }
}
