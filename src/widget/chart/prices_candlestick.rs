use itertools::Itertools;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::text::Span;
use tui::widgets::canvas::{Canvas, Line, Rectangle};
use tui::widgets::{Block, Borders, StatefulWidget, Widget};

use crate::common::{Price, TimeFrame};
use crate::draw::{add_padding, PaddingDirection};
use crate::theme::style;
use crate::widget::StockState;
use crate::{HIDE_PREV_CLOSE, THEME};

#[derive(Debug)]
struct Candle {
    open: f64,
    close: f64,
    high: f64,
    low: f64,
}

pub struct PricesCandlestickChart<'a> {
    pub loaded: bool,
    pub data: &'a [Price],
    pub is_summary: bool,
    pub show_x_labels: bool,
}

impl<'a> StatefulWidget for PricesCandlestickChart<'a> {
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

        let mut data = self.data.to_vec();
        data.push(Price {
            close: state.current_price(),
            open: state.current_price(),
            high: state.current_price(),
            low: state.current_price(),
            ..Default::default()
        });

        let (min, max) = state.min_max(&data);
        let (start, end) = state.start_end();
        let x_bounds = state.x_bounds(start, end, &data);

        // x_layout[0] - chart + y labels
        // x_layout[1] - (x labels)
        let x_layout = Layout::default()
            .constraints(if self.show_x_labels {
                &[Constraint::Min(0), Constraint::Length(1)][..]
            } else {
                &[Constraint::Min(0)][..]
            })
            .split(area);

        // layout[0] - Y lables
        // layout[1] - chart
        let mut layout = Layout::default()
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
            .split(x_layout[0]);

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

            let labels = state.x_labels(area.width, start, end, self.data);
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

        let width = layout[1].width - 1;
        let num_candles = width / 2;

        let candles = data
            .iter()
            .flat_map(|p| vec![*p; num_candles as usize])
            .chunks(x_bounds[1] as usize)
            .into_iter()
            .map(|c| {
                let prices = c.filter(|p| p.close.gt(&0.0)).collect::<Vec<_>>();

                if prices.is_empty() {
                    return None;
                }

                let open = prices.get(0).unwrap().open;
                let close = prices.iter().last().unwrap().close;
                let high = prices
                    .iter()
                    .max_by(|a, b| a.high.partial_cmp(&b.high).unwrap())
                    .unwrap()
                    .high;
                let low = prices
                    .iter()
                    .min_by(|a, b| a.low.partial_cmp(&b.low).unwrap())
                    .unwrap()
                    .low;

                Some(Candle {
                    open,
                    close,
                    high,
                    low,
                })
            })
            .collect::<Vec<_>>();

        if self.loaded {
            Canvas::default()
                .background_color(THEME.background())
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
                .x_bounds([0.0, num_candles as f64 * 4.0])
                .y_bounds(state.y_bounds(min, max))
                .paint(move |ctx| {
                    if state.time_frame == TimeFrame::Day1
                        && self.loaded
                        && !*HIDE_PREV_CLOSE
                        && state.prev_close_price.is_some()
                    {
                        ctx.draw(&Line {
                            x1: 0.0,
                            x2: num_candles as f64 * 4.0,
                            y1: state.prev_close_price.unwrap(),
                            y2: state.prev_close_price.unwrap(),
                            color: THEME.gray(),
                        })
                    }

                    ctx.layer();

                    for (idx, candle) in candles.iter().enumerate() {
                        if let Some(candle) = candle {
                            let color = if candle.close.gt(&candle.open) {
                                THEME.profit()
                            } else {
                                THEME.loss()
                            };

                            ctx.draw(&Rectangle {
                                x: idx as f64 * 4.0 + 1.0,
                                y: candle.open.min(candle.close),
                                width: 2.0,
                                height: candle.open.max(candle.close)
                                    - candle.open.min(candle.close),
                                color,
                            });

                            ctx.draw(&Line {
                                x1: idx as f64 * 4.0 + 2.0,
                                x2: idx as f64 * 4.0 + 2.0,
                                y1: candle.low,
                                y2: candle.open.min(candle.close),
                                color,
                            });

                            ctx.draw(&Line {
                                x1: idx as f64 * 4.0 + 2.0,
                                x2: idx as f64 * 4.0 + 2.0,
                                y1: candle.high,
                                y2: candle.open.max(candle.close),
                                color,
                            });
                        }
                    }
                })
                .render(layout[1], buf);
        } else {
            Block::default()
                .borders(if self.show_x_labels {
                    Borders::LEFT | Borders::BOTTOM
                } else {
                    Borders::LEFT
                })
                .border_style(style().fg(THEME.border_axis()))
                .render(layout[1], buf);
        }
    }
}
