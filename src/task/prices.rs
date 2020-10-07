use super::*;
use crate::common::{chart_data_to_prices, Price, TimeFrame};

use api::{model::ChartMeta, Interval};
use async_std::sync::Arc;
use futures::future::BoxFuture;

/// Returns an array of prices, depending on the TimeFrame chosen
pub struct Prices {
    symbol: String,
    time_frame: TimeFrame,
}

impl Prices {
    pub fn new(symbol: String, time_frame: TimeFrame) -> Prices {
        Prices { symbol, time_frame }
    }
}

impl AsyncTask for Prices {
    type Input = (String, TimeFrame, api::Client);
    type Response = (ChartMeta, Vec<Price>);

    fn update_interval(&self) -> Option<Duration> {
        Some(self.time_frame.update_interval())
    }

    fn input(&self) -> Self::Input {
        (self.symbol.clone(), self.time_frame, api::Client::new())
    }

    fn task<'a>(input: Arc<Self::Input>) -> BoxFuture<'a, Option<Self::Response>> {
        Box::pin(async move {
            let symbol = &input.0;
            let time_frame = input.1;
            let client = &input.2;

            let interval = match time_frame {
                TimeFrame::Day1 => Interval::Minute1,
                TimeFrame::Week1 => Interval::Minute5,
                TimeFrame::Month1 => Interval::Minute30,
                TimeFrame::Month3 => Interval::Minute60,
                TimeFrame::Month6 => Interval::Minute60,
                _ => Interval::Day1,
            };

            if let Ok(response) = client
                .get_chart_data(&symbol, interval, time_frame.as_range())
                .await
            {
                Some((response.meta.clone(), chart_data_to_prices(response)))
            } else {
                None
            }
        })
    }
}
