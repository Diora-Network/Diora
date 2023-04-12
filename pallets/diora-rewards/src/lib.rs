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


//! # Crowdloan Rewards Pallet
//!
//! This pallet issues rewards to citizens who participated in a crowdloan on the backing relay
//! chain (eg Kusama) in order to help this parachain acquire a parachain slot.
//!
//! ## Monetary Policy
//!
//! This is simple and mock for now. We can do whatever we want.
//! This pallet stores a constant  "reward ratio" which is the number of reward tokens to pay per
//! contributed token. In our cases this is "3",.
//! Vesting is also linear. No tokens are vested at genesis and they unlock linearly until a
//! predecided block number. Vesting computations happen on demand when payouts are requested. So
//! no block weight is ever wasted on this, and there is no "base-line" cost of updating vestings.
//! Like I said, we can anything we want there. Even a non-linear reward curve to disincentivize
//! whales.
//!
//! ## Payout Mechanism
//!
//! The current payout mechanism requires contributors to claim their payouts. Because they are
//! paying the transaction fees for this themselves, they can do it as often as every block, or
//! wait and claim the entire thing once it is fully vested. We could consider auto payouts if we
//! want.
//!
//! ## Sourcing Contribution Information
//!
//! The pallet can learn about the crowdloan contributions in several ways.
//!
//! * **Through the initialize_reward_vec extrinsic*
//!
//! The simplest way is to call the initialize_contributors_list through sudo call.
//! This makes sense in a scenario where the crowdloan took place entirely offchain.
//! This extrinsic initializes the associated and unassociated stoerage with the provided data
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod benchmarking;
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;
pub mod weights;
pub use weights::WeightInfo;


