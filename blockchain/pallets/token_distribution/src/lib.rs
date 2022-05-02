#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod exponential_issuance;
pub use exponential_issuance::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use frame_support::{
        sp_runtime::traits::Saturating,
        traits::{Currency, Get},
    };

    pub type FractalId = u64;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: Currency<Self::AccountId>;

        type TotalIssuance: Get<BalanceOf<Self>>;
        type IssuanceHalfLife: Get<Self::BlockNumber>;
        type IssuanceCompleteAt: Get<Self::BlockNumber>;
    }

    pub trait TokenDistribution<T: Config> {
        fn take_from(purpose: u8) -> BalanceOf<T>;
        fn return_to(purpose: u8, amount: BalanceOf<T>);
    }

    #[pallet::storage]
    pub type ArtificiallyIssued<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type DestinationWeights<T: Config> =
        StorageMap<_, Blake2_128Concat, Destination<T::AccountId>, u32, ValueQuery>;

    #[pallet::storage]
    pub type PurposeBalances<T: Config> =
        StorageMap<_, Blake2_128Concat, u8, BalanceOf<T>, ValueQuery>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::metadata(BalanceOf<T> = "Balance")]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {}

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    pub enum Destination<A> {
        Address(A),
        Purpose(u8),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(0, 1),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn set_weight(
            origin: OriginFor<T>,
            address: Destination<T::AccountId>,
            #[pallet::compact] weight: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            DestinationWeights::<T>::insert(address, weight);
            Ok(())
        }

        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(1, 1),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn increment_artificially_issued(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ArtificiallyIssued::<T>::mutate(|a| {
                *a += amount;
            });
            Ok(())
        }

        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(1, 1),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn mint(
            origin: OriginFor<T>,
            address: T::AccountId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            T::Currency::deposit_creating(&address, amount);
            ArtificiallyIssued::<T>::mutate(|a| {
                *a += amount;
            });

            Ok(())
        }
    }

    impl<T: Config> TokenDistribution<T> for Pallet<T> {
        fn take_from(purpose: u8) -> BalanceOf<T> {
            PurposeBalances::<T>::take(purpose)
        }

        fn return_to(purpose: u8, amount: BalanceOf<T>) {
            PurposeBalances::<T>::insert(purpose, amount);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        BalanceOf<T>: num_traits::PrimInt + core::iter::Sum,
        BlockNumberFor<T>: num_traits::PrimInt,
    {
        fn on_finalize(block_number: BlockNumberFor<T>) {
            let total_weight: u32 = DestinationWeights::<T>::iter_values().sum();
            if total_weight == 0 {
                return;
            }

            // Using Issuance like this makes it _technically_ possible for
            // consensus to fail if the CPU's floating point calculations are
            // different.
            //
            // If this becomes a problem, we can have this value derived from
            // an extrinsic that an authoritative account sets. Similar to how
            // the timestamp pallet works.
            let issuance = crate::Issuance {
                total: T::TotalIssuance::get(),
                half_life: T::IssuanceHalfLife::get(),
                complete_at: T::IssuanceCompleteAt::get(),
            };

            let should_be_issued =
                issuance.total_issued_by(block_number) + ArtificiallyIssued::<T>::get();
            let already_issued =
                T::Currency::total_issuance() + PurposeBalances::<T>::iter_values().sum();

            let unit_balance =
                should_be_issued.saturating_sub(already_issued) / total_weight.into();

            for (dest, weight) in DestinationWeights::<T>::iter() {
                let to_this = unit_balance * weight.into();
                match dest {
                    Destination::Address(a) => {
                        T::Currency::deposit_creating(&a, to_this);
                    }
                    Destination::Purpose(p) => {
                        PurposeBalances::<T>::mutate(&p, |balance| {
                            *balance += to_this;
                        });
                    }
                }
            }
        }
    }
}
