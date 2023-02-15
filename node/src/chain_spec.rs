use cumulus_primitives_core::ParaId;
use diora_runtime::{
    constants::currency::{DIR, SUPPLY_FACTOR},
    AccountId, AuthorFilterConfig, AuthorMappingConfig, Balance, BalancesConfig, BaseFeeConfig,
    DefaultBaseFeePerGas, EligibilityValue, EthereumChainIdConfig, GenesisConfig, InflationInfo,
    NimbusId, ParachainInfoConfig, ParachainStakingConfig, Perbill, Range, Signature, SudoConfig,
    SystemConfig, WASM_BINARY,
};
use hex_literal::hex;

use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_pair_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

pub fn diora_local_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DIOR".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::from_genesis(
        // Name
        "Diora Local Testnet",
        // ID
        "diora_local_testnet",
        ChainType::Local,
        move || {
            diora_genesis(
                // initial collators.
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_from_seed::<NimbusId>("Alice"),
                        250 * DIR * SUPPLY_FACTOR,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_from_seed::<NimbusId>("Bob"),
                        250 * DIR * SUPPLY_FACTOR,
                    ),
                ],
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
                4202.into(),
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some("diora-local"),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "rococo".into(), // You MUST set this to the correct network!
            para_id: 4202,
        },
    )
}

pub fn diora_rococo_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DIOR".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::from_genesis(
        // Name
        "Diora rococo",
        // ID
        "Diora rococo",
        ChainType::Live,
        move || {
            diora_genesis(
                // initial collators.
                vec![
                    (
                        hex!["5ccd918cbf5e1d876641b5967b943a659e7a0ef415ca677ec457160a21a4ad7e"]
                            .into(),
                        hex!["5ccd918cbf5e1d876641b5967b943a659e7a0ef415ca677ec457160a21a4ad7e"]
                            .unchecked_into(),
                        250 * DIR * SUPPLY_FACTOR,
                    ),
                    (
                        hex!["72d5cd6efc6c3338a03b86834c6dbc81e0878742a8039e52270d586945698b5f"]
                            .into(),
                        hex!["72d5cd6efc6c3338a03b86834c6dbc81e0878742a8039e52270d586945698b5f"]
                            .unchecked_into(),
                        250 * DIR * SUPPLY_FACTOR,
                    ),
                    (
                        hex!["96d0d050dda960781a621678e34e69c5012f37ff9fcd6e1b8e6aed19a6a3402e"]
                            .into(),
                        hex!["96d0d050dda960781a621678e34e69c5012f37ff9fcd6e1b8e6aed19a6a3402e"]
                            .unchecked_into(),
                        250 * DIR * SUPPLY_FACTOR,
                    ),
                ],
                vec![
                    hex!["5ccd918cbf5e1d876641b5967b943a659e7a0ef415ca677ec457160a21a4ad7e"].into(),
                    hex!["72d5cd6efc6c3338a03b86834c6dbc81e0878742a8039e52270d586945698b5f"].into(),
                    hex!["96d0d050dda960781a621678e34e69c5012f37ff9fcd6e1b8e6aed19a6a3402e"].into(),
                ],
                4202.into(),
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some("diora-rococo"),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "rococo".into(), // You MUST set this to the correct network!
            para_id: 4202,
        },
    )
}
pub fn diora_inflation_config() -> InflationInfo<Balance> {
    let annual = Range {
        min: Perbill::from_percent(4),
        ideal: Perbill::from_percent(5),
        max: Perbill::from_percent(5),
    };
    let round = Range {
        min: Perbill::from_percent(0),
        ideal: Perbill::from_percent(0),
        max: Perbill::from_percent(0),
    };
    InflationInfo {
        // staking expectations
        expect: Range {
            min: 100_000 * DIR * SUPPLY_FACTOR,
            ideal: 200_000 * DIR * SUPPLY_FACTOR,
            max: 500_000 * DIR * SUPPLY_FACTOR,
        },
        // annual inflation
        annual,
        round,
    }
}

fn diora_genesis(
    candidates: Vec<(AccountId, NimbusId, Balance)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 50_00000 * DIR))
                .collect(),
        },
        parachain_info: ParachainInfoConfig { parachain_id: id },
        parachain_system: Default::default(),
        ethereum_chain_id: EthereumChainIdConfig { chain_id: 201u64 },
        evm: Default::default(),
        ethereum: Default::default(),
        base_fee: BaseFeeConfig::new(
            DefaultBaseFeePerGas::get(),
            false,
            sp_runtime::Permill::from_parts(125_000),
        ),
        council: Default::default(),
        democracy: Default::default(),
        technical_committee: Default::default(),
        treasury: Default::default(),
        parachain_staking: ParachainStakingConfig {
            candidates: candidates
                .iter()
                .cloned()
                .map(|(account, _, bond)| (account, bond))
                .collect(),
            delegations: vec![],
            inflation_config: diora_inflation_config(),
        },
        author_mapping: AuthorMappingConfig {
            mappings: candidates
                .iter()
                .cloned()
                .map(|(account_id, author_id, _)| (author_id, account_id))
                .collect(),
        },
        author_filter: AuthorFilterConfig {
            eligible_count: EligibilityValue::default(),
        },
    }
}
