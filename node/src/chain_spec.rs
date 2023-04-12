// This file is part of Diora.

// Copyright (C) 2019-2022 Diora-Network.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use cumulus_primitives_core::ParaId;
use diora_runtime::{
	constants::currency::{DIOR, SUPPLY_FACTOR},
	AccountId, AuthorFilterConfig, AuthorMappingConfig, Balance, BalancesConfig, EligibilityValue,
	EthereumChainIdConfig, GenesisConfig, InflationInfo, NimbusId, ParachainInfoConfig,
	ParachainStakingConfig, Perbill, Range, Signature, SudoConfig, SystemConfig, HOURS,
	WASM_BINARY,
};
use hex_literal::hex;

use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Percent,
};

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
						250 * DIOR * SUPPLY_FACTOR,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_from_seed::<NimbusId>("Bob"),
						250 * DIOR * SUPPLY_FACTOR,
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
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
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
						250 * DIOR * SUPPLY_FACTOR,
					),
					(
						hex!["72d5cd6efc6c3338a03b86834c6dbc81e0878742a8039e52270d586945698b5f"]
							.into(),
						hex!["72d5cd6efc6c3338a03b86834c6dbc81e0878742a8039e52270d586945698b5f"]
							.unchecked_into(),
						250 * DIOR * SUPPLY_FACTOR,
					),
					(
						hex!["96d0d050dda960781a621678e34e69c5012f37ff9fcd6e1b8e6aed19a6a3402e"]
							.into(),
						hex!["96d0d050dda960781a621678e34e69c5012f37ff9fcd6e1b8e6aed19a6a3402e"]
							.unchecked_into(),
						250 * DIOR * SUPPLY_FACTOR,
					),
				],
				vec![
					// Candidates
					hex!["5ccd918cbf5e1d876641b5967b943a659e7a0ef415ca677ec457160a21a4ad7e"].into(),
					hex!["72d5cd6efc6c3338a03b86834c6dbc81e0878742a8039e52270d586945698b5f"].into(),
					hex!["96d0d050dda960781a621678e34e69c5012f37ff9fcd6e1b8e6aed19a6a3402e"].into(),
					// Faucet 5STim9aZszaS1GSj7QHniMBZb4qbCVJNXTnHZmSfiEgw9XB7
					hex!["7cd8f5644cdb7d1e70ba38587f968b61be9eb69198037b7359bc545e2231d9cd"].into(),
					// Developer
					hex!["aea48c27a7f703a7f8acedf15b43e8fcbad0b7846e5fe32a0b2b75cb81d75306"].into(),
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

const COLLATOR_COMMISSION: Perbill = Perbill::from_percent(20);
const PARACHAIN_BOND_RESERVE_PERCENT: Percent = Percent::from_percent(30);
const BLOCKS_PER_ROUND: u32 = 6 * HOURS;
const NUM_SELECTED_CANDIDATES: u32 = 8;
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
			min: 100_000 * DIOR * SUPPLY_FACTOR,
			ideal: 200_000 * DIOR * SUPPLY_FACTOR,
			max: 500_000 * DIOR * SUPPLY_FACTOR,
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
			code: WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(
				// 5G1h51S53gWS4ARhRjgNAW1jQcvX4gY2GP8G7DaBh6RmVsQC
				hex!["5ccd918cbf5e1d876641b5967b943a659e7a0ef415ca677ec457160a21a4ad7e"].into(),
			),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 50_00000 * DIOR)).collect(),
		},
		parachain_info: ParachainInfoConfig { parachain_id: id },
		parachain_system: Default::default(),
		ethereum_chain_id: EthereumChainIdConfig { chain_id: 201u64 },
		evm: Default::default(),
		ethereum: Default::default(),
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
			collator_commission: COLLATOR_COMMISSION,
			parachain_bond_reserve_percent: PARACHAIN_BOND_RESERVE_PERCENT,
			blocks_per_round: BLOCKS_PER_ROUND,
			num_selected_candidates: NUM_SELECTED_CANDIDATES,
		},
		author_mapping: AuthorMappingConfig {
			mappings: candidates
				.iter()
				.cloned()
				.map(|(account_id, author_id, _)| (author_id, account_id))
				.collect(),
		},
		author_filter: AuthorFilterConfig { eligible_count: EligibilityValue::default() },
	}
}
