use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use futures::AsyncReadExt;
use http::{header, Request, Uri};
use isahc::{AsyncReadResponseExt, HttpClient};
use serde::de::DeserializeOwned;

use crate::model::{Chart, ChartData, Company, CompanyData, CrumbData, Options, OptionsHeader};
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

    async fn get<T: DeserializeOwned>(&self, url: Uri, cookie: Option<String>) -> Result<T> {
        let mut req = Request::builder().method(http::Method::GET).uri(url);

        if let Some(cookie) = cookie {
            req = req.header(header::COOKIE, cookie);
        }

        let res = self
            .client
            .send_async(req.body(())?)
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

        let response: Chart = self.get(url, None).await?;

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

    pub async fn get_company_data(
        &self,
        symbol: &str,
        crumb_data: CrumbData,
    ) -> Result<CompanyData> {
        let mut params = HashMap::new();
        params.insert("modules", "price,assetProfile".to_string());
        params.insert("crumb", crumb_data.crumb);

        let url = self.get_url(
            Version::V10,
            &format!("finance/quoteSummary/{}", symbol),
            Some(params),
        )?;

        let response: Company = self.get(url, Some(crumb_data.cookie)).await?;

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

        let response: Options = self.get(url, None).await?;

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

        let response: Options = self.get(url, None).await?;

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

    pub async fn get_crumb(&self) -> Result<CrumbData> {
        let res = self
            .client
            .get_async("https://fc.yahoo.com")
            .await
            .context("Failed to get request")?;

        let Some(cookie) = res
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|header| header.to_str().ok())
            .and_then(|s| s.split_once(';').map(|(value, _)| value))
        else {
            bail!("Couldn't fetch cookie");
        };

        let request = Request::builder()
            .uri(self.get_url(Version::V1, "test/getcrumb", None)?)
            .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36")
            .header(header::COOKIE, cookie)
            .method(http::Method::GET)
            .body(())?;
        let mut res = self.client.send_async(request).await?;

        let crumb = res.text().await?;

        Ok(CrumbData {
            cookie: cookie.to_string(),
            crumb,
        })
    }
}

impl Default for Client {
    fn default() -> Client {
        #[allow(unused_mut)]
        let mut builder = HttpClient::builder();

        #[cfg(target_os = "android")]
        {
            use isahc::config::{Configurable, SslOption};

            builder = builder.ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS);
        }

        let client = builder.build().unwrap();

        let base = String::from("https://query1.finance.yahoo.com");

        Client { client, base }
    }
}

#[derive(Debug, Clone)]
pub enum Version {
    V1,
    V7,
    V8,
    V10,
}

impl Version {
    fn as_str(&self) -> &'static str {
        match self {
            Version::V1 => "v1",
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
            let data = client
                .get_company_data(
                    symbol,
                    CrumbData {
                        cookie: "".into(),
                        crumb: "".into(),
                    },
                )
                .await;

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
