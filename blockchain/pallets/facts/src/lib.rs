#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn fact)]
    pub type Fact<T> = StorageValue<_, sp_std::vec::Vec<u8>>;

    #[pallet::storage]
    pub type FractalAuthoritativeAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The `AccountId` of the sudo key.
        pub fractal_authoritative_account_facts: T::AccountId,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                fractal_authoritative_account_facts: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T>
    where
        T::AccountId: Clone,
    {
        fn build(&self) {
            FractalAuthoritativeAccount::<T>::put(self.fractal_authoritative_account_facts.clone());
        }
    }


    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FactStored(sp_std::vec::Vec<u8>, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        StorageOverflow,
    }


    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn store_fact(origin: OriginFor<T>, fact: sp_std::vec::Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            <Fact<T>>::put(fact.clone());

            Self::deposit_event(Event::FactStored(fact, who));

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        BlockNumberFor<T>: num_traits::PrimInt,
    {
        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            Weight::default()
        }
    }
}
