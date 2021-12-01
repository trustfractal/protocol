use crate::data::*;
use std::collections::HashMap;

pub mod identities;

pub trait Indexer: Send {
    fn begin_block(&mut self, _number: u64) {}
    fn visit_extrinsic(&mut self, _extrinsic: &Extrinsic) {}
}

pub fn indexers() -> HashMap<String, Box<dyn Indexer>> {
    let mut map = HashMap::new();
    map.insert(
        "count_identities".to_string(),
        Box::new(identities::CountIdentities::default()) as Box<dyn Indexer>,
    );
    map
}
