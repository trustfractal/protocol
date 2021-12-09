use serde::*;
use shared_lru::MemorySize;

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub hash: String,
    pub parent: String,
    pub number: u64,
}

impl MemorySize for Block {
    fn bytes(&self) -> usize {
        MemorySize::bytes(&self.hash)
            + MemorySize::bytes(&self.parent)
            + MemorySize::bytes(&self.number)
    }
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Extrinsic {
    pub block: String,
    pub index_in_block: u64,

    pub nonce: u64,
    pub signer: Option<String>,

    pub section: String,
    pub method: String,
    pub success: bool,

    pub args: serde_json::Value,
    pub error: Option<serde_json::Value>,
}

impl MemorySize for Extrinsic {
    fn bytes(&self) -> usize {
        MemorySize::bytes(&self.block)
            + MemorySize::bytes(&self.index_in_block)
            + MemorySize::bytes(&self.nonce)
            + MemorySize::bytes(&self.signer)
            + MemorySize::bytes(&self.section)
            + MemorySize::bytes(&self.method)
            + MemorySize::bytes(&self.success)
            + MemorySize::bytes(&self.args)
            + MemorySize::bytes(&self.error)
    }
}
