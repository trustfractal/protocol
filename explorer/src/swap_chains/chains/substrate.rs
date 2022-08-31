use serde::*;
use sp_core::{crypto::AccountId32, *};
use std::str::FromStr;
use substrate_api_client::*;

use super::*;

pub struct Substrate {
    api: Api<sr25519::Pair>,
}

impl Substrate {
    pub fn new(url: impl AsRef<str>) -> anyhow::Result<Self> {
        Ok(Substrate {
            api: Api::new(url.as_ref().to_string())?,
        })
    }

    fn balance_at_block(
        &self,
        account: &AccountId32,
        block: impl Into<Option<Hash>>,
    ) -> anyhow::Result<u128> {
        let account_data =
            self.api
                .get_storage_map("System", "Account", account.encode(), block.into())?;
        let account_info = match account_data {
            None => return Ok(0),
            Some(b) => b.decode::<AccountInfo>()?,
        };

        Ok(account_info.data.free)
    }
}

impl Chain for Substrate {
    fn info(&self) -> ChainInfo {
        ChainInfo {
            id: String::from("substrate"),
            name: String::from("Substrate"),
        }
    }
}

impl Receiver for Substrate {
    fn create_receive_request(&self) -> (SwapState, Option<Sidecar>) {
        let (pair, secret_key, _) = sr25519::Pair::generate_with_phrase(None);

        let account_id: AccountId32 = pair.public().into();
        let state = SwapState::AwaitingReceive {
            receive_address: account_id.to_string(),
            payment_request: format!("fcl_substrate:{}", account_id),
        };

        let mut sidecar = Sidecar::default();
        sidecar
            .set(
                "substrate/receive",
                ReceiveSidecar {
                    secret_key,
                    receive_address: account_id.to_string(),
                },
            )
            .unwrap();

        (state, Some(sidecar))
    }

    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        match &swap.state {
            SwapState::AwaitingReceive { .. } => {}
            _ => return Ok(true),
        };

        let balance = self.balance_at_block(&get_receive_account(swap)?, None)?;
        Ok(balance > 0)
    }

    fn has_finalized(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        match &swap.state {
            SwapState::AwaitingReceive { .. } => return Ok(false),
            SwapState::Finalizing { .. } => {}
            SwapState::Finished { .. } => return Ok(true),
        }

        let finalized_head = self
            .api
            .get_finalized_head()?
            .ok_or(anyhow::anyhow!("No finalized head"))?;

        let finalized_balance = self.balance_at_block(&get_receive_account(swap)?, finalized_head)?;
        Ok(finalized_balance > 0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ReceiveSidecar {
    secret_key: String,
    receive_address: String,
}

#[derive(Decode, Debug)]
struct AccountInfo {
    _nonce: u32,
    _consumers: u32,
    _providers: u32,
    _sufficients: u32,
    data: AccountData,
}

#[derive(Decode, Debug)]
struct AccountData {
    free: u128,
    _reserved: u128,
    _misc_frozen: u128,
}

fn get_receive_account(swap: &Swap) -> anyhow::Result<AccountId32> {
    let address = swap
        .secret_sidecar
        .get::<ReceiveSidecar>("substrate/receive")?
        .ok_or_else(|| anyhow::anyhow!("Missing sidecar for substrate receive"))?
        .receive_address;

    AccountId32::from_str(&address).map_err(|e| anyhow::anyhow!("Error parsing address: {}", e))
}
