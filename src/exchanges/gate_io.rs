use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct GateIo {
    pairs: CurrencyPairList,
}

impl<'a> GateIo {
    pub fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}_{}", pair.quote, pair.base).to_lowercase()
        });

        GateIo { pairs }
    }
}

#[async_trait]
impl Exchange for GateIo {
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
                    ask: ticker.lowest_ask.take(),
                    bid: ticker.highest_bid.take(),
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
    result: String,
    last: Option<String>,
    lowest_ask: Option<String>,
    highest_bid: Option<String>,
    percent_change: Option<String>,
    base_volume: Option<String>,
    quote_volume: Option<String>,
    high_24hr: Option<String>,
    low_24hr: Option<String>,
}

const TICKERS_ENDPOINT: &'static str = "https://data.gateio.life/api2/1/tickers";
