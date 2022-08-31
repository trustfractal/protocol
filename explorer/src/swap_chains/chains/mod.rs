use super::{ChainInfo, Sidecar, Swap, SwapState};

mod substrate;

mod test;
use test::Test;

pub type ReceiverRef = &'static dyn Receiver;
pub type SenderRef = &'static dyn Sender;

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

lazy_static::lazy_static! {
    static ref RECEIVERS: Vec<Box<dyn Receiver>> = vec![
        Box::new(Test),
        Box::new(substrate::Substrate::new(fractal_protocol_url()).unwrap()),
    ];

    static ref SENDERS: Vec<Box<dyn Sender>> = vec![
        Box::new(Test),
    ];
}

pub fn receivers() -> impl Iterator<Item = ReceiverRef> {
    RECEIVERS.iter().map(|r| r.as_ref())
}

pub fn senders() -> impl Iterator<Item = SenderRef> {
    SENDERS.iter().map(|s| s.as_ref())
}

pub fn receiver(id: &str) -> anyhow::Result<ReceiverRef> {
    receivers()
        .find(|r| r.info().id == id)
        .ok_or_else(|| anyhow::anyhow!("Unrecognized receiver {}", id))
}

pub fn sender(id: &str) -> anyhow::Result<SenderRef> {
    senders()
        .find(|r| r.info().id == id)
        .ok_or_else(|| anyhow::anyhow!("Unrecognized sender {}", id))
}

fn fractal_protocol_url() -> String {
    std::env::var("SUBSTRATE_CHAIN_URL")
        .unwrap_or_else(|_| "wss://nodes.mainnet.fractalprotocol.com:443".to_string())
}
