use derive_more::*;
use serde::*;
use shared_lru::MemorySize;

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ramhorns::Content)]
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

#[derive(Deserialize, Clone, PartialEq, Eq, Debug, Deref, DerefMut)]
#[serde(deny_unknown_fields)]
pub struct Extrinsic {
    #[serde(flatten)]
    #[deref]
    #[deref_mut]
    pub without_json: ExtrinsicNoJson,

    pub args: serde_json::Value,
    pub error: Option<serde_json::Value>,
}

// TODO(shelbyd): Remove once ramhorns supports serde_json::Value.
#[derive(Deserialize, Clone, PartialEq, Eq, Debug, ramhorns::Content)]
#[serde(deny_unknown_fields)]
pub struct ExtrinsicNoJson {
    pub hash: String,

    pub block: String,
    pub index_in_block: u64,

    pub nonce: u64,
    pub signer: Option<String>,

    pub section: String,
    pub method: String,
    pub success: bool,
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
