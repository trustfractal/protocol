use actix_web::*;
use block_pool::Pool;
use std::sync::Arc;

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

pub async fn find_by_id(
    id: String,
    pg: web::Data<Pool<postgres::Client>>,
) -> anyhow::Result<Option<Swap>> {
    retry_blocking(move || {
        let pg = &mut pg.take();
        let queried = pg.query_opt("SELECT json FROM swaps WHERE id = $1", &[&id])?;
        let row = match queried {
            Some(r) => r,
            None => return Ok(None),
        };
        let json = row.get("json");
        let swap: Swap = serde_json::from_value(json)?;

        Ok(Some(swap))
    })
    .await
}

pub async fn update(swap: Swap, pg: web::Data<Pool<postgres::Client>>) -> anyhow::Result<Swap> {
    let swap = Arc::new(swap);

    let clone = swap.clone();
    retry_blocking(move || -> anyhow::Result<_> {
        let pg = &mut pg.take();

        let json = serde_json::to_value(&*clone)?;
        pg.execute("UPDATE swaps SET json=$2 WHERE id=$1", &[&clone.id, &json])?;

        Ok(())
    })
    .await?;

    Ok(Arc::try_unwrap(swap).unwrap())
}
