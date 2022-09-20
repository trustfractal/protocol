use super::*;

use crate::swap_chains::evm;

pub struct EvmBurner {
    chain: &'static evm::Chain,
}

impl EvmBurner {
    pub fn new(chain: &'static evm::Chain) -> anyhow::Result<Self> {
        Ok(EvmBurner { chain })
    }
}

impl Chain for EvmBurner {
    fn info(&self) -> ChainInfo {
        self.chain.info.clone()
    }
}

impl Receiver for EvmBurner {
    fn create_receive_request(&self, id: &str) -> (SwapState, Option<Sidecar>) {
        let state = SwapState::AwaitingReceive(PaymentRequest::Metamask {
            chain_id: self.chain.chain_id,
            transactions: vec![
                evm::Transaction {
                    contract_address: evm::address_str(&self.chain.token_contract),
                    contract_abi: evm::token_abi(),
                    method: "approve".to_string(),
                    params: vec![
                        serde_json::Value::from(evm::address_str(&self.chain.burner_contract)),
                        serde_json::Value::from("user_amount"),
                    ],
                },
                evm::Transaction {
                    contract_address: evm::address_str(&self.chain.burner_contract),
                    contract_abi: evm::burner_abi(),
                    method: "burn".to_string(),
                    params: vec![
                        serde_json::Value::from(id),
                        serde_json::Value::from("user_amount"),
                    ],
                },
            ],
        });
        (state, None)
    }

    fn has_received(&self, _swap: &mut Swap) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn finalized_amount(&self, _swap: &mut Swap) -> anyhow::Result<Option<Balance>> {
        unimplemented!("finalized_amount");
    }

    fn after_finalized(&self, _: &mut Swap, _: Balance) -> anyhow::Result<()> {
        unimplemented!("after_finalized");
    }
}
