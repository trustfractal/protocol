use crate::data::*;
use postgres::Client;
use std::collections::HashMap;

pub mod address_extrinsics;
pub mod block_timestamps;
pub mod id_to_entity;
pub mod identities;

/// Implementors of this interface should assume that they may revisit previously visited
/// blocks/extrinsics.
pub trait Indexer: Send {
    /// Whenever this value changes, and the first time run, this indexer will be restarted from
    /// block 0 and `[version_upgrade]` will be called.
    fn version(&mut self) -> u32;

    /// Called when the `[version]` value changes or the first time this indexer is run. This is
    /// the recommended place to put DB upgrade logic.
    fn version_upgrade(&mut self, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called when this Indexer instance is beginning to run. Instance initialization should be
    /// done here.
    fn begin(&mut self, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called when a new block is visited.
    fn begin_block(&mut self, _block: &Block, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called after all extrinsics/events in a block.
    fn end_block(&mut self, _block: &Block, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called for each extrinsic in a block.
    fn visit_extrinsic(&mut self, _extrinsic: &Extrinsic, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called regularly. After this call completes, the indexing system will consider this indexer
    /// to have "completed" up to the latest block. An indexer can store work in memory and wait
    /// for this call to commit them to the database. Guaranteed to be called after some call to
    /// `end_block`.
    fn commit(&mut self, _pg: &mut Client) -> anyhow::Result<()> {
        Ok(())
    }
}

pub fn indexers() -> HashMap<String, Box<dyn Indexer>> {
    let mut map = HashMap::new();

    map.insert(
        "address_extrinsics".to_string(),
        Box::new(address_extrinsics::AddressExtrinsics::default()) as Box<dyn Indexer>,
    );
    map.insert(
        "block_timestamps".to_string(),
        Box::new(block_timestamps::BlockTimestamps::default()) as Box<dyn Indexer>,
    );
    map.insert(
        "count_identities".to_string(),
        Box::new(identities::CountIdentities::default()) as Box<dyn Indexer>,
    );
    map.insert(
        "id_to_entity".to_string(),
        Box::new(id_to_entity::IdToEntity::default()) as Box<dyn Indexer>,
    );

    map
}
