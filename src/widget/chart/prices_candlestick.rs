#![allow(unused_variables)]
#![allow(unused_imports)]

use std::convert::identity;
use std::fmt::LowerExp;

use itertools::Itertools;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Style;
use tui::symbols::Marker;
use tui::text::{Span, Spans};
use tui::widgets::canvas::{Canvas, Line, Rectangle};
use tui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, StatefulWidget, Widget,
};

use crate::common::{
    cast_as_dataset, cast_historical_as_price, remove_zeros_lows, zeros_as_pre, Price, TimeFrame,
    TradingPeriod,
};
use crate::draw::{add_padding, PaddingDirection};
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

    #[allow(clippy::clippy::unnecessary_unwrap)]
    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !self.is_summary {
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(THEME.border_secondary))
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
            .split(area);

        // Draw labels
        {
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(THEME.border_axis))
                .render(layout[0], buf);
            layout[0] = add_padding(layout[0], 1, PaddingDirection::Right);

            let mut y_labels = state.y_labels(min, max);

            let height = layout[0].height as usize;
            let top = 0;
            let mid = (height - 1) / 2;
            let bottom = height - 1;

            let mut labels = vec![Spans::default(); height];
            if let Some(label) = labels.get_mut(top) {
                *label = Spans::from(y_labels.pop().unwrap());
            }
            if let Some(label) = labels.get_mut(mid) {
                *label = Spans::from(y_labels.pop().unwrap());
            }
            if let Some(label) = labels.get_mut(bottom) {
                *label = Spans::from(y_labels.pop().unwrap());
            }

            Paragraph::new(labels).render(layout[0], buf);
        }

        let width = area.width;
        let num_candles = width / 2;
        let chunk_size = (x_bounds[1] / num_candles as f64).ceil() as usize;

        let candles = data
            .iter()
            .chunks(chunk_size)
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

        Canvas::default()
            .x_bounds([0.0, num_candles as f64 * 4.0])
            .y_bounds(state.y_bounds(min, max))
            .paint(move |ctx| {
                if state.time_frame == TimeFrame::Day1
                    && self.loaded
                    && !*HIDE_PREV_CLOSE
                    && state.prev_close_price.is_some()
                {
                    let num_points = (end - start) / 60 + 1;

                    ctx.draw(&Line {
                        x1: 0.0,
                        x2: num_candles as f64 * 4.0,
                        y1: state.prev_close_price.unwrap(),
                        y2: state.prev_close_price.unwrap(),
                        color: THEME.gray,
                    })
                }

                ctx.layer();

                for (idx, candle) in candles.iter().enumerate() {
                    if let Some(candle) = candle {
                        let color = if candle.close.gt(&candle.open) {
                            THEME.profit
                        } else {
                            THEME.loss
                        };

                        ctx.draw(&Rectangle {
                            x: idx as f64 * 4.0 + 1.0,
                            y: candle.open.min(candle.close),
                            width: 2.0,
                            height: candle.open.max(candle.close) - candle.open.min(candle.close),
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
    }
}
