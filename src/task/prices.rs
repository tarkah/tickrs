use super::*;
use crate::common::{chart_data_to_prices, Price, TimeFrame};

use api::Interval;
use async_std::task;
use crossbeam_channel::{bounded, select, unbounded};
use std::time::Instant;

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
    type Response = Vec<Price>;

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

            let mut last_updated = Instant::now();

            let interval = match time_frame {
                TimeFrame::Day1 => Interval::Minute1,
                TimeFrame::Week1 => Interval::Minute5,
                TimeFrame::Month1 => Interval::Minute30,
                TimeFrame::Month3 => Interval::Minute60,
                TimeFrame::Month6 => Interval::Minute60,
                _ => Interval::Day1,
            };

            // Send it initially
            if let Ok(response) = client
                .get_chart_data(&symbol, interval, time_frame.as_range())
                .await
            {
                let prices = chart_data_to_prices(response);

                let _ = response_sender.send(prices);
            }

            loop {
                // Only send it adter update interval has passed
                if last_updated.elapsed() >= update_interval {
                    if let Ok(response) = client
                        .get_chart_data(&symbol, interval, time_frame.as_range())
                        .await
                    {
                        let prices = chart_data_to_prices(response);

                        let _ = response_sender.send(prices);
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
