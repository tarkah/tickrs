use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::symbols::Marker;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, StatefulWidget, Widget};

use crate::common::{
    cast_as_dataset, cast_historical_as_price, zeros_as_pre, Price, TimeFrame, TradingPeriod,
};
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

impl<'a> StatefulWidget for PricesLineChart<'a> {
    type State = StockState;

    #[allow(clippy::clippy::unnecessary_unwrap)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (min, max) = state.min_max(&self.data);
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

        let x_labels = if self.show_x_labels {
            state.x_labels(area.width, start, end, &self.data)
        } else {
            vec![]
        };

        let trading_period = state.current_trading_period(&self.data);

        let (reg_prices, pre_prices, post_prices) = if self.loaded {
            let (start_idx, end_idx) = state.regular_start_end_idx(&self.data);

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
                    .data(&data),
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
                    .data(&data),
            );
        }

        if let Some(data) = prev_close_line.as_ref() {
            datasets.insert(
                0,
                Dataset::default()
                    .marker(Marker::Braille)
                    .style(style().fg(THEME.gray()))
                    .graph_type(GraphType::Line)
                    .data(&data),
            );
        }

        let mut chart = Chart::new(datasets)
            .style(style())
            .x_axis({
                let axis = Axis::default().bounds(state.x_bounds(start, end, &self.data));

                if self.show_x_labels && self.loaded && !self.is_summary {
                    axis.labels(x_labels).style(style().fg(THEME.border_axis()))
                } else {
                    axis
                }
            })
            .y_axis(
                Axis::default()
                    .bounds(state.y_bounds(min, max))
                    .labels(state.y_labels(min, max))
                    .style(style().fg(THEME.border_axis())),
            );

        if !self.is_summary {
            chart = chart.block(
                Block::default()
                    .style(style().fg(THEME.border_secondary()))
                    .borders(Borders::TOP)
                    .border_style(style()),
            );
        }

        chart.render(area, buf);
    }
}
