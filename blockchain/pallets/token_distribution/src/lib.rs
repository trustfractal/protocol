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

    #[pallet::storage]
    pub type ArtificiallyIssued<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type ArtificiallyBurned<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type DestinationWeights<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::metadata(BalanceOf<T> = "Balance")]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(0, 1),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn set_weight(
            origin: OriginFor<T>,
            address: T::AccountId,
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
            Self::increase_artificial(amount);
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
            Self::increase_artificial(amount);

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn burn(origin: OriginFor<T>, amount: Option<BalanceOf<T>>) -> DispatchResult {
            let address = ensure_signed(origin)?;

            let slash = match amount {
                Some(amount) => core::cmp::min(amount, T::Currency::free_balance(&address)),
                None => T::Currency::free_balance(&address),
            };
            T::Currency::slash(&address, slash);

            Self::decrease_artificial(slash);

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn increase_artificial(amount: BalanceOf<T>) {
            let after_from = ArtificiallyBurned::<T>::mutate(|a| {
                let orig_a = *a;
                *a = a.saturating_sub(amount);
                amount - orig_a
            });
            ArtificiallyIssued::<T>::mutate(|a| {
                *a += after_from;
            });
        }

        fn decrease_artificial(amount: BalanceOf<T>) {
            let after_from = ArtificiallyIssued::<T>::mutate(|a| {
                let orig_a = *a;
                *a = a.saturating_sub(amount);
                amount - orig_a
            });
            ArtificiallyBurned::<T>::mutate(|a| {
                *a += after_from;
            });
        }

        fn needed_to_reach(total: BalanceOf<T>) -> BalanceOf<T> {
            let should_be_issued = total + ArtificiallyIssued::<T>::get();
            let already_issued = T::Currency::total_issuance() + ArtificiallyBurned::<T>::get();

            should_be_issued.saturating_sub(already_issued)
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

            let needed = Self::needed_to_reach(issuance.total_issued_by(block_number));
            let unit_balance = needed / total_weight.into();

            for (address, weight) in DestinationWeights::<T>::iter() {
                let to_this = unit_balance * weight.into();
                T::Currency::deposit_creating(&address, to_this);
            }
        }
    }
}
