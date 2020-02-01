use {hashbrown::HashMap, std::error::Error};

use crate::{prelude::Ticker, Settings};

#[async_trait]
pub trait Exchange {
    fn new(settings: &Settings) -> Self
    where
        Self: Sized;

    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>>;
}
