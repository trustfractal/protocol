use super::*;
use crate::swap_chains::{evm, Event};

use secp256k1::SecretKey;
use web3::{
    contract::{Contract, Options},
    types::*,
    *,
};

pub struct EvmMintable {
    info: ChainInfo,
    web3: Web3<transports::Http>,

    contract_address: Address,
    minting_key: SecretKey,

    explorer_url: String,
}

impl EvmMintable {
    pub fn new(
        url: String,
        explorer_url: String,
        contract_address: String,
        private_key: String,
        info: ChainInfo,
    ) -> anyhow::Result<Self> {
        let minting_key = if private_key.starts_with("0x") {
            &private_key[2..]
        } else {
            &private_key
        }
        .parse()?;
        Ok(EvmMintable {
            info,
            web3: Web3::new(transports::Http::new(&url).unwrap()),
            contract_address: contract_address.parse()?,
            minting_key,
            explorer_url,
        })
    }
}

impl Chain for EvmMintable {
    fn info(&self) -> ChainInfo {
        self.info.clone()
    }
}

impl Sender for EvmMintable {
    fn send(&self, swap: &mut Swap, amount: Balance) -> anyhow::Result<SwapState> {
        block_on(async {
            let contract = Contract::from_json(
                self.web3.eth(),
                self.contract_address,
                &serde_json::to_vec(&evm::token_abi())?,
            )?;

            let user_address: Address = swap.user.send_address.parse()?;
            let receipt = contract
                .signed_call_with_confirmations(
                    "mint",
                    (user_address, U256::from(amount)),
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
}

fn block_on<F: core::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
