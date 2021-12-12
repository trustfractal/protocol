use crate::data::*;

use actix_web::{error::*, *};
use postgres::Client;
use serde::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct IdToEntity {
    uncommitted: HashMap<String, EntityId>,
    current_block_number: u64,
}

impl super::Indexer for IdToEntity {
    fn version(&mut self) -> u32 {
        1
    }

    fn version_upgrade(&mut self, pg: &mut Client) -> anyhow::Result<()> {
        pg.execute(
            "CREATE TABLE IF NOT EXISTS
            id_to_entity (
                id VARCHAR PRIMARY KEY,
                entity_json VARCHAR NOT NULL
            )",
            &[],
        )?;
        pg.execute("TRUNCATE id_to_entity", &[])?;
        Ok(())
    }

    fn begin_block(&mut self, block: &Block, _: &mut Client) -> anyhow::Result<()> {
        self.current_block_number = block.number;
        self.uncommitted
            .insert(block.hash.clone(), EntityId::Block(block.number));
        Ok(())
    }

    fn visit_extrinsic(&mut self, extr: &Extrinsic, _: &mut Client) -> anyhow::Result<()> {
        self.uncommitted.insert(
            extr.hash.clone(),
            EntityId::Extrinsic(self.current_block_number, extr.index_in_block),
        );
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
            "INSERT INTO id_to_entity (id, entity_json) VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET entity_json = $2",
        )?;
        for (string, entity) in self.uncommitted.drain() {
            let entity_json = serde_json::to_string(&entity)?;
            pg.execute(&stmt, &[&string, &entity_json])?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum EntityId {
    Block(u64),
    Extrinsic(u64, u64),
    Address(String),
    FractalId(String),
}

impl EntityId {
    fn web_path(&self) -> String {
        match self {
            EntityId::Block(number) => format!("/block/{}", number),
            EntityId::Extrinsic(block, index) => format!("/block/{}/extrinsic/{}", block, index),
            EntityId::Address(encoded) => format!("/address/{}", encoded),
            EntityId::FractalId(id) => format!("/fractal_id/{}", id),
        }
    }
}

#[actix_web::get("/{id}")]
pub async fn redirect_id(
    id: web::Path<(String,)>,
    pg: web::Data<block_pool::Pool<Client>>,
) -> actix_web::Result<impl Responder> {
    let (id,) = id.into_inner();
    let mut pg = pg.take();

    let mut matches = pg
        .query(
            "SELECT entity_json FROM id_to_entity WHERE id LIKE $1 LIMIT 2",
            &[&format!("{}%", id)],
        )
        .map_err(ErrorInternalServerError)?
        .into_iter()
        .map(|row| serde_json::from_str(row.get(&"entity_json")));

    let first: Option<EntityId> = matches
        .next()
        .transpose()
        .map_err(ErrorInternalServerError)?;
    let second = matches.next();

    if let (Some(entity), None) = (first, second) {
        Ok(HttpResponse::MovedPermanently()
            .header(http::header::LOCATION, entity.web_path())
            .finish())
    } else {
        Err(ErrorNotFound(id))
    }
}
