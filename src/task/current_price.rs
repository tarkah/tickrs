use super::*;

use async_std::sync::Arc;
use futures::future::BoxFuture;

/// Returns the current price, only if it has changed
pub struct CurrentPrice {
    symbol: String,
}

impl CurrentPrice {
    pub fn new(symbol: String) -> CurrentPrice {
        CurrentPrice { symbol }
    }
}

impl AsyncTask for CurrentPrice {
    type Input = (String, api::Client);
    type Response = f32;

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }

    fn input(&self) -> Self::Input {
        (self.symbol.clone(), api::Client::new())
    }

    fn task<'a>(input: Arc<Self::Input>) -> BoxFuture<'a, Option<Self::Response>> {
        Box::pin(async move {
            let symbol = &input.0;
            let client = &input.1;

            if let Ok(response) = client.get_company_data(symbol).await {
                Some(response.price.regular_market_price.price)
            } else {
                None
            }
        })
    }
}
