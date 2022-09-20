use super::*;

use crate::swap_chains::evm;

pub struct EvmBurner {
    info: ChainInfo,
    chain_id: u32,
    token_contract_address: String,
    burn_contract_address: String,
}

impl EvmBurner {
    pub fn new(
        info: ChainInfo,
        chain_id: String,
        token_contract_address: String,
        burn_contract_address: String,
    ) -> anyhow::Result<Self> {
        Ok(EvmBurner {
            info,
            chain_id: chain_id.parse()?,
            token_contract_address,
            burn_contract_address,
        })
    }
}

impl Chain for EvmBurner {
    fn info(&self) -> ChainInfo {
        self.info.clone()
    }
}

impl Receiver for EvmBurner {
    fn create_receive_request(&self, id: &str) -> (SwapState, Option<Sidecar>) {
        let state = SwapState::AwaitingReceive(PaymentRequest::Metamask {
            chain_id: self.chain_id,
            transactions: vec![
                evm::Transaction {
                    contract_address: self.token_contract_address.clone(),
                    contract_abi: evm::token_abi(),
                    method: "approve".to_string(),
                    params: vec![
                        serde_json::Value::from(self.burn_contract_address.as_str()),
                        serde_json::Value::from("user_amount"),
                    ],
                },
                evm::Transaction {
                    contract_address: self.burn_contract_address.clone(),
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
