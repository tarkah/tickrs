use std::collections::HashMap;

use super::*;
use crate::common::TimeFrame;
use crate::task::*;

pub struct DefaultTimestampService {
    handle: AsyncTaskHandle<HashMap<TimeFrame, Vec<i64>>>,
}

impl DefaultTimestampService {
    pub fn new() -> DefaultTimestampService {
        let task = DefaultTimestamps::new();
        let handle = task.connect();

        DefaultTimestampService { handle }
    }
}

impl Service for DefaultTimestampService {
    type Update = HashMap<TimeFrame, Vec<i64>>;

    fn updates(&self) -> Vec<Self::Update> {
        self.handle.response().try_iter().collect()
    }
}
