use itertools::Itertools;
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::{bar, Marker};
use tui::widgets::{
    Axis, BarChart, Block, Borders, Chart, Dataset, GraphType, Paragraph, StatefulWidget, Text,
    Widget,
};

use super::stock::StockState;
use crate::common::*;
use crate::draw::{add_padding, PaddingDirection};
use crate::{ENABLE_PRE_POST, HIDE_PREV_CLOSE, SHOW_VOLUMES};

pub struct StockSummaryWidget {}

impl StatefulWidget for StockSummaryWidget {
    type State = StockState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.use_cache {
            for (idx, cell) in buf.content.iter_mut().enumerate() {
                let x = idx as u16 % buf.area.width;
                let y = idx as u16 / buf.area.width;

                if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
                {
                    if let Some(cached_cell) = state.cached_content.get(idx) {
                        *cell = cached_cell.clone();
                    }
                }
            }

            state.use_cache = false;
            return;
        }

        let data = state.prices().collect::<Vec<_>>();
        let pct_change = state.pct_change(&data);

        let enable_pre_post = *ENABLE_PRE_POST.read().unwrap();
        let show_volumes = *SHOW_VOLUMES.read().unwrap();

        let loaded = state.loaded();

        let (company_name, currency) = match state.profile.as_ref() {
            Some(profile) => (
                profile.price.short_name.as_str(),
                profile.price.currency.as_deref().unwrap_or("USD"),
            ),
            None => ("", ""),
        };

        let loading_indicator = ".".repeat(state.loading_tick);

        let title = &format!(
            " {}{}",
            state.symbol,
            if state.profile.is_some() {
                format!(" - {}", company_name)
            } else {
                "".to_string()
            }
        );
        Block::default()
            .title(&format!(
                " {}{} ",
                &title[..24.min(title.len())],
                if loaded {
                    "".to_string()
                } else {
                    format!("{:<4}", loading_indicator)
                }
            ))
            .borders(Borders::TOP)
            .render(area, buf);

