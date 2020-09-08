use super::block;
use crate::draw::{add_padding, PaddingDirection};
use crate::service::{self, Service};

use api::model::{OptionsData, OptionsQuote};
use chrono::NaiveDateTime;
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{
    Block, Borders, List, ListState, Paragraph, Row, StatefulWidget, Table, TableState, Text,
    Widget,
};

#[derive(Clone, Copy, PartialEq)]
enum OptionType {
    Call,
    Put,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SelectionMode {
    Dates,
    Options,
}

pub struct OptionsState {
    options_service: service::options::OptionsService,
    exp_dates: Vec<i64>,
    exp_date: Option<i64>,
    pub data: Option<OptionsData>,
    selected_type: OptionType,
    pub selection_mode: SelectionMode,
    selected_option: Option<usize>,
    quote: Option<OptionsQuote>,
}

impl OptionsState {
    pub fn new(symbol: String) -> OptionsState {
        let options_service = service::options::OptionsService::new(symbol);

        OptionsState {
            options_service,
            exp_dates: vec![],
            exp_date: None,
            data: None,
            selected_type: OptionType::Call,
            selection_mode: SelectionMode::Dates,
            selected_option: None,
            quote: None,
        }
    }

    fn set_exp_date(&mut self, date: i64) {
        self.exp_date = Some(date);

        self.options_service.set_expiration_date(date);

        self.data.take();
        self.selected_option.take();
    }

    pub fn toggle_option_type(&mut self) {
        match self.selected_type {
            OptionType::Call => self.selected_type = OptionType::Put,
            OptionType::Put => self.selected_type = OptionType::Call,
        }

        self.set_selected_as_closest();
    }

    fn set_selected_as_closest(&mut self) {
        let selected_range = match self.selected_type {
            OptionType::Call => &self.data.as_ref().unwrap().calls[..],
            OptionType::Put => &self.data.as_ref().unwrap().puts[..],
        };

        let market_price = if let Some(ref quote) = self.quote {
            quote.regular_market_price
        } else {
            0.0
        };

        let mut closest_idx = selected_range
            .iter()
            .position(|c| c.strike < market_price)
            .unwrap_or_default();

        if closest_idx > 0 && self.selected_type == OptionType::Call {
            closest_idx -= 1;
        }

        self.selected_option = Some(closest_idx);
    }

    pub fn previous_date(&mut self) {
        if let Some(idx) = self
            .exp_dates
            .iter()
            .position(|d| *d == self.exp_date.unwrap_or_default())
        {
            let new_idx = if idx == 0 {
                self.exp_dates.len() - 1
            } else {
                idx - 1
            };

            self.set_exp_date(self.exp_dates[new_idx]);
        }
    }

    pub fn next_date(&mut self) {
        if let Some(idx) = self
            .exp_dates
            .iter()
            .position(|d| *d == self.exp_date.unwrap_or_default())
        {
            let new_idx = (idx + 1) % self.exp_dates.len();

            self.set_exp_date(self.exp_dates[new_idx]);
        }
    }

    pub fn previous_option(&mut self) {
        if let Some(idx) = self.selected_option {
            let option_range = if self.selected_type == OptionType::Call {
                &self.data.as_ref().unwrap().calls[..]
            } else {
                &self.data.as_ref().unwrap().puts[..]
            };

            let new_idx = if idx == 0 {
                option_range.len() - 1
            } else {
                idx - 1
            };

            self.selected_option = Some(new_idx);
        }
    }

    pub fn next_option(&mut self) {
        if let Some(idx) = self.selected_option {
            let option_range = if self.selected_type == OptionType::Call {
                &self.data.as_ref().unwrap().calls[..]
            } else {
                &self.data.as_ref().unwrap().puts[..]
            };

            let new_idx = (idx + 1) % option_range.len();

            self.selected_option = Some(new_idx);
        }
    }

    pub fn selection_mode_left(&mut self) {
        if self.selection_mode == SelectionMode::Options {
            self.selection_mode = SelectionMode::Dates;
        }
    }

    pub fn selection_mode_right(&mut self) {
        if self.selection_mode == SelectionMode::Dates {
            self.selection_mode = SelectionMode::Options;
        }
    }

