use fractal_explorer::{indexing, ingested};
use postgres::Client;
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

    // Default 200MB
    #[structopt(long, default_value = "200000000")]
    cache_byte_limit: usize,
}

impl Options {
    fn postgres(&self) -> anyhow::Result<Client> {
        fractal_explorer::postgres::connect(&self.postgres)
    }
}

fn main() -> anyhow::Result<()> {
    ::simple_logger::init_with_level(log::Level::Info)?;

    let options = Options::from_args();
    create_status_table(&options)?;

    let indexers = indexing::indexers();

    let lru_storage = shared_lru::SharedLru::with_byte_limit(options.cache_byte_limit);
    let ingested = Arc::new(ingested::Ingested::create(
        &lru_storage,
        options.postgres()?,
    ));

    let (error_tx, error_rx) = channel();
    let unpark = UnparkOnDrop::current();

    for (id, indexer) in indexers {
        let error_tx = error_tx.clone();
        let unpark = unpark.clone();
        let options = options.clone();
        let ingested = Arc::clone(&ingested);
        Builder::new().name(id.to_string()).spawn(move || {
            match run_indexer(&id, indexer, &ingested, options) {
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
    let mut pg = options.postgres()?;
    pg.execute(
        "
          CREATE TABLE IF NOT EXISTS
          indexing_status (
            id VARCHAR PRIMARY KEY,
            latest_block INT,
            version INT NOT NULL
          )
        ",
        &[],
    )?;
    pg.execute(
        "
        DO $$
        BEGIN
          IF EXISTS(SELECT *
            FROM information_schema.columns
            WHERE table_name='indexing_status' and column_name='storage_version')
          THEN
              ALTER TABLE indexing_status RENAME COLUMN storage_version TO version;
          END IF;
        END $$;
        ",
        &[],
    )?;

    Ok(())
}

fn run_indexer(
    id: &str,
    mut indexer: Box<dyn indexing::Indexer>,
    ingested: &ingested::Ingested,
    options: Options,
) -> anyhow::Result<Never> {
    let mut pg = options.postgres()?;

    let version = get_version(id, &mut pg)?;
    if version != Some(indexer.version()) {
        log::info!(
            "Transitioning '{id}' from {version:?} to {new_version}",
            id = id,
            version = version,
            new_version = indexer.version()
        );
        indexer.version_upgrade(&mut pg)?;
        save_version(id, indexer.version(), &mut pg)?;
        save_latest_block_number(id, None, &mut pg)?;
    }
    log::info!("Starting '{}'", id);

    indexer.begin(&mut pg)?;
    let mut one_second = Interval::new(Duration::from_secs(1));

    loop {
        let latest_ingested = match ingested.latest(&mut pg)? {
            Some(i) => i,
            None => {
                log::info!("No ingested blocks, waiting");
                sleep(Duration::from_millis(options.no_blocks_wait_ms));
                log::info!("Done waiting");
                continue;
            }
        };
        let latest_block_number = latest_block_number(id, &mut pg)?;
        if let Some(latest_for_indexer) = latest_block_number {
            use std::cmp::Ordering;
            match latest_for_indexer.cmp(&latest_ingested) {
                Ordering::Equal => {
                    sleep(Duration::from_millis(options.caught_up_sleep_ms));
                    continue;
                }
                Ordering::Greater => {
                    // Ingestion restarted, we should reindex.
                    save_latest_block_number(id, None, &mut pg)?;
                    continue;
                }
                Ordering::Less => {}
            }
        }

        let next_block = latest_block_number.map(|b| b + 1).unwrap_or(0);

        for block_number in next_block..=latest_ingested {
            let block = ingested
                .load_block(block_number)?
                .expect("loaded block that doesn't exist");

            indexer.begin_block(&block, &mut pg)?;

            let mut index = 0;
            while let Some(extr) = ingested.load_extrinsic(block_number, index)? {
                index += 1;
                indexer.visit_extrinsic(&extr, &mut pg)?;
            }

            indexer.end_block(&block, &mut pg)?;

            if one_second.is_time() {
                indexer.commit(&mut pg)?;
                save_latest_block_number(id, Some(block_number), &mut pg)?;

                log::info!("Indexer '{}' finished block {}", id, block_number);
                break;
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
    #[allow(clippy::wrong_self_convention)]
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

fn latest_block_number(id: &str, pg: &mut Client) -> anyhow::Result<Option<u64>> {
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

    let number = row.get::<_, Option<i32>>(&"latest_block").map(|n| n as u64);
    Ok(number)
}

fn save_latest_block_number(id: &str, number: Option<u64>, pg: &mut Client) -> anyhow::Result<()> {
    pg.execute(
        "
        INSERT INTO indexing_status (id, latest_block) VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE SET latest_block = $2
    ",
        &[&id, &number.map(|n| n as i32)],
    )?;

    Ok(())
}

fn get_version(id: &str, pg: &mut Client) -> anyhow::Result<Option<u32>> {
    let row = match pg
        .query("SELECT version FROM indexing_status WHERE id = $1", &[&id])?
        .into_iter()
        .next()
    {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(row.get::<_, i32>(&"version") as u32))
}

fn save_version(id: &str, version: u32, pg: &mut Client) -> anyhow::Result<()> {
    pg.execute(
        "
        INSERT INTO indexing_status (id, version) VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE SET version = $2
    ",
        &[&id, &(version as i32)],
    )?;

    Ok(())
}
