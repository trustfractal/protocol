pub mod fractal_store;
pub use fractal_store::{Database, FractalStore};

mod kv_set;

pub mod test;

pub type Hash = ();

pub enum Given {
    RootIs(Hash),
}

pub enum Proposition {
    ObjectIsValue(Vec<u8>, Vec<u8>),
}

pub struct Proof {}

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
