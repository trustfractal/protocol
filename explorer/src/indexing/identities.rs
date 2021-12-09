use postgres::{fallible_iterator::FallibleIterator, types::ToSql};
use std::collections::BTreeMap;

use super::*;
use crate::data::Extrinsic;

#[derive(Default)]
pub struct CountIdentities {
    current_block: Option<u64>,
    previous_count: i32,
    this_count: i32,
}

impl Indexer for CountIdentities {
    fn version(&mut self) -> u32 {
        2
    }

    fn version_upgrade(&mut self, pg: &mut Client) -> anyhow::Result<()> {
        log::info!("Dropping identity_first_seen");
        pg.execute("DROP TABLE IF EXISTS identity_first_seen", &[])?;
        log::info!("Dropping unique_identity_counts");
        pg.execute("DROP TABLE IF EXISTS unique_identity_counts", &[])?;

        log::info!("Creating identity_first_seen");
        pg.execute(
            "
            CREATE TABLE IF NOT EXISTS
            identity_first_seen (
                id VARCHAR PRIMARY KEY,
                block INT NOT NULL
            )
        ",
            &[],
        )?;

        log::info!("Creating unique_identity_counts");
        pg.execute(
            "
            CREATE TABLE IF NOT EXISTS
            unique_identity_counts (
                block_number INT PRIMARY KEY,
                id_count INT NOT NULL
            )
        ",
            &[],
        )?;

        Ok(())
    }

    fn begin_block(&mut self, block: &Block, pg: &mut Client) -> anyhow::Result<()> {
        self.current_block = Some(block.number);
        self.previous_count = pg
            .query_opt(
                "SELECT id_count FROM unique_identity_counts WHERE block_number = $1",
                &[&(block.number as i32 - 1)],
            )?
            .map(|row| row.get(&"id_count"))
            .unwrap_or(0);
        self.this_count = 0;

        Ok(())
    }

    fn visit_extrinsic(&mut self, extrinsic: &Extrinsic, pg: &mut Client) -> anyhow::Result<()> {
        let relevant = extrinsic.section == "fractalMinting"
            && extrinsic.method == "registerIdentity"
            && extrinsic.success;
        if !relevant {
            return Ok(());
        }

        let id = extrinsic.args[0].as_str().expect("arg 0 is string");
        let block = self.current_block.unwrap() as i32;

        let already = pg.query_opt(
            "SELECT 1 FROM identity_first_seen WHERE id = $1 AND block < $2",
            &[&id, &block],
        )?;
        if already.is_none() {
            pg.execute(
                "INSERT INTO identity_first_seen (id, block) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET block = $2",
                &[&id, &block],
            )?;
            self.this_count += 1;
        }

        Ok(())
    }

    fn end_block(&mut self, block: &Block, pg: &mut Client) -> anyhow::Result<()> {
        pg.execute(
            "INSERT INTO unique_identity_counts (block_number, id_count)
            VALUES ($1, $2)
            ON CONFLICT (block_number) DO UPDATE SET id_count = $2",
            &[
                &(block.number as i32),
                &(self.previous_count + self.this_count),
            ],
        )?;

        Ok(())
    }
}

/// Returns map of block_number to id_count.
pub fn get_counts(
    data_points: usize,
    include_previous: impl IntoIterator<Item = u64>,
    pg: &mut Client,
) -> anyhow::Result<BTreeMap<u64, u64>> {
    let max_block = pg
        .query_one(
            "SELECT MAX(block_number) AS max FROM unique_identity_counts",
            &[],
        )?
        .get::<_, i32>("max");
    let size = max_block as usize / data_points;

    let include_blocks = include_previous
        .into_iter()
        .filter_map(|delta| max_block.checked_sub(delta as i32))
        .filter(|&block| block >= 0)
        .chain(std::iter::once(max_block))
        .collect::<Vec<_>>();

    let mut rows = pg.query_raw(
        "SELECT block_number, id_count
        FROM unique_identity_counts
        WHERE block_number % $1 = 0 OR
            block_number = ANY($2)",
        [&(size as i32) as &dyn ToSql, &include_blocks],
    )?;

    let mut map = BTreeMap::new();

    while let Some(row) = rows.next()? {
        map.insert(
            row.get::<_, i32>("block_number") as u64,
            row.get::<_, i32>("id_count") as u64,
        );
    }
    Ok(map)
}
