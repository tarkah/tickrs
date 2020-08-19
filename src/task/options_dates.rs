use super::*;

use async_std::task;
use crossbeam_channel::{bounded, select, unbounded};

use std::time::{Duration, Instant};

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
    type Response = Vec<i64>;

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(60 * 60))
    }

    fn connect(&self) -> AsyncTaskHandle<Self::Response> {
        let (drop_sender, drop_receiver) = bounded::<()>(1);
        let (response_sender, response_receiver) = unbounded::<Self::Response>();

        let update_interval = self.update_interval().unwrap();

        let symbol = self.symbol.to_owned();

        let _handle = task::spawn(async move {
            let client = api::Client::new();

            let mut last_updated = Instant::now();

            //Send it initially
            if let Ok(response) = client.get_options_expiration_dates(&symbol).await {
                let _ = response_sender.send(response);
            }

            loop {
                if last_updated.elapsed() >= update_interval {
                    if let Ok(response) = client.get_options_expiration_dates(&symbol).await {
                        let _ = response_sender.send(response);
                    }

                    last_updated = Instant::now();
                }

                // Break this loop to drop if drop msg received
                select! {
                    recv(drop_receiver) -> drop => if let Ok(()) = drop {
                        break;
                    },
                    default() => (),
                }

                task::sleep(Duration::from_secs(1)).await;
            }
        });

        AsyncTaskHandle {
            _handle: Some(_handle),
            drop_sender: Some(drop_sender),
            response: response_receiver,
        }
    }
}
