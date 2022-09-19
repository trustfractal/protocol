use actix_web::*;
use block_pool::Pool;

use crate::{retry_blocking, swap_chains::Swap};

pub async fn insert_swap(swap: Swap, pg: web::Data<Pool<postgres::Client>>) -> anyhow::Result<()> {
    retry_blocking(move || {
        let pg = &mut pg.take();

        let id = &swap.id;
        let json = serde_json::to_value(&swap)?;

        let do_insert = |pg: &mut postgres::Client| {
            pg.execute("INSERT INTO swaps (id, json) VALUES ($1, $2)", &[id, &json])
        };

        if let Err(e) = do_insert(pg) {
            if let Some(db_error) = e.as_db_error() {
                if db_error.message() == "relation \"swaps\" does not exist" {
                    pg.execute(
                        "CREATE TABLE swaps (
                            id TEXT PRIMARY KEY NOT NULL,
                            json JSON NOT NULL
                        )",
                        &[],
                    )?;
                    do_insert(pg)?;
                    return Ok(());
                }
            }

            anyhow::bail!(e);
        }

        Ok(())
    })
    .await
}

/// Run the provided function with the swap with the provided id (if found) in a transaction,
/// preventing multiple instances from performing side effects concurrently.
///
/// If no swap with the provided id is found, returns `None` immediately.
pub fn run_locked(
    pg: &mut postgres::Client,
    id: &str,
    f: impl FnOnce(&mut Swap) -> anyhow::Result<()>,
) -> anyhow::Result<Option<Swap>> {
    let mut txn = pg.transaction()?;

    let queried = txn.query_opt("SELECT json FROM swaps WHERE id = $1 FOR UPDATE", &[&id])?;
    let row = match queried {
        Some(r) => r,
        None => return Ok(None),
    };
    log::info!("Retrieved swap {}", id);
    let json = row.get("json");
    let mut swap: Swap = serde_json::from_value(json)?;
    let original_swap = swap.clone();

    // Don't return result immediately so we can commit or roll-back the transaction.
    let result = f(&mut swap);

    if swap == original_swap {
        log::info!("No changes for swap {}", id);
        txn.rollback()?;
    } else {
        log::info!("Updating swap {}", id);
        txn.execute(
            "UPDATE swaps SET json=$2 WHERE id=$1",
            &[&id, &serde_json::to_value(&swap)?],
        )?;
        txn.commit()?;
    }

    result.map(|_| Some(swap))
}
