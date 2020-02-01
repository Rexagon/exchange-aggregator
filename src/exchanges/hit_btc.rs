use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct HitBtc {
    pairs: CurrencyPairList,
}

#[async_trait]
impl Exchange for HitBtc {
    fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}{}", pair.quote, pair.base).to_uppercase()
        });

        HitBtc { pairs }
    }

    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let response: Vec<TickersResponseItem> =
            reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for mut ticker in response {
            let pair = match self.pairs.find(&ticker.symbol) {
                Some(pair) => pair,
                None => continue,
            };

            result.insert(
                pair.to_string(),
                Ticker {
                    ask: ticker.ask.take(),
                    bid: ticker.bid.take(),
                    last: ticker.last.take(),
                },
            );
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TickersResponseItem {
    symbol: String,
    ask: Option<String>,
    bid: Option<String>,
    open: Option<String>,
    last: Option<String>,
    low: Option<String>,
    high: Option<String>,
    volume: String,
    volume_quote: String,
    timestamp: String,
}

const TICKERS_ENDPOINT: &'static str = "https://api.hitbtc.com/api/2/public/ticker";
