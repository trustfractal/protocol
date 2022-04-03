pub mod fractal_store;
pub(crate) use fractal_store::*;
pub use fractal_store::{Database, FractalStore};

mod kv_set;
mod merkle_tree;

pub mod test;

pub type Hash = [u8; 64];

#[derive(Debug)]
pub enum Given {
    RootIs(Hash),
}

#[derive(Debug)]
pub enum Proposition {
    ObjectIsValue(Vec<u8>, Vec<u8>),
    HashInObjectTree(Hash),
}

#[derive(Debug)]
pub enum Proof {
    Empty,
    ObjectValue {
        hash_exists: Box<Proof>,
        object_id: Vec<u8>,
        value: Vec<u8>,
    },
}

impl Proof {
    pub fn serialize(&self) -> Vec<u8> {
        unimplemented!("serialize");
    }
}

pub struct ProofChecker {}

impl ProofChecker {
    pub fn new(_given: Given) -> Self {
        unimplemented!("new");
    }

    pub fn verify(&self, _proposition: Proposition, _proof: &[u8]) -> bool {
        unimplemented!("verify");
    }
}
