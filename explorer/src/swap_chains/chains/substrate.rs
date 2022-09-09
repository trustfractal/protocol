#![allow(unreachable_code, unused_variables)]

use serde::*;
use sp_core::{crypto::AccountId32, *};
use std::{str::FromStr, sync::RwLock};
use substrate_api_client::*;

use super::{Balance, *};

pub struct Substrate {
    url: String,
    // We maintain a mutable connected client to gracefully handle when the chain is inaccessible.
    connected_api: RwLock<Option<Api<sr25519::Pair>>>,
}

impl Substrate {
    pub fn new(url: impl AsRef<str>) -> Self {
        let url = url.as_ref().to_string();
        Substrate {
            url,
            connected_api: RwLock::new(None),
        }
    }

    fn api_call<R>(
        &self,
        call: impl FnOnce(&Api<sr25519::Pair>) -> Result<R, ApiClientError>,
    ) -> anyhow::Result<R> {
        if self.connected_api.read().unwrap().is_none() {
            let mut lock = self.connected_api.write().unwrap();
            *lock = Some(Api::new(self.url.clone())?);
        }

        let lock = self.connected_api.read().unwrap();
        let api = &lock.as_ref().unwrap();
        match call(api) {
            Ok(r) => Ok(r),
            Err(e) => {
                if let ApiClientError::Disconnected(_) = &e {
                    drop(lock);
                    log::info!("Disconnected from {}, dropping client", self.url);
                    *self.connected_api.write().unwrap() = None;
                }
                Err(e.into())
            }
        }
    }

    fn balance_at_block(
        &self,
        account: &AccountId32,
        block: impl Into<Option<Hash>>,
    ) -> anyhow::Result<Balance> {
        let account_data = self.api_call(|api| {
            api.get_storage_map("System", "Account", account.encode(), block.into())
        })?;
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

    fn finalized_amount(&self, swap: &mut Swap) -> anyhow::Result<Option<Balance>> {
        match &swap.state {
            SwapState::AwaitingReceive { .. } => return Ok(None),
            SwapState::Finalizing { .. } => {}
            SwapState::Sending { .. } | SwapState::Finished { .. } => unreachable!(),
        }

        let finalized_head = self
            .api_call(|api| api.get_finalized_head())?
            .ok_or(anyhow::anyhow!("No finalized head"))?;

        let finalized_balance =
            self.balance_at_block(&get_receive_account(swap)?, finalized_head)?;

        if finalized_balance > 0 {
            Ok(Some(finalized_balance))
        } else {
            Ok(None)
        }
    }

    fn after_finalized(&self, swap: &mut Swap, amount: Balance) -> anyhow::Result<()> {
        let sidecar = get_receive_sidecar(swap)?;
        let (signer, _) =
            sr25519::Pair::from_phrase(&sidecar.secret_key, None).expect("valid pair key");

        let signer_api = Api::new(self.url.clone())?.set_signer(signer)?;
        let txn = compose_extrinsic(
            &signer_api,
            "FractalTokenDistribution",
            "burn",
            (Some(amount),),
        );

        todo!();

        Ok(())
    }
}

impl Sender for Substrate {
    // TODO(shelbyd): Mint tokens.
    fn send(&self, swap: &mut Swap, received_amount: Balance) -> anyhow::Result<SwapState> {
        let key = swap
            .secret_sidecar
            .get::<ReceiveSidecar>("substrate/receive")?
            .ok_or_else(|| anyhow::anyhow!("Missing sidecar for substrate receive"))?
            .secret_key;
        let (pair, _) = sr25519::Pair::from_phrase(&key, None).expect("valid pair key");
        let temp_api = Api::new(self.url.clone())?.set_signer(pair)?;

        let to = AccountId32::from_str(&swap.user.send_address).expect("valid user address");

        todo!("Correct txn");
        let txn = compose_extrinsic(
            &temp_api,
            "Balances",
            "transfer",
            (
                GenericAddress::Id(to),
                parity_scale_codec::Compact(received_amount),
            ),
        );

        let hash = self
            .api_call(|api| api.send_extrinsic(txn.hex_encode(), XtStatus::InBlock))?
            .expect("extrinsic will result in hash");
        let hash_str = format!("{:x}", hash);

        Ok(SwapState::Finished {
            txn_link: format!("https://explorer.fractalprotocol.com/{}", hash_str),
            txn_id: hash_str,
        })
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
    free: Balance,
    _reserved: Balance,
    _misc_frozen: Balance,
}

fn get_receive_sidecar(swap: &Swap) -> anyhow::Result<ReceiveSidecar> {
    swap.secret_sidecar
        .get::<ReceiveSidecar>("substrate/receive")?
        .ok_or_else(|| anyhow::anyhow!("Missing sidecar for substrate receive"))
}

fn get_receive_account(swap: &Swap) -> anyhow::Result<AccountId32> {
    AccountId32::from_str(&get_receive_sidecar(swap)?.receive_address)
        .map_err(|e| anyhow::anyhow!("Error parsing address: {}", e))
}
