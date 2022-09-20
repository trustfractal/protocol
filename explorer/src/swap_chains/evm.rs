use super::*;

use serde::{Deserialize, Serialize};
use web3::{
    contract::{Contract, Options},
    types::*,
    *,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub contract_address: String,
    pub contract_abi: serde_json::Value,
    pub params: Vec<serde_json::Value>,
    pub method: String,
}

pub fn token_abi() -> serde_json::Value {
    TOKEN_ABI.clone()
}

pub fn burner_abi() -> serde_json::Value {
    BURNER_ABI.clone()
}

lazy_static::lazy_static! {
    static ref TOKEN_ABI: serde_json::Value = {
        let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../evm/artifacts/contracts/FCLToken.sol/FCLToken.json"
        )).unwrap();
        json["abi"].take()
    };

    static ref BURNER_ABI: serde_json::Value = {
        let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../evm/artifacts/contracts/FCLBurner.sol/FCLBurner.json"
        )).unwrap();
        json["abi"].take()
    };
}

pub struct Chain {
    pub web3: Web3<transports::Http>,
    pub info: ChainInfo,
    pub chain_id: u32,

    pub token_contract: Address,
    pub burner_contract: Address,
}

pub fn address_str(addr: &Address) -> String {
    format!("{:?}", addr)
}
