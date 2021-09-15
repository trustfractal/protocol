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
        "FCL Mainnet",
        // ID
        "fcl_mainnet",
        ChainType::Live,
        move || {
            mainnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                // https://www.shawntabrizi.com/substrate-js-utilities/
                vec![
                    (
                        // 5FCLaubL8NeerKq8LPVYJPJe6Z8JP9ayCT5LSNUXaStmjShw
                        hex!["8a870e7805780fe7e2d499a679c3465da86b04a2ec7e51379391a8d81326b467"]
                            .unchecked_into(),
                        // 5FCLgrPgkUbrCEnvdCCrkNNExsQyznTyuVrheQhpdPbSs29j
                        hex!["8a8766cc3bece48c8e014f896a6d60183b2db6d41896d590ff0e4c004b10bb9c"]
                            .unchecked_into(),
                    ),
                    (
                        // 5FCLauianC9jJitTqpWmNDMp677FkirT4xojK7GLQ6YERgQX
                        hex!["8a870e8038d0504ea60a980c72e89896252b969e98d769002a01bf00e26b1212"]
                            .unchecked_into(),
                        // 5FCLgrr1N9HWMJnLicNyUoKCrGE99csRwDYyyUD6C9xwzjm4
                        hex!["8a8766e9feca3ed817e8355b967a7e151489e67011b0204ba3d96bb23dd609d3"]
                            .unchecked_into(),
                    ),
                    (
                        // 5FCLauc7k38LpiA6JJKid2n7r3JjkH2EFC2FqmEXi7tLhUBP
                        hex!["8a870e78e91e6ba9a8068a6e04c65bae0b9469cae549c5ad9d1fb91813fdb235"]
                            .unchecked_into(),
                        // 5FCLgr3aRRHCJLXeoH5vbz7NGuQ3E3hLg44KPn4LaRMb7ub9
                        hex!["8a8766b57f2391f4f6e2fed09b1ea8fde72948f988494ec39e91562812c5dee5"]
                            .unchecked_into(),
                    ),
                ],
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
