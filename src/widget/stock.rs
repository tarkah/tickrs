use super::block;
use crate::service::{self, Service};
use crate::TimeFrame;

use api::model::{CompanyProfile, HistoricalDay};

use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, Marker, Paragraph, Tabs, Text, Widget,
};

pub struct StockWidget {
    symbol: String,
    stock_service: service::stock::StockService,
    profile: Option<CompanyProfile>,
    current_price: f32,
    prices: Vec<HistoricalDay>,
    time_frame: TimeFrame,
}

impl StockWidget {
    pub fn new(symbol: String) -> StockWidget {
        let time_frame = TimeFrame::Day1;

        let stock_service = service::stock::StockService::new(symbol.clone(), time_frame);

        StockWidget {
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
                service::stock::Update::CompanyProfile(profile) => {
                    self.profile = profile;
                }
            }
        }
    }

    fn min_max(&self) -> (f32, f32) {
        let mut data: Vec<_> = self.prices.iter().map(cast_historical_as_price).collect();
        data.push(self.current_price);

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
            TimeFrame::Day1 => [0.0, 1000.0], // Need to update once intra ready
            _ => [0.0, (self.prices.len() + 1) as f64],
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

        let open = self.prices.first().map(|d| d.open).unwrap();
        self.current_price / open - 1.0
    }
}

impl Widget for StockWidget {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let pct_change = self.pct_change();

        // Draw widget block
        {
            let company_name = match self.profile.as_ref() {
                Some(profile) => &profile.company_name,
                None => "",
            };

            block::new(&format!(" {} - {} ", self.symbol, company_name)).draw(area, buf);
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

        chunks[1] = add_left_padding(chunks[1], 2);
        chunks[2] = add_left_padding(chunks[2], 2);
        chunks[3] = add_left_padding(chunks[3], 2);

        // Draw company info
        {
            let (high, low) = self.high_low();

            let company_info = [
                Text::raw("c: "),
                Text::styled(
                    format!("${:.2}", self.current_price),
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
                .draw(chunks[1], buf);
        }

        // Draw graph
        {
            let (min, max) = self.min_max();

            let mut prices: Vec<_> = self.prices.iter().map(cast_historical_as_price).collect();
            prices.push(self.current_price);

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
                .x_axis(Axis::default().bounds(self.x_bounds()))
                .y_axis(
                    Axis::default()
                        .bounds(self.y_bounds(min, max))
                        .labels(&self.y_labels(min, max))
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
                .draw(chunks[2], buf);
        }

        // Draw time frame tabs
        {
            Tabs::default()
                .block(Block::default().borders(Borders::TOP))
                .titles(&TimeFrame::tab_names())
                .select(self.time_frame.idx())
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow))
                .draw(chunks[3], buf);
        }
    }
}

fn cast_as_dataset(input: (usize, &f32)) -> (f64, f64) {
    ((input.0 + 1) as f64, *input.1 as f64)
}

fn cast_historical_as_price(input: &HistoricalDay) -> f32 {
    input.close
}

fn add_left_padding(mut rect: Rect, n: u16) -> Rect {
    rect.x += n;
    rect.width -= n * 2;
    rect
}
