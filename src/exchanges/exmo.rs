use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct Exmo {
    pairs: CurrencyPairList,
}

impl Exmo {
    pub fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}_{}", pair.quote, pair.base).to_uppercase()
        });

        Exmo { pairs }
    }
}

#[async_trait]
impl Exchange for Exmo {
    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let mut response: HashMap<String, TickersResponseItem> =
            reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for (symbol, item) in &self.pairs.items {
            let ticker = match response.get_mut(symbol) {
                Some(ticker) => ticker,
                None => continue,
            };

            result.insert(
                item.pair.to_string(),
                Ticker {
                    ask: ticker.sell_price.take(),
                    bid: ticker.buy_price.take(),
                    last: ticker.last_trade.take(),
                },
            );
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct TickersResponseItem {
    buy_price: Option<String>,
    sell_price: Option<String>,
    last_trade: Option<String>,
    high: Option<String>,
    low: Option<String>,
    avg: Option<String>,
    vol: Option<String>,
    vol_curr: Option<String>,
    updated: u64,
}

const TICKERS_ENDPOINT: &'static str = "https://api.exmo.com/v1/ticker/";
