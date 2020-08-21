use super::*;

use api::model;
use async_std::sync::Arc;

use std::time::Duration;

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
    type Input = (String, i64, api::Client);
    type Response = model::OptionsHeader;

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }

    fn input(&self) -> Self::Input {
        (self.symbol.clone(), self.date, api::Client::new())
    }

    fn task(
        input: Arc<Self::Input>,
    ) -> Pin<Box<dyn Future<Output = Option<Self::Response>> + Send>> {
        Box::pin(async move {
            let symbol = &input.0;
            let date = input.1;
            let client = &input.2;

            client
                .get_options_for_expiration_date(symbol, date)
                .await
                .ok()
        })
    }
}
