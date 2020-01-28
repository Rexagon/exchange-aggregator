use {
    futures::{stream, StreamExt},
    hashbrown::HashMap,
    std::error::Error,
};

use crate::exchanges::CurrencyPair;
use crate::{
    exchanges::{Exchange, Ticker},
    Settings,
};

pub struct Yobit<'a> {
    endpoints: Vec<Endpoint<'a>>,
}

impl<'a> Yobit<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        let endpoints = settings.currency_pairs.iter().map(Endpoint::new).collect();

        Yobit { endpoints }
    }
}

#[async_trait]
impl<'a> Exchange for Yobit<'a> {
    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let responses: Vec<reqwest::Result<TickerResponse>> = stream::iter(
            self.endpoints
                .iter()
                .filter(|endpoint| endpoint.is_active)
                .map(|endpoint| {
                    async move {
                        reqwest::get(&endpoint.url)
                            .await?
                            .json::<TickerResponse>()
                            .await
                    }
                }),
        )
        .fold(
            Vec::with_capacity(self.endpoints.len()),
            |mut responses, fut| {
                async {
                    responses.push(fut.await);
                    responses
                }
            },
        )
        .await;

        let mut result = HashMap::new();

        for (response, endpoint) in responses.iter().zip(self.endpoints.iter_mut()) {
            let response = match response {
                Ok(response) => response,
                Err(_) => {
                    endpoint.is_active = false;
                    continue;
                }
            };

            match response {
                TickerResponse::Success { ticker } => {
                    result.insert(
                        endpoint.currency_pair.to_string(),
                        Ticker {
                            ask: ticker.buy.map(|x| x.to_string()),
                            bid: ticker.sell.map(|x| x.to_string()),
                            last: ticker.last.map(|x| x.to_string()),
                        },
                    );
                }
                _ => {
                    endpoint.is_active = false;
                }
            }
        }

        Ok(result)
    }
}

struct Endpoint<'a> {
    currency_pair: &'a CurrencyPair,
    url: String,
    is_active: bool,
}

impl<'a> Endpoint<'a> {
    fn new(currency_pair: &'a CurrencyPair) -> Self {
        let symbol = format!("{}_{}", currency_pair.quote, currency_pair.base).to_lowercase();

        Endpoint {
            currency_pair,
            url: format!("{}/{}/ticker", TICKER_BASE_ENDPOINT, symbol),
            is_active: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TickerResponse {
    Success { ticker: YobitTicker },
    Error { error: String },
}

#[derive(Debug, Deserialize)]
struct YobitTicker {
    high: Option<f64>,
    low: Option<f64>,
    avg: Option<f64>,
    vol: Option<f64>,
    vol_cur: Option<f64>,
    last: Option<f64>,
    buy: Option<f64>,
    sell: Option<f64>,
    updated: u64,
    server_time: u64,
}

const TICKER_BASE_ENDPOINT: &'static str = "https://yobit.net/api/2";
