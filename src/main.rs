#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate async_trait;

mod exchanges;

use {
    futures::{stream, StreamExt},
    hashbrown::HashMap,
    std::time::Duration,
    tokio::time::{self, Instant},
};

use crate::exchanges::{
    Binance, CurrencyPair, Exchange, ExchangeSettings, Exmo, HitBtc, LiveCoin, Ticker, Yobit,
};

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
pub struct Settings {
    pub currency_pairs: Vec<CurrencyPair>,
    pub exchanges: Exchanges,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("appsettings.json"))
        .expect("Unable to read appsettings.json");

    let settings = settings.try_into::<Settings>()?;

    let mut exchanges: Vec<(&str, Box<dyn Exchange>)> = vec![
        ("Binance", Box::new(Binance::new(&settings))),
        ("EXMO", Box::new(Exmo::new(&settings))),
        ("HitBTC", Box::new(HitBtc::new(&settings))),
        ("Livecoin", Box::new(LiveCoin::new(&settings))),
        ("YoBit", Box::new(Yobit::new(&settings))),
    ];
    let exchange_count = exchanges.len();

    loop {
        let now = Instant::now();
        println!("Tick");

        let tickers = stream::iter(exchanges.iter_mut().map(|(name, exchange)| {
            async move {
                let tickers = exchange.request_tickers().await;
                tickers.map(|data| (*name, data))
            }
        }))
        .fold(
            Vec::<(&str, HashMap<String, Ticker>)>::with_capacity(exchange_count),
            |mut result, fut| {
                async {
                    result.extend(fut.await);
                    result
                }
            },
        )
        .await;

        let mut result = HashMap::new();
        for (name, tickers) in tickers {
            result.insert(name, tickers);
        }

        println!("{:?}", result);

        time::delay_until(now + Duration::from_secs(1)).await
    }
}
