use super::{evm, Balance, ChainInfo, PaymentRequest, Sidecar, Swap, SwapState};

mod evm_burner;
mod evm_mintable;
mod substrate;
mod test;

pub type ReceiverRef = &'static dyn Receiver;
pub type SenderRef = &'static dyn Sender;

pub trait Chain: Sync + Send {
    fn info(&self) -> ChainInfo;
}

/// For all these methods, the swaps are still persisted even if the function returns `Err(_)`.
pub trait Receiver: Chain {
    fn create_receive_request(&self, id: &str) -> (SwapState, Option<Sidecar>);
    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool>;
    fn finalized_amount(&self, swap: &mut Swap) -> anyhow::Result<Option<Balance>>;

    /// This is potentially called more than once (if machines fail or sending fails).
    /// Therefore, it must be idempotent.
    fn after_finalized(&self, swap: &mut Swap, amount: Balance) -> anyhow::Result<()>;
}

pub trait Sender: Chain {
    /// This is potentially called more than once (if machines fail or sending fails).
    /// Therefore, it must be idempotent.
    fn send(&self, swap: &mut Swap, amount: Balance) -> anyhow::Result<SwapState>;

    fn is_valid(&self, address: &str) -> bool;
}

lazy_static::lazy_static! {
    static ref TEST: test::Test = test::Test;
    static ref SUBSTRATE: substrate::Substrate = substrate::Substrate::new(
        env_or("SUBSTRATE_CHAIN_URL", "wss://main.devnet.fractalprotocol.com:443"),
        env_or("SUBSTRATE_MINTING_KEY", "//Alice"),
    );

    static ref ACALA: evm::Chain = acala_chain().unwrap();

    static ref ACALA_SENDER: evm_mintable::EvmMintable = evm_mintable::EvmMintable::new(
        &*ACALA,
        env_or("ACALA_EXPLORER_URL", "http://acala.subscan.io"),
        env_or(
            "ACALA_FCL_MINTER_KEY",
            // Known account 1
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
        ),
    ).unwrap();

    static ref ACALA_RECEIVER: evm_burner::EvmBurner = evm_burner::EvmBurner::new(
        &*ACALA,
        env_or("ACALA_CONFIRMATIONS_REQUIRED", "12"),
    ).unwrap();

    static ref RECEIVERS: Vec<&'static dyn Receiver> = vec![
        &*TEST,
        &*SUBSTRATE,
        &*ACALA_RECEIVER,
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

fn env_or(env: &str, fallback: &str) -> String {
    std::env::var(env).unwrap_or_else(|_| fallback.to_string())
}

fn acala_chain() -> anyhow::Result<evm::Chain> {
    Ok(evm::Chain {
        info: ChainInfo {
            id: "acala".to_string(),
            name: "Acala".to_string(),
        },
        chain_id: env_or("ACALA_CHAIN_ID", "31337").parse()?,

        burner_contract: env_or(
            "ACALA_BURNER_ADDRESS",
            "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0",
        )
        .trim_start_matches("0x")
        .parse()?,
        token_contract: env_or(
            "ACALA_FCL_TOKEN_ADDRESS",
            "0x5FbDB2315678afecb367f032d93F642f64180aa3",
        )
        .trim_start_matches("0x")
        .parse()?,

        web3: web3::Web3::new(web3::transports::Http::new(&env_or(
            "ACALA_URL",
            "http://127.0.0.1:8545",
        ))?),

        decimals: env_or("ACALA_FCL_TOKEN_DECIMALS", "18").parse()?,
    })
}
