use fractal_protocol_blockchain_runtime::{
    AccountId, AuraConfig, BalancesConfig, FractalMintingConfig, GenesisConfig, GrandpaConfig,
    Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

use hex_literal::hex;
use sp_core::crypto::UncheckedInto;
use std::convert::TryInto;

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

pub fn live_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Main wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Live",
        // ID
        "live",
        ChainType::Live,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    (
                        AuraId::from_slice(&hex_literal::hex![
                            // 5FCLaubL8NeerKq8LPVYJPJe6Z8JP9ayCT5LSNUXaStmjShw
                            "8a870e7805780fe7e2d499a679c3465da86b04a2ec7e51379391a8d81326b467"
                        ]),
                        GrandpaId::from_slice(&hex_literal::hex![
                            // 5D8jfFQ7WZXTCyYLCRdojFMfztDq9qri4ZPAEZZRH22KhoCP
                            "2f4f48eda1a0e8dec6f2d5c4396d6565bda97c419b12af112183fe25e7d8fd43"
                        ]),
                    ),
                ],
                // Sudo account
                AccountId::new(hex_literal::hex![
                    // 5FCLsu6FTwi9cafruqVbEo8uUEqPHaf6oqNLhSTn5CkbrQ3o
                    "8a880aedd15c00bfa62220f014d12ff818534e99bd298c5c1a68ac221fc2471e"
                ]),
                // Pre-funded accounts
                vec![],
                FractalMintingConfig {
                    fractal_authoritative_account: AccountId::new(hex_literal::hex![
                        // 5FCLidfiL1wcTSXvecm1PrrP3N3jUxAgpdDGJQRAqA5pk1K3
                        "8a8781412df00f8be33f7428dfdcd467957a5142f1308b45036126443fc42635"
                    ]),
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

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

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
        None,
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
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    fractal_authoritative_account: AccountId,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: vec![],
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
        fractal_minting: FractalMintingConfig {
            fractal_authoritative_account,
        },
    }
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Mainnet",
        // ID
        "mainnet",
        ChainType::Custom("FCL Mainnet".into()),
        move || {
            mainnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                // https://www.shawntabrizi.com/substrate-js-utilities/
                //
                vec![(
                    // 5FCLaubL8NeerKq8LPVYJPJe6Z8JP9ayCT5LSNUXaStmjShw
                    hex!["8a870e7805780fe7e2d499a679c3465da86b04a2ec7e51379391a8d81326b467"]
                        .unchecked_into(),
                    // same key
                    hex!["8a870e7805780fe7e2d499a679c3465da86b04a2ec7e51379391a8d81326b467"]
                        .unchecked_into(),
                )],
                // Sudo account
                // 5FCLsu6FTwi9cafruqVbEo8uUEqPHaf6oqNLhSTn5CkbrQ3o
                hex!["8a880aedd15c00bfa62220f014d12ff818534e99bd298c5c1a68ac221fc2471e"].into(),
                // 5FCLidfiL1wcTSXvecm1PrrP3N3jUxAgpdDGJQRAqA5pk1K3
                hex!["8a8781412df00f8be33f7428dfdcd467957a5142f1308b45036126443fc42635"].into(),
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
