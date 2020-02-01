use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct Okex {
    pairs: CurrencyPairList,
}

#[async_trait]
impl Exchange for Okex {
    fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}-{}", pair.quote, pair.base).to_uppercase()
        });

        Okex { pairs }
    }

    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let response: Vec<TickersResponseItem> =
            reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for mut ticker in response {
            let pair = match self.pairs.find(&ticker.instrument_id) {
                Some(pair) => pair,
                None => continue,
            };

            result.insert(
                pair.to_string(),
                Ticker {
                    ask: ticker.best_ask.take(),
                    bid: ticker.best_bid.take(),
                    last: ticker.last.take(),
                },
            );
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct TickersResponseItem {
    instrument_id: String,
    last: Option<String>,
    last_qty: Option<String>,
    best_ask: Option<String>,
    best_bid: Option<String>,
    best_ask_size: Option<String>,
    best_bid_size: Option<String>,
    open_24h: Option<String>,
    high_24h: Option<String>,
    low_24h: Option<String>,
    base_volume_24h: Option<String>,
    quote_volume_24h: Option<String>,
    timestamp: String,
}

const TICKERS_ENDPOINT: &'static str = "https://www.okex.com/api/spot/v3/instruments/ticker";
