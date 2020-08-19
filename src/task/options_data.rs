use super::*;

use api::model;
use async_std::task;
use crossbeam_channel::{bounded, select, unbounded};

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
    type Response = model::OptionsHeader;

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }

    fn connect(&self) -> AsyncTaskHandle<Self::Response> {
        let (drop_sender, drop_receiver) = bounded::<()>(1);
        let (response_sender, response_receiver) = unbounded::<Self::Response>();

        let update_interval = self.update_interval().unwrap();

        let symbol = self.symbol.to_owned();
        let date = self.date;

        let _handle = task::spawn(async move {
            let client = api::Client::new();

            loop {
                if let Ok(response) = client.get_options_for_expiration_date(&symbol, date).await {
                    let _ = response_sender.send(response);
                }

                // Break this loop to drop if drop msg received
                select! {
                    recv(drop_receiver) -> drop => if let Ok(()) = drop {
                        break;
                    },
                    default() => (),
                }

                task::sleep(update_interval).await;
            }
        });

        AsyncTaskHandle {
            _handle: Some(_handle),
            drop_sender: Some(drop_sender),
            response: response_receiver,
        }
    }
}
