use super::block;
use crate::common::{Price, TimeFrame};
use crate::draw::{add_padding, PaddingDirection};
use crate::service::{self, Service};

use api::model::CompanyData;

use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::Marker;
use tui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, StatefulWidget, Tabs, Text, Widget,
};

const X_SCALE: usize = 1;

pub struct StockState {
    symbol: String,
    stock_service: service::stock::StockService,
    profile: Option<CompanyData>,
    current_price: f32,
    prices: Vec<Price>,
    time_frame: TimeFrame,
}

impl StockState {
    pub fn new(symbol: String) -> StockState {
        let time_frame = TimeFrame::Day1;

        let stock_service = service::stock::StockService::new(symbol.clone(), time_frame);

        StockState {
            symbol,
            stock_service,
            profile: None,
            current_price: 0.0,
            prices: vec![],
            time_frame,
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

    fn set_time_frame(&mut self, time_frame: TimeFrame) {
        self.time_frame = time_frame;
        self.prices.drain(..);

        self.stock_service.update_time_frame(time_frame);
    }

    pub fn update(&mut self) {
        let updates = self.stock_service.updates();

        for update in updates {
            match update {
                service::stock::Update::NewPrice(price) => {
                    self.current_price = price;
                }
                service::stock::Update::Prices(prices) => {
                    self.prices = prices;
                }
                service::stock::Update::CompanyData(data) => {
                    self.profile = data;
                }
            }
        }
    }

    fn min_max(&self) -> (f32, f32) {
        let mut data: Vec<_> = self.prices.iter().map(cast_historical_as_price).collect();
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

    fn high_low(&self) -> (f32, f32) {
        let mut data = self.prices.clone();

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

    fn x_bounds(&self) -> [f64; 2] {
        match self.time_frame {
            TimeFrame::Day1 => [0.0, (390 * X_SCALE) as f64],
            _ => [0.0, ((self.prices.len() * X_SCALE) + 1) as f64],
        }
    }

    fn y_bounds(&self, min: f32, max: f32) -> [f64; 2] {
        [(min - 0.05) as f64, (max + 0.05) as f64]
    }

    fn y_labels(&self, min: f32, max: f32) -> Vec<String> {
        vec![
            format!("{:.2}", (min - 0.05)),
            format!("{:.2}", ((min - 0.05) + (max + 0.05)) / 2.0),
            format!("{:.2}", max + 0.05),
        ]
    }

    fn pct_change(&self) -> f32 {
        if self.prices.is_empty() {
            return 0.0;
        }

        let baseline = if self.time_frame == TimeFrame::Day1 {
            if let Some(profile) = &self.profile {
                profile.price.regular_market_previous_close.price
            } else {
                self.prices.first().map(|d| d.close).unwrap()
            }
        } else {
            self.prices.first().map(|d| d.close).unwrap()
        };

        self.current_price / baseline - 1.0
    }
}

pub struct StockWidget {}

impl StatefulWidget for StockWidget {
    type State = StockState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let pct_change = state.pct_change();

        // Draw widget block
        {
            let company_name = match state.profile.as_ref() {
                Some(profile) => &profile.price.short_name,
                None => "",
            };

            block::new(&format!(" {} - {} ", state.symbol, company_name)).render(area, buf);
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
            let (high, low) = state.high_low();

            let company_info = [
                Text::raw("c: "),
                Text::styled(
                    format!("${:.2}", state.current_price),
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
                Text::raw("h: "),
                Text::styled(
                    format!("${:.2}\n", high),
                    Style::default().fg(Color::LightCyan),
                ),
                Text::raw("l: "),
                Text::styled(
                    format!("${:.2}", low),
                    Style::default().fg(Color::LightCyan),
                ),
            ];

            Paragraph::new(company_info.iter())
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Left)
                .wrap(true)
                .render(chunks[1], buf);
        }

        // Draw graph
        {
            let (min, max) = state.min_max();

            let mut prices: Vec<_> = state.prices.iter().map(cast_historical_as_price).collect();
            prices.pop();
            prices.push(state.current_price);
            zeros_as_pre(&mut prices);

            // Need more than one price for GraphType::Line to work
            let graph_type = if prices.len() <= 2 {
                GraphType::Scatter
            } else {
                GraphType::Line
            };

            Chart::<String, String>::default()
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .border_style(Style::default()),
                )
                .x_axis(Axis::default().bounds(state.x_bounds()))
                .y_axis(
                    Axis::default()
                        .bounds(state.y_bounds(min, max))
                        .labels(&state.y_labels(min, max))
                        .style(Style::default().fg(Color::LightBlue)),
                )
                .datasets(&[Dataset::default()
                    .marker(Marker::Braille)
                    .style(Style::default().fg(if pct_change >= 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    }))
                    .graph_type(graph_type)
                    .data(
                        &prices
                            .iter()
                            .enumerate()
                            .map(cast_as_dataset)
                            .collect::<Vec<(f64, f64)>>(),
                    )])
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

fn cast_as_dataset(input: (usize, &f32)) -> (f64, f64) {
    (((input.0 * X_SCALE) + 1) as f64, *input.1 as f64)
}

fn cast_historical_as_price(input: &Price) -> f32 {
    input.close
}

fn zeros_as_pre(prices: &mut [f32]) {
    if prices.len() <= 1 {
        return;
    }

    let zero_indexes = prices
        .iter()
        .enumerate()
        .filter_map(|(idx, price)| if *price == 0.0 { Some(idx) } else { None })
        .collect::<Vec<usize>>();

    for idx in zero_indexes {
        if idx == 0 {
            prices[0] = prices[1];
        } else {
            prices[idx] = prices[idx - 1];
        }
    }
}

fn remove_zeros(prices: Vec<f32>) -> Vec<f32> {
    prices.into_iter().filter(|x| x.ne(&0.0)).collect()
}

fn remove_zeros_lows(prices: Vec<Price>) -> Vec<Price> {
    prices.into_iter().filter(|x| x.low.ne(&0.0)).collect()
}
