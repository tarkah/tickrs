use anyhow::{bail, Result};
use serde::Deserialize;

pub(crate) enum ResponseType {
    Historical,
    HistoricalMinimal,
    Current,
    Company,
}

pub(crate) enum Response {
    Historical(Historical),
    HistoricalMinimal(HistoricalMinimal),
    Current(Current),
    Company(Company),
}

impl ResponseType {
    pub fn deserialize(&self, body: &[u8]) -> Result<Response> {
        match self {
            ResponseType::Historical => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::Historical(deser)),
                Err(e) => bail!(e),
            },
            ResponseType::HistoricalMinimal => match serde_json::from_slice(body) {
                Ok(deser) => Ok(Response::HistoricalMinimal(deser)),
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
pub struct Historical {
    pub symbol: String,
    pub historical: Vec<HistoricalDay>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct HistoricalDay {
    pub date: chrono::NaiveDate,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub adj_close: f32,
    pub volume: f64,
    pub unadjusted_volume: f64,
    pub change: f32,
    pub change_percent: f32,
    pub vwap: f32,
    pub change_over_time: f32,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct Current {
    pub symbol: String,
    pub price: f32,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct HistoricalMinimal {
    pub symbol: String,
    pub historical: Vec<HistoricalDayMinimal>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Clone)]
pub struct HistoricalDayMinimal {
    pub date: chrono::NaiveDate,
    pub close: f32,
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
