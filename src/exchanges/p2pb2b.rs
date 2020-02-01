use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct P2pb2b {
    pairs: CurrencyPairList,
}

#[async_trait]
impl Exchange for P2pb2b {
    fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}_{}", pair.quote, pair.base).to_uppercase()
        });

        P2pb2b { pairs }
    }

    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let mut response: ApiResponse = reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for (symbol, item) in &self.pairs.items {
            let response_item = match response.result.get_mut(symbol) {
                Some(response_item) => response_item,
                None => continue,
            };

            result.insert(
                item.pair.to_string(),
                Ticker {
                    ask: response_item.ticker.ask.take(),
                    bid: response_item.ticker.bid.take(),
                    last: response_item.ticker.last.take(),
                },
            );
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    success: bool,
    message: String,
    result: HashMap<String, ApiResponseItem>,
}

#[derive(Debug, Deserialize)]
struct ApiResponseItem {
    at: u64,
    ticker: CurrencyPairTicker,
}

#[derive(Debug, Deserialize)]
struct CurrencyPairTicker {
    bid: Option<String>,
    ask: Option<String>,
    low: Option<String>,
    high: Option<String>,
    last: Option<String>,
    vol: Option<String>,
}

const TICKERS_ENDPOINT: &'static str = "https://api.p2pb2b.io/api/v1/public/tickers";
