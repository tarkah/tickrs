use super::*;
use crate::TimeFrame;

use api::model::HistoricalDay;
use async_std::task;
use chrono::Local;
use crossbeam_channel::{bounded, select, unbounded};

/// Returns an array of prices, depending on the TimeFrame chosen
pub struct Prices {
    symbol: String,
    time_frame: TimeFrame,
}

impl Prices {
    pub fn new(symbol: String, time_frame: TimeFrame) -> Prices {
        Prices { symbol, time_frame }
    }
}

impl AsyncTask for Prices {
    type Response = Vec<HistoricalDay>;

    fn update_interval(&self) -> Option<Duration> {
        Some(self.time_frame.update_interval())
    }

    fn connect(&self) -> AsyncTaskHandle<Self::Response> {
        let (drop_sender, drop_receiver) = bounded::<()>(1);
        let (response_sender, response_receiver) = unbounded::<Self::Response>();

        let update_interval = self.update_interval().unwrap();

        let time_frame = self.time_frame;
        let symbol = self.symbol.to_owned();

        let _handle = task::spawn(async move {
            let client = api::Client::new();

            loop {
                let today = Local::today().naive_local();
                let as_of = time_frame.as_of_date();

                if let Ok(response) = client.get_historical_from_to(&symbol, as_of, today).await {
                    let _ = response_sender.send(response.historical);
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
