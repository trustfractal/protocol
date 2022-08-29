use serde::*;
use sp_core::*;

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

        let mut sidecar = Sidecar::default();
        sidecar
            .set("substrate/receive", ReceiveSidecar { secret_key })
            .unwrap();

        // let signer = subxt::tx::PairSigner::new(pair);

        unimplemented!("create_receive_request");
    }

    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        unimplemented!("has_received");
    }

    fn has_finalized(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        unimplemented!("has_finalized");
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ReceiveSidecar {
    secret_key: String,
}
