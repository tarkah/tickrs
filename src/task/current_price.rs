use super::*;

use async_std::task;
use crossbeam_channel::{bounded, select, unbounded};

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
    type Response = f32;

    fn update_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }

    fn connect(&self) -> AsyncTaskHandle<Self::Response> {
        let (drop_sender, drop_receiver) = bounded::<()>(1);
        let (response_sender, response_receiver) = unbounded::<Self::Response>();

        let update_interval = self.update_interval().unwrap();

        let symbol = self.symbol.to_owned();

        let _handle = task::spawn(async move {
            let client = api::Client::new();

            let mut last_price = 0.0;

            loop {
                if let Ok(response) = client.get_current(&symbol).await {
                    let current_price = response.price;

                    if last_price.ne(&current_price) {
                        let _ = response_sender.send(current_price);

                        last_price = current_price;
                    }
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
