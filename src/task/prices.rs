use async_std::sync::Arc;
use futures::future::BoxFuture;

use super::*;
use crate::api::model::ChartMeta;
use crate::common::{chart_data_to_prices, Price, TimeFrame};

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
    type Input = (String, TimeFrame);
    type Response = (TimeFrame, ChartMeta, Vec<Price>);

    fn update_interval(&self) -> Option<Duration> {
        Some(self.time_frame.update_interval())
    }

    fn input(&self) -> Self::Input {
        (self.symbol.clone(), self.time_frame)
    }

    fn task<'a>(input: Arc<Self::Input>) -> BoxFuture<'a, Option<Self::Response>> {
        Box::pin(async move {
            let symbol = &input.0;
            let time_frame = input.1;

            let interval = time_frame.api_interval();

            let include_pre_post = time_frame == TimeFrame::Day1;

            if let Ok(response) = crate::CLIENT
                .get_chart_data(symbol, interval, time_frame.as_range(), include_pre_post)
                .await
            {
                Some((
                    time_frame,
                    response.meta.clone(),
                    chart_data_to_prices(response),
                ))
            } else {
                None
            }
        })
    }
}
