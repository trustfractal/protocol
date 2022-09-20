use super::*;

use crate::{block_on, swap_chains::evm};

use web3::{
    contract::Options,
    types::{BlockId, U256},
};

pub struct EvmBurner {
    chain: &'static evm::Chain,
    confirmations_required: u64,
}

impl EvmBurner {
    pub fn new(chain: &'static evm::Chain, confirmations_required: String) -> anyhow::Result<Self> {
        Ok(EvmBurner {
            chain,
            confirmations_required: confirmations_required.parse()?,
        })
    }

    async fn burned_at_block(
        &self,
        id: &str,
        block: impl Into<Option<BlockId>>,
    ) -> anyhow::Result<U256> {
        let contract = self.chain.burner_contract()?;

        Ok(contract
            .query(
                "amountBurnedById",
                id.to_string(),
                None,
                Options::default(),
                block.into(),
            )
            .await?)
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
            erc20_decimals: self.chain.decimals,
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

    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        block_on(async {
            let burned = self.burned_at_block(&swap.id, None).await?;
            Ok(burned > 0.into())
        })
    }

    fn finalized_amount(&self, swap: &mut Swap) -> anyhow::Result<Option<Balance>> {
        block_on(async {
            let block_number = self.chain.web3.eth().block_number().await?;
            let confirmed_block = block_number - self.confirmations_required;
            let burned = self
                .burned_at_block(&swap.id, Some(confirmed_block.into()))
                .await?;
            let as_balance = self.chain.erc20_to_balance(burned)?;
            log::info!("Swap {} has finalized {}", swap.id, as_balance);

            Ok(if as_balance > 0 {
                Some(as_balance)
            } else {
                None
            })
        })
    }

    fn after_finalized(&self, _: &mut Swap, _: Balance) -> anyhow::Result<()> {
        Ok(())
    }
}
