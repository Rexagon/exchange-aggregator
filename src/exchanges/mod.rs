pub mod hit_btc;

pub use hit_btc::*;

use hashbrown::HashMap;

#[derive(Debug, Deserialize)]
pub struct Exchanges {
    pub hit_btc: Option<ExchangeSettings>,
    pub yobit: Option<ExchangeSettings>,
    pub live_coin: Option<ExchangeSettings>,
    pub exmo: Option<ExchangeSettings>,
    pub binance: Option<ExchangeSettings>,
    pub polonex: Option<ExchangeSettings>,
    pub gate_io: Option<ExchangeSettings>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeSettings {
    pub public_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Ticker {
    pub ask: Option<String>,
    pub bid: Option<String>,
    pub last: Option<String>,
}

#[async_trait]
pub trait Exchange {
    async fn request_tickers(&self) -> Result<HashMap<String, Ticker>, Box<dyn std::error::Error>>;
}

#[derive(Debug, Deserialize)]
pub struct CurrencyPair {
    pub base: String,
    pub quote: String,
}

impl ToString for CurrencyPair {
    fn to_string(&self) -> String {
        format!("{}_{}", self.quote, self.base)
    }
}

pub struct CurrencyPairList<'a>(Vec<(&'a CurrencyPair, String)>);

impl<'a> CurrencyPairList<'a> {
    pub fn new<F>(currency_pairs: &'a Vec<CurrencyPair>, symbol_predicate: F) -> Self
    where
        F: Fn(&CurrencyPair) -> String,
    {
        CurrencyPairList(
            currency_pairs
                .iter()
                .map(|pair| (pair, symbol_predicate(pair)))
                .collect(),
        )
    }

    pub fn find(&self, symbol: &String) -> Option<&CurrencyPair> {
        self.0.iter().find_map(|(pair, pair_symbol)| {
            if *pair_symbol == *symbol {
                Some(*pair)
            } else {
                None
            }
        })
    }
}
