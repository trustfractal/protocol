use crate::Hash;

pub(crate) struct MerkleTree {}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree {}
    }

    pub fn update(&mut self, _hash: Hash) {
        unimplemented!("update");
    }

    pub fn root_hash(&self) -> Hash {
        unimplemented!("root_hash");
    }
}

// use blake2::digest::VariableOutput;
//
// let hasher = blake2::Blake2bVar::new(64).unwrap();
//
// let mut buf = Hash::default();
// hasher.finalize_variable(&mut buf).unwrap();
//
// Ok(buf)
