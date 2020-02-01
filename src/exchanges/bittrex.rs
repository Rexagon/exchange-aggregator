use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};

pub struct Bittrex {
    pairs: CurrencyPairList,
}

#[async_trait]
impl Exchange for Bittrex {
    fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("{}-{}", pair.base, pair.quote)
        });

        Bittrex { pairs }
    }

    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let response: ApiResponse = reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for ticker in response.result {
            let pair = match self.pairs.find(&ticker.market_name) {
                Some(pair) => pair,
                None => continue,
            };

            result.insert(
                pair.to_string(),
                Ticker {
                    ask: ticker.ask.map(|x| x.to_string()),
                    bid: ticker.bid.map(|x| x.to_string()),
                    last: ticker.last.map(|x| x.to_string()),
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
    result: Vec<TickersResponseItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TickersResponseItem {
    market_name: String,
    high: Option<f64>,
    low: Option<f64>,
    volume: Option<f64>,
    last: Option<f64>,
    base_volume: Option<f64>,
    time_stamp: String,
    bid: Option<f64>,
    ask: Option<f64>,
    open_buy_orders: u32,
    open_sell_orders: u32,
    prev_day: Option<f64>,
}

const TICKERS_ENDPOINT: &'static str = "https://api.bittrex.com/api/v1.1/public/getmarketsummaries";
