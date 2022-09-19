use super::{Balance, ChainInfo, Sidecar, Swap, SwapState};

mod evm_mintable;
mod substrate;
mod test;

pub type ReceiverRef = &'static dyn Receiver;
pub type SenderRef = &'static dyn Sender;

pub trait Chain: Sync + Send {
    fn info(&self) -> ChainInfo;
}

pub trait Receiver: Chain {
    fn create_receive_request(&self) -> (SwapState, Option<Sidecar>);
    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool>;
    fn finalized_amount(&self, swap: &mut Swap) -> anyhow::Result<Option<Balance>>;

    // This is potentially called more than once (if machines fail or sending fails).
    // Therefore, it must be idempotent.
    fn after_finalized(&self, swap: &mut Swap, amount: Balance) -> anyhow::Result<()>;
}

pub trait Sender: Chain {
    fn send(&self, swap: &mut Swap, amount: Balance) -> anyhow::Result<SwapState>;
}

lazy_static::lazy_static! {
    static ref TEST: test::Test = test::Test;
    static ref SUBSTRATE: substrate::Substrate = substrate::Substrate::new(
        fractal_protocol_url(),
        fractal_protocol_minting_key(),
    );
    static ref ACALA_SENDER: evm_mintable::EvmMintable = evm_mintable::EvmMintable::new(
        acala_url(),
        acala_explorer_url(),
        acala_fcl_token_address(),
        acala_fcl_minter_key(),
        ChainInfo {
            name: "Acala".to_string(),
            id: "acala_sender".to_string(),
        },
    ).unwrap();

    static ref RECEIVERS: Vec<&'static dyn Receiver> = vec![
        &*TEST,
        &*SUBSTRATE,
    ];

    static ref SENDERS: Vec<&'static dyn Sender> = vec![
        &*TEST,
        &*SUBSTRATE,
        &*ACALA_SENDER,
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
    std::env::var("SUBSTRATE_CHAIN_URL").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string())
}

fn fractal_protocol_minting_key() -> String {
    std::env::var("SUBSTRATE_MINTING_KEY").unwrap_or_else(|_| "//Alice".to_string())
}

fn acala_url() -> String {
    std::env::var("ACALA_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string())
}

fn acala_explorer_url() -> String {
    std::env::var("ACALA_EXPLORER_URL").unwrap_or_else(|_| "http://acala.subscan.io".to_string())
}

fn acala_fcl_token_address() -> String {
    std::env::var("ACALA_FCL_TOKEN_ADDRESS")
        .unwrap_or_else(|_| "0x5FbDB2315678afecb367f032d93F642f64180aa3".to_string())
}

fn acala_fcl_minter_key() -> String {
    std::env::var("ACALA_FCL_MINTER_KEY").unwrap_or_else(|_| {
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_string()
    })
}
