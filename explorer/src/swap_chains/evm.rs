use serde::{Deserialize, Serialize};

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
