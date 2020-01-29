use {
    futures::{
        task::{Context, Poll},
        Future, FutureExt, Stream,
    },
    hashbrown::HashMap,
    std::{error::Error, pin::Pin, time::Duration},
    tokio::time::Instant,
};

use crate::{prelude::*, Exchange};

type TickersResult = Result<HashMap<String, Ticker>, Box<dyn Error>>;
type ExchangeFuture = dyn Future<Output = (&'static str, Box<dyn Exchange>, TickersResult)>;

pub struct Aggregator {
    futures: Vec<(Pin<Box<ExchangeFuture>>, bool)>,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            futures: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &'static str, mut exchange: Box<dyn Exchange>) {
        let future = Box::pin(async move {
            let tickers_result = exchange.request_tickers().await;
            (name, exchange, tickers_result)
        });

        self.futures.push((future, false));
    }
}

impl Stream for Aggregator {
    type Item = HashMap<&'static str, HashMap<String, Ticker>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let deadline = Instant::now() + Duration::from_millis(900);

        let mut result = HashMap::new();

        let this = self.get_mut();

        for (_, checked) in &mut this.futures {
            *checked = false;
        }

        while Instant::now() < deadline {
            for (future, checked) in &mut this.futures {
                if *checked {
                    continue;
                }

                let (name, tickers) = match future.poll_unpin(cx) {
                    Poll::Ready((name, mut exchange, tickers)) => {
                        *future = Box::pin(async move {
                            let tickers_result = exchange.request_tickers().await;
                            (name, exchange, tickers_result)
                        });

                        *checked = true;

                        (name, tickers)
                    }
                    Poll::Pending => continue,
                };

                if let Ok(tickers) = tickers {
                    result.insert(name, tickers);
                }
            }

            std::thread::sleep(Duration::from_millis(10));
        }

        Poll::Ready(Some(result))
    }
}
