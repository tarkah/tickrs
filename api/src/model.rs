use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use anyhow::Result;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Chart {
    pub chart: ChartStatus,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartStatus {
    pub result: Option<Vec<ChartData>>,
    pub error: Option<Error>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartData {
    pub meta: ChartMeta,
    pub timestamp: Vec<i64>,
    pub indicators: ChartIndicators,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartMeta {
    pub instrument_type: Option<String>,
    pub regular_market_price: f64,
    pub chart_previous_close: f64,
    pub current_trading_period: Option<ChartCurrentTradingPeriod>,
}

impl Hash for ChartMeta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.instrument_type.hash(state);
        self.regular_market_price.to_bits().hash(state);
        self.chart_previous_close.to_bits().hash(state);
        self.current_trading_period.hash(state);
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ChartCurrentTradingPeriod {
    pub regular: ChartTradingPeriod,
    pub pre: ChartTradingPeriod,
    pub post: ChartTradingPeriod,
}

#[derive(Debug, Deserialize, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ChartTradingPeriod {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartIndicators {
    pub quote: Vec<ChartQuote>,
    pub adjclose: Option<Vec<ChartAdjClose>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartAdjClose {
    #[serde(deserialize_with = "deserialize_vec")]
    pub adjclose: Vec<f64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartQuote {
    #[serde(deserialize_with = "deserialize_vec")]
    pub close: Vec<f64>,
    #[serde(deserialize_with = "deserialize_vec")]
    pub volume: Vec<u64>,
    #[serde(deserialize_with = "deserialize_vec")]
    pub high: Vec<f64>,
    #[serde(deserialize_with = "deserialize_vec")]
    pub low: Vec<f64>,
    #[serde(deserialize_with = "deserialize_vec")]
    pub open: Vec<f64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    #[serde(rename = "quoteSummary")]
    pub company: CompanyStatus,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompanyStatus {
    pub result: Option<Vec<CompanyData>>,
    pub error: Option<Error>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompanyData {
    #[serde(rename = "assetProfile")]
    pub profile: Option<CompanyProfile>,
    pub price: CompanyPrice,
}

#[derive(Debug, Deserialize, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    pub website: Option<String>,
    pub industry: Option<String>,
    pub sector: Option<String>,
    #[serde(rename = "longBusinessSummary")]
    pub description: Option<String>,
    #[serde(rename = "fullTimeEmployees")]
    pub employees: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompanyPrice {
    pub symbol: String,
    pub short_name: String,
    pub long_name: Option<String>,
    pub regular_market_price: CompanyMarketPrice,
    pub regular_market_previous_close: CompanyMarketPrice,
    pub post_market_price: CompanyPostMarketPrice,
    pub regular_market_volume: CompanyMarketPrice,
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompanyMarketPrice {
    #[serde(rename = "raw")]
    pub price: f64,
    pub fmt: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompanyPostMarketPrice {
    #[serde(rename = "raw")]
    pub price: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub option_chain: OptionsStatus,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionsStatus {
    pub result: Option<Vec<OptionsHeader>>,
    pub error: Option<Error>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionsHeader {
    pub quote: OptionsQuote,
    pub expiration_dates: Vec<i64>,
    pub options: Vec<OptionsData>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionsQuote {
    pub regular_market_price: f64,
}

impl Hash for OptionsQuote {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.regular_market_price.to_bits().hash(state);
    }
}

#[derive(Debug, Deserialize, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub struct OptionsData {
    pub expiration_date: i64,
    pub calls: Vec<OptionsContract>,
    pub puts: Vec<OptionsContract>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionsContract {
    pub strike: f64,
    pub last_price: f64,
    pub change: f64,
    #[serde(default)]
    pub percent_change: f64,
    pub volume: Option<u64>,
    pub open_interest: Option<u64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub implied_volatility: Option<f64>,
    pub in_the_money: Option<bool>,
    pub currency: Option<String>,
}

impl Hash for OptionsContract {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.strike.to_bits().hash(state);
        self.last_price.to_bits().hash(state);
        self.change.to_bits().hash(state);
        self.percent_change.to_bits().hash(state);
        self.volume.hash(state);
        self.open_interest.hash(state);
        self.bid.map(|f| f.to_bits()).hash(state);
        self.ask.map(|f| f.to_bits()).hash(state);
        self.implied_volatility.map(|f| f.to_bits()).hash(state);
        self.in_the_money.hash(state);
        self.currency.hash(state);
    }
}

fn deserialize_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    struct SeqVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for SeqVisitor<T>
    where
        T: Deserialize<'de> + Default,
    {
        type Value = Vec<T>;

        fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
            fmt.write_str("default vec")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut vec = Vec::new();
            while let Ok(Some(elem)) = seq.next_element::<Option<T>>() {
                vec.push(elem.unwrap_or_default());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(SeqVisitor(PhantomData))
}
