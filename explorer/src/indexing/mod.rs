use crate::data::*;
use postgres::Client;
use std::collections::HashMap;

pub mod identities;

pub trait Indexer: Send {
    fn storage_version(&mut self) -> u32;

    fn version_upgrade(&mut self, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    fn begin(&mut self, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    fn begin_block(&mut self, _block: &Block, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    fn end_block(&mut self, _block: &Block, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    fn visit_extrinsic(&mut self, _extrinsic: &Extrinsic, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }
}

pub fn indexers() -> HashMap<String, Box<dyn Indexer>> {
    let mut map = HashMap::new();
    map.insert(
        "count_identities".to_string(),
        Box::new(identities::CountIdentities::default()) as Box<dyn Indexer>,
    );
    map
}
