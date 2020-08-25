use super::stock::StockState;
use crate::common::*;
use crate::draw::{add_padding, PaddingDirection};

use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::Marker;
use tui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, StatefulWidget, Text, Widget,
};

pub struct StockSummaryWidget {}

impl StatefulWidget for StockSummaryWidget {
    type State = StockState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let pct_change = state.pct_change();

        let (company_name, currency) = match state.profile.as_ref() {
            Some(profile) => (
                profile.price.short_name.as_str(),
                profile.price.currency.as_deref().unwrap_or("USD"),
            ),
            None => ("", ""),
        };

        let title = format!("{} - {}", state.symbol, company_name);
        Block::default()
            .title(&format!(" {} ", &title[..24.min(title.len())]))
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::White))
            .render(area, buf);

        let mut layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(0)].as_ref())
            .split(area);

        {
            layout[0] = add_padding(layout[0], 2, PaddingDirection::Top);
            layout[0] = add_padding(layout[0], 1, PaddingDirection::Left);
            layout[0] = add_padding(layout[0], 2, PaddingDirection::Right);

            let (high, low) = state.high_low();

            let prices = [
                Text::raw("c: "),
                Text::styled(
                    format!("{:.2} {}\n", state.current_price, currency),
                    Style::default().modifier(Modifier::BOLD).fg(Color::Yellow),
                ),
                Text::raw("h: "),
                Text::styled(
                    format!("{:.2}\n", high),
                    Style::default().fg(Color::LightCyan),
                ),
                Text::raw("l: "),
                Text::styled(format!("{:.2}", low), Style::default().fg(Color::LightCyan)),
            ];

            let pct = [Text::styled(
                format!("  {:.2}%", pct_change * 100.0),
                Style::default()
                    .modifier(Modifier::BOLD)
                    .fg(if pct_change >= 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    }),
            )];

            Paragraph::new(prices.iter())
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Left)
                .render(layout[0], buf);

            Paragraph::new(pct.iter())
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Right)
                .render(layout[0], buf);
        }

        // Draw graph
        {
            layout[1] = add_padding(layout[1], 1, PaddingDirection::Left);
            layout[1] = add_padding(layout[1], 1, PaddingDirection::Top);

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
                .block(Block::default().border_style(Style::default()))
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
                .render(layout[1], buf);
        }
    }
}
