use hashbrown::HashMap;
use std::error::Error;
use std::pin::Pin;

use crate::exchanges::{CurrencyPairList, Exchange, Ticker};
use crate::Settings;

pub struct HitBtc<'a> {
    pairs: CurrencyPairList<'a>,
}

impl<'a> HitBtc<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}{}", pair.quote, pair.base).to_uppercase()
        });

        HitBtc { pairs }
    }
}

#[async_trait]
impl<'a> Exchange for HitBtc<'a> {
    async fn request_tickers(&self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let response: Vec<TickersResponseItem> =
            reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for mut ticker in response {
            let pair = match self.pairs.find(&ticker.symbol) {
                Some(pair) => pair,
                None => continue,
            };

            result.insert(
                ticker.symbol,
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
