use super::*;
use crate::common::*;
use crate::task::*;

use api::model::{ChartTradingPeriod, CompanyData};

pub struct StockService {
    symbol: String,
    current_price_handle: AsyncTaskHandle<f32>,
    prices_handle: AsyncTaskHandle<(Option<ChartTradingPeriod>, Vec<Price>)>,
    company_handle: AsyncTaskHandle<CompanyData>,
}

impl StockService {
    pub fn new(symbol: String, time_frame: TimeFrame) -> StockService {
        let task = CurrentPrice::new(symbol.clone());
        let current_price_handle = task.connect();

        let task = Prices::new(symbol.clone(), time_frame);
        let prices_handle = task.connect();

        let task = Company::new(symbol.clone());
        let company_handle = task.connect();

        StockService {
            symbol,
            current_price_handle,
            prices_handle,
            company_handle,
        }
    }

    pub fn update_time_frame(&mut self, time_frame: TimeFrame) {
        let task = Prices::new(self.symbol.clone(), time_frame);
        let prices_handle = task.connect();

        self.prices_handle = prices_handle;
    }
}

#[derive(Debug)]
pub enum Update {
    NewPrice(f32),
    Prices((Option<ChartTradingPeriod>, Vec<Price>)),
    CompanyData(CompanyData),
}

impl Service for StockService {
    type Update = Update;

    fn updates(&self) -> Vec<Self::Update> {
        let mut updates = vec![];

        let current_price_updates = self
            .current_price_handle
            .response()
            .try_iter()
            .map(Update::NewPrice);
        updates.extend(current_price_updates);

        let prices_updates = self.prices_handle.response().try_iter().map(Update::Prices);
        updates.extend(prices_updates);

        let company_updates = self
            .company_handle
            .response()
            .try_iter()
            .map(Update::CompanyData);
        updates.extend(company_updates);

        updates
    }
}
