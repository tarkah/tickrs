use crate::{
    model::{CompanyProfile, Current, HistoricalDaily, HistoricalInterval, Response, ResponseType},
    Interval,
};
use anyhow::{bail, Context, Result};
use futures::AsyncReadExt;
use http::Uri;
use isahc::HttpClient;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Client {
    client: HttpClient,
    base: String,
    version: Version,
}

impl Client {
    pub fn new() -> Self {
        Client::default()
    }

    fn get_url(&self, path: &str, params: Option<HashMap<&str, String>>) -> http::Uri {
        if let Some(params) = params {
            let params = serde_urlencoded::to_string(params).unwrap_or_else(|_| String::from(""));
            let uri = format!(
                "{}/{}/{}?{}",
                self.base,
                self.version.as_str(),
                path,
                params
            );
            uri.parse::<Uri>().unwrap()
        } else {
            let uri = format!("{}/{}/{}", self.base, self.version.as_str(), path);
            uri.parse::<Uri>().unwrap()
        }
    }

    async fn get(&self, url: Uri, response_type: ResponseType) -> Result<Response> {
        let res = self
            .client
            .get_async(url)
            .await
            .context("Failed to get request")?;

        let mut body = res.into_body();
        let mut bytes = Vec::new();
        body.read_to_end(&mut bytes).await?;

        let response = response_type.deserialize(&bytes)?;

        Ok(response)
    }

    pub async fn get_current(&self, symbol: &str) -> Result<Current> {
        let url = self.get_url(&format!("stock/real-time-price/{}", symbol), None);
        let response_type = ResponseType::Current;

        let _response = self.get(url, response_type).await?;

        if let Response::Current(response) = _response {
            return Ok(response);
        }
        bail!("Failed to get current price for {}", symbol);
    }

    pub async fn get_historical_daily_from_to(
        &self,
        symbol: &str,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> Result<HistoricalDaily> {
        let from = from.format("%Y-%m-%d").to_string();
        let to = to.format("%Y-%m-%d").to_string();

        let mut params = HashMap::new();
        params.insert("from", from);
        params.insert("to", to);

        let url = self.get_url(&format!("historical-price-full/{}", symbol), Some(params));

        let response_type = ResponseType::HistoricalDaily;

        let _response = self.get(url, response_type).await?;

        if let Response::HistoricalDaily(response) = _response {
            return Ok(response);
        }
        bail!("Failed to get historical data for {}", symbol);
    }

    pub async fn get_historical_interval(
        &self,
        symbol: &str,
        interval: Interval,
    ) -> Result<HistoricalInterval> {
        let url = self.get_url(&format!("historical-chart/{}/{}", interval, symbol), None);

        let response_type = ResponseType::HistoricalInterval;

        let _response = self.get(url, response_type).await?;

        if let Response::HistoricalInterval(response) = _response {
            return Ok(response);
        }
        bail!("Failed to get historical data for {}", symbol);
    }

    pub async fn get_company_profile(&self, symbol: &str) -> Result<CompanyProfile> {
        let url = self.get_url(&format!("company/profile/{}", symbol), None);
        let response_type = ResponseType::Company;

        let _response = self.get(url, response_type).await?;

        if let Response::Company(response) = _response {
            return Ok(response.profile);
        }
        bail!("Failed to get company data for {}", symbol);
    }
}

impl Default for Client {
    fn default() -> Client {
        let client = HttpClient::new().unwrap();

        let base = String::from("https://financialmodelingprep.com/api");

        Client {
            client,
            base,
            version: Version::V3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Version {
    V3,
}

impl Version {
    fn as_str(&self) -> &'static str {
        match self {
            Version::V3 => "v3",
        }
    }
}
