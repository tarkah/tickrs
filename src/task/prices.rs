use super::*;
use crate::TimeFrame;

use api::{model::Price, Interval};
use async_std::task;
use chrono::{
    offset::TimeZone,
    {Local, NaiveDateTime, Utc},
};
use chrono_tz::US::Eastern;
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
                TimeFrame::Day1 => Some(Interval::Minute1),
                TimeFrame::Week1 => Some(Interval::Minute15),
                TimeFrame::Month1 => Some(Interval::Minute30),
                _ => None,
            };

            // Send it initially
            if let Some(interval) = interval {
                if let Ok(response) = client.get_historical_interval(&symbol, interval).await {
                    let mut prices = response.prices;
                    prices.reverse();

                    if time_frame == TimeFrame::Day1 {
                        prices = prices
                            .into_iter()
                            .filter(|price| {
                                let today_utc = Utc::today();
                                let today = today_utc.with_timezone(&Eastern);

                                let datetime =
                                    NaiveDateTime::parse_from_str(&price.date, "%Y-%m-%d %H:%M:%S")
                                        .unwrap();
                                let date = Eastern.from_local_datetime(&datetime).unwrap().date();

                                today == date
                            })
                            .collect::<Vec<_>>();
                    }

                    let _ = response_sender.send(prices);
                }
            } else {
                let today = Local::today().naive_local();
                let as_of = time_frame.as_of_date();

                if let Ok(response) = client
                    .get_historical_daily_from_to(&symbol, as_of, today)
                    .await
                {
                    let _ = response_sender.send(response.prices);
                }
            }

            loop {
                // Only send it adter update interval has passed
                if last_updated.elapsed() >= update_interval {
                    if let Some(interval) = interval {
                        if let Ok(response) =
                            client.get_historical_interval(&symbol, interval).await
                        {
                            let mut prices = response.prices;
                            prices.reverse();

                            if time_frame == TimeFrame::Day1 {
                                prices = prices
                                    .into_iter()
                                    .filter(|price| {
                                        let today_utc = Utc::today();
                                        let today = today_utc.with_timezone(&Eastern);

                                        let datetime = NaiveDateTime::parse_from_str(
                                            &price.date,
                                            "%Y-%m-%d %H:%M:%S",
                                        )
                                        .unwrap();
                                        let date =
                                            Eastern.from_local_datetime(&datetime).unwrap().date();
                                        today == date
                                    })
                                    .collect::<Vec<_>>();
                            }

                            let _ = response_sender.send(prices);
                        }
                    } else {
                        let today = Local::today().naive_local();
                        let as_of = time_frame.as_of_date();
                        if let Ok(response) = client
                            .get_historical_daily_from_to(&symbol, as_of, today)
                            .await
                        {
                            let _ = response_sender.send(response.prices);
                        }
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
