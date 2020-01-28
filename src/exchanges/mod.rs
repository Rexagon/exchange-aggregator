pub mod hit_btc;
pub mod yobit;

pub use hit_btc::*;
pub use yobit::*;

use hashbrown::HashMap;

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
    async fn request_tickers(
        &mut self,
    ) -> Result<HashMap<String, Ticker>, Box<dyn std::error::Error>>;
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

pub struct CurrencyPairList<'a> {
    pub items: Vec<CurrencyPairListItem<'a>>,
}

impl<'a> CurrencyPairList<'a> {
    pub fn new<F>(currency_pairs: &'a Vec<CurrencyPair>, symbol_predicate: F) -> Self
    where
        F: Fn(&CurrencyPair) -> String,
    {
        let items = currency_pairs
            .iter()
            .map(|pair| CurrencyPairListItem {
                pair,
                symbol: symbol_predicate(pair),
                is_active: true,
            })
            .collect();

        CurrencyPairList { items }
    }

    pub fn find(&self, symbol: &String) -> Option<&CurrencyPair> {
        self.items.iter().find_map(|item| {
            if item.symbol == *symbol {
                Some(item.pair)
            } else {
                None
            }
        })
    }
}

pub struct CurrencyPairListItem<'a> {
    pub pair: &'a CurrencyPair,
    pub symbol: String,
    pub is_active: bool,
}
