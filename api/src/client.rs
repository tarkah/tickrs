use crate::model::{Company, Current, Historical, HistoricalMinimal, Response, ResponseType};
use anyhow::{bail, Context, Result};
use futures::AsyncReadExt;
use http::Request;
use http::Uri;
use http_client::native::NativeClient;
use http_client::{Body, HttpClient};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Client {
    client: NativeClient,
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
        let request = Request::builder()
            .method("GET")
            .uri(url)
            .body(Body::empty())
            .unwrap();

        let res = self
            .client
            .send(request)
            .await
            .context("Failed to get request")?;

        let mut body = res.into_body();
        let mut bytes = Vec::new();
        body.read_to_end(&mut bytes).await?;

        let response = response_type.deserialize(&bytes)?;

        Ok(response)
    }

    pub async fn get_historical(&self, symbol: &str) -> Result<Historical> {
        let url = self.get_url(&format!("historical-price-full/{}", symbol), None);
        let response_type = ResponseType::Historical;

        let _response = self.get(url, response_type).await?;

        if let Response::Historical(response) = _response {
            return Ok(response);
        }
        bail!("Failed to get historical data for {}", symbol);
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

    pub async fn get_historical_minimal_from_to(
        &self,
        symbol: &str,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> Result<HistoricalMinimal> {
        let from = from.format("%Y-%m-%d").to_string();
        let to = to.format("%Y-%m-%d").to_string();

        let mut params = HashMap::new();
        params.insert("serietype", "line".to_owned());
        params.insert("from", from);
        params.insert("to", to);

        let url = self.get_url(&format!("historical-price-full/{}", symbol), Some(params));
        let response_type = ResponseType::HistoricalMinimal;

        let _response = self.get(url, response_type).await?;

        if let Response::HistoricalMinimal(response) = _response {
            return Ok(response);
        }
        bail!("Failed to get historical data for {}", symbol);
    }

    pub async fn get_historical_from_to(
        &self,
        symbol: &str,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> Result<Historical> {
        let from = from.format("%Y-%m-%d").to_string();
        let to = to.format("%Y-%m-%d").to_string();

        let mut params = HashMap::new();
        params.insert("from", from);
        params.insert("to", to);

        let url = self.get_url(&format!("historical-price-full/{}", symbol), Some(params));
        let response_type = ResponseType::Historical;

        let _response = self.get(url, response_type).await?;

        if let Response::Historical(response) = _response {
            return Ok(response);
        }
        bail!("Failed to get historical data for {}", symbol);
    }

    pub async fn get_company(&self, symbol: &str) -> Result<Company> {
        let url = self.get_url(&format!("company/profile/{}", symbol), None);
        let response_type = ResponseType::Company;

        let _response = self.get(url, response_type).await?;

        if let Response::Company(response) = _response {
            return Ok(response);
        }
        bail!("Failed to get company data for {}", symbol);
    }
}

impl Default for Client {
    fn default() -> Client {
        let client = NativeClient::new();

        #[cfg(not(test))]
        let base = String::from("https://financialmodelingprep.com/api");

        #[cfg(test)]
        let base = mockito::server_url();

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
