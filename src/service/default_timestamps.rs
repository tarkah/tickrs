use super::*;
use crate::common::Timestamps;
use crate::task::*;

pub struct DefaultTimestampService {
    handle: AsyncTaskHandle<Timestamps>,
}

impl DefaultTimestampService {
    pub fn new() -> DefaultTimestampService {
        let task = DefaultTimestamps::new();
        let handle = task.connect();

        DefaultTimestampService { handle }
    }
}

impl Service for DefaultTimestampService {
    type Update = Timestamps;

    fn updates(&self) -> Vec<Self::Update> {
        self.handle.response().try_iter().collect()
    }

    fn pause(&self) {
        self.handle.pause();
    }

    fn resume(&self) {
        self.handle.resume();
    }
}
