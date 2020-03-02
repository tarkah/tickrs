use anyhow::{bail, Result};
use serde::Deserialize;

pub(crate) enum ResponseType {
    HistoricalDaily,
    HistoricalInterval,
    Current,
    Company,
}

pub(crate) enum Response {
    HistoricalDaily(HistoricalDaily),
    HistoricalInterval(HistoricalInterval),
    Current(Current),
    Company(Company),
}

impl ResponseType {
    pub fn deserialize(&self, body: &[u8]) -> Result<Response> {
        match self {
            ResponseType::HistoricalDaily => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::HistoricalDaily(deser)),
                Err(e) => bail!(e),
            },
            ResponseType::HistoricalInterval => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::HistoricalInterval(deser)),
                Err(e) => bail!(e),
            },
            ResponseType::Current => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::Current(deser)),
                Err(e) => bail!(e),
            },
            ResponseType::Company => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::Company(deser)),
                Err(e) => bail!(e),
            },
        }
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct HistoricalDaily {
    pub symbol: String,
    #[serde(rename = "historical")]
    pub prices: Vec<Price>,
}

#[serde(rename_all = "camelCase", transparent)]
#[derive(Debug, Deserialize, Clone)]
pub struct HistoricalInterval {
    pub prices: Vec<Price>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct Price {
    pub date: String, //chrono::NaiveDate,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f64,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct Current {
    pub symbol: String,
    pub price: f32,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct Company {
    pub symbol: String,
    pub profile: CompanyProfile,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct CompanyProfile {
    pub price: f32,
    pub beta: Option<String>,
    pub vol_avg: String,
    pub mkt_cap: String,
    pub company_name: String,
    pub description: String,
}
