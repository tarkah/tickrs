use super::*;

use api::model::CompanyData;
use async_std::sync::Arc;
use futures::Future;
use std::pin::Pin;

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
    type Input = (String, api::Client);
    type Response = CompanyData;

    fn update_interval(&self) -> Option<Duration> {
        None
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

            client.get_company_data(symbol).await.ok()
        })
    }
}
