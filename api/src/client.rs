use crate::{
    model::{ChartData, CompanyData, Response, ResponseType},
    Interval, Range,
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
}

impl Client {
    pub fn new() -> Self {
        Client::default()
    }

    fn get_url(
        &self,
        version: Version,
        path: &str,
        params: Option<HashMap<&str, String>>,
    ) -> http::Uri {
        if let Some(params) = params {
            let params = serde_urlencoded::to_string(params).unwrap_or_else(|_| String::from(""));
            let uri = format!("{}/{}/{}?{}", self.base, version.as_str(), path, params);
            uri.parse::<Uri>().unwrap()
        } else {
            let uri = format!("{}/{}/{}", self.base, version.as_str(), path);
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

    pub async fn get_chart_data(
        &self,
        symbol: &str,
        interval: Interval,
        range: Range,
    ) -> Result<ChartData> {
        let mut params = HashMap::new();
        params.insert("interval", format!("{}", interval));
        params.insert("range", format!("{}", range));

        let url = self.get_url(
            Version::V8,
            &format!("finance/chart/{}", symbol),
            Some(params),
        );

        let response_type = ResponseType::Chart;

        let mut _response = self.get(url, response_type).await?;

        if let Response::Chart(response) = _response {
            if let Some(err) = response.chart.error {
                bail!(
                    "Error getting chart data for {}: {}",
                    symbol,
                    err.description
                );
            }

            if let Some(mut result) = response.chart.result {
                if result.len() == 1 {
                    return Ok(result.remove(0));
                }
            }
        }
        bail!("Failed to get chart data for {}", symbol);
    }

    pub async fn get_company_data(&self, symbol: &str) -> Result<CompanyData> {
        let mut params = HashMap::new();
        params.insert("modules", "price,assetProfile".to_string());

        let url = self.get_url(
            Version::V10,
            &format!("finance/quoteSummary/{}", symbol),
            Some(params),
        );
        let response_type = ResponseType::Company;

        let mut _response = self.get(url, response_type).await?;

        if let Response::Company(response) = _response {
            if let Some(err) = response.company.error {
                bail!(
                    "Error getting company data for {}: {}",
                    symbol,
                    err.description
                );
            }

            if let Some(mut result) = response.company.result {
                if result.len() == 1 {
                    return Ok(result.remove(0));
                }
            }
        }
        bail!("Failed to get company data for {}", symbol);
    }
}

impl Default for Client {
    fn default() -> Client {
        let client = HttpClient::new().unwrap();

        let base = String::from("https://query1.finance.yahoo.com");

        Client { client, base }
    }
}

#[derive(Debug, Clone)]
pub enum Version {
    V8,
    V10,
}

impl Version {
    fn as_str(&self) -> &'static str {
        match self {
            Version::V8 => "v8",
            Version::V10 => "v10",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_company_data() {
        let client = Client::new();

        let symbols = vec!["SPY", "AAPL", "AMD", "TSLA"];

        for symbol in symbols {
            let data = client.get_company_data(symbol).await;

            if let Err(e) = data {
                println!("{}", e);

                panic!();
            }
        }
    }

    #[async_std::test]
    async fn test_chart_data() {
        let client = Client::new();

        let combinations = vec![
            (Range::Year5, Interval::Minute1),
            (Range::Day1, Interval::Minute1),
            (Range::Day5, Interval::Minute5),
            (Range::Month1, Interval::Minute30),
            (Range::Month3, Interval::Minute60),
            (Range::Month6, Interval::Minute60),
            (Range::Year1, Interval::Day1),
            (Range::Year5, Interval::Day1),
        ];

        let ticker = "ATVI";

        for (idx, (range, interval)) in combinations.iter().enumerate() {
            let data = client.get_chart_data(ticker, *interval, *range).await;

            if let Err(e) = data {
                println!("{}", e);

                if idx > 0 {
                    panic!();
                }
            } else if idx == 0 {
                panic!();
            }
        }
    }
}
