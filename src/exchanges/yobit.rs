use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct Yobit {
    pairs: CurrencyPairList,
    endpoints: Vec<String>,
    current_endpoint: usize,
}

impl Yobit {
    pub fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}_{}", pair.quote, pair.base).to_lowercase()
        });

        let part_count = (pairs.items.len() + PAIRS_PER_REQUEST - 1) / PAIRS_PER_REQUEST;
        let mut endpoints = Vec::with_capacity(part_count);

        for part in 0..part_count {
            let mut parameters = pairs
                .items
                .iter()
                .skip(part * PAIRS_PER_REQUEST)
                .take(PAIRS_PER_REQUEST)
                .fold(String::new(), |acc, (symbol, _)| acc + symbol + "-");

            parameters.pop();

            let endpoint = format!(
                "{}/{}?{}",
                TICKER_BASE_ENDPOINT, parameters, ENDPOINT_PARAMETER
            );

            endpoints.push(endpoint);
        }

        Yobit {
            pairs,
            endpoints,
            current_endpoint: 0,
        }
    }
}

#[async_trait]
impl Exchange for Yobit {
    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let endpoint = &self.endpoints[self.current_endpoint];
        self.current_endpoint = (self.current_endpoint + 1) % self.endpoints.len();

        let mut response: HashMap<String, TickerResponseItem> =
            reqwest::get(endpoint).await?.json().await?;

        let mut result = HashMap::new();

        for (symbol, item) in &self.pairs.items {
            let ticker = match response.get_mut(symbol) {
                Some(ticker) => ticker,
                None => continue,
            };

            result.insert(
                item.pair.to_string(),
                Ticker {
                    ask: ticker.sell.map(|x| x.to_string()),
                    bid: ticker.buy.map(|x| x.to_string()),
                    last: ticker.last.map(|x| x.to_string()),
                },
            );
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct TickerResponseItem {
    high: Option<f64>,
    low: Option<f64>,
    avg: Option<f64>,
    vol: Option<f64>,
    vol_cur: Option<f64>,
    last: Option<f64>,
    buy: Option<f64>,
    sell: Option<f64>,
    updated: u64,
}

const PAIRS_PER_REQUEST: usize = 50;
const TICKER_BASE_ENDPOINT: &'static str = "https://yobit.net/api/3/ticker";
const ENDPOINT_PARAMETER: &'static str = "ignore_invalid=1";
