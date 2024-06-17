use async_std::sync::Arc;
use futures::future::BoxFuture;

use super::*;
use crate::YAHOO_CRUMB;

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
    type Input = String;
    type Response = (f64, Option<f64>, String);

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }

    fn input(&self) -> Self::Input {
        self.symbol.clone()
    }

    fn task<'a>(input: Arc<Self::Input>) -> BoxFuture<'a, Option<Self::Response>> {
        Box::pin(async move {
            let symbol = input.as_ref();

            let crumb = YAHOO_CRUMB.read().await.clone();

            if let Some(crumb) = crumb {
                if let Ok(response) = crate::CLIENT.get_company_data(symbol, crumb).await {
                    let regular_price = response.price.regular_market_price.price;

                    let post_price = response.price.post_market_price.price;

                    let volume = response.price.regular_market_volume.fmt.unwrap_or_default();

                    return Some((regular_price, post_price, volume));
                }
            }

            None
        })
    }
}
