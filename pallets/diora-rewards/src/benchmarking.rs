//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as DioraRewards;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec};
use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
    initialize_contributors_list {
        let alice: T::AccountId = account("alice", 0, SEED);
        let bob: T::AccountId = account("bob", 0, SEED);
        let list = vec![(alice, 100u32.into()), (bob, 300u32.into())];
    }: _(RawOrigin::Root, list)

    complete_initialization {
        let alice: T::AccountId = account("alice", 0, SEED);
        let bob: T::AccountId = account("bob", 0, SEED);
        let list = vec![(alice, 100u32.into()), (bob, 300u32.into())];
        let _ = DioraRewards::<T>::initialize_contributors_list(<T as frame_system::Config>::Origin::from(RawOrigin::Root), list);
    }: _(RawOrigin::Root, 300u32.into())

    claim_rewards {
        let alice: T::AccountId = account("alice", 0, SEED);
        let bob: T::AccountId = account("bob", 0, SEED);
        let list = vec![(alice.clone(), 100u32.into()), (bob.clone(), 300u32.into())];
        let _ = DioraRewards::<T>::initialize_contributors_list(<T as frame_system::Config>::Origin::from(RawOrigin::Root), list);
        let _ = DioraRewards::<T>::complete_initialization(<T as frame_system::Config>::Origin::from(RawOrigin::Root), 300u32.into());
    }: _(RawOrigin::Signed(alice))
}

impl_benchmark_test_suite!(DioraRewards, crate::mock::new_test_ext(), crate::mock::Test,);
