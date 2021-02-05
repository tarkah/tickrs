use std::collections::HashMap;
use std::convert::identity;

use async_std::sync::Arc;
use futures::future::{join_all, BoxFuture};

use super::*;
use crate::common::TimeFrame;

/// Default timestamps to reference for stocks that haven't been around as long
/// as the interval we are trying to graph
pub struct DefaultTimestamps {}

impl DefaultTimestamps {
    pub fn new() -> DefaultTimestamps {
        DefaultTimestamps {}
    }
}

impl AsyncTask for DefaultTimestamps {
    type Input = ();
    type Response = HashMap<TimeFrame, Vec<i64>>;

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(60 * 15))
    }

    fn input(&self) -> Self::Input {}

    fn task<'a>(_input: Arc<Self::Input>) -> BoxFuture<'a, Option<Self::Response>> {
        Box::pin(async move {
            let client = Arc::new(api::Client::new());

            let symbol = "SPY";

            let tasks = TimeFrame::ALL[1..].iter().map(|timeframe| {
                let client_clone = client.clone();

                async move {
                    let interval = timeframe.api_interval();
                    let range = timeframe.as_range();

                    if let Ok(chart) = client_clone
                        .get_chart_data(symbol, interval, range, false)
                        .await
                    {
                        Some((*timeframe, chart.timestamp))
                    } else {
                        None
                    }
                }
            });

            Some(
                join_all(tasks)
                    .await
                    .into_iter()
                    .filter_map(identity)
                    .collect(),
            )
        })
    }
}
