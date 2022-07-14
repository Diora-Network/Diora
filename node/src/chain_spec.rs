use cumulus_primitives_core::ParaId;
use diora_runtime::{
    constants::currency::{DIR, SUPPLY_FACTOR},
    AccountId, AuthorFilterConfig, Balance, BalancesConfig, BaseFeeConfig, DefaultBaseFeePerGas,
    EligibilityValue, EthereumChainIdConfig, GenesisConfig, InflationInfo, NimbusId,
    ParachainInfoConfig, ParachainStakingConfig, Perbill, PotentialAuthorSetConfig, Range,
    Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public, H160, U256};
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

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> NimbusId {
    get_pair_from_seed::<NimbusId>(seed)
}

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

pub fn development_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DIR".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                // initial collators.
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
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
                2000.into(),
            )
        },
        vec![],
        None,
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "rococo".into(), // You MUST set this to the correct network!
            para_id: 2000,
        },
    )
}

pub fn testnet_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DIR".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::from_genesis(
        // Name
        "Testnet",
        // ID
        "testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                // initial collators.
                vec![
                    (
                        H160::from(hex_literal::hex!["e782fE6487d55904244A955775da4662220Bb2AB"]),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        H160::from(hex_literal::hex!["3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"]),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                vec![
			                H160::from(hex_literal::hex!["e782fE6487d55904244A955775da4662220Bb2AB"]),
					H160::from(hex_literal::hex!["3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"]),
					H160::from(hex_literal::hex!["798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc"]),
                                        H160::from(hex_literal::hex!["773539d4Ac0e786233D90A233654ccEE26a613D9"]),
                ],
                4000.into(),
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
            para_id: 4000,
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
fn testnet_genesis(
    authorities: Vec<(AccountId, NimbusId)>,
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
                .map(|k| (k, 500_000 * DIR))
                .collect(),
        },
        parachain_info: ParachainInfoConfig { parachain_id: id },
        author_filter: AuthorFilterConfig {
            eligible_count: EligibilityValue::default(),
        },
        potential_author_set: PotentialAuthorSetConfig {
            mapping: authorities,
        },
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
        // author_mapping: Default::default(),
        parachain_staking: ParachainStakingConfig {
                           // Alice -> Alith
                (
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_from_seed::<NimbusId>("Alice"),
                    1_000 * DIR * SUPPLY_FACTOR,
                ),
                // Bob -> Baltithar
                (
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_from_seed::<NimbusId>("Bob"),
                    1_000 * DIR * SUPPLY_FACTOR,
                ),
            ]
            .iter()
            .cloned()
            .map(|(account, _, bond)| (account, bond))
            .collect(),
            delegations: vec![],
            inflation_config: diora_inflation_config(),
        },
    }
}
