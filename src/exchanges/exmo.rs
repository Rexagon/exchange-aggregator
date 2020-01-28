use {hashbrown::HashMap, std::error::Error};

use crate::{
    exchanges::{CurrencyPairList, Exchange, Ticker},
    Settings,
};

pub struct Exmo<'a> {
    pairs: CurrencyPairList<'a>,
}

impl<'a> Exmo<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}_{}", pair.quote, pair.base).to_uppercase()
        });

        Exmo { pairs }
    }
}

#[async_trait]
impl<'a> Exchange for Exmo<'a> {
    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let mut response: HashMap<String, TickersResponseItem> =
            reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for item in &self.pairs.items {
            let ticker = match response.get_mut(&item.symbol) {
                Some(ticker) => ticker,
                None => continue,
            };

            result.insert(
                item.pair.to_string(),
                Ticker {
                    ask: ticker.buy_price.take(),
                    bid: ticker.sell_price.take(),
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
