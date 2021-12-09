use crate::data::*;
use postgres::Client;
use std::collections::HashMap;

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
}

pub fn indexers() -> HashMap<String, Box<dyn Indexer>> {
    let mut map = HashMap::new();

    map.insert(
        "count_identities".to_string(),
        Box::new(identities::CountIdentities::default()) as Box<dyn Indexer>,
    );

    map
}
