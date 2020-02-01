#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate log;

mod aggregator;
mod exchange;
mod exchanges;
mod prelude;

pub use exchange::*;

use {
    hashbrown::HashMap,
    std::{error::Error, time::Duration},
    tokio::stream::StreamExt,
    tokio::time::{self, Instant},
};

use crate::{aggregator::Aggregator, exchanges::*, prelude::*};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub publish_endpoint: String,
    pub exchanges: Option<HashMap<String, bool>>,
    pub currency_pairs: Vec<CurrencyPair>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("settings.json"))
        .expect("Unable to read settings.json");

    let settings = settings.try_into::<Settings>()?;

    let mut aggregator = Aggregator::new();
    aggregator.try_create::<Binance>("Binance", &settings);
    aggregator.try_create::<Bittrex>("Bittrex", &settings);
    aggregator.try_create::<Exmo>("EXMO", &settings);
    aggregator.try_create::<GateIo>("gate.io", &settings);
    aggregator.try_create::<HitBtc>("HitBTC", &settings);
    aggregator.try_create::<LiveCoin>("Livecoin", &settings);
    aggregator.try_create::<Okex>("OKEx", &settings);
    aggregator.try_create::<P2pb2b>("p2pb2b", &settings);
    aggregator.try_create::<Polonex>("Polonex", &settings);
    aggregator.try_create::<Yobit>("YoBit", &settings);

    loop {
        let now = Instant::now();
        info!("Tick");

        let result = aggregator.next().await.unwrap();
        info!("{:?}", result);

        let client = reqwest::Client::new();
        let response = client
            .post(&settings.publish_endpoint)
            .json(&result)
            .send()
            .await;

        if let Err(response) = response {
            error!("Unable to publish: {:?}", response);
        }

        time::delay_until(now + Duration::from_millis(1000)).await
    }
}
