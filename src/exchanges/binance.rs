use {futures::TryFutureExt, hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct Binance<'a> {
    pairs: CurrencyPairList<'a>,
}

impl<'a> Binance<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}{}", pair.quote, pair.base).to_uppercase()
        });

        Binance { pairs }
    }
}

#[async_trait]
impl<'a> Exchange for Binance<'a> {
    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let (orders_response, prices_response): (
            reqwest::Result<Vec<BookTickerItem>>,
            reqwest::Result<Vec<PriceTickerItem>>,
        ) = futures::future::join(
            reqwest::get(ORDERS_ENDPOINT)
                .and_then(|response| response.json::<Vec<BookTickerItem>>()),
            reqwest::get(PRICES_ENDPOINT)
                .and_then(|response| response.json::<Vec<PriceTickerItem>>()),
        )
        .await;

        let (orders_response, prices_response) = (orders_response?, prices_response?);

        let mut result = HashMap::new();

        for mut item in orders_response {
            let pair = match self.pairs.find(&item.symbol) {
                Some(pair) => pair,
                None => continue,
            };

            result.insert(
                pair.to_string(),
                Ticker {
                    ask: item.ask_price.take(),
                    bid: item.bid_price.take(),
                    last: None,
                },
            );
        }

        for mut item in prices_response {
            let pair = match self.pairs.find(&item.symbol) {
                Some(pair) => pair,
                None => continue,
            };

            if let Some(ticker) = result.get_mut(&pair.to_string()) {
                ticker.last = item.price.take();
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BookTickerItem {
    symbol: String,
    bid_price: Option<String>,
    bid_qty: Option<String>,
    ask_price: Option<String>,
    ask_qty: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriceTickerItem {
    symbol: String,
    price: Option<String>,
}

const ORDERS_ENDPOINT: &'static str = "https://api.binance.com/api/v3/ticker/bookTicker";
const PRICES_ENDPOINT: &'static str = "https://api.binance.com/api/v3/ticker/price";