    pub fn update(&mut self) {
        let updates = self.options_service.updates();

        for update in updates {
            match update {
                service::options::Update::ExpirationDates(dates) => {
                    let prev_len = self.exp_dates.len();

                    self.exp_dates = dates;

                    if prev_len == 0 && !self.exp_dates.is_empty() {
                        self.set_exp_date(self.exp_dates[0]);
                    }
                }
                service::options::Update::OptionsData(mut header) => {
                    if header.options.len() == 1 {
                        header.options[0].calls.reverse();
                        header.options[0].puts.reverse();

                        self.quote = Some(header.quote);
                        self.data = Some(header.options.remove(0));

                        if self.selected_option.is_none() {
                            self.set_selected_as_closest();
                        }
                    }
                }
            }
        }
    }
}

pub struct OptionsWidget {}

impl StatefulWidget for OptionsWidget {
    type State = OptionsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        block::new(" Options ", None).render(area, buf);

        // chunks[0] - call / put selector
        // chunks[1] - option info
        // chunks[2] - remainder (date selector | option selector)
        let mut chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(8),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(area);

        // Draw call / put selector
        {
            let call_put_selector = [
                Text::styled(
                    "Call",
                    Style::default().fg(Color::Green).modifier(
                        if state.selected_type == OptionType::Call {
                            Modifier::BOLD | Modifier::UNDERLINED
                        } else {
                            Modifier::empty()
                        },
                    ),
                ),
                Text::raw(" | "),
                Text::styled(
                    "Put",
                    Style::default().fg(Color::Red).modifier(
                        if state.selected_type == OptionType::Put {
                            Modifier::BOLD | Modifier::UNDERLINED
                        } else {
                            Modifier::empty()
                        },
                    ),
                ),
            ];

            chunks[0] = add_padding(chunks[0], 2, PaddingDirection::Left);
            chunks[0] = add_padding(chunks[0], 2, PaddingDirection::Right);
            chunks[0] = add_padding(chunks[0], 1, PaddingDirection::Top);

            Paragraph::new(call_put_selector.iter())
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(false)
                .render(chunks[0], buf);

            Block::default()
                .borders(Borders::BOTTOM)
                .render(chunks[0], buf);
        }

