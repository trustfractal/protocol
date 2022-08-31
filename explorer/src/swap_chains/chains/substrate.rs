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
            .set("substrate/receive", ReceiveSidecar { secret_key })
            .unwrap();

        (state, Some(sidecar))
    }

    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        let receive_address = match &swap.state {
            SwapState::AwaitingReceive {
                receive_address, ..
            } => receive_address,
            _ => return Ok(true),
        };
        let receive_account = AccountId32::from_str(receive_address)
            .map_err(|e| anyhow::anyhow!("Error parsing receive address {}", e))?;
        let account_data =
            self.api
                .get_storage_map("System", "Account", receive_account.encode(), None)?;
        let account_info = match account_data {
            None => return Ok(false),
            Some(b) => b.decode::<AccountInfo>()?,
        };

        Ok(account_info.data.free > 0)
    }

    fn has_finalized(&self, _swap: &mut Swap) -> anyhow::Result<bool> {
        unimplemented!("has_finalized");
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ReceiveSidecar {
    secret_key: String,
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
