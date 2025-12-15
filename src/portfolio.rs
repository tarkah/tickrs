use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct PortfolioItem {
    pub quantity: f64,
    pub average_price: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(transparent)]
pub struct Portfolio {
    pub items: HashMap<String, PortfolioItem>,
}

impl PortfolioItem {
    pub fn calculate_ticker_profit_loss(&self, current_price: f64) -> (f64, f64) {
        let invested = self.quantity * self.average_price;
        let current = self.quantity * current_price;
        let profit_loss = current - invested;
        let profit_loss_pct = if self.average_price > 0.0 {
            (current_price / self.average_price - 1.0) * 100.0
        } else {
            0.0
        };

        (profit_loss, profit_loss_pct)
    }
}
