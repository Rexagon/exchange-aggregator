#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate async_trait;

mod exchanges;

use std::time::Duration;
use tokio::time::{self, Instant};

use crate::exchanges::{CurrencyPair, Exchange, ExchangeSettings, HitBtc, LiveCoin, Yobit};

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

    let mut hit_btc = HitBtc::new(&settings);
    let mut live_coin = LiveCoin::new(&settings);
    let mut yobit: Yobit = Yobit::new(&settings);

    loop {
        let now = Instant::now();
        println!("Tick");

        let result = hit_btc.request_tickers().await;
        println!("HitBtc: {:?}", result);

        let result = live_coin.request_tickers().await;
        println!("LiveCoin: {:?}", result);

        let result = yobit.request_tickers().await;
        println!("Yobit: {:?}", result);

        time::delay_until(now + Duration::from_secs(1)).await
    }
}
