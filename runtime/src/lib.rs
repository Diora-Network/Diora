#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{Decode, Encode};
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
    transaction_validity::{TransactionSource, TransactionValidity, TransactionValidityError},
    ApplyExtrinsicResult, MultiSignature, Percent,
};

pub use nimbus_primitives::{CanAuthor, NimbusId};
pub use pallet_author_slot_filter::EligibilityValue;
pub use parachain_staking::{inflation, InflationInfo, Range};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
    construct_runtime, match_types, parameter_types,
    traits::{
        ConstBool, ConstU128, ConstU32, EnsureOneOf, EqualPrivilegeOnly, Everything, OnUnbalanced,
    },
    weights::{
        constants::{RocksDbWeight, WEIGHT_PER_SECOND},
        ConstantMultiplier, IdentityFee, Weight,
    },
    PalletId,
};
use frame_system::EnsureRoot;
use pallet_balances::NegativeImbalance;

pub use sp_runtime::{MultiAddress, Perbill, Permill, RuntimeDebug};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot Imports
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

// XCM Imports
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter,
    EnsureXcmOrigin, FixedWeightBounds, IsConcrete, LocationInverter, NativeAsset,
    ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
    SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
};
use xcm_executor::{Config, XcmExecutor};

mod pallet_account_set;

/// Import the template pallet.
pub use pallet_template;

pub mod constants;
pub use constants::currency::*;

// EVM
use fp_rpc::TransactionStatus;
use frame_support::traits::Imbalance;
use pallet_ethereum::{Call::transact, Transaction as EthereumTransaction};
use pallet_evm::{
    Account as EVMAccount, EnsureAddressNever, EnsureAddressRoot, FeeCalculator,
    HashedAddressMapping, Runner,
};
use sp_core::{H160, H256, U256};
use sp_runtime::traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf};

mod precompiles;
pub use precompiles::DioraPrecompiles;

pub use session_keys_primitives::*;

pub use nimbus_primitives::AccountLookup;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    fp_self_contained::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = fp_self_contained::CheckedExtrinsic<AccountId, Call, SignedExtra, H160>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsReversedWithSystemFirst,
    OnRuntimeUpgrade,
>;

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
        UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        )
    }
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(
        &self,
        transaction: pallet_ethereum::Transaction,
    ) -> opaque::UncheckedExtrinsic {
        let extrinsic = UncheckedExtrinsic::new_unsigned(
            pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
        );
        let encoded = extrinsic.encode();
        opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
            .expect("Encoded extrinsic is always valid")
    }
}

impl fp_self_contained::SelfContainedCall for Call {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            Call::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<Call>,
        len: usize,
    ) -> Option<TransactionValidity> {
        match self {
            Call::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<Call>,
        len: usize,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.pre_dispatch_self_contained(info, dispatch_info, len),
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
        match self {
            call @ Call::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(call.dispatch(
                Origin::from(pallet_ethereum::RawOrigin::EthereumTransaction(info)),
            )),
            _ => None,
        }
    }
}

