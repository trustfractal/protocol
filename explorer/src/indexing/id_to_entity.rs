use crate::data::*;
use postgres::Client;
use serde::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct IdToEntity {
    uncommitted: HashMap<String, EntityId>,
}

impl super::Indexer for IdToEntity {
    fn version(&mut self) -> u32 {
        0
    }

    fn version_upgrade(&mut self, pg: &mut Client) -> anyhow::Result<()> {
        pg.execute(
            "
            CREATE TABLE IF NOT EXISTS
            id_to_entity (
                id VARCHAR PRIMARY KEY,
                entity_json VARCHAR NOT NULL
            )
        ",
            &[],
        )?;
        Ok(())
    }

    fn begin_block(&mut self, block: &Block, _: &mut Client) -> anyhow::Result<()> {
        self.uncommitted
            .insert(block.hash.clone(), EntityId::Block(block.hash.clone()));
        Ok(())
    }

    fn visit_extrinsic(&mut self, extr: &Extrinsic, _: &mut Client) -> anyhow::Result<()> {
        self.uncommitted
            .insert(extr.hash.clone(), EntityId::Extrinsic(extr.hash.clone()));
        if let Some(signer) = &extr.signer {
            self.uncommitted
                .insert(signer.to_string(), EntityId::Address(signer.to_string()));
        }
        match (extr.section.as_ref(), extr.method.as_ref()) {
            ("fractalMinting", "registerIdentity") => {
                let fractal_id = extr.args[0]
                    .as_str()
                    .expect("registerIdentity args[0] should be string");
                self.uncommitted.insert(
                    fractal_id.to_string(),
                    EntityId::FractalId(fractal_id.to_string()),
                );
            }
            ("balances", "transfer") => {
                let dest = &extr.args[0]["id"];
                if let Some(destination) = dest.as_str() {
                    self.uncommitted.insert(
                        destination.to_string(),
                        EntityId::Address(destination.to_string()),
                    );
                } else {
                    log::warn!("Got transfer without id: {:?}", extr);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn end_block(&mut self, _: &Block, pg: &mut Client) -> anyhow::Result<()> {
        let stmt = pg.prepare(
            "INSERT INTO id_to_entity (id, entity_json) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )?;
        for (string, entity) in self.uncommitted.drain() {
            let entity_json = serde_json::to_string(&entity)?;
            pg.execute(&stmt, &[&string, &entity_json])?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub enum EntityId {
    Block(String),
    Extrinsic(String),
    Address(String),
    FractalId(String),
}
