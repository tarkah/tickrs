use super::{block, OptionsState};
use crate::common::*;
use crate::draw::{add_padding, PaddingDirection};
use crate::service::{self, Service};
use crate::{HIDE_PREV_CLOSE, HIDE_TOGGLE, SHOW_X_LABELS, TIME_FRAME};

use api::model::CompanyData;

use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::Marker;
use tui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, StatefulWidget, Tabs, Text, Widget,
};

pub struct StockState {
    pub symbol: String,
    pub stock_service: service::stock::StockService,
    pub profile: Option<CompanyData>,
    pub current_price: f32,
    pub prices: [Vec<Price>; 7],
    pub time_frame: TimeFrame,
    pub show_options: bool,
    pub options: Option<OptionsState>,
}

impl StockState {
    pub fn new(symbol: String) -> StockState {
        let time_frame = *TIME_FRAME;

        let stock_service = service::stock::StockService::new(symbol.clone(), time_frame);

        StockState {
            symbol,
            stock_service,
            profile: None,
            current_price: 0.0,
            prices: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
            time_frame,
            show_options: false,
            options: None,
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn time_frame_up(&mut self) {
        self.set_time_frame(self.time_frame.up());
    }

    pub fn time_frame_down(&mut self) {
        self.set_time_frame(self.time_frame.down());
    }

    pub fn set_time_frame(&mut self, time_frame: TimeFrame) {
        self.time_frame = time_frame;

        self.stock_service.update_time_frame(time_frame);
    }

    pub fn prices(&self) -> &[Price] {
        &self.prices[self.time_frame.idx()]
    }

    pub fn update(&mut self) {
        let updates = self.stock_service.updates();

        for update in updates {
            match update {
                service::stock::Update::NewPrice(price) => {
                    self.current_price = price;
                }
                service::stock::Update::Prices(prices) => {
                    self.prices[self.time_frame.idx()] = prices;
                }
                service::stock::Update::CompanyData(data) => {
                    self.profile = Some(data);
                }
            }
        }
    }

    pub fn toggle_options(&mut self) {
        self.show_options = !self.show_options;

        if self.options.is_some() {
            self.options.take();
        } else {
            self.options = Some(OptionsState::new(self.symbol.clone()));
        }
    }

    pub fn min_max(&self) -> (f32, f32) {
        let mut data: Vec<_> = self.prices().iter().map(cast_historical_as_price).collect();
        data.pop();
        data.push(self.current_price);
        data = remove_zeros(data);

        data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut min = data.first().unwrap_or(&0.0);
        let mut max = data.last().unwrap_or(&1.0);

        if self.current_price.le(&min) {
            min = &self.current_price;
        }

        if self.current_price.gt(&max) {
            max = &self.current_price;
        }

        (*min, *max)
    }

    pub fn high_low(&self) -> (f32, f32) {
        let mut data = self.prices().to_vec();

        data.sort_by(|a, b| a.high.partial_cmp(&b.high).unwrap());
        let mut max = data.last().map(|d| d.high).unwrap_or(0.0);

        data = remove_zeros_lows(data);
        data.sort_by(|a, b| a.low.partial_cmp(&b.low).unwrap());
        let mut min = data.first().map(|d| d.low).unwrap_or(0.0);

        if self.current_price.le(&min) {
            min = self.current_price;
        }

        if self.current_price.gt(&max) {
            max = self.current_price;
        }

        (max, min)
    }

    pub fn x_bounds(&self) -> [f64; 2] {
        match self.time_frame {
            TimeFrame::Day1 => [0.0, 390.0],
            _ => [0.0, (self.prices().len() + 1) as f64],
        }
    }

    pub fn x_labels(&self, width: u16) -> Vec<String> {
        let mut labels = vec![];

        let dates = if self.time_frame == TimeFrame::Day1 {
            MarketHours::default().collect()
        } else {
            self.prices().iter().map(|p| p.date).collect::<Vec<_>>()
        };

        if dates.is_empty() {
            return labels;
        }

        let label_len = dates
            .get(0)
            .map_or(0, |d| self.time_frame.format_time(*d).len())
            + 5;
        let num_labels = width as usize / label_len;
        let chunk_size = (dates.len() as f32 / (num_labels - 1) as f32).ceil() as usize;

        for (idx, chunk) in dates.chunks(chunk_size).enumerate() {
            if idx == 0 {
                labels.push(
                    chunk
                        .get(0)
                        .map_or("".to_string(), |d| self.time_frame.format_time(*d)),
                );
            }

            labels.push(
                chunk
                    .get(chunk.len() - 1)
                    .map_or("".to_string(), |d| self.time_frame.format_time(*d)),
            );
        }

        labels
    }

    pub fn y_bounds(&self, min: f32, max: f32) -> [f64; 2] {
        [(min - 0.05) as f64, (max + 0.05) as f64]
    }

    pub fn y_labels(&self, min: f32, max: f32) -> Vec<String> {
        vec![
            format!("{:>7.2}", (min - 0.05)),
            format!("{:>7.2}", ((min - 0.05) + (max + 0.05)) / 2.0),
            format!("{:>7.2}", max + 0.05),
        ]
    }

    pub fn pct_change(&self) -> f32 {
        if self.prices().is_empty() {
            return 0.0;
        }

        let baseline = if self.time_frame == TimeFrame::Day1 {
            if let Some(profile) = &self.profile {
                profile.price.regular_market_previous_close.price
            } else {
                self.prices().first().map(|d| d.close).unwrap()
            }
        } else {
            self.prices().first().map(|d| d.close).unwrap()
        };

        self.current_price / baseline - 1.0
    }
}

pub struct StockWidget {}

impl StatefulWidget for StockWidget {
    type State = StockState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let pct_change = state.pct_change();

        let show_x_labes = SHOW_X_LABELS.read().map_or(false, |l| *l);

        let (company_name, currency) = match state.profile.as_ref() {
            Some(profile) => (
                profile.price.short_name.as_str(),
                profile.price.currency.as_deref().unwrap_or("USD"),
            ),
            None => ("", ""),
        };

        // Draw widget block
        {
            block::new(&format!(" {} - {} ", state.symbol, company_name), None).render(area, buf);
        }

        // chunks[0] - Top Padding
        // chunks[1] - Company Info
        // chunks[2] - Graph - fill remaining space
        // chunks[3] - Time Frame Tabs
        // chunks[4] - Bottom Padding
        let mut chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Length(5),
                    Constraint::Min(0),
                    Constraint::Length(2),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(area);

        chunks[1] = add_padding(chunks[1], 2, PaddingDirection::Left);
        chunks[1] = add_padding(chunks[1], 2, PaddingDirection::Right);

        chunks[2] = add_padding(chunks[2], 2, PaddingDirection::Left);
        chunks[2] = add_padding(chunks[2], 2, PaddingDirection::Right);

        chunks[3] = add_padding(chunks[3], 2, PaddingDirection::Left);
        chunks[3] = add_padding(chunks[3], 2, PaddingDirection::Right);

        // Draw company info
        {
            let mut info_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(15)].as_ref())
                .split(chunks[1]);
            info_chunks[1].y -= 1;
            info_chunks[1].height += 1;

            let (high, low) = state.high_low();

            let company_info = [
                Text::styled("c: ", Style::default()),
                Text::styled(
                    format!("{:.2} {}", state.current_price, currency),
                    Style::default().modifier(Modifier::BOLD).fg(Color::Yellow),
                ),
                Text::styled(
                    format!("  {:.2}%\n\n", pct_change * 100.0),
                    Style::default()
                        .modifier(Modifier::BOLD)
                        .fg(if pct_change >= 0.0 {
                            Color::Green
                        } else {
                            Color::Red
                        }),
                ),
                Text::styled("h: ", Style::default()),
                Text::styled(
                    format!("{:.2}\n", high),
                    Style::default().fg(Color::LightCyan),
                ),
                Text::styled("l: ", Style::default()),
                Text::styled(format!("{:.2}", low), Style::default().fg(Color::LightCyan)),
            ];

            Paragraph::new(company_info.iter())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .wrap(true)
                .render(info_chunks[0], buf);

            if !*HIDE_TOGGLE {
                let toggle_block = block::new(" Toggle ", None);
                toggle_block.render(info_chunks[1], buf);
                info_chunks[1].x += 2;
                info_chunks[1].width -= 2;

                let toggle_info = [
                    Text::styled(
                        "\n\nOptions  'o'\n",
                        Style::default().bg(if state.show_options {
                            Color::DarkGray
                        } else {
                            Color::Reset
                        }),
                    ),
                    Text::styled("Summary  's'\n", Style::default()),
                    Text::styled(
                        "X Labels 'x'",
                        Style::default().bg(if show_x_labes {
                            Color::DarkGray
                        } else {
                            Color::Reset
                        }),
                    ),
                ];

                Paragraph::new(toggle_info.iter())
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left)
                    .wrap(false)
                    .render(info_chunks[1], buf);
            }
        }

        // Draw graph
        {
            let (min, max) = state.min_max();

            let mut prices: Vec<_> = state
                .prices()
                .iter()
                .map(cast_historical_as_price)
                .collect();

            prices.pop();
            prices.push(state.current_price);
            zeros_as_pre(&mut prices);

            // Need more than one price for GraphType::Line to work
            let graph_type = if prices.len() <= 2 {
                GraphType::Scatter
            } else {
                GraphType::Line
            };

            let x_labels = if show_x_labes {
                state.x_labels(chunks[2].width)
            } else {
                vec![]
            };

            let data_1 = prices
                .iter()
                .enumerate()
                .map(cast_as_dataset)
                .collect::<Vec<(f64, f64)>>();

            let data_2 = if state.time_frame == TimeFrame::Day1
                && state.profile.is_some()
                && !*HIDE_PREV_CLOSE
            {
                Some(
                    (0..391)
                        .map(|i| {
                            (
                                (i + 1) as f64,
                                state
                                    .profile
                                    .as_ref()
                                    .unwrap()
                                    .price
                                    .regular_market_previous_close
                                    .price as f64,
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            };

            let mut datasets = vec![Dataset::default()
                .marker(Marker::Braille)
                .style(Style::default().fg(if pct_change >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                }))
                .graph_type(graph_type)
                .data(&data_1)];

            if let Some(data) = data_2.as_ref() {
                datasets.insert(
                    0,
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::DarkGray))
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            Chart::<String, String>::default()
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .border_style(Style::default()),
                )
                .x_axis({
                    let axis = Axis::default().bounds(state.x_bounds());

                    if show_x_labes {
                        axis.labels(&x_labels)
                            .style(Style::default().fg(Color::LightBlue))
                    } else {
                        axis
                    }
                })
                .y_axis(
                    Axis::default()
                        .bounds(state.y_bounds(min, max))
                        .labels(&state.y_labels(min, max))
                        .style(Style::default().fg(Color::LightBlue)),
                )
                .datasets(&datasets)
                .render(chunks[2], buf);
        }

        // Draw time frame tabs
        {
            Tabs::default()
                .block(Block::default().borders(Borders::TOP))
                .titles(&TimeFrame::tab_names())
                .select(state.time_frame.idx())
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow))
                .render(chunks[3], buf);
        }
    }
}
