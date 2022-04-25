use crate as fractal_data_capture;
use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        FractalMinting: fractal_data_capture::{Pallet, Call, Storage, Config<T>, Event<T>},
        FractalTokenDistribution: fractal_token_distribution::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<<Test as pallet_balances::Config>::Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const MintEveryNBlocks: u64 = 10;

    pub const TotalIssuance: u64 = 420_000_000;
    pub const IssuanceHalfLife: u64 = 600;
    pub const IssuanceCompleteAt: u64 = 10_000;
    pub const MaxRewardPerUser: u64 = 420_000;
}

impl fractal_data_capture::Config for Test {
    type Event = Event;

    type MintEveryNBlocks = MintEveryNBlocks;

    type MaxRewardPerUser = MaxRewardPerUser;
    type TokenDistribution = FractalTokenDistribution;
}

impl fractal_token_distribution::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type TotalIssuance = TotalIssuance;
    type IssuanceHalfLife = IssuanceHalfLife;
    type IssuanceCompleteAt = IssuanceCompleteAt;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    GenesisConfig {
        fractal_data_capture: crate::GenesisConfig {
            fractal_authoritative_account: 123,
        },
        ..Default::default()
    }
    .build_storage()
    .unwrap()
    .into()
}
