use super::{evm, Balance, ChainInfo, PaymentRequest, Sidecar, Swap, SwapState};

mod evm_burner;
mod evm_mintable;
mod substrate;

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
    static ref SUBSTRATE: substrate::Substrate = substrate::Substrate::new(
        env_or("SUBSTRATE_CHAIN_URL", "wss://main.devnet.fractalprotocol.com:443"),
        env_or("SUBSTRATE_MINTING_KEY", "//Alice"),
    );

    static ref GNOSIS: evm::Chain = gnosis_chain().unwrap();

    static ref GNOSIS_SENDER: evm_mintable::EvmMintable = evm_mintable::EvmMintable::new(
        &*GNOSIS,
        env_or("GNOSIS_EXPLORER_URL", "https://blockscout.com/xdai/mainnet"),
        env_or(
            "GNOSIS_FCL_MINTER_KEY",
            // Known account 1
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
        ),
    ).unwrap();

    static ref GNOSIS_RECEIVER: evm_burner::EvmBurner = evm_burner::EvmBurner::new(
        &*GNOSIS,
        env_or("GNOSIS_CONFIRMATIONS_REQUIRED", "3"),
    ).unwrap();

    static ref ETHEREUM: evm::Chain = ethereum_chain().unwrap();

    static ref ETHEREUM_RECEIVER: evm_burner::EvmBurner = evm_burner::EvmBurner::new(
        &*ETHEREUM,
        env_or("ETHEREUM_CONFIRMATIONS_REQUIRED", "3"),
    ).unwrap();

    static ref RECEIVERS: Vec<&'static dyn Receiver> = vec![
        &*SUBSTRATE,
        &*GNOSIS_RECEIVER,
        &*ETHEREUM_RECEIVER,
    ];

    static ref SENDERS: Vec<&'static dyn Sender> = vec![
        &*SUBSTRATE,
        &*GNOSIS_SENDER,
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
            can_bridge_to: vec![
                String::from("substrate"),
            ],
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

fn gnosis_chain() -> anyhow::Result<evm::Chain> {
    Ok(evm::Chain {
        info: ChainInfo {
            id: "gnosis".to_string(),
            name: "Gnosis".to_string(),
            can_bridge_to: vec![
                String::from("substrate"),
            ],
        },
        chain_id: env_or("GNOSIS_CHAIN_ID", "100").parse()?,

        burner_contract: env_or(
            "GNOSIS_BURNER_ADDRESS",
            "0x265B056E3Ec5fDC08FB79d37cc9a2551d1c1c231",
        )
        .trim_start_matches("0x")
        .parse()?,
        token_contract: env_or(
            "GNOSIS_FCL_TOKEN_ADDRESS",
            "0xb2B90d3C7A9EB291c4fA06cFc1EFE5AdDdCa7FD4",
        )
        .trim_start_matches("0x")
        .parse()?,

        web3: web3::Web3::new(web3::transports::Http::new(&env_or(
            "GNOSIS_URL",
            "https://rpc.gnosischain.com",
        ))?),

        decimals: env_or("GNOSIS_FCL_TOKEN_DECIMALS", "18").parse()?,
    })
}

fn ethereum_chain() -> anyhow::Result<evm::Chain> {
    Ok(evm::Chain {
        info: ChainInfo {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            can_bridge_to: vec![
                String::from("gnosis"),
                String::from("substrate")
            ],
        },
        chain_id: env_or("ETHEREUM_CHAIN_ID", "1").parse()?,

        burner_contract: env_or(
            "ETHEREUM_BURNER_ADDRESS",
            "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0",
        )
        .trim_start_matches("0x")
        .parse()?,
        token_contract: env_or(
            "ETHEREUM_FCL_TOKEN_ADDRESS",
            "0x5FbDB2315678afecb367f032d93F642f64180aa3",
        )
        .trim_start_matches("0x")
        .parse()?,

        web3: web3::Web3::new(web3::transports::Http::new(&env_or(
            "ETHEREUM_URL",
            "http://127.0.0.1:8545",
        ))?),

        decimals: env_or("ETHEREUM_FCL_TOKEN_DECIMALS", "18").parse()?,
    })
}
