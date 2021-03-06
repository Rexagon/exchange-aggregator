use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct LiveCoin {
    pairs: CurrencyPairList,
}

#[async_trait]
impl Exchange for LiveCoin {
    fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}/{}", pair.quote, pair.base).to_uppercase()
        });

        LiveCoin { pairs }
    }

    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let response: Vec<TickersResponseItem> =
            reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for ticker in response {
            let pair = match self.pairs.find(&ticker.symbol) {
                Some(pair) => pair,
                None => continue,
            };

            result.insert(
                pair.to_string(),
                Ticker {
                    ask: ticker.best_ask.map(|x| x.to_string()),
                    bid: ticker.best_bid.map(|x| x.to_string()),
                    last: ticker.last.map(|x| x.to_string()),
                },
            );
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct TickersResponseItem {
    cur: String,
    symbol: String,
    last: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    volume: Option<f64>,
    vwap: Option<f64>,
    max_bid: Option<f64>,
    min_ask: Option<f64>,
    best_bid: Option<f64>,
    best_ask: Option<f64>,
}

const TICKERS_ENDPOINT: &'static str = "https://api.livecoin.net/exchange/ticker";
