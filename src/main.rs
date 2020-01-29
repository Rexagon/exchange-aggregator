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

pub use aggregator::*;
pub use exchange::*;

use {
    std::{error::Error, time::Duration},
    tokio::stream::StreamExt,
    tokio::time::{self, Instant},
};

use crate::{exchanges::*, prelude::*, Aggregator};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub currency_pairs: Vec<CurrencyPair>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_LOG", "info");

    env_logger::init();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("appsettings.json"))
        .expect("Unable to read appsettings.json");

    let settings = settings.try_into::<Settings>()?;

    let mut aggregator = Aggregator::new();
    aggregator.add("Binance", Box::new(Binance::new(&settings)));
    aggregator.add("EXMO", Box::new(Exmo::new(&settings)));
    aggregator.add("gate.io", Box::new(GateIo::new(&settings)));
    aggregator.add("HitBTC", Box::new(HitBtc::new(&settings)));
    aggregator.add("Livecoin", Box::new(LiveCoin::new(&settings)));
    aggregator.add("Polonex", Box::new(Polonex::new(&settings)));
    aggregator.add("YoBit", Box::new(Yobit::new(&settings)));

    loop {
        let now = Instant::now();
        info!("Tick");

        let result = aggregator.next().await.unwrap();
        info!("{:?}", result);

        time::delay_until(now + Duration::from_millis(1000)).await
    }
}
