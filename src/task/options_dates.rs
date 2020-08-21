use super::*;

use async_std::sync::Arc;

use std::time::Duration;

/// Returns options expiration dates for a company
pub struct OptionsDates {
    symbol: String,
}

impl OptionsDates {
    pub fn new(symbol: String) -> OptionsDates {
        OptionsDates { symbol }
    }
}

impl AsyncTask for OptionsDates {
    type Input = (String, api::Client);
    type Response = Vec<i64>;

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(60 * 60))
    }

    fn input(&self) -> Self::Input {
        (self.symbol.clone(), api::Client::new())
    }

    fn task(
        input: Arc<Self::Input>,
    ) -> Pin<Box<dyn Future<Output = Option<Self::Response>> + Send>> {
        Box::pin(async move {
            let symbol = &input.0;
            let client = &input.1;

            client.get_options_expiration_dates(symbol).await.ok()
        })
    }
}
