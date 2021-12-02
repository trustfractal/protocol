use fractal_explorer::{indexing, ingested};
use postgres::{Client, NoTls};
use std::{
    sync::{atomic::*, mpsc::*, Arc},
    thread::*,
    time::*,
};
use structopt::StructOpt;

#[derive(StructOpt, Clone)]
struct Options {
    #[structopt(long)]
    postgres: String,

    #[structopt(long, default_value = "200")]
    caught_up_sleep_ms: u64,

    #[structopt(long, default_value = "2000")]
    no_blocks_wait_ms: u64,
}

impl Options {
    fn postgres(&self) -> anyhow::Result<Client> {
        Ok(self
            .postgres
            .parse::<postgres::Config>()?
            .ssl_mode(postgres::config::SslMode::Disable)
            .connect(NoTls)?)
    }
}

fn main() -> anyhow::Result<()> {
    ::simple_logger::init_with_level(log::Level::Info)?;

    let options = Options::from_args();
    create_status_table(&options)?;

    let indexers = indexing::indexers();

    let (error_tx, error_rx) = channel();
    let unpark = UnparkOnDrop::current();

    for (id, indexer) in indexers {
        let error_tx = error_tx.clone();
        let unpark = unpark.clone();
        let options = options.clone();
        Builder::new().name(id.to_string()).spawn(move || {
            match run_indexer(&id, indexer, options) {
                Err(e) => {
                    error_tx.send(e).unwrap();
                }
                Ok(_) => unreachable!(),
            }
            drop(unpark);
        })?;
    }

    unpark.park();

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
    fn current() -> Self {
        UnparkOnDrop {
            thread: std::thread::current(),
            spurious: Arc::new(AtomicBool::new(true)),
        }
    }

    fn park(&self) {
        while self.spurious.load(Ordering::SeqCst) {
            std::thread::park();
        }
    }
}

impl Drop for UnparkOnDrop {
    fn drop(&mut self) {
        self.spurious.store(false, Ordering::SeqCst);
        self.thread.unpark();
    }
}

enum Never {}

fn create_status_table(options: &Options) -> anyhow::Result<()> {
    options.postgres()?.execute(
        "
      CREATE TABLE IF NOT EXISTS
      indexing_status (
        id VARCHAR PRIMARY KEY,
        latest_block INT
      )
    ",
        &[],
    )?;

    Ok(())
}

fn run_indexer(
    id: &str,
    mut indexer: Box<dyn indexing::Indexer>,
    options: Options,
) -> anyhow::Result<Never> {
    let mut pg = options.postgres()?;
    indexer.begin(&mut pg)?;
    let mut log_every = Interval::new(Duration::from_secs(1));

    loop {
        let latest_ingested = match ingested::latest(&mut pg)? {
            Some(i) => i,
            None => {
                log::info!("No ingested blocks, waiting");
                sleep(Duration::from_millis(options.no_blocks_wait_ms));
                continue;
            }
        };
        let latest_for_indexer = latest_for_indexer(id, &mut pg)?;
        if latest_for_indexer == Some(latest_ingested) {
            sleep(Duration::from_millis(options.caught_up_sleep_ms));
            continue;
        }

        let next_block = latest_for_indexer.map(|x| x + 1).unwrap_or(0);

        for block_number in next_block..=latest_ingested {
            let block = ingested::load_block(block_number, &mut pg)?
                .expect("loaded block that doesn't exist");
            indexer.begin_block(&block, &mut pg)?;

            let mut index = 0;
            while let Some(extr) = ingested::load_extrinsic(block_number, index, &mut pg)? {
                indexer.visit_extrinsic(&extr, &mut pg)?;
                index += 1;
            }

            indexer.end_block(&block, &mut pg)?;
            save_latest_for_indexer(id, block_number, &mut pg)?;
            if log_every.is_time() {
                log::info!("Indexer '{}' finished block {}", id, block_number);
            }
        }
    }
}

struct Interval {
    every: Duration,
    last: Option<Instant>,
}

impl Interval {
    fn new(every: Duration) -> Self {
        Interval { every, last: None }
    }

    fn is_time(&mut self) -> bool {
        if let Some(last) = self.last {
            if last.elapsed() < self.every {
                return false;
            }
        }

        self.last = Some(Instant::now());
        true
    }
}

fn latest_for_indexer(id: &str, pg: &mut Client) -> anyhow::Result<Option<u64>> {
    let row = match pg
        .query(
            "SELECT latest_block FROM indexing_status WHERE id = $1",
            &[&id],
        )?
        .into_iter()
        .next()
    {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(row.get::<_, i32>(&"latest_block") as u64))
}

fn save_latest_for_indexer(id: &str, number: u64, pg: &mut Client) -> anyhow::Result<()> {
    pg.execute(
        "
        INSERT INTO indexing_status (id, latest_block) VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE SET latest_block = $2
    ",
        &[&id, &(number as i32)],
    )?;

    Ok(())
}
