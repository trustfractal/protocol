use serde::*;
use sp_core::*;
use subxt::{*, tx::PairSigner};

use super::*;

pub struct Substrate {}

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

        let signer = PairSigner::<SubstrateConfig, _>::new(pair);
        let state = SwapState::AwaitingReceive {
            receive_address: signer.account_id().to_string(),
            payment_request: format!("fcl_substrate:{}", signer.account_id()),
        };

        let mut sidecar = Sidecar::default();
        sidecar
            .set("substrate/receive", ReceiveSidecar { secret_key })
            .unwrap();

        (state, Some(sidecar))
    }

    fn has_received(&self, _swap: &mut Swap) -> anyhow::Result<bool> {
        log::warn!("Unimplemented: has_received");
        Ok(false)
    }

    fn has_finalized(&self, _swap: &mut Swap) -> anyhow::Result<bool> {
        unimplemented!("has_finalized");
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ReceiveSidecar {
    secret_key: String,
}
