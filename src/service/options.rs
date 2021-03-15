use super::*;
use crate::api::model;
use crate::task::*;

pub struct OptionsService {
    symbol: String,
    expiration_dates_handle: AsyncTaskHandle<Vec<i64>>,
    options_data_handle: Option<AsyncTaskHandle<model::OptionsHeader>>,
}

impl OptionsService {
    pub fn new(symbol: String) -> OptionsService {
        let task = OptionsDates::new(symbol.clone());
        let expiration_dates_handle = task.connect();

        OptionsService {
            symbol,
            expiration_dates_handle,
            options_data_handle: None,
        }
    }

    pub fn set_expiration_date(&mut self, expiration_date: i64) {
        let task = OptionsData::new(self.symbol.clone(), expiration_date);
        let options_data_handle = task.connect();

        self.options_data_handle = Some(options_data_handle);
    }
}

#[derive(Debug)]
pub enum Update {
    ExpirationDates(Vec<i64>),
    OptionsData(model::OptionsHeader),
}

impl Service for OptionsService {
    type Update = Update;

    fn updates(&self) -> Vec<Self::Update> {
        let mut updates = vec![];

        let expiration_dates_updates = self
            .expiration_dates_handle
            .response()
            .try_iter()
            .map(Update::ExpirationDates);
        updates.extend(expiration_dates_updates);

        if let Some(ref options_data_handle) = self.options_data_handle {
            let options_data_updates = options_data_handle
                .response()
                .try_iter()
                .map(Update::OptionsData);
            updates.extend(options_data_updates);
        }

        updates
    }

    fn pause(&self) {
        self.expiration_dates_handle.pause();
        if let Some(handle) = self.options_data_handle.as_ref() {
            handle.pause();
        }
    }

    fn resume(&self) {
        self.expiration_dates_handle.resume();
        if let Some(handle) = self.options_data_handle.as_ref() {
            handle.resume();
        }
    }
}
