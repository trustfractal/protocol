use super::*;
use crate::{
    block_on,
    swap_chains::{evm, Event},
};

use secp256k1::SecretKey;
use web3::{contract::Options, types::*};

pub struct EvmMintable {
    chain: &'static evm::Chain,
    minting_key: SecretKey,
    explorer_url: String,
}

impl EvmMintable {
    pub fn new(
        chain: &'static evm::Chain,
        explorer_url: String,
        private_key: String,
    ) -> anyhow::Result<Self> {
        Ok(EvmMintable {
            chain,
            minting_key: private_key.trim_start_matches("0x").parse()?,
            explorer_url,
        })
    }
}

impl Chain for EvmMintable {
    fn info(&self) -> ChainInfo {
        self.chain.info.clone()
    }
}

impl Sender for EvmMintable {
    fn send(&self, swap: &mut Swap, amount: Balance) -> anyhow::Result<SwapState> {
        block_on(async {
            let user_address: Address = swap.user.send_address.parse()?;

            let contract = self.chain.token_contract()?;
            let receipt = contract
                .signed_call_with_confirmations(
                    "mint",
                    (user_address, self.chain.balance_to_erc20(amount)),
                    Options::default(),
                    1,
                    &self.minting_key,
                )
                .await?;

            let hash = format!("{:?}", receipt.transaction_hash);
            swap.push_event(Event::generic("evm_transaction_receipt", receipt)?);

            Ok(SwapState::Finished {
                txn_link: format!("{}/{}", self.explorer_url, hash),
                txn_id: hash,
            })
        })
    }

    fn is_valid(&self, address: &str) -> bool {
        let address: Result<Address, _> = address.trim_start_matches("0x").parse();
        address.is_ok()
    }
}
