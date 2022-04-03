use parity_scale_codec::{Decode, Encode};
use parity_scale_codec_derive::{Decode, Encode};

pub mod fractal_store;
pub(crate) use fractal_store::*;
pub use fractal_store::{Database, FractalStore};

mod kv_set;
mod merkle_tree;

pub mod test;

pub type Hash = [u8; 64];

#[derive(Debug, PartialEq, Eq)]
pub enum Given {
    RootIs(Hash),
}

#[derive(Debug)]
pub enum Proposition {
    ObjectIsValue(Vec<u8>, Vec<u8>),
    HashInObjectTree(Hash),
}

#[derive(Debug, Encode, Decode)]
pub enum Proof {
    Empty,
}

impl Proof {
    pub fn serialize(&self) -> Vec<u8> {
        self.encode()
    }
}

pub struct ProofChecker {
    given: Given,
}

impl ProofChecker {
    pub fn new(given: Given) -> Self {
        ProofChecker { given }
    }

    pub fn verify(&self, proposition: Proposition, mut proof: &[u8]) -> bool {
        let proof = match Proof::decode(&mut proof) {
            Ok(p) => p,
            Err(_) => return false,
        };

        self.verify_proposition(proposition, proof)
    }

    fn verify_proposition(&self, prop: Proposition, proof: Proof) -> bool {
        match (prop, proof) {
            (Proposition::ObjectIsValue(object_id, value), p) => {
                let object_hash = crate::fractal_store::object_hash(&object_id, &value);
                self.verify_proposition(Proposition::HashInObjectTree(object_hash), p)
            }
            (Proposition::HashInObjectTree(hash), Proof::Empty) => {
                self.given == Given::RootIs(hash)
            }
        }
    }
}
