use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::*, Exchange, Settings};
use serde::export::TryFrom;
use std::convert::TryInto;

pub struct Bitfinex {
    pairs: CurrencyPairList,
}

#[async_trait]
impl Exchange for Bitfinex {
    fn new(settings: &Settings) -> Self {
        let pairs = CurrencyPairList::new(&settings.currency_pairs, |pair| {
            format!("t{}{}", pair.quote.to_uppercase(), pair.base.to_uppercase())
        });

        Bitfinex { pairs }
    }

    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>> {
        let response: Vec<Vec<ResponseItemComponent>> =
            reqwest::get(TICKERS_ENDPOINT).await?.json().await?;

        let mut result = HashMap::new();

        for data in response {
            if data.len() != COMPONENT_COUNT {
                continue;
            }

            let pair = match try_get_symbol(&data) {
                Ok(symbol) => match self.pairs.find(symbol) {
                    Some(pair) => pair,
                    None => continue,
                },
                _ => continue,
            };

            if let Ok(ticker) = data.try_into() {
                result.insert(pair.to_string(), ticker);
            }
        }

        Ok(result)
    }
}

impl TryFrom<Vec<ResponseItemComponent>> for Ticker {
    type Error = ();

    fn try_from(value: Vec<ResponseItemComponent>) -> Result<Self, Self::Error> {
        let ask = try_get_number(&value, ASK_COMPONENT)?;
        let bid = try_get_number(&value, BID_COMPONENT)?;
        let last = try_get_number(&value, LAST_PRICE_COMPONENT)?;

        Ok(Ticker {
            ask: ask.map(|x| x.to_string()),
            bid: bid.map(|x| x.to_string()),
            last: last.map(|x| x.to_string()),
        })
    }
}

fn try_get_symbol(value: &Vec<ResponseItemComponent>) -> Result<&String, ()> {
    match value[SYMBOL_COMPONENT] {
        ResponseItemComponent::Symbol(ref x) => Ok(x),
        _ => Err(()),
    }
}

fn try_get_number(value: &Vec<ResponseItemComponent>, component: usize) -> Result<Option<f64>, ()> {
    match value[component] {
        ResponseItemComponent::Number(x) => Ok(x),
        _ => Err(()),
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponseItemComponent {
    Symbol(String),
    Number(Option<f64>),
}

const TICKERS_ENDPOINT: &'static str = "https://api-pub.bitfinex.com/v2/tickers?symbols=ALL";

const SYMBOL_COMPONENT: usize = 0;
const BID_COMPONENT: usize = 1;
//const BID_SIZE_COMPONENT: usize = 2;
const ASK_COMPONENT: usize = 3;
//const ASK_SIZE_COMPONENT: usize = 4;
//const DAILY_CHANGE_COMPONENT: usize = 5;
//const DAILY_CHANGE_RELATIVE_COMPONENT: usize = 6;
const LAST_PRICE_COMPONENT: usize = 7;
//const VOLUME_COMPONENT: usize = 8;
//const HIGH_COMPONENT: usize = 9;
//const LOW_COMPONENT: usize = 10;
const COMPONENT_COUNT: usize = 11;
