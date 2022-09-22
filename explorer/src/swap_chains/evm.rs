use super::*;

use serde::{Deserialize, Serialize};
use web3::{contract::Contract, types::*, *};

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
            "./FCLToken_contract.json"
        )).unwrap();
        json["abi"].take()
    };

    static ref BURNER_ABI: serde_json::Value = {
        let mut json: serde_json::Value = serde_json::from_slice(include_bytes!(
            "./FCLBurner_contract.json"
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

    pub decimals: u8,
}

impl Chain {
    pub fn token_contract(&self) -> anyhow::Result<Contract<transports::Http>> {
        Ok(Contract::from_json(
            self.web3.eth(),
            self.token_contract,
            &serde_json::to_vec(&*TOKEN_ABI)?,
        )?)
    }

    pub fn burner_contract(&self) -> anyhow::Result<Contract<transports::Http>> {
        Ok(Contract::from_json(
            self.web3.eth(),
            self.burner_contract,
            &serde_json::to_vec(&*BURNER_ABI)?,
        )?)
    }

    pub fn erc20_to_balance(&self, erc20: U256) -> anyhow::Result<Balance> {
        let balance = decimals_from_to(erc20, self.decimals, 12);

        core::convert::TryFrom::try_from(balance).map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn balance_to_erc20(&self, balance: Balance) -> U256 {
        decimals_from_to(U256::from(balance), 12, self.decimals)
    }
}

pub fn address_str(addr: &Address) -> String {
    format!("{:?}", addr)
}

fn decimals_from_to(v: U256, from_decimals: u8, to_decimals: u8) -> U256 {
    use core::cmp::Ordering::*;

    match from_decimals.cmp(&to_decimals) {
        Equal => v,
        Less => {
            let factor: U256 = (10u128.pow((to_decimals - from_decimals) as u32)).into();
            v * &factor
        }
        Greater => {
            let factor: U256 = (10u128.pow((from_decimals - to_decimals) as u32)).into();
            v / &factor
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod decimals_from_to {
        use super::*;

        #[test]
        fn zero_is_zero() {
            assert_eq!(decimals_from_to(0.into(), 1, 1), 0.into());
        }

        #[test]
        fn more_decimals_in_from() {
            assert_eq!(decimals_from_to(1_000.into(), 4, 1), 1.into());
        }

        #[test]
        fn more_decimals_in_to() {
            assert_eq!(decimals_from_to(1_000.into(), 4, 7), 1_000_000.into());
        }
    }
}
