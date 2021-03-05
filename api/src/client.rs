use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use futures::AsyncReadExt;
use http::Uri;
use isahc::HttpClient;
use serde::de::DeserializeOwned;

use crate::model::{Chart, ChartData, Company, CompanyData, Options, OptionsHeader};
use crate::{Interval, Range};

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
    ) -> Result<http::Uri> {
        if let Some(params) = params {
            let params = serde_urlencoded::to_string(params).unwrap_or_else(|_| String::from(""));
            let uri = format!("{}/{}/{}?{}", self.base, version.as_str(), path, params);
            Ok(uri.parse::<Uri>()?)
        } else {
            let uri = format!("{}/{}/{}", self.base, version.as_str(), path);
            Ok(uri.parse::<Uri>()?)
        }
    }

    async fn get<T: DeserializeOwned>(&self, url: Uri) -> Result<T> {
        let res = self
            .client
            .get_async(url)
            .await
            .context("Failed to get request")?;

        let mut body = res.into_body();
        let mut bytes = Vec::new();
        body.read_to_end(&mut bytes).await?;

        let response = serde_json::from_slice(&bytes)?;

        Ok(response)
    }

    pub async fn get_chart_data(
        &self,
        symbol: &str,
        interval: Interval,
        range: Range,
        include_pre_post: bool,
    ) -> Result<ChartData> {
        let mut params = HashMap::new();
        params.insert("interval", format!("{}", interval));
        params.insert("range", format!("{}", range));

        if include_pre_post {
            params.insert("includePrePost", format!("{}", true));
        }

        let url = self.get_url(
            Version::V8,
            &format!("finance/chart/{}", symbol),
            Some(params),
        )?;

        let response: Chart = self.get(url).await?;

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

        bail!("Failed to get chart data for {}", symbol);
    }

    pub async fn get_company_data(&self, symbol: &str) -> Result<CompanyData> {
        let mut params = HashMap::new();
        params.insert("modules", "price,assetProfile".to_string());

        let url = self.get_url(
            Version::V10,
            &format!("finance/quoteSummary/{}", symbol),
            Some(params),
        )?;

        let response: Company = self.get(url).await?;

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

        bail!("Failed to get company data for {}", symbol);
    }

    pub async fn get_options_expiration_dates(&self, symbol: &str) -> Result<Vec<i64>> {
        let url = self.get_url(Version::V7, &format!("finance/options/{}", symbol), None)?;

        let response: Options = self.get(url).await?;

        if let Some(err) = response.option_chain.error {
            bail!(
                "Error getting options data for {}: {}",
                symbol,
                err.description
            );
        }

        if let Some(mut result) = response.option_chain.result {
            if result.len() == 1 {
                let options_header = result.remove(0);
                return Ok(options_header.expiration_dates);
            }
        }

        bail!("Failed to get options data for {}", symbol);
    }

    pub async fn get_options_for_expiration_date(
        &self,
        symbol: &str,
        expiration_date: i64,
    ) -> Result<OptionsHeader> {
        let mut params = HashMap::new();
        params.insert("date", format!("{}", expiration_date));

        let url = self.get_url(
            Version::V7,
            &format!("finance/options/{}", symbol),
            Some(params),
        )?;

        let response: Options = self.get(url).await?;

        if let Some(err) = response.option_chain.error {
            bail!(
                "Error getting options data for {}: {}",
                symbol,
                err.description
            );
        }

        if let Some(mut result) = response.option_chain.result {
            if result.len() == 1 {
                let options_header = result.remove(0);

                return Ok(options_header);
            }
        }

        bail!("Failed to get options data for {}", symbol);
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
    V7,
    V8,
    V10,
}

impl Version {
    fn as_str(&self) -> &'static str {
        match self {
            Version::V7 => "v7",
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

        let symbols = vec!["SPY", "AAPL", "AMD", "TSLA", "ES=F", "BTC-USD", "DX-Y.NYB"];

        for symbol in symbols {
            let data = client.get_company_data(symbol).await;

            if let Err(e) = data {
                println!("{}", e);

                panic!();
            }
        }
    }

    #[async_std::test]
    async fn test_options_data() {
        let client = Client::new();

        let symbol = "SPY";

        let exp_dates = client.get_options_expiration_dates(symbol).await;

        match exp_dates {
            Err(e) => {
                println!("{}", e);

                panic!();
            }
            Ok(dates) => {
                for date in dates {
                    let options = client.get_options_for_expiration_date(symbol, date).await;

                    if let Err(e) = options {
                        println!("{}", e);

                        panic!();
                    }
                }
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

        let ticker = "SPY";

        for (idx, (range, interval)) in combinations.iter().enumerate() {
            let data = client.get_chart_data(ticker, *interval, *range, true).await;

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
