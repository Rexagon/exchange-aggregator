use {hashbrown::HashMap, std::error::Error};

use crate::prelude::Ticker;

#[async_trait]
pub trait Exchange {
    async fn request_tickers(&mut self) -> Result<HashMap<String, Ticker>, Box<dyn Error>>;
}
