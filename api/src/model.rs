use anyhow::{bail, Result};
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;
use std::marker::PhantomData;

pub(crate) enum ResponseType {
    Chart,
    Company,
    Options,
}

pub(crate) enum Response {
    Chart(Chart),
    Company(Company),
    Options(Options),
}

impl ResponseType {
    pub fn deserialize(&self, body: &[u8]) -> Result<Response> {
        match self {
            ResponseType::Chart => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::Chart(deser)),
                Err(e) => bail!(e),
            },
            ResponseType::Company => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::Company(deser)),
                Err(e) => bail!(e),
            },
            ResponseType::Options => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::Options(deser)),
                Err(e) => bail!(e),
            },
        }
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct Chart {
    pub chart: ChartStatus,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct ChartStatus {
    pub result: Option<Vec<ChartData>>,
    pub error: Option<Error>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct Error {
    pub code: String,
    pub description: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct ChartData {
    pub meta: ChartMeta,
    pub timestamp: Vec<i64>,
    pub indicators: ChartIndicators,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct ChartMeta {
    pub regular_market_price: f32,
    pub chart_previous_close: f32,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct ChartIndicators {
    pub quote: Vec<ChartQuote>,
    pub adjclose: Option<Vec<ChartAdjClose>>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct ChartAdjClose {
    pub adjclose: Vec<f32>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct ChartQuote {
    #[serde(deserialize_with = "deserialize_vec")]
    pub close: Vec<f32>,
    #[serde(deserialize_with = "deserialize_vec")]
    pub volume: Vec<u32>,
    #[serde(deserialize_with = "deserialize_vec")]
    pub high: Vec<f32>,
    #[serde(deserialize_with = "deserialize_vec")]
    pub low: Vec<f32>,
    #[serde(deserialize_with = "deserialize_vec")]
    pub open: Vec<f32>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct Company {
    #[serde(rename = "quoteSummary")]
    pub company: CompanyStatus,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct CompanyStatus {
    pub result: Option<Vec<CompanyData>>,
    pub error: Option<Error>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct CompanyData {
    #[serde(rename = "assetProfile")]
    pub profile: CompanyProfile,
    pub price: CompanyPrice,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct CompanyProfile {
    pub website: Option<String>,
    pub industry: Option<String>,
    pub sector: Option<String>,
    #[serde(rename = "longBusinessSummary")]
    pub description: Option<String>,
    #[serde(rename = "fullTimeEmployees")]
    pub employees: Option<u64>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct CompanyPrice {
    pub symbol: String,
    pub short_name: String,
    pub long_name: String,
    pub regular_market_price: CompanyRegularMarketPrice,
    pub regular_market_previous_close: CompanyRegularMarketPreviousClose,
    pub currency: String
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct CompanyRegularMarketPrice {
    #[serde(rename = "raw")]
    pub price: f32,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct CompanyRegularMarketPreviousClose {
    #[serde(rename = "raw")]
    pub price: f32,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct Options {
    pub option_chain: OptionsStatus,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct OptionsStatus {
    pub result: Option<Vec<OptionsHeader>>,
    pub error: Option<Error>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct OptionsHeader {
    pub quote: OptionsQuote,
    pub expiration_dates: Vec<i64>,
    pub options: Vec<OptionsData>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct OptionsQuote {
    pub regular_market_price: f32,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct OptionsData {
    pub expiration_date: i64,
    pub calls: Vec<OptionsContract>,
    pub puts: Vec<OptionsContract>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct OptionsContract {
    pub strike: f32,
    pub last_price: f32,
    pub change: f32,
    pub percent_change: f32,
    pub volume: Option<u32>,
    pub open_interest: Option<u32>,
    pub bid: Option<f32>,
    pub ask: Option<f32>,
    pub implied_volatility: Option<f32>,
    pub in_the_money: Option<bool>,
    pub currency: String
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
