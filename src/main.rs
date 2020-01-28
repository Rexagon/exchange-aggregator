#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate async_trait;

mod exchanges;

use std::collections::HashMap;
use std::time::Duration;
use tokio::prelude::*;
use tokio::time::{self, Instant};

use crate::exchanges::{CurrencyPair, Exchange, Exchanges, HitBtc};

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

    let hit_btc = HitBtc::new(&settings);

    loop {
        let now = Instant::now();
        println!("Tick");

        let result = hit_btc.request_tickers().await;
        println!("HitBtc: {:?}", result);

        time::delay_until(now + Duration::from_secs(1)).await
    }
}