        let mut layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(0)].as_ref())
            .split(area);

        {
            layout[0] = add_padding(layout[0], 1, PaddingDirection::Top);
            layout[0] = add_padding(layout[0], 1, PaddingDirection::Left);
            layout[0] = add_padding(layout[0], 2, PaddingDirection::Right);

            let (high, low) = state.high_low(&data);
            let vol = state.reg_mkt_volume.clone().unwrap_or_default();

            let prices = [
                Text::styled("c: ", Style::default()),
                Text::styled(
                    if loaded {
                        format!("{:.2} {}\n", state.current_price(), currency)
                    } else {
                        "\n".to_string()
                    },
                    Style::default().modifier(Modifier::BOLD).fg(Color::Yellow),
                ),
                Text::styled("h: ", Style::default()),
                Text::styled(
                    if loaded {
                        format!("{:.2}\n", high)
                    } else {
                        "\n".to_string()
                    },
                    Style::default().fg(Color::LightCyan),
                ),
                Text::styled("l: ", Style::default()),
                Text::styled(
                    if loaded {
                        format!("{:.2}\n\n", low)
                    } else {
                        "\n\n".to_string()
                    },
                    Style::default().fg(Color::LightCyan),
                ),
                Text::styled("v: ", Style::default()),
                Text::styled(
                    if loaded { vol } else { "".to_string() },
                    Style::default().fg(Color::LightCyan),
                ),
            ];

            let pct = [Text::styled(
                if loaded {
                    format!("  {:.2}%", pct_change * 100.0)
                } else {
                    "".to_string()
                },
                Style::default()
                    .modifier(Modifier::BOLD)
                    .fg(if pct_change >= 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    }),
            )];

            Paragraph::new(prices.iter())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .render(layout[0], buf);

            Paragraph::new(pct.iter())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Right)
                .render(layout[0], buf);
        }

        // Draw graph
        {
            layout[1] = add_padding(layout[1], 1, PaddingDirection::Left);
            layout[1] = add_padding(layout[1], 1, PaddingDirection::Top);

            let (min, max) = state.min_max(&data);
            let (start, end) = state.start_end();

            let mut prices: Vec<_> = data.iter().map(cast_historical_as_price).collect();

            prices.pop();
            prices.push(state.current_price());
            zeros_as_pre(&mut prices);

            // Need more than one price for GraphType::Line to work
            let graph_type = if prices.len() <= 2 {
                GraphType::Scatter
            } else {
                GraphType::Line
            };

            let trading_period = state.current_trading_period(&data);

            let (reg_prices, pre_prices, post_prices) = if loaded {
                let (start_idx, end_idx) = state.regular_start_end_idx(&data);

                if enable_pre_post && state.time_frame == TimeFrame::Day1 {
                    (
                        prices
                            .iter()
                            .enumerate()
                            .filter(|(idx, _)| {
                                if let Some(start) = start_idx {
                                    *idx >= start
                                } else {
                                    false
                                }
                            })
                            .filter(|(idx, _)| {
                                if let Some(end) = end_idx {
                                    *idx <= end
                                } else {
                                    true
                                }
                            })
                            .map(cast_as_dataset)
                            .collect::<Vec<(f64, f64)>>(),
                        {
                            let pre_end_idx = if let Some(start_idx) = start_idx {
                                start_idx
                            } else {
                                prices.len()
                            };

                            if pre_end_idx > 0 {
                                Some(
                                    prices
                                        .iter()
                                        .enumerate()
                                        .filter(|(idx, _)| *idx <= pre_end_idx)
                                        .map(cast_as_dataset)
                                        .collect::<Vec<(f64, f64)>>(),
                                )
                            } else {
                                None
                            }
                        },
                        {
                            if let Some(post_start_idx) = end_idx {
                                Some(
                                    prices
                                        .iter()
                                        .enumerate()
                                        .filter(|(idx, _)| *idx >= post_start_idx)
                                        .map(cast_as_dataset)
                                        .collect::<Vec<(f64, f64)>>(),
                                )
                            } else {
                                None
                            }
                        },
                    )
                } else {
                    (
                        prices
                            .iter()
                            .enumerate()
                            .map(cast_as_dataset)
                            .collect::<Vec<(f64, f64)>>(),
                        None,
                        None,
                    )
                }
            } else {
                (vec![], None, None)
            };

            let prev_close_line = if state.time_frame == TimeFrame::Day1
                && loaded
                && !*HIDE_PREV_CLOSE
                && state.prev_close_price.is_some()
            {
                let num_points = (end - start) / 60 + 1;

                Some(
                    (0..num_points)
                        .map(|i| ((i + 1) as f64, state.prev_close_price.unwrap()))
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            };

            let mut datasets = vec![Dataset::default()
                .marker(Marker::Braille)
                .style(Style::default().fg(
                    if trading_period != TradingPeriod::Regular && enable_pre_post {
                        Color::DarkGray
                    } else if pct_change >= 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    },
                ))
                .graph_type(graph_type)
                .data(&reg_prices)];

            if let Some(data) = post_prices.as_ref() {
                datasets.push(
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(
                            Style::default().fg(if trading_period != TradingPeriod::Post {
                                Color::DarkGray
                            } else if pct_change >= 0.0 {
                                Color::Green
                            } else {
                                Color::Red
                            }),
                        )
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            if let Some(data) = pre_prices.as_ref() {
                datasets.insert(
                    0,
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(
                            Style::default().fg(if trading_period != TradingPeriod::Pre {
                                Color::DarkGray
                            } else if pct_change >= 0.0 {
                                Color::Green
                            } else {
                                Color::Red
                            }),
                        )
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            if let Some(data) = prev_close_line.as_ref() {
                datasets.insert(
                    0,
                    Dataset::default()
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::DarkGray))
                        .graph_type(GraphType::Line)
                        .data(&data),
                );
            }

            // graph_chunks[0] = prices
            // graph_chunks[1] = volume
            let graph_chunks = if show_volumes {
                Layout::default()
                    .constraints([Constraint::Length(5), Constraint::Length(1)].as_ref())
                    .split(layout[1])
            } else {
                vec![layout[1]]
            };

            if show_volumes {
                let mut volume_chunks = graph_chunks[1];
                volume_chunks.height += 1;

                let x_offset = if !loaded { 8 } else { 9 };
                volume_chunks.x += x_offset;

                if volume_chunks.width > x_offset + 1 {
                    volume_chunks.width -= x_offset + 1;

                    let width = volume_chunks.width;
                    let num_bars = width as usize;

                    let volumes = state.volumes(&data);
                    let vol_count = volumes.len();

                    if vol_count > 0 {
                        let volumes = data
                            .iter()
                            .map(|p| [p.volume].repeat(num_bars))
                            .flatten()
                            .chunks(vol_count)
                            .into_iter()
                            .map(|c| ("", c.sum::<u64>() / vol_count as u64))
                            .collect::<Vec<_>>();

                        volume_chunks.x -= 1;

                        Block::default()
                            .borders(Borders::LEFT)
                            .border_style(Style::default().fg(Color::Blue))
                            .render(volume_chunks, buf);

                        volume_chunks.x += 1;

                        BarChart::default()
                            .bar_gap(0)
                            .bar_set(bar::NINE_LEVELS)
                            .style(Style::default().fg(Color::DarkGray))
                            .data(&volumes)
                            .render(volume_chunks, buf);
                    }
                }
            }

            Chart::<String, String>::default()
                .block(Block::default().border_style(Style::default()))
                .x_axis(Axis::default().bounds(state.x_bounds(start, end, &data)))
                .y_axis(
                    Axis::default()
                        .bounds(state.y_bounds(min, max))
                        .labels(&state.y_labels(min, max))
                        .style(Style::default().fg(Color::LightBlue)),
                )
                .datasets(&datasets)
                .render(graph_chunks[0], buf);
        }

        // Cache current area, buf and reset use_cache flag
        state.cached_area = area;
        state.cached_content = buf.content.clone();
        state.use_cache = false;
    }
}
