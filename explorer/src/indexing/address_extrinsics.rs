use crate::data::*;

use postgres::{types::ToSql, Client};
use std::collections::HashMap;

#[derive(Default)]
pub struct AddressExtrinsics {
    current_block: Option<u64>,
    uncommitted: HashMap<String, Vec<(u64, u64)>>,
}

impl super::Indexer for AddressExtrinsics {
    fn version(&mut self) -> u32 {
        1
    }

    fn version_upgrade(&mut self, pg: &mut Client) -> anyhow::Result<()> {
        pg.execute(
            "CREATE TABLE IF NOT EXISTS
            address_extrinsics (
                block BIGINT NOT NULL,
                index BIGINT NOT NULL,
                address VARCHAR,
                PRIMARY KEY(block, index)
            )",
            &[],
        )?;
        pg.execute("TRUNCATE address_extrinsics", &[])?;
        pg.execute(
            "CREATE INDEX IF NOT EXISTS address_extrinsics_address
            ON address_extrinsics (address)",
            &[],
        )?;
        Ok(())
    }

    fn begin_block(&mut self, block: &Block, _: &mut Client) -> anyhow::Result<()> {
        self.current_block = Some(block.number);
        Ok(())
    }

    fn visit_extrinsic(&mut self, extr: &Extrinsic, _: &mut Client) -> anyhow::Result<()> {
        let entry = match &extr.signer {
            Some(s) => self
                .uncommitted
                .entry(s.to_string())
                .or_insert_with(Vec::new),
            None => return Ok(()),
        };

        entry.push((self.current_block.unwrap(), extr.index_in_block));

        Ok(())
    }

    fn end_block(&mut self, _: &Block, pg: &mut Client) -> anyhow::Result<()> {
        let stmt = pg.prepare(
            "INSERT INTO address_extrinsics (address, block, index) VALUES ($1, $2, $3)
            ON CONFLICT (block, index) DO UPDATE SET address = $1",
        )?;
        for (address, transactions) in self.uncommitted.drain() {
            for (block, index) in transactions {
                pg.execute(
                    &stmt,
                    &[
                        &address as &(dyn ToSql + Sync),
                        &(block as i64),
                        &(index as i64),
                    ],
                )?;
            }
        }
        Ok(())
    }
}

pub fn for_address(address: &str, pg: &mut Client) -> anyhow::Result<Vec<Extrinsic>> {
    let extrinsics = pg
        .query(
            "SELECT extrinsic_json.json FROM address_extrinsics
            JOIN extrinsic_json
                ON address_extrinsics.block = extrinsic_json.block_number
                AND address_extrinsics.index = extrinsic_json.index
            WHERE address = $1
            ORDER BY extrinsic_json.block_number DESC, extrinsic_json.index DESC",
            &[&address],
        )?
        .into_iter()
        .map(|row| serde_json::from_str(row.get(&"json")))
        .collect::<Result<_, _>>()?;
    Ok(extrinsics)
}
