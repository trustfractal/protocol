pub mod data;
pub mod indexing;
pub mod ingested;
pub mod pages;
pub mod postgres;
pub mod swap_chains;

use actix_web::{error::*, *};

pub async fn retry_blocking<R, E, F>(f: F) -> Result<R, E>
where
    F: FnMut() -> Result<R, E> + Send + 'static,
    R: Send + 'static,
    E: Send + core::fmt::Debug + 'static,
{
    let mut tries = 3;
    let f = std::sync::Arc::new(std::sync::Mutex::new(f));
    loop {
        let f = std::sync::Arc::clone(&f);
        let result = web::block(move || {
            let mut f = f.lock().unwrap();
            f()
        })
        .await;

        match result {
            Ok(v) => return Ok(v),
            Err(BlockingError::Error(e)) => return Err(e),
            Err(BlockingError::Canceled) => {
                tries -= 1;
                if tries == 0 {
                    panic!("Blocking call cancelled.");
                } else {
                    log::warn!("Blocking call cancelled.");
                }
            }
        }
    }
}

pub fn block_on<F: core::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
