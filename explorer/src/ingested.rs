use postgres::{fallible_iterator::FallibleIterator, types::ToSql, Client};
use shared_lru::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::data::{Block, Extrinsic};

fn get_key(key: impl AsRef<str>, pg: &mut Client) -> anyhow::Result<Option<String>> {
    Ok(pg
        .query_opt(
            "SELECT value FROM key_values WHERE key = $1",
            &[&key.as_ref()],
        )?
        .map(|row| row.get::<_, &str>(&"value").to_string()))
}

pub struct Ingested {
    blocks: LruCache<u64, Block>,
    extrinsics: LruCache<(u64, u64), Option<Extrinsic>>,
    client: Mutex<Client>,
}

impl Ingested {
    pub fn create(lru: &Arc<SharedLru>, client: Client) -> Ingested {
        Ingested {
            blocks: lru.make_cache(),
            extrinsics: lru.make_cache(),
            client: Mutex::new(client),
        }
    }

    pub fn latest(&self, pg: &mut Client) -> anyhow::Result<Option<u64>> {
        Ok(get_key("ingestion/fully_ingested", pg)?
            .map(|v| v.parse())
            .transpose()?)
    }

    pub fn load_block(&self, number: u64) -> anyhow::Result<Option<Block>> {
        match self.blocks.get(&number) {
            Some(block) => Ok(Some(block.clone())),
            None => {
                log::debug!("Loading block {}", number);
                Ok(self.load(number)?.map(|(block, _extrinsics)| block))
            }
        }
    }

    fn load(&self, load_block: u64) -> anyhow::Result<Option<(Block, HashMap<u64, Extrinsic>)>> {
        let mut client = self.client.lock().unwrap();
        let mut limit: i64 = 10_000;

        loop {
            // TODO(shelbyd): Could merge these queries to not waste time processing unnecessary
            // blocks.
            let mut blocks = client.query_raw(
                "SELECT number, json FROM block_json WHERE number >= $1
                ORDER BY number
                LIMIT $2",
                [&(load_block as i64), &limit],
            )?;

            let mut result: Option<(Block, HashMap<u64, Extrinsic>)> = None;

            while let Some(block_row) = blocks.next()? {
                let block: Block = serde_json::from_str(block_row.get(&"json"))?;
                let number = block_row.get::<_, i64>(&"number") as u64;

                if number == load_block {
                    result = Some((block.clone(), HashMap::new()));
                }

                self.blocks.insert(number, block);
            }
            drop(blocks);

            let mut extrinsics = client.query_raw(
                "SELECT block_number, index, json FROM extrinsic_json WHERE block_number >= $1
                ORDER BY block_number, index
                LIMIT $2",
                [&(load_block as i64) as &dyn ToSql, &limit],
            )?;

            let mut largest_unseen_extrinsic = HashMap::<u64, u64>::new();
            let mut extrinsic_count = 0;
            while let Some(extr_row) = extrinsics.next()? {
                extrinsic_count += 1;
                let extrinsic: Extrinsic = serde_json::from_str(extr_row.get(&"json"))?;

                let block_number = extr_row.get::<_, i64>(&"block_number") as u64;
                let index = extr_row.get::<_, i64>(&"index") as u64;

                if block_number == load_block {
                    result
                        .as_mut()
                        .expect("should have visited block")
                        .1
                        .insert(index, extrinsic.clone());
                }

                self.extrinsics
                    .insert((block_number, index), Some(extrinsic));

                let unseen = largest_unseen_extrinsic.entry(block_number).or_default();
                *unseen = std::cmp::max(index + 1, *unseen);
            }

            let fewer_extrinsics_than_limit = extrinsic_count < limit;

            for (block, unseen) in &largest_unseen_extrinsic {
                if largest_unseen_extrinsic.contains_key(&(block + 1))
                    || fewer_extrinsics_than_limit
                {
                    self.extrinsics.insert((*block, *unseen), None);
                }
            }

            let seen_extrinsics_for_next_block =
                largest_unseen_extrinsic.keys().any(|&k| k > load_block);
            if seen_extrinsics_for_next_block || fewer_extrinsics_than_limit {
                return Ok(result);
            }

            // There may be more extrinsics to return. Double the limit and try again.
            // This is unlikely to be hit.
            limit *= 2;
            log::info!("Did not load enough extrinsics, now loading {}", limit);
        }
    }

    pub fn load_extrinsic(&self, block: u64, index: u64) -> anyhow::Result<Option<Extrinsic>> {
        if let Some(extr) = self.extrinsics.get(&(block, index)) {
            return Ok(extr.clone());
        }
        log::debug!("Loading extrinsic {}/{}", block, index);
        Ok(self
            .load(block)?
            .and_then(|(_block, mut extrs)| extrs.remove(&index)))
    }
}
