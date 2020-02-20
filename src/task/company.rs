use super::*;

use api::model::CompanyProfile;
use async_std::task;
use crossbeam_channel::bounded;

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
    type Response = Option<CompanyProfile>;

    fn update_interval(&self) -> Option<Duration> {
        None
    }

    fn connect(&self) -> AsyncTaskHandle<Self::Response> {
        let (response_sender, response_receiver) = bounded::<Self::Response>(1);

        let symbol = self.symbol.to_owned();

        let _handle = task::spawn(async move {
            let client = api::Client::new();

            if let Ok(response) = client.get_company(&symbol).await {
                let _ = response_sender.send(Some(response.profile));
            } else {
                let _ = response_sender.send(None);
            }
        });

        AsyncTaskHandle {
            _handle: None,
            drop_sender: None,
            response: response_receiver,
        }
    }
}