#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement},
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::{AccountIdConversion, AtLeast32BitUnsigned, BlockNumberProvider, Saturating},
        Perbill, SaturatedConversion,
    };
    use sp_std::{prelude::*, vec::Vec};
    use sp_core::H160;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // Diora Crowdloan rewards pallet
    #[pallet::pallet]
    #[pallet::without_storage_info]
    // The crowdloan rewards pallet
    pub struct Pallet<T>(PhantomData<T>);

    pub const PALLET_ID: PalletId = PalletId(*b"DioraRew");

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The currency in which the rewards will be paid (probably the parachain native currency)
        type Currency: Currency<Self::AccountId>;
        /// Check the contributor list and ending lease is already? default is false
        type Initialized: Get<bool>;
        /// tracking the vesting process
        type VestingBlockNumber: AtLeast32BitUnsigned + Parameter + Default + Into<BalanceOf<Self>>;
        /// The notion of time that will be used for vesting. Probably
        /// either the relay chain or sovereign chain block number.
        type VestingBlockProvider: BlockNumberProvider<BlockNumber = Self::VestingBlockNumber>;
        /// the first reward percentage of total reward
        type FirstVestPercentage: Get<Perbill>;
        /// this parameter control the max contributor list length
        #[pallet::constant]
        type MaxContributorsNumber: Get<u32>;
        /// runtime weights.
        type WeightInfo: WeightInfo;
    }

    /// Record the contributor's reward info
    /// - total_reward: the total reward balance based on the contribution(contribution * 3)
    /// - claimed_reward: the balance claimed by the current account
    /// - track_block_number: the block number where the reward was last claimed
    #[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct RewardInfo<T: Config> {
        pub total_reward: BalanceOf<T>,
        pub claimed_reward: BalanceOf<T>,
        pub track_block_number: T::VestingBlockNumber,
    }

    #[pallet::storage]
    #[pallet::storage_prefix = "InitBlock"]
    #[pallet::getter(fn init_vesting_block)]
    /// Vesting block height at the initialization of the pallet
    type InitVestingBlock<T: Config> = StorageValue<_, T::VestingBlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::storage_prefix = "EndBlock"]
    #[pallet::getter(fn end_vesting_block)]
    /// Vesting block height at the initialization of the pallet
    type EndVestingBlock<T: Config> = StorageValue<_, T::VestingBlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn initialized)]
    pub type Initialized<T: Config> = StorageValue<_, bool, ValueQuery, T::Initialized>;

    #[pallet::storage]
    #[pallet::getter(fn total_contributors)]
    /// store total number of contributors
    type TotalContributors<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Record contributor's info (total reward, claimed reward, track block number)
    #[pallet::storage]
    #[pallet::getter(fn rewards_info)]
    type ContributorsInfo<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, RewardInfo<T>, OptionQuery>;

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// complete the initialized
        InitializationIsCompleted,
        /// Invalid contributor account (not exist in contributor list)
        NotInContributorList,
        /// Not set the Ending lease block
        NotCompleteInitialization,
        /// Ending lease block setting invalid (should higher than the init block)
        InvalidEndingLeaseBlock,
        /// current account claimed all the reward, no reward left
        NoLeftRewards,
        /// too many contributors when put the contributor list into the storage
        TooManyContributors,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// update contributor's reward info (accountId, total reward, claimed reward)
        UpdateContributorsInfo(T::AccountId, BalanceOf<T>, BalanceOf<T>),
        /// distribute reward (source account, destination account, amount)
        DistributeReward(T::AccountId, T::AccountId, BalanceOf<T>),
        /// set the ending lease block
        EndleasingBlock(T::VestingBlockNumber),
    }

    // This hook is in charge of initializing the vesting height at the first block of the parachain
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: <T as frame_system::Config>::BlockNumber) {
            // record the block number of relaychain when our parachain launch it as the InitVestingBlock number
            // at this time, our reward computation is starting...
            if n == 1u32.into() {
                <InitVestingBlock<T>>::put(T::VestingBlockProvider::current_block_number());
            }
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The amount of funds in this pallet
        pub funded_amount: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                funded_amount: 1u32.into(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        // This sets the pre-funds of this Reward pallet(In DIORA, we set 0. this pallet's fund will transfered by the sudo account)
        fn build(&self) {
            T::Currency::deposit_creating(&Pallet::<T>::account_id(), self.funded_amount);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// contributors claim their rewards by this call
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::claim_rewards())]
        pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // ensure three things:
            // check current acccount is in contributor list
            // if exist, have the access to claim his reward
            let contribute_info =
                <ContributorsInfo<T>>::get(who.clone()).ok_or(Error::<T>::NotInContributorList)?;

            // ensure we have set the ending lease block and init the contributor list, which means we can start claiming
            let initialized = <Initialized<T>>::get();
            ensure!(initialized == true, <Error<T>>::NotCompleteInitialization);

            // if contributor's claimed reward reach his total reward, no rewards will distribute to them
            ensure!(
                contribute_info.claimed_reward < contribute_info.total_reward,
                <Error<T>>::NoLeftRewards,
            );
            // all the check passed, compute the reward will distribute.
            // compute the total linear reward block(ending lease block- init lease block)
            let total_reward_period = <EndVestingBlock<T>>::get() - <InitVestingBlock<T>>::get();

            // Get the current block used for vesting purposes
            let now = T::VestingBlockProvider::current_block_number();

            // Get the current block used for vesting purposes and check the current block number is in the lease.
            // if in the lease, the computation baseline is current blocknumber, otherwise is EndVestingBlock
            let track_now = if now >= <EndVestingBlock<T>>::get() {
                <EndVestingBlock<T>>::get()
            } else {
                T::VestingBlockProvider::current_block_number()
            };

            // the fist reward distributed to the contributor by the percentage(this percent currently is 20%) total reward
            // as you can see, if you contribute more, the more first reward you will claim
            let first_reward = T::FirstVestPercentage::get() * contribute_info.total_reward;

            // the linear reward indeed
            let left_linear_reward = contribute_info.total_reward.saturating_sub(first_reward);
            // compute the linear block period by the last tracked block number
            let curr_linear_reward_period = track_now
                .clone()
                .saturating_sub(contribute_info.track_block_number.clone());
            // compute the linear reward by the linear block period
            let current_linear_reward = left_linear_reward
                .saturating_mul(curr_linear_reward_period.into())
                / total_reward_period.into();

            // Get the comming reward
            let coming_reward = if contribute_info.claimed_reward == 0u32.into() {
                // if current user never claim the rewards, distribute `fisrt reward` + `current
                // linear block reward`.

                // update the claimed reward and track block number
                let new_contribute_info = RewardInfo {
                    total_reward: contribute_info.total_reward,
                    claimed_reward: first_reward + current_linear_reward,
                    track_block_number: now.clone(),
                };
                Self::update_contribute_info(who.clone(), new_contribute_info);
                first_reward + current_linear_reward
            } else {
                // if current user have got some rewards, but the lease is not ending, get the
                // latest linear block reward compute by the block period: now block number - last
                // track block number

                // if reach or higher the end lease block, the claimed reward < total reward, distribute the left reward
                if now >= <EndVestingBlock<T>>::get() {
                    let new_contribute_info = RewardInfo {
                        total_reward: contribute_info.total_reward,
                        claimed_reward: contribute_info.claimed_reward
                            + (contribute_info.total_reward - contribute_info.claimed_reward),
                        track_block_number: now.clone(),
                    };
                    Self::update_contribute_info(who.clone(), new_contribute_info);
                    contribute_info.total_reward - contribute_info.claimed_reward
                } else {
                    let new_contribute_info = RewardInfo {
                        total_reward: contribute_info.total_reward,
                        claimed_reward: contribute_info.claimed_reward + current_linear_reward,
                        track_block_number: now.clone(),
                    };
                    Self::update_contribute_info(who.clone(), new_contribute_info);
                    current_linear_reward
                }
            };

            // distribute comming reward to contributor
            Self::distribute_to_contributors(who.clone(), coming_reward.saturated_into::<u128>())?;
            Self::deposit_event(<Event<T>>::DistributeReward(
                Self::account_id(),
                who.clone(),
                coming_reward.saturated_into::<BalanceOf<T>>(),
            ));
            Ok(().into())
        }

        ///  Initialize contributor's rewards info which is a contributors vec
        ///  this operation should be execute by sudo user
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::initialize_contributors_list())]
        pub fn initialize_contributors_list(
            origin: OriginFor<T>,
            contributor_list: Vec<(T::AccountId, BalanceOf<T>)>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let initialized = <Initialized<T>>::get();
            ensure!(initialized == false, <Error<T>>::InitializationIsCompleted);
            // ensure the number don't exceed contributor list length
            ensure!(
                contributor_list.len() as u32 <= T::MaxContributorsNumber::get(),
                <Error<T>>::TooManyContributors,
            );

            // Total number of contributors
            let mut total_contributors = TotalContributors::<T>::get();

            // update the contributors list
            for (contributor_account, contribution_value) in &contributor_list {
                // compute contributor's total rewards
                let total_reward = (contribution_value.saturating_mul(3u32.into()))
                    .saturated_into::<BalanceOf<T>>();
                // initialize the contrbutor's rewards info
                // default: claimed_reward is 0, track_block_number is the InitVestingBlock
                let reward_info = RewardInfo {
                    total_reward,
                    claimed_reward: 0u128.saturated_into::<BalanceOf<T>>(),
                    track_block_number: <InitVestingBlock<T>>::get(),
                };
                // insert the contributor info
                <ContributorsInfo<T>>::insert(contributor_account.clone(), reward_info.clone());
                Self::deposit_event(Event::UpdateContributorsInfo(
                    contributor_account.clone(),
                    total_reward,
                    0u128.saturated_into::<BalanceOf<T>>(),
                ));
                // update the total contributors number
                total_contributors += 1;
                TotalContributors::<T>::put(total_contributors);
            }

            Ok(().into())
        }

        /// Update the ending lease block
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::complete_initialization())]
        pub fn complete_initialization(
            origin: OriginFor<T>,
            lease_ending_block: T::VestingBlockNumber,
        ) -> DispatchResult {
            // only sudo
            ensure_root(origin)?;
            let initialized = <Initialized<T>>::get();
            ensure!(initialized == false, <Error<T>>::InitializationIsCompleted);
            // ending lease block should higher than the init lease block
            ensure!(
                lease_ending_block > <InitVestingBlock<T>>::get(),
                <Error<T>>::InvalidEndingLeaseBlock,
            );

            <EndVestingBlock<T>>::put(lease_ending_block.clone());
            <Initialized<T>>::put(true);
            Self::deposit_event(<Event<T>>::EndleasingBlock(lease_ending_block.clone()));

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// get the pallet's account
        pub fn account_id() -> T::AccountId {
            PALLET_ID.into_account_truncating()
        }

        /// distributed by Pallet account
        pub fn distribute_to_contributors(
            contributor_account: T::AccountId,
            value: u128,
        ) -> DispatchResult {
            T::Currency::transfer(
                &Self::account_id(),
                &contributor_account,
                value.saturated_into(),
                ExistenceRequirement::AllowDeath,
            )?;
            Ok(().into())
        }

        /// update the contributor's info
        pub fn update_contribute_info(contributor: T::AccountId, reward_info: RewardInfo<T>) {
            // insert the contributor info
            <ContributorsInfo<T>>::insert(contributor.clone(), reward_info.clone());
            Self::deposit_event(Event::<T>::UpdateContributorsInfo(
                contributor.clone(),
                reward_info.total_reward,
                reward_info.claimed_reward,
            ));
        }
    }
}