        // selector_chunks[0] - date selector
        // selector_chunks[1] - option selector
        let mut selector_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(13), Constraint::Min(0)].as_ref())
            .split(chunks[2]);

        // Draw date selector
        {
            selector_chunks[0] = add_padding(selector_chunks[0], 2, PaddingDirection::Left);
            selector_chunks[0] = add_padding(selector_chunks[0], 1, PaddingDirection::Bottom);

            Block::default()
                .borders(Borders::RIGHT)
                .render(selector_chunks[0], buf);

            let dates = state.exp_dates.iter().map(|d| {
                let date = NaiveDateTime::from_timestamp(*d, 0).date();
                Text::raw(date.format("%b-%d-%y").to_string())
            });

            let list = List::new(dates)
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .highlight_style(Style::default().bg(
                    if state.selection_mode == SelectionMode::Dates {
                        Color::LightBlue
                    } else {
                        Color::DarkGray
                    },
                ));

            let mut list_state = ListState::default();
            if let Some(idx) = state
                .exp_dates
                .iter()
                .position(|d| *d == state.exp_date.unwrap_or_default())
            {
                list_state.select(Some(idx));
            }

            Paragraph::new([Text::styled("Date", Style::default().fg(Color::Cyan))].iter())
                .render(selector_chunks[0], buf);

            selector_chunks[0] = add_padding(selector_chunks[0], 2, PaddingDirection::Top);

            <List<_> as StatefulWidget>::render(list, selector_chunks[0], buf, &mut list_state);
        }

        // Draw options data
        {
            selector_chunks[1] = add_padding(selector_chunks[1], 1, PaddingDirection::Left);
            selector_chunks[1] = add_padding(selector_chunks[1], 1, PaddingDirection::Bottom);

            if let Some(ref data) = state.data {
                let selected_data = if state.selected_type == OptionType::Call {
                    &data.calls[..]
                } else {
                    &data.puts[..]
                };

                let rows = selected_data.iter().map(|d| {
                    Row::StyledData(
                        vec![
                            format!("{: <7.2}", d.strike),
                            format!("{: <7.2}", d.last_price),
                            format!("{: >7.2}%", d.percent_change),
                        ]
                        .into_iter(),
                        Style::default().fg(if d.percent_change >= 0.0 {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                    )
                });

                let table = Table::new(["Strike", "Price", "% Change"].iter(), rows)
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .header_style(Style::default().fg(Color::Cyan).bg(Color::Black))
                    .highlight_style(Style::default().bg(
                        if state.selection_mode == SelectionMode::Options {
                            Color::LightBlue
                        } else {
                            Color::DarkGray
                        },
                    ))
                    .widths(&[
                        Constraint::Length(8),
                        Constraint::Length(8),
                        Constraint::Min(0),
                    ])
                    .column_spacing(2);

                let mut table_state = TableState::default();
                if let Some(idx) = state.selected_option {
                    table_state.select(Some(idx));
                }

                <Table<_, _> as StatefulWidget>::render(
                    table,
                    selector_chunks[1],
                    buf,
                    &mut table_state,
                );
            }
        }

        // Draw selected option info
        {
            chunks[1] = add_padding(chunks[1], 2, PaddingDirection::Left);
            chunks[1] = add_padding(chunks[1], 2, PaddingDirection::Right);

            Block::default()
                .borders(Borders::BOTTOM)
                .render(chunks[1], buf);

            if let Some(idx) = state.selected_option {
                let option_range = if state.selected_type == OptionType::Call {
                    &state.data.as_ref().unwrap().calls[..]
                } else {
                    &state.data.as_ref().unwrap().puts[..]
                };

                if let Some(option) = option_range.get(idx) {
                    let mut columns = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(20), Constraint::Length(20)].as_ref())
                        .split(chunks[1]);

                    columns[1] = add_padding(columns[1], 2, PaddingDirection::Left);

                    let currency = option.currency.as_deref().unwrap_or("USD");

                    let gap_strike = 19 - (format!("{:.2} {}", option.strike, currency).len() + 7);
                    let gap_last = 15 - (format!("{:.2}", option.last_price).len() + 6);
                    let gap_ask = 15 - (format!("{:.2}", option.ask.unwrap_or_default()).len() + 4);
                    let gap_bid = 15 - (format!("{:.2}", option.bid.unwrap_or_default()).len() + 4);
                    let gap_volume =
                        18 - (format!("{}", option.volume.unwrap_or_default()).len() + 7);
                    let gap_open_int =
                        18 - (format!("{}", option.open_interest.unwrap_or_default()).len() + 9);
                    let gap_impl_vol = 18
                        - (format!(
                            "{:.0}%",
                            option.implied_volatility.unwrap_or_default() * 100.0
                        )
                        .len()
                            + 11);

                    let column_0 = [
                        Text::raw(format!(
                            "strike:{}{:.2} {}\n\n",
                            " ".repeat(gap_strike),
                            option.strike,
                            currency
                        )),
                        Text::raw(format!(
                            "price:{}{:.2}\n\n",
                            " ".repeat(gap_last),
                            option.last_price,
                        )),
                        Text::raw(format!(
                            "bid:{}{:.2}\n\n",
                            " ".repeat(gap_ask),
                            option.bid.unwrap_or_default(),
                        )),
                        Text::raw(format!(
                            "ask:{}{:.2}",
                            " ".repeat(gap_bid),
                            option.ask.unwrap_or_default(),
                        )),
                    ];

                    let column_1 = [
                        Text::raw(format!(
                            "volume:{}{}\n\n",
                            " ".repeat(gap_volume),
                            option.volume.unwrap_or_default(),
                        )),
                        Text::raw(format!(
                            "interest:{}{}\n\n",
                            " ".repeat(gap_open_int),
                            option.open_interest.unwrap_or_default()
                        )),
                        Text::raw(format!(
                            "volatility:{}{:.0}%",
                            " ".repeat(gap_impl_vol),
                            option.implied_volatility.unwrap_or_default() * 100.0
                        )),
                    ];

                    Paragraph::new(column_0.iter()).render(columns[0], buf);
                    Paragraph::new(column_1.iter()).render(columns[1], buf);
                }
            }
        }
    }
}
