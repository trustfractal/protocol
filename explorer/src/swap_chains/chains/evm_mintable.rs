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

            let block_number = self.chain.web3.eth().block_number().await?;
            let receipt = contract
                .signed_call_with_confirmations(
                    "mint",
                    (user_address, self.chain.balance_to_erc20(amount)),
                    Options::with(|options| {
                        if self.chain.info.id == "acala" {
                            let (gas_price, gas_limit) = calculate_acala_gas(block_number);
                            options.gas = Some(gas_limit);
                            options.gas_price = Some(gas_price);
                        }
                    }),
                    1,
                    &self.minting_key,
                )
                .await?;

            let hash = format!("{:?}", receipt.transaction_hash);
            swap.push_event(Event::generic("evm_transaction_receipt", receipt)?);

            Ok(SwapState::Finished {
                txn_link: format!("{}/tx/{}", self.explorer_url, hash),
                txn_id: hash,
            })
        })
    }

    fn is_valid(&self, address: &str) -> bool {
        let address: Result<Address, _> = address.trim_start_matches("0x").parse();
        address.is_ok()
    }
}

// Copied from Acala docs:
// https://evmdocs.acala.network/tutorials/hardhat-tutorials/advancedescrow-tutorial#transaction-helper-utility
fn calculate_acala_gas(block_number: U64) -> (U256, U256) {
    let tx_fee_per_gas = U256::from_str_radix("199999946752", 10).unwrap();
    let storage_byte_deposit = U256::from_str_radix(&std::env::var("ACALA_STORAGE_BYTE_DEPOSIT").unwrap_or("100000000000000".to_string()), 10).unwrap();
    let gas_limit = U256::from_str_radix("31000000", 10).unwrap();
    let storage_limit = U256::from_str_radix("64001", 10).unwrap();
    let valid_until = U256::from((block_number + 100).as_u64());

    fn div_round_up(numerator: U256, denominator: impl Into<U256>) -> U256 {
        let denominator = denominator.into();

        let div = numerator / denominator;
        let modulo = numerator % denominator;

        if modulo == 0.into() {
            div
        } else {
            div + 1
        }
    }

    let storage_entry_limit = div_round_up(storage_limit, 64);
    let storage_entry_deposit = storage_byte_deposit * 64;
    let tx_gas_limit = storage_entry_deposit / tx_fee_per_gas * storage_entry_limit + gas_limit;

    let block_period = div_round_up(valid_until, 30);
    let shifted_block_period = block_period << 16;
    let tx_gas_price = tx_fee_per_gas + shifted_block_period + storage_entry_limit;

    (tx_gas_price, tx_gas_limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod calculate_acala_gas {
        use super::*;

        #[test]
        fn gas_limit_block_from_local_testing() {
            assert_eq!(
                calculate_acala_gas(2_736_423.into()).1,
                U256::from_str_radix("63032000", 10).unwrap(),
            );
        }

        #[test]
        fn gas_price_local_testing() {
            assert_eq!(
                calculate_acala_gas(2_736_423.into()).0,
                U256::from_str_radix("205978010601", 10).unwrap(),
            );
        }
    }
}
