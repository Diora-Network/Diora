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


//! Test utilities

use crate::{self as pallet_diora_rewards, Config};
use cumulus_primitives_core::relay_chain::BlockNumber as RelayChainBlockNumber;
use cumulus_primitives_core::PersistedValidationData;
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::{
    construct_runtime,
    dispatch::UnfilteredDispatchable,
    inherent::{InherentData, ProvideInherent},
    parameter_types,
    traits::{ConstBool, ConstU32, GenesisBuild, Nothing, OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;
use sp_core::H256;
use sp_io;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use sp_std::convert::{From, TryInto};

pub type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        DioraRewards: pallet_diora_rewards::{Pallet, Call, Storage, Event<T>, Config<T>},
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
    }
);

parameter_types! {
    pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}

impl cumulus_pallet_parachain_system::Config for Test {
    type SelfParaId = ParachainId;
    type RuntimeEvent = RuntimeEvent;
    type OnSystemEvent = ();
    type OutboundXcmpMessageSource = ();
    type XcmpMessageHandler = ();
    type ReservedXcmpWeight = ();
    type DmpMessageHandler = ();
    type ReservedDmpWeight = ();
    type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Nothing;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type BlockNumber = u64;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type MaxLocks = ();
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const TestFirstVestPercentage: Perbill = Perbill::from_percent(20);
    pub const TestMaxContributorsNumber: u32 = 5;
}

// diora reward pallet config
impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Initialized = ConstBool<false>;
    type FirstVestPercentage = TestFirstVestPercentage;
    type MaxContributorsNumber = TestMaxContributorsNumber;
    type VestingBlockNumber = RelayChainBlockNumber;
    type VestingBlockProvider =
        cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
    type WeightInfo = ();
}

fn genesis(funded_amount: Balance) -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_diora_rewards::GenesisConfig::<Test> { funded_amount }
        .assimilate_storage(&mut storage)
        .expect("Pallet balances storage can be assimilated");

    let mut ext = sp_io::TestExternalities::from(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

// Genesis initialize 10000 DIORA
pub(crate) fn empty() -> sp_io::TestExternalities {
    genesis(10000u32.into())
}

pub(crate) fn roll_to(n: u64) {
    while System::block_number() < n {
        // Relay chain Stuff. I might actually set this to a number different than N
        let sproof_builder = RelayStateSproofBuilder::default();
        let (relay_parent_storage_root, relay_chain_state) =
            sproof_builder.into_state_root_and_proof();
        let vfp = PersistedValidationData {
            relay_parent_number: (System::block_number() + 1u64) as RelayChainBlockNumber,
            relay_parent_storage_root,
            ..Default::default()
        };
        let inherent_data = {
            let mut inherent_data = InherentData::default();
            let system_inherent_data = ParachainInherentData {
                validation_data: vfp.clone(),
                relay_chain_state,
                downward_messages: Default::default(),
                horizontal_messages: Default::default(),
            };
            inherent_data
                .put_data(
                    cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
                    &system_inherent_data,
                )
                .expect("failed to put VFP inherent");
            inherent_data
        };

        ParachainSystem::on_initialize(System::block_number());
        ParachainSystem::create_inherent(&inherent_data)
            .expect("got an inherent")
            .dispatch_bypass_filter(RawOrigin::None.into())
            .expect("dispatch succeeded");
        ParachainSystem::on_finalize(System::block_number());

        DioraRewards::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        DioraRewards::on_initialize(System::block_number());
    }
}
