use async_std::task::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use std::time::Duration;

mod company;
mod current_price;
mod prices;

pub use company::Company;
pub use current_price::CurrentPrice;
pub use prices::Prices;

/// Trait to define a type that spawns an Async Task to complete background
/// work.
pub trait AsyncTask {
    type Response;

    fn update_interval(&self) -> Option<Duration>;
    fn connect(&self) -> AsyncTaskHandle<Self::Response>;
}

pub struct AsyncTaskHandle<R> {
    _handle: Option<JoinHandle<()>>,
    drop_sender: Option<Sender<()>>,
    pub response: Receiver<R>,
}

impl<R> Drop for AsyncTaskHandle<R> {
    fn drop(&mut self) {
        if let Some(ref drop_sender) = self.drop_sender {
            let _ = drop_sender.send(());
        }
    }
}
