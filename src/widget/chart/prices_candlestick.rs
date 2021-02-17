#![allow(unused_variables)]
#![allow(unused_imports)]

use std::convert::identity;

use itertools::Itertools;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::symbols::Marker;
use tui::widgets::canvas::{Canvas, Rectangle};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, StatefulWidget, Widget};

use crate::common::{
    cast_as_dataset, cast_historical_as_price, remove_zeros_lows, zeros_as_pre, Price, TimeFrame,
    TradingPeriod,
};
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
}

impl<'a> StatefulWidget for PricesCandlestickChart<'a> {
    type State = StockState;

    #[allow(clippy::clippy::unnecessary_unwrap)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (min, max) = state.min_max(&self.data);
        let (start, end) = state.start_end();
        let x_bounds = state.x_bounds(start, end, self.data);

        let width = area.width;
        let num_candles = width / 2;
        let chunk_size = (x_bounds[1] / num_candles as f64).ceil() as usize;

        let candles = self
            .data
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
            .block(
                Block::default()
                    .style(Style::default().fg(THEME.border_secondary))
                    .borders(Borders::TOP)
                    .border_style(Style::default()),
            )
            .x_bounds([0.0, num_candles as f64 * 4.0])
            .y_bounds(state.y_bounds(min, max))
            .paint(move |ctx| {
                for (idx, candle) in candles.iter().enumerate() {
                    if let Some(candle) = candle {
                        ctx.draw(&Rectangle {
                            x: idx as f64 * 4.0 + 1.0,
                            y: candle.open.min(candle.close),
                            width: 2.0,
                            height: candle.open.max(candle.close) - candle.open.min(candle.close),
                            color: if candle.close.gt(&candle.open) {
                                THEME.profit
                            } else {
                                THEME.loss
                            },
                        });
                    }
                }
            })
            .render(area, buf);
    }
}