pub struct OnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for OnRuntimeUpgrade {
    fn on_runtime_upgrade() -> u64 {
        frame_support::migrations::migrate_from_pallet_version_to_storage_version::<
            AllPalletsWithSystem,
        >(&RocksDbWeight::get())
    }
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    use sp_runtime::{generic, traits::BlakeTwo256};

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        //TODO this was called author_inherent in the old runtime.
        // Can I just rename it like this?
        pub nimbus: AuthorInherent,
        pub vrf: session_keys_primitives::VrfSessionKey,
    }
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("diora-parachain"),
    impl_name: create_runtime_str!("diora-parachain"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u16 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Runtime version.
    type Version = Version;
    /// Converts a module to an index of this module in the runtime.
    type PalletInfo = PalletInfo;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = Everything;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// The action to take on a Runtime Upgrade
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 0;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const TransactionByteFee: Balance = TRANSACTION_BYTE_FEE;
    pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction =
        pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees<Runtime>>;
    // type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type WeightToFee = constants::fee::WeightToFee;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
}
parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 7 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}
type CouncilInstance = pallet_collective::Instance1;
type TechCommitteeInstance = pallet_collective::Instance2;
impl pallet_collective::Config<CouncilInstance> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
    pub const TechnicalMaxProposals: u32 = 100;
    pub const TechnicalMaxMembers: u32 = 100;
}
impl pallet_collective::Config<TechCommitteeInstance> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = TechnicalMotionDuration;
    type MaxProposals = TechnicalMaxProposals;
    type MaxMembers = TechnicalMaxMembers;
    type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
}
impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type PreimageProvider = ();
    type NoPreimagePostponement = ();
}

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 7 * DAYS;
    pub const VotingPeriod: BlockNumber = 14 * DAYS;
    pub const FastTrackVotingPeriod: BlockNumber = 1 * DAYS;
    pub const InstantAllowed: bool = true;
    pub const MinimumDeposit: Balance = 4 * DIR * SUPPLY_FACTOR;
    pub const EnactmentPeriod: BlockNumber = 2 * DAYS;
    pub const CooloffPeriod: BlockNumber = 7 * DAYS;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = STORAGE_BYTE_FEE;
    pub const MaxVotes: u32 = 100;
    pub const MaxProposals: u32 = 100;
}
impl pallet_democracy::Config for Runtime {
    type Proposal = Call;
    type Event = Event;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 1, 2>;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 1, 2>;
    type InstantOrigin =
        pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 3, 5>;
    type InstantAllowed = InstantAllowed;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    type CancellationOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>,
    >;
    type CancelProposalOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 3, 5>,
    >;
    type BlacklistOrigin = EnsureRoot<AccountId>;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cooloff period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechCommitteeInstance>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilInstance>;
    type Slash = ();
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = MaxVotes;
    type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
    type MaxProposals = MaxProposals;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const SpendPeriod: BlockNumber = 6 * DAYS;
    pub const TreasuryPalletId: PalletId = PalletId(*b"pcx/trsy");
    pub const MaxApprovals: u32 = 100;
}
impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>,
    >;
    type RejectOrigin = EnsureOneOf<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
    >;
    type Event = Event;
    type OnSlash = Treasury;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ();
    type SpendPeriod = SpendPeriod;
    type Burn = ();
    type BurnDestination = ();
    type SpendFunds = ();
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
    type ProposalBondMaximum = ();
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnSystemEvent = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl parachain_info::Config for Runtime {}

