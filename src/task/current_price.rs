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
    type Response = (f64, Option<f64>);

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
                let regular_price = response.price.regular_market_price.price;

                let post_price = response.price.post_market_price.price;

                Some((regular_price, post_price))
            } else {
                None
            }
        })
    }
}
