use super::{ChainInfo, Sidecar, Swap, SwapState};

mod substrate;

mod test;
use test::Test;

pub trait Chain: Sync + Send {
    fn info(&self) -> ChainInfo;
}

pub trait Receiver: Chain {
    fn create_receive_request(&self) -> (SwapState, Option<Sidecar>);
    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool>;
    fn has_finalized(&self, swap: &mut Swap) -> anyhow::Result<bool>;
}

pub trait Sender: Chain {
    fn send(&self, swap: &mut Swap) -> anyhow::Result<SwapState>;
}

pub fn receivers() -> impl Iterator<Item = Box<dyn Receiver>> {
    vec![
        Box::new(Test) as Box<dyn Receiver>,
        Box::new(substrate::Substrate {}),
    ]
    .into_iter()
}

pub fn senders() -> impl Iterator<Item = Box<dyn Sender>> {
    vec![Box::new(Test) as Box<dyn Sender>].into_iter()
}

pub fn receiver(id: &str) -> anyhow::Result<Box<dyn Receiver>> {
    receivers()
        .find(|r| r.info().id == id)
        .ok_or_else(|| anyhow::anyhow!("Unrecognized receiver {}", id))
}

pub fn sender(id: &str) -> anyhow::Result<Box<dyn Sender>> {
    senders()
        .find(|r| r.info().id == id)
        .ok_or_else(|| anyhow::anyhow!("Unrecognized sender {}", id))
}
