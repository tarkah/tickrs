use std::time::{Duration, Instant};

use async_std::sync::Arc;
use async_std::task;
use crossbeam_channel::{unbounded, Receiver};
use futures::future::BoxFuture;
use task::JoinHandle;

pub use self::company::Company;
pub use self::current_price::CurrentPrice;
pub use self::default_timestamps::DefaultTimestamps;
pub use self::options_data::OptionsData;
pub use self::options_dates::OptionsDates;
pub use self::prices::Prices;
use crate::{DATA_RECEIVED, UPDATE_INTERVAL};

mod company;
mod current_price;
mod default_timestamps;
mod options_data;
mod options_dates;
mod prices;

/// Trait to define a type that spawns an Async Task to complete background
/// work.
pub trait AsyncTask: 'static {
    type Input: Send + Sync;
    type Response: Send;

    /// Interval that `task` should be executed at
    ///
    /// If `None` is returned, the task will only get executed once then exit
    fn update_interval(&self) -> Option<Duration>;

    /// Input data needed for the `task`
    fn input(&self) -> Self::Input;

    /// Defines the async task that will get executed and return` Response`
    fn task<'a>(input: Arc<Self::Input>) -> BoxFuture<'a, Option<Self::Response>>;

    /// Runs the task on the async runtime and returns a handle to query updates from
    fn connect(&self) -> AsyncTaskHandle<Self::Response> {
        let (response_sender, response_receiver) = unbounded::<Self::Response>();
        let data_received = DATA_RECEIVED.0.clone();

        let update_interval = self.update_interval();
        let input = Arc::new(self.input());

        let handle = task::spawn(async move {
            let mut last_updated = Instant::now();

            // Execute the task initially and request a redraw to display this data
            if let Some(response) = <Self as AsyncTask>::task(input.clone()).await {
                let _ = response_sender.send(response);
                let _ = data_received.try_send(());
            }

            // If no update interval is defined, exit task
            let update_interval = if let Some(interval) = update_interval {
                interval.max(Duration::from_secs(*UPDATE_INTERVAL))
            } else {
                return;
            };

            // Execute task every update interval
            loop {
                if last_updated.elapsed() >= update_interval {
                    if let Some(response) = <Self as AsyncTask>::task(input.clone()).await {
                        let _ = response_sender.send(response);
                        let _ = data_received.try_send(());
                    }

                    last_updated = Instant::now();
                }

                // Free up some cycles
                task::sleep(Duration::from_secs(1)).await;
            }
        });

        AsyncTaskHandle {
            response: response_receiver,
            handle: Some(handle),
        }
    }
}

pub struct AsyncTaskHandle<R> {
    response: Receiver<R>,
    handle: Option<JoinHandle<()>>,
}

impl<R> AsyncTaskHandle<R> {
    pub fn response(&self) -> &Receiver<R> {
        &self.response
    }
}

impl<R> Drop for AsyncTaskHandle<R> {
    fn drop(&mut self) {
        let handle = self.handle.take().unwrap();
        task::block_on(async { handle.cancel().await });
    }
}
