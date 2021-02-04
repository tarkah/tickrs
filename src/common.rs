use std::str::FromStr;
use std::time::Duration;

use chrono::{Local, TimeZone, Utc};
use itertools::izip;

use crate::api::model::ChartData;
use crate::api::Range;

#[derive(PartialEq, Clone, Copy, PartialOrd, Debug)]
pub enum TimeFrame {
    Day1,
    Week1,
    Month1,
    Month3,
    Month6,
    Year1,
    Year5,
}

impl FromStr for TimeFrame {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use TimeFrame::*;

        match s {
            "1D" => Ok(Day1),
            "1W" => Ok(Week1),
            "1M" => Ok(Month1),
            "3M" => Ok(Month3),
            "6M" => Ok(Month6),
            "1Y" => Ok(Year1),
            "5Y" => Ok(Year5),
            _ => Err("Valid time frames are: '1D', '1W', '1M', '3M', '6M', '1Y', '5Y'"),
        }
    }
}

impl TimeFrame {
    pub fn idx(self) -> usize {
        match self {
            TimeFrame::Day1 => 0,
            TimeFrame::Week1 => 1,
            TimeFrame::Month1 => 2,
            TimeFrame::Month3 => 3,
            TimeFrame::Month6 => 4,
            TimeFrame::Year1 => 5,
            TimeFrame::Year5 => 6,
        }
    }

    pub const fn tab_names() -> [&'static str; 7] {
        ["1D", "1W", "1M", "3M", "6M", "1Y", "5Y"]
    }

    pub fn update_interval(self) -> Duration {
        match self {
            TimeFrame::Day1 => Duration::from_secs(60),
            TimeFrame::Week1 => Duration::from_secs(60 * 5),
            TimeFrame::Month1 => Duration::from_secs(60 * 30),
            TimeFrame::Month3 => Duration::from_secs(60 * 60),
            TimeFrame::Month6 => Duration::from_secs(60 * 60),
            TimeFrame::Year1 => Duration::from_secs(60 * 60 * 24),
            TimeFrame::Year5 => Duration::from_secs(60 * 60 * 24),
        }
    }

    pub fn up(self) -> TimeFrame {
        match self {
            TimeFrame::Day1 => TimeFrame::Week1,
            TimeFrame::Week1 => TimeFrame::Month1,
            TimeFrame::Month1 => TimeFrame::Month3,
            TimeFrame::Month3 => TimeFrame::Month6,
            TimeFrame::Month6 => TimeFrame::Year1,
            TimeFrame::Year1 => TimeFrame::Year5,
            TimeFrame::Year5 => TimeFrame::Day1,
        }
    }

    pub fn down(self) -> TimeFrame {
        match self {
            TimeFrame::Day1 => TimeFrame::Year5,
            TimeFrame::Week1 => TimeFrame::Day1,
            TimeFrame::Month1 => TimeFrame::Week1,
            TimeFrame::Month3 => TimeFrame::Month1,
            TimeFrame::Month6 => TimeFrame::Month3,
            TimeFrame::Year1 => TimeFrame::Month6,
            TimeFrame::Year5 => TimeFrame::Year1,
        }
    }

    pub fn as_range(self) -> Range {
        match self {
            TimeFrame::Day1 => Range::Day1,
            TimeFrame::Week1 => Range::Day5,
            TimeFrame::Month1 => Range::Month1,
            TimeFrame::Month3 => Range::Month3,
            TimeFrame::Month6 => Range::Month6,
            TimeFrame::Year1 => Range::Year1,
            TimeFrame::Year5 => Range::Year5,
        }
    }

    pub fn format_time(&self, timestamp: i64) -> String {
        let utc_date = Utc.timestamp(timestamp, 0);
        let local_date = utc_date.with_timezone(&Local);

        let fmt = match self {
            TimeFrame::Day1 => "%H:%M",
            TimeFrame::Week1 => "%m-%d %H:%M",
            _ => "%F",
        };

        local_date.format(fmt).to_string()
    }
}

#[derive(Clone, Copy)]
pub struct MarketHours(pub i64, pub i64);

impl Default for MarketHours {
    fn default() -> Self {
        MarketHours(52200, 75600)
    }
}

impl Iterator for MarketHours {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == self.1 {
            None
        } else {
            let result = Some(self.0);
            self.0 += 60;
            result
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradingPeriod {
    Pre,
    Regular,
    Post,
}

#[derive(Debug, Clone, Copy)]
pub struct Price {
    pub close: f64,
    pub volume: u64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub date: i64,
}

pub fn chart_data_to_prices(mut chart_data: ChartData) -> Vec<Price> {
    if chart_data.indicators.quote.len() != 1 {
        return vec![];
    }

    let quote = chart_data.indicators.quote.remove(0);
    let timestamps = chart_data.timestamp;

    izip!(
        &quote.close,
        &quote.volume,
        &quote.high,
        &quote.low,
        &quote.open,
        &timestamps,
    )
    .map(|(c, v, h, l, o, t)| Price {
        close: *c,
        volume: *v,
        high: *h,
        low: *l,
        open: *o,
        date: *t,
    })
    .collect()
}

pub fn cast_as_dataset(input: (usize, &f64)) -> (f64, f64) {
    ((input.0 + 1) as f64, *input.1)
}

pub fn cast_historical_as_price(input: &Price) -> f64 {
    input.close
}

pub fn zeros_as_pre(prices: &mut [f64]) {
    if prices.len() <= 1 {
        return;
    }

    let zero_indexes = prices
        .iter()
        .enumerate()
        .filter_map(|(idx, price)| if *price == 0.0 { Some(idx) } else { None })
        .collect::<Vec<usize>>();

    for idx in zero_indexes {
        if idx == 0 {
            prices[0] = prices[1];
        } else {
            prices[idx] = prices[idx - 1];
        }
    }
}

pub fn remove_zeros(prices: Vec<f64>) -> Vec<f64> {
    prices.into_iter().filter(|x| x.ne(&0.0)).collect()
}

pub fn remove_zeros_lows(prices: Vec<Price>) -> Vec<Price> {
    prices.into_iter().filter(|x| x.low.ne(&0.0)).collect()
}
