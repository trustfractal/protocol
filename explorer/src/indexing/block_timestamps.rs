use crate::data::*;

use postgres::{types::ToSql, Client};
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime},
};

#[derive(Default)]
pub struct BlockTimestamps {
    current_block: Option<(u64, String)>,
    timestamp: Option<SystemTime>,
}

impl super::Indexer for BlockTimestamps {
    fn version(&mut self) -> u32 {
        1
    }

    fn version_upgrade(&mut self, pg: &mut Client) -> anyhow::Result<()> {
        pg.execute(
            "CREATE TABLE IF NOT EXISTS
            block_timestamps (
                number BIGINT PRIMARY KEY,
                hash VARCHAR NOT NULL,
                timestamp TIMESTAMP NOT NULL
            )",
            &[],
        )?;
        pg.execute("TRUNCATE block_timestamps", &[])?;
        pg.execute(
            "CREATE INDEX IF NOT EXISTS block_timestamps_hash
            ON block_timestamps (hash)",
            &[],
        )?;
        Ok(())
    }

    fn begin_block(&mut self, block: &Block, _: &mut Client) -> anyhow::Result<()> {
        self.current_block = Some((block.number, block.hash.clone()));
        self.timestamp = None;

        Ok(())
    }

    fn visit_extrinsic(&mut self, extr: &Extrinsic, _: &mut Client) -> anyhow::Result<()> {
        if extr.section == "timestamp" && extr.method == "set" {
            assert_eq!(self.timestamp, None);

            let secs_since_epoch = extr.args[0].as_i64().unwrap() / 1000;
            let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(secs_since_epoch as u64);
            self.timestamp = Some(timestamp);
        }

        Ok(())
    }

    fn end_block(&mut self, _: &Block, pg: &mut Client) -> anyhow::Result<()> {
        let (number, hash) = self.current_block.take().unwrap();

        if let Some(time) = self.timestamp {
            pg.execute(
                "INSERT INTO block_timestamps
                (number, hash, timestamp) VALUES ($1, $2, $3)
                ON CONFLICT (number) DO UPDATE SET hash = $2, timestamp = $3",
                &[&(number as i64) as &(dyn ToSql + Sync), &hash, &time],
            )?;
        } else {
            log::warn!("Block {} does not have a timestamp", number);
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum BlockId {
    Hash(String),
}

pub fn get(
    blocks: impl IntoIterator<Item = BlockId>,
    pg: &mut postgres::Client,
) -> anyhow::Result<HashMap<BlockId, SystemTime>> {
    let blocks = blocks.into_iter().collect::<Vec<_>>();

    let hashes = blocks
        .iter()
        .filter_map(|id| match id {
            BlockId::Hash(h) => Some(h),
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Ok(pg
        .query(
            "SELECT hash, timestamp FROM block_timestamps WHERE hash = ANY($1)",
            &[&hashes],
        )?
        .into_iter()
        .map(|row| {
            Ok((
                BlockId::Hash(row.get::<_, String>(&"hash")),
                row.get(&"timestamp"),
            ))
        })
        .collect::<anyhow::Result<_>>()?)
}
