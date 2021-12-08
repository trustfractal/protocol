use serde::*;

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub hash: String,
    pub parent: String,
    pub number: u64,
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
