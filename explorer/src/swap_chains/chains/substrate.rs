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
        unimplemented!("create_receive_request");
    }

    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        unimplemented!("has_received");
    }

    fn has_finalized(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        unimplemented!("has_finalized");
    }
}