parameter_types! {
    pub const RocLocation: MultiLocation = MultiLocation::parent();
    pub const RelayNetwork: NetworkId = NetworkId::Any;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the parent `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = CurrencyAdapter<
    // Use this currency:
    Balances,
    // Use this currency when it is a fungible asset matching the given location or name:
    IsConcrete<RocLocation>,
    // Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
    LocationToAccountId,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We don't track any teleports.
    (),
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognised.
    RelayChainAsNative<RelayChainOrigin, Origin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognised.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    // Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
    // transaction from the Root origin.
    ParentAsSuperuser<Origin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<Origin>,
);

parameter_types! {
    // One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
    pub UnitWeightCost: Weight = 1_000_000_000;
    pub const MaxInstructions: u32 = 100;
}

match_types! {
    pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Here } |
        MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
    };
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
    // ^^^ Parent and its exec plurality get free execution
);

pub struct XcmConfig;
impl Config for XcmConfig {
    type Call = Call;
    type XcmSender = XcmRouter;
    // How to withdraw and deposit an asset.
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type IsReserve = NativeAsset;
    type IsTeleporter = NativeAsset; // Should be enough to allow teleportation of ROC
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type Trader = UsingComponents<IdentityFee<Balance>, RocLocation, AccountId, Balances, ()>;
    type ResponseHandler = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
}

parameter_types! {
    pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Everything;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type Origin = Origin;
    type Call = Call;

    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = ();
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

impl pallet_author_inherent::Config for Runtime {
    type AccountLookup = AuthorMapping;
    type EventHandler = ParachainStaking;
    type CanAuthor = AuthorFilter;
    // We start a new slot each time we see a new relay block.
    type SlotBeacon = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
    type WeightInfo = pallet_author_inherent::weights::SubstrateWeight<Runtime>;
}

impl pallet_author_slot_filter::Config for Runtime {
    type Event = Event;
    type RandomnessSource = RandomnessCollectiveFlip;
    type PotentialAuthors = ParachainStaking;
    type WeightInfo = pallet_author_slot_filter::weights::SubstrateWeight<Runtime>;
}

// This is a simple session key manager. It should probably either work with, or be replaced
// entirely by pallet sessions
impl pallet_author_mapping::Config for Runtime {
    type Event = Event;
    type DepositCurrency = Balances;
    type DepositAmount = ConstU128<{ 100 * 1000000000000000000 }>;
    type Keys = VrfId;
    type WeightInfo = pallet_author_mapping::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    /// Default fixed percent a collator takes off the top of due rewards
    pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
    /// Default percent of inflation set aside for parachain bond every round
    pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
}

impl parachain_staking::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
    /// Minimum round length is 2 minutes (10 * 12 second block times)
    type MinBlocksPerRound = ConstU32<10>;
    /// Blocks per round
    type DefaultBlocksPerRound = ConstU32<{ 6 * HOURS }>;
    /// Rounds before the collator leaving the candidates request can be executed
    type LeaveCandidatesDelay = ConstU32<{ 4 * 7 }>;
    /// Rounds before the candidate bond increase/decrease can be executed
    type CandidateBondLessDelay = ConstU32<{ 4 * 7 }>;
    /// Rounds before the delegator exit can be executed
    type LeaveDelegatorsDelay = ConstU32<{ 4 * 7 }>;
    /// Rounds before the delegator revocation can be executed
    type RevokeDelegationDelay = ConstU32<{ 4 * 7 }>;
    /// Rounds before the delegator bond increase/decrease can be executed
    type DelegationBondLessDelay = ConstU32<{ 4 * 7 }>;
    /// Rounds before the reward is paid
    type RewardPaymentDelay = ConstU32<2>;
    /// Minimum collators selected per round, default at genesis and minimum forever after
    type MinSelectedCandidates = ConstU32<8>;
    /// Maximum top delegations per candidate
    type MaxTopDelegationsPerCandidate = ConstU32<300>;
    /// Maximum bottom delegations per candidate
    type MaxBottomDelegationsPerCandidate = ConstU32<50>;
    /// Maximum delegations per delegator
    type MaxDelegationsPerDelegator = ConstU32<100>;
    type DefaultCollatorCommission = DefaultCollatorCommission;
    type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
    /// Minimum stake required to become a collator
    type MinCollatorStk = ConstU128<{ 1000 * DIR * SUPPLY_FACTOR }>;
    /// Minimum stake required to be reserved to be a candidate
    type MinCandidateStk = ConstU128<{ 1000 * DIR * SUPPLY_FACTOR }>;
    /// Minimum stake required to be reserved to be a delegator
    type MinDelegation = ConstU128<{ 500 * MILLIDIR * SUPPLY_FACTOR }>;
    /// Minimum stake required to be reserved to be a delegator
    type MinDelegatorStk = ConstU128<{ 500 * MILLIDIR * SUPPLY_FACTOR }>;
    type OnCollatorPayout = ();
    type OnNewRound = ();
    type WeightInfo = parachain_staking::weights::SubstrateWeight<Runtime>;
}

impl pallet_account_set::Config for Runtime {}

/// Configure the pallet template in pallets/template.
impl pallet_template::Config for Runtime {
    type Event = Event;
}

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

parameter_types! {
    pub BlockGasLimit: U256
        = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
    pub PrecompilesValue: DioraPrecompiles<Runtime> = DioraPrecompiles::<_>::new();
}

pub struct DioraGasWeightMapping;
impl pallet_evm::GasWeightMapping for DioraGasWeightMapping {
    fn gas_to_weight(gas: u64) -> Weight {
        gas.saturating_mul(WEIGHT_PER_GAS)
    }
    fn weight_to_gas(weight: Weight) -> u64 {
        u64::try_from(weight.wrapping_div(WEIGHT_PER_GAS)).unwrap_or(u32::MAX as u64)
    }
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
    R: pallet_balances::Config + pallet_treasury::Config,
    pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
{
    // this seems to be called for substrate-based transactions
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
        if let Some(fees) = fees_then_tips.next() {
            // for fees, 80% are burned, 20% to the treasury
            let (_, to_treasury) = fees.ration(80, 20);
            // Balances pallet automatically burns dropped Negative Imbalances by decreasing
            // total_supply accordingly
            <pallet_treasury::Pallet<R> as OnUnbalanced<_>>::on_unbalanced(to_treasury);
        }
    }

    // this is called from pallet_evm for Ethereum-based transactions
    // (technically, it calls on_unbalanced, which calls this when non-zero)
    fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
        // Balances pallet automatically burns dropped Negative Imbalances by decreasing
        // total_supply accordingly
        let (_, to_treasury) = amount.ration(80, 20);
        <pallet_treasury::Pallet<R> as OnUnbalanced<_>>::on_unbalanced(to_treasury);
    }
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = BaseFee;
    type GasWeightMapping = DioraGasWeightMapping;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressRoot<AccountId>;
    type WithdrawOrigin = EnsureAddressNever<AccountId>;
    type AddressMapping = HashedAddressMapping<BlakeTwo256>;
    type Currency = Balances;
    type Event = Event;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type PrecompilesType = DioraPrecompiles<Runtime>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = EthereumChainId;
    type OnChargeTransaction = pallet_evm::EVMCurrencyAdapter<Balances, DealWithFees<Runtime>>;
    // type OnChargeTransaction = pallet_evm::EVMCurrencyAdapter<Balances, ()>;
    type BlockGasLimit = BlockGasLimit;
    type FindAuthor = ();
}

impl pallet_ethereum::Config for Runtime {
    type Event = Event;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}
impl pallet_ethereum_chain_id::Config for Runtime {}

parameter_types! {
    pub DefaultBaseFeePerGas: U256 = U256::from(450_000_000_000u128);
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
    fn lower() -> Permill {
        Permill::zero()
    }
    fn ideal() -> Permill {
        Permill::from_parts(500_000)
    }
    fn upper() -> Permill {
        Permill::from_parts(1_000_000)
    }
}

impl pallet_base_fee::Config for Runtime {
    type Event = Event;
    type Threshold = BaseFeeThreshold;
    // Tells `pallet_base_fee` whether to calculate a new BaseFee `on_finalize` or not.
    type IsActive = ConstBool<false>;
    type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
}
parameter_types! {
    pub const BlockPerEra: BlockNumber = 1 * DAYS;
    pub const RegisterDeposit: Balance = 1000 * DIR;
    pub const MaxNumberOfStakersPerContract: u32 = 16384;
    pub const MinimumStakingAmount: Balance = 500 * DIR;
    pub const MinimumRemainingAmount: Balance = 1 * DIR;
    pub const MaxEraStakeValues: u32 = 5;
    pub const MaxUnlockingChunks: u32 = 4;
    pub const UnbondingPeriod: u32 = 10;
    pub const DappsStakingPalletId: PalletId = PalletId(*b"py/dpsst");
}
impl pallet_dapps_staking::Config for Runtime {
    type Currency = Balances;
    type BlockPerEra = BlockPerEra;
    type SmartContract = SmartContract<AccountId>;
    type RegisterDeposit = RegisterDeposit;
    type Event = Event;
    type WeightInfo = pallet_dapps_staking::weights::SubstrateWeight<Runtime>;
    type MaxNumberOfStakersPerContract = MaxNumberOfStakersPerContract;
    type MinimumStakingAmount = MinimumStakingAmount;
    type PalletId = DappsStakingPalletId;
    type MaxUnlockingChunks = MaxUnlockingChunks;
    type UnbondingPeriod = UnbondingPeriod;
    type MinimumRemainingAmount = MinimumRemainingAmount;
    type MaxEraStakeValues = MaxEraStakeValues;
}
/// Multi-VM pointer to smart contract instance.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, scale_info::TypeInfo)]
pub enum SmartContract<AccountId> {
    /// EVM smart contract instance.
    Evm(sp_core::H160),
    /// Wasm smart contract instance.
    Wasm(AccountId),
}

impl<AccountId> Default for SmartContract<AccountId> {
    fn default() -> Self {
        SmartContract::Evm(H160::repeat_byte(0x00))
    }
}

#[cfg(not(feature = "runtime-benchmarks"))]
impl<AccountId> pallet_dapps_staking::IsContract for SmartContract<AccountId> {
    fn is_valid(&self) -> bool {
        match self {
            SmartContract::Wasm(_account) => false,
            SmartContract::Evm(account) => Evm::account_codes(&account).len() > 0,
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
impl<AccountId> pallet_dapps_staking::IsContract for SmartContract<AccountId> {
    fn is_valid(&self) -> bool {
        match self {
            SmartContract::Wasm(_account) => false,
            SmartContract::Evm(_account) => true,
        }
    }
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        // System support stuff.
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
        ParachainSystem: cumulus_pallet_parachain_system::{
            Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
        } = 1,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 2,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
        ParachainInfo: parachain_info::{Pallet, Storage, Config} = 4,
        Utility: pallet_utility::{Pallet, Call, Storage, Event} = 5,
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 6,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 7,
        // Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 8,

        // Monetary stuff.
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 11,

        // Nimbus support. The order of these are important and shall not change.
        ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 20,
        AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent} = 21,
        AuthorFilter: pallet_author_slot_filter::{Pallet, Storage, Event, Config} = 22,
        PotentialAuthorSet: pallet_account_set::{Pallet, Storage, Config<T>} = 23,
        DappsStaking: pallet_dapps_staking::{Pallet, Call, Storage, Event<T>} = 24,
        AuthorMapping: pallet_author_mapping::{Pallet, Call, Config<T>, Storage, Event<T>} = 25,

        // XCM helpers.
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 30,
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 31,
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,

        // Template
        TemplatePallet: pallet_template::{Pallet, Call, Storage, Event<T>}  = 40,

        // Ethereum compatibility
        EthereumChainId: pallet_ethereum_chain_id::{Pallet, Call, Storage, Config} = 50,
        Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>} = 51,
        Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin} = 52,
        BaseFee: pallet_base_fee::{Pallet, Call, Storage, Config<T>, Event} = 54,

        // Governance stuff.
        Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 60,
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 61,
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 62,
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 65,
    }
);

impl_runtime_apis! {
    impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
        fn chain_id() -> u64 {
            <Runtime as pallet_evm::Config>::ChainId::chain_id()
        }

        fn account_basic(address: H160) -> EVMAccount {
            let (account, _) = Evm::account_basic(&address);
            account
        }

        fn gas_price() -> U256 {
            let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
            gas_price
        }

        fn account_code_at(address: H160) -> Vec<u8> {
            Evm::account_codes(address)
        }

        fn author() -> H160 {
            <pallet_evm::Pallet<Runtime>>::find_author()
        }

        fn storage_at(address: H160, index: U256) -> H256 {
            let mut tmp = [0u8; 32];
            index.to_big_endian(&mut tmp);
            Evm::account_storages(address, H256::from_slice(&tmp[..]))
        }

        fn call(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

            let is_transactional = false;
            let validate = true;
            <Runtime as pallet_evm::Config>::Runner::call(
                        from,
                        to,
                        data,
                        value,
                        gas_limit.low_u64(),
                        max_fee_per_gas,
                        max_priority_fee_per_gas,
                        nonce,
                        access_list.unwrap_or_default(),
                        is_transactional,
                        validate,
                        config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
                    ).map_err(|err| err.error.into())
                }

        fn create(
            from: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

            let is_transactional = false;
            let validate = true;
            <Runtime as pallet_evm::Config>::Runner::create(
                        from,
                        data,
                        value,
                        gas_limit.low_u64(),
                        max_fee_per_gas,
                        max_priority_fee_per_gas,
                        nonce,
                        access_list.unwrap_or_default(),
                        is_transactional,
                        validate,
                        config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
                    ).map_err(|err| err.error.into())
                }


        fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
            Ethereum::current_transaction_statuses()
        }

        fn current_block() -> Option<pallet_ethereum::Block> {
            Ethereum::current_block()
        }

        fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
            Ethereum::current_receipts()
        }

        fn current_all() -> (
            Option<pallet_ethereum::Block>,
            Option<Vec<pallet_ethereum::Receipt>>,
            Option<Vec<TransactionStatus>>
        ) {
            (
                Ethereum::current_block(),
                Ethereum::current_receipts(),
                Ethereum::current_transaction_statuses()
            )
        }

        fn extrinsic_filter(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> Vec<EthereumTransaction> {
            xts.into_iter().filter_map(|xt| match xt.0.function {
                Call::Ethereum(transact { transaction }) => Some(transaction),
                _ => None
            }).collect::<Vec<EthereumTransaction>>()
        }

        fn elasticity() -> Option<Permill> {
            Some(BaseFee::elasticity())
        }
    }

    impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
        fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
            UncheckedExtrinsic::new_unsigned(
                pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
            )
        }
    }
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info(header)
        }
    }

    impl nimbus_primitives::NimbusApi<Block> for Runtime {
                fn can_author(
                    author: nimbus_primitives::NimbusId,
                    slot: u32,
                    parent_header: &<Block as BlockT>::Header
                ) -> bool {
                    let block_number = parent_header.number + 1;

                    // The Moonbeam runtimes use an entropy source that needs to do some accounting
                    // work during block initialization. Therefore we initialize it here to match
                    // the state it will be in when the next block is being executed.
                    use frame_support::traits::OnInitialize;
                    System::initialize(
                        &block_number,
                        &parent_header.hash(),
                        &parent_header.digest,
                    );
                    RandomnessCollectiveFlip::on_initialize(block_number);

                    // Because the staking solution calculates the next staking set at the beginning
                    // of the first block in the new round, the only way to accurately predict the
                    // authors is to compute the selection during prediction.
                    use nimbus_primitives::AccountLookup;
                    if parachain_staking::Pallet::<Self>::round().should_update(block_number) {
                        let author_account_id = if let Some(account) =
                            AuthorMapping::lookup_account(&author) {
                            account
                        } else {
                            // return false if author mapping not registered like in can_author impl
                            return false
                        };
                        // predict eligibility post-selection by computing selection results now
                        let (eligible, _) =
                            pallet_author_slot_filter::compute_pseudo_random_subset::<Self>(
                                parachain_staking::Pallet::<Self>::compute_top_candidates(),
                                &slot
                            );
                        eligible.contains(&author_account_id)
                    } else {
                        AuthorInherent::can_author(&author, &slot)
                    }
                }
            }

            // We also implement the old AuthorFilterAPI to meet the trait bounds on the client side.
            impl nimbus_primitives::AuthorFilterAPI<Block, NimbusId> for Runtime {
                fn can_author(_: NimbusId, _: u32, _: &<Block as BlockT>::Header) -> bool {
                    panic!("AuthorFilterAPI is no longer supported. Please update your client.")
                }
            }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
            list_benchmark!(list, extra, pallet_balances, Balances);
            list_benchmark!(list, extra, pallet_timestamp, Timestamp);
            list_benchmark!(list, extra, pallet_collator_selection, CollatorSelection);

            let storage_info = AllPalletsWithSystem::storage_info();

            return (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}

            use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
            impl cumulus_pallet_session_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_collator_selection, CollatorSelection);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
    fn check_inherents(
        block: &Block,
        relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
    ) -> sp_inherents::CheckInherentsResult {
        let relay_chain_slot = relay_state_proof
            .read_slot()
            .expect("Could not read the relay chain slot from the proof");

        let inherent_data =
            cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
                relay_chain_slot,
                sp_std::time::Duration::from_secs(6),
            )
            .create_inherent_data()
            .expect("Could not create the timestamp inherent data");

        inherent_data.check_extrinsics(block)
    }
}

cumulus_pallet_parachain_system::register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = pallet_author_inherent::BlockExecutor::<Runtime, Executive>,
    CheckInherents = CheckInherents,
}
