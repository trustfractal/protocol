use fractal_explorer::indexing;
use std::{
    sync::{atomic::*, mpsc::*, Arc},
    thread::*,
};

fn main() -> anyhow::Result<()> {
    let indexers = indexing::indexers();

    let (error_tx, error_rx) = channel();
    let unpark = UnparkOnDrop {
        thread: current(),
        spurious: Arc::new(AtomicBool::new(true)),
    };

    for (id, indexer) in indexers {
        let error_tx = error_tx.clone();
        let unpark = unpark.clone();
        Builder::new().name(id.to_string()).spawn(move || {
            match run_indexer(&id, indexer) {
                Err(e) => {
                    error_tx.send(e).unwrap();
                }
                Ok(_) => unreachable!(),
            }
            drop(unpark);
        })?;
    }

    while unpark.spurious() {
        park();
    }

    match error_rx.try_recv() {
        Ok(e) => Err(e),
        Err(_) => anyhow::bail!("A thread panicked"),
    }
}

#[derive(Clone)]
struct UnparkOnDrop {
    thread: Thread,
    spurious: Arc<AtomicBool>,
}

impl UnparkOnDrop {
    fn spurious(&self) -> bool {
        self.spurious.load(Ordering::SeqCst)
    }
}

impl Drop for UnparkOnDrop {
    fn drop(&mut self) {
        self.spurious.store(false, Ordering::SeqCst);
        self.thread.unpark();
    }
}

enum Never {}

fn run_indexer(id: &str, indexer: Box<dyn indexing::Indexer>) -> anyhow::Result<Never> {
    unimplemented!("run_indexer");
}
