use async_std::sync::Arc;
use futures::future::BoxFuture;

use super::*;
use crate::api::model::CompanyData;
use crate::YAHOO_CRUMB;

/// Returns a companies profile information. Only needs to be returned once.
pub struct Company {
    symbol: String,
}

impl Company {
    pub fn new(symbol: String) -> Company {
        Company { symbol }
    }
}

impl AsyncTask for Company {
    type Input = String;
    type Response = CompanyData;

    fn update_interval(&self) -> Option<Duration> {
        None
    }

    fn input(&self) -> Self::Input {
        self.symbol.clone()
    }

    fn task<'a>(input: Arc<Self::Input>) -> BoxFuture<'a, Option<Self::Response>> {
        Box::pin(async move {
            let symbol = input.as_ref();

            let crumb = YAHOO_CRUMB.read().await.clone();

            if let Some(crumb) = crumb {
                crate::CLIENT.get_company_data(symbol, crumb).await.ok()
            } else {
                None
            }
        })
    }
}
