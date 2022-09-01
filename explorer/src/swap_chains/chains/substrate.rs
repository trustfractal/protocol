use serde::*;
use sp_core::{crypto::AccountId32, *};
use std::str::FromStr;
use substrate_api_client::*;

use super::{Balance, *};

pub struct Substrate {
    api: Api<sr25519::Pair>,
    url: String,
}

impl Substrate {
    pub fn new(url: impl AsRef<str>) -> anyhow::Result<Self> {
        let url = url.as_ref().to_string();
        Ok(Substrate {
            api: Api::new(url.clone())?,
            url,
        })
    }

    fn balance_at_block(
        &self,
        account: &AccountId32,
        block: impl Into<Option<Hash>>,
    ) -> anyhow::Result<Balance> {
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

    // TODO(shelbyd): Burn after finalized.
    fn finalized_amount(&self, swap: &mut Swap) -> anyhow::Result<Option<Balance>> {
        match &swap.state {
            SwapState::AwaitingReceive { .. } => return Ok(None),
            SwapState::Finalizing { .. } => {}
            SwapState::Finished { .. } => unreachable!(),
        }

        let finalized_head = self
            .api
            .get_finalized_head()?
            .ok_or(anyhow::anyhow!("No finalized head"))?;

        let finalized_balance =
            self.balance_at_block(&get_receive_account(swap)?, finalized_head)?;

        if finalized_balance > 0 {
            Ok(Some(finalized_balance))
        } else {
            Ok(None)
        }
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
            .api
            .send_extrinsic(txn.hex_encode(), XtStatus::InBlock)?
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

fn get_receive_account(swap: &Swap) -> anyhow::Result<AccountId32> {
    let address = swap
        .secret_sidecar
        .get::<ReceiveSidecar>("substrate/receive")?
        .ok_or_else(|| anyhow::anyhow!("Missing sidecar for substrate receive"))?
        .receive_address;

    AccountId32::from_str(&address).map_err(|e| anyhow::anyhow!("Error parsing address: {}", e))
}
