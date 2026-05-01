use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, StatefulWidget, Widget};

use crate::common::{
    cast_as_dataset, cast_historical_as_price, zeros_as_pre, Price, TimeFrame, TradingPeriod,
};
use crate::draw::{add_padding, PaddingDirection};
use crate::theme::style;
use crate::widget::StockState;
use crate::{HIDE_PREV_CLOSE, THEME};

pub struct PricesLineChart<'a> {
    pub loaded: bool,
    pub enable_pre_post: bool,
    pub show_x_labels: bool,
    pub is_profit: bool,
    pub is_summary: bool,
    pub data: &'a [Price],
}

impl StatefulWidget for PricesLineChart<'_> {
    type State = StockState;

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width <= 9 || area.height <= 3 {
            return;
        }

        if !self.is_summary {
            Block::default()
                .borders(Borders::TOP)
                .border_style(style().fg(THEME.border_secondary()))
                .render(area, buf);
            area = add_padding(area, 1, PaddingDirection::Top);
        }

        let (min, max) = state.min_max(self.data);
        let (start, end) = state.start_end();

        let mut prices: Vec<_> = self.data.iter().map(cast_historical_as_price).collect();

        prices.pop();
        prices.push(state.current_price());
        zeros_as_pre(&mut prices);

        // Need more than one price for GraphType::Line to work
        let graph_type = if prices.len() <= 2 {
            GraphType::Scatter
        } else {
            GraphType::Line
        };

        let trading_period = state.current_trading_period(self.data);

        let (reg_prices, pre_prices, post_prices) = if self.loaded {
            let (start_idx, end_idx) = state.regular_start_end_idx(self.data);

            if self.enable_pre_post && state.time_frame == TimeFrame::Day1 {
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
                        end_idx.map(|post_start_idx| {
                            prices
                                .iter()
                                .enumerate()
                                .filter(|(idx, _)| *idx >= post_start_idx)
                                .map(cast_as_dataset)
                                .collect::<Vec<(f64, f64)>>()
                        })
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
            && self.loaded
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
            .style(style().fg(
                if trading_period != TradingPeriod::Regular && self.enable_pre_post {
                    THEME.gray()
                } else if self.is_profit {
                    THEME.profit()
                } else {
                    THEME.loss()
                },
            ))
            .graph_type(graph_type)
            .data(&reg_prices)];

        if let Some(data) = post_prices.as_ref() {
            datasets.push(
                Dataset::default()
                    .marker(Marker::Braille)
                    .style(style().fg(if trading_period != TradingPeriod::Post {
                        THEME.gray()
                    } else if self.is_profit {
                        THEME.profit()
                    } else {
                        THEME.loss()
                    }))
                    .graph_type(GraphType::Line)
                    .data(data),
            );
        }

        if let Some(data) = pre_prices.as_ref() {
            datasets.insert(
                0,
                Dataset::default()
                    .marker(Marker::Braille)
                    .style(style().fg(if trading_period != TradingPeriod::Pre {
                        THEME.gray()
                    } else if self.is_profit {
                        THEME.profit()
                    } else {
                        THEME.loss()
                    }))
                    .graph_type(GraphType::Line)
                    .data(data),
            );
        }

        if let Some(data) = prev_close_line.as_ref() {
            datasets.insert(
                0,
                Dataset::default()
                    .marker(Marker::Braille)
                    .style(style().fg(THEME.gray()))
                    .graph_type(GraphType::Line)
                    .data(data),
            );
        }

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .style(style())
                    .borders(if self.show_x_labels {
                        Borders::LEFT | Borders::BOTTOM
                    } else {
                        Borders::LEFT
                    })
                    .border_style(style().fg(THEME.border_axis())),
            )
            .style(style())
            .x_axis(Axis::default().bounds(state.x_bounds(start, end, self.data)))
            .y_axis(Axis::default().bounds(state.y_bounds(min, max)));

        // x_layout[0] - chart + y labels
        // x_layout[1] - (x labels)
        let x_layout: Vec<Rect> = Layout::default()
            .constraints(if self.show_x_labels {
                &[Constraint::Min(0), Constraint::Length(1)][..]
            } else {
                &[Constraint::Min(0)][..]
            })
            .split(area)
            .to_vec();

        // layout[0] - Y lables
        // layout[1] - chart
        let mut layout: Vec<Rect> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(if !self.loaded {
                    8
                } else if self.show_x_labels {
                    match state.time_frame {
                        TimeFrame::Day1 => 9,
                        TimeFrame::Week1 => 12,
                        _ => 11,
                    }
                } else {
                    9
                }),
                Constraint::Min(0),
            ])
            .split(x_layout[0])
            .to_vec();

        // Fix for border render
        layout[1].x = layout[1].x.saturating_sub(1);
        layout[1].width += 1;

        // Draw x labels
        if self.show_x_labels && self.loaded {
            // Fix for y label render
            layout[0] = add_padding(layout[0], 1, PaddingDirection::Bottom);

            let mut x_area = x_layout[1];
            x_area.x = layout[1].x + 1;
            x_area.width = layout[1].width - 1;

            let labels = state.x_labels(area.width, self.data);
            let total_width = labels.iter().map(Span::width).sum::<usize>() as u16;
            let labels_len = labels.len() as u16;
            if total_width < x_area.width && labels_len > 1 {
                for (i, label) in labels.iter().enumerate() {
                    buf.set_span(
                        x_area.left() + i as u16 * (x_area.width - 1) / (labels_len - 1)
                            - label.width() as u16,
                        x_area.top(),
                        label,
                        label.width() as u16,
                    );
                }
            }
        }

        // Draw y labels
        if self.loaded {
            let y_area = layout[0];

            let labels = state.y_labels(min, max);
            let labels_len = labels.len() as u16;
            for (i, label) in labels.iter().enumerate() {
                let dy = i as u16 * (y_area.height - 1) / (labels_len - 1);
                if dy < y_area.bottom() {
                    buf.set_span(
                        y_area.left(),
                        y_area.bottom() - 1 - dy,
                        label,
                        label.width() as u16,
                    );
                }
            }
        }

        chart.render(layout[1], buf);
    }
}
