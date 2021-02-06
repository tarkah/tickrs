use std::time::Duration;

use async_std::sync::Arc;
use futures::future::BoxFuture;

use super::*;
use crate::api::model;

/// Returns options data for a company
pub struct OptionsData {
    symbol: String,
    date: i64,
}

impl OptionsData {
    pub fn new(symbol: String, date: i64) -> OptionsData {
        OptionsData { symbol, date }
    }
}

impl AsyncTask for OptionsData {
    type Input = (String, i64);
    type Response = model::OptionsHeader;

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }

    fn input(&self) -> Self::Input {
        (self.symbol.clone(), self.date)
    }

    fn task<'a>(input: Arc<Self::Input>) -> BoxFuture<'a, Option<Self::Response>> {
        Box::pin(async move {
            let symbol = &input.0;
            let date = input.1;

            crate::CLIENT
                .get_options_for_expiration_date(symbol, date)
                .await
                .ok()
        })
    }
}
