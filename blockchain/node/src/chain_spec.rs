use fractal_protocol_blockchain_runtime::{
    AccountId, AuraConfig, BalancesConfig, FractalMintingConfig, GenesisConfig, GrandpaConfig,
    Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

use hex_literal::hex;
use sp_core::crypto::UncheckedInto;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn fractal_properties() -> Properties {
    let mut p = Properties::new();
    // p.insert("ss58format".into(), <number>.into());
    p.insert("tokenDecimals".into(), 12.into());
    p.insert("tokenSymbol".into(), "FCL".into());
    p
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    let props = fractal_properties();
    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                FractalMintingConfig {
                    fractal_authoritative_account: get_account_id_from_seed::<sr25519::Public>(
                        "Ferdie",
                    ),
                },
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(props),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                FractalMintingConfig {
                    fractal_authoritative_account: get_account_id_from_seed::<sr25519::Public>(
                        "Ferdie",
                    ),
                },
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    fractal_minting: FractalMintingConfig,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        pallet_aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        pallet_grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        pallet_sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        fractal_minting,
    }
}

/// FCL
fn mainnet_genesis(
    wasm_binary: &[u8],
    initial_aura_authorities: Vec<AuraId>,
    initial_grandpa_authorities: Vec<(GrandpaId, u64)>,
    root_key: AccountId,
    fractal_authoritative_account: AccountId,
    seeded_accounts: Vec<(AccountId, Balance)>,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: BalancesConfig {
            balances: seeded_accounts,
        },
        pallet_aura: AuraConfig {
            authorities: initial_aura_authorities,
        },
        pallet_grandpa: GrandpaConfig {
            authorities: initial_grandpa_authorities,
        },
        pallet_sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        fractal_minting: FractalMintingConfig {
            fractal_authoritative_account,
        },
    }
}

/// Balance of an account.
pub type Balance = u128;
pub const UNIT_BALANCE: Balance = 1_000_000_000_000;

pub fn mainnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?;
    let props = fractal_properties();

    Ok(ChainSpec::from_genesis(
        // Name
        "FCL Mainnet",
        // ID
        "fcl_mainnet",
        ChainType::Live,
        move || {
            mainnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    // 5FCLauyC2GbRz1WLANFLYuUvfS4RoRqeBFgDFxUTpZ2Ek61d
                    hex!["8a870e90bd2411e7a734680ebda3ddb2b9c9432c77766cf2cf8b9e999e433543"]
                        .unchecked_into(),
                    // 5FCLauYR7adiBYHXjV9KFCNSFcVhYJfgrAuhwtK3iFDgwBHm
                    hex!["8a870e74b9f94bf5d055c9fec5ec95f589be647ad3da500a39fbdc95a452042e"]
                        .unchecked_into(),
                    // 5FCLauXLWS1pvawBGyRSn8ymmdrGfc7bYLEJcfEkrGPWQojQ
                    hex!["8a870e738186c9ab6bdad0982569dcf91fc54ecd764cf5fe94e599b4da6b3b79"]
                        .unchecked_into(),
                ],
                vec![
                    // 5FCLgrebZjCAhqCsYSf8sFjHaYMLHw1zhZQKGK4zZPCm7ZSp
                    (
                        hex!["8a8766dd17f2e9fd00d27767b43aae9279934f36b9a76320a6bc0841c614cc9a"]
                            .unchecked_into(),
                        1,
                    ),
                    // 5FCLgrungfcxZyHPQbn2s5V8twDJ8oGGVKiSDWjeXG3SHfXz
                    (
                        hex!["8a8766ee45586978246297444a92beb0d5cfae8b92351766db869666bc0c8538"]
                            .unchecked_into(),
                        1,
                    ),
                    // 5FCLgrQcKX43W5nSPZ9WnTCX2EosYMdxQK238RVuBzVHvW2C
                    (
                        hex!["8a8766cd4746732e0f4b13d70c541bf8a4e31f895d7915f1d8211ddfc7000341"]
                            .unchecked_into(),
                        1,
                    ),
                ],
                // Sudo account
                // 5FCLsuKWKGLDMDUmTke8pbBTDqwxmFri4z6utanCHno1WxJH
                hex!["8a880afcce745e98eb27dd016eb9a050a0c73097f91768fdffe6f3c99fca8f6d"].into(),
                // Fractal authoritative account
                // 5FCLidfDWqbDHZNbqGzAiTLJTY3zLrYZEor9MDUWXiD3hE5E
                hex!["8a8781409e238a4b2a794fb7decee78007a8adee55ecf7888ed49b4ed64e4425"].into(),
                vec![(
                    // Fractal bridge account seeded with the 5 million units burned from ERC-20.
                    // 5FCLbrPcvVNBvKkiwEa8DQxyGTW4BPBVaDG2Sq3LRJv5y59x
                    hex!["8a871c81b98ef76617dcfe3bdc5af1b150f20553094bd98dce9da8eb4048655d"].into(),
                    5_000_000 * UNIT_BALANCE,
                )],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(props),
        // Extensions
        None,
    ))
}
