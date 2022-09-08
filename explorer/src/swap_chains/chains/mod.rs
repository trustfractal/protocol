use super::{Balance, ChainInfo, Sidecar, Swap, SwapState, Txn};

mod substrate;
mod test;

pub type ReceiverRef = &'static dyn Receiver;
pub type SenderRef = &'static dyn Sender;

pub trait Chain: Sync + Send {
    fn info(&self) -> ChainInfo;

    fn ensure_submitted(&self, _txn: &Txn) -> anyhow::Result<()> {
        todo!("ensure_submitted");
    }
}

pub trait Receiver: Chain {
    fn create_receive_request(&self) -> (SwapState, Option<Sidecar>);
    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool>;
    fn finalized_amount(&self, swap: &mut Swap) -> anyhow::Result<Option<Balance>>;

    fn post_finalize_txns(&self, _swap: &mut Swap) -> anyhow::Result<Vec<Txn>> {
        Ok(Vec::new())
    }
}

pub trait Sender: Chain {
    fn send_txns(
        &self,
        swap: &mut Swap,
        received_amount: Balance,
    ) -> anyhow::Result<(SwapState, Vec<Txn>)>;
}

lazy_static::lazy_static! {
    static ref TEST: test::Test = test::Test;
    static ref SUBSTRATE: substrate::Substrate = substrate::Substrate::new(fractal_protocol_url());

    static ref RECEIVERS: Vec<&'static dyn Receiver> = vec![
        &*TEST,
        &*SUBSTRATE,
    ];

    static ref SENDERS: Vec<&'static dyn Sender> = vec![
        &*TEST,
        &*SUBSTRATE,
    ];
}

pub fn receivers() -> impl Iterator<Item = ReceiverRef> {
    RECEIVERS.iter().map(|&r| r)
}

pub fn senders() -> impl Iterator<Item = SenderRef> {
    SENDERS.iter().map(|&s| s)
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
