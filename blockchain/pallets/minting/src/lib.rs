#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use core::convert::TryInto;
    use frame_support::{
        traits::{Currency, Get},
        weights::Weight,
    };
    use frame_system::ensure_signed;
    use sp_runtime::traits::CheckedDiv;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: Currency<Self::AccountId>;

        type MaxRewardPerUser: Get<BalanceOf<Self>>;
        type MaxMintPerPeriod: Get<BalanceOf<Self>>;

        type MintEveryNBlocks: Get<Self::BlockNumber>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type NextMintingRewards<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::metadata(BalanceOf<T> = "Balance")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Some amount of balance was minted among the number of provided accounts.
        /// [amount, number_of_accounts]
        Minted(BalanceOf<T>, u32),
    }

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register the origin for minting in the next minting period.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_for_minting(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            NextMintingRewards::<T>::insert(who, true);

            Ok(Default::default())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            Weight::default()
        }

        fn on_finalize(block_number: BlockNumberFor<T>) {
            if block_number % T::MintEveryNBlocks::get() != 0u32.into() {
                return;
            }

            // TODO(shelbyd): Don't iterate whole storage just to count.
            let accounts: u32 = NextMintingRewards::<T>::iter()
                .count()
                .try_into()
                .unwrap_or(core::u32::MAX);

            let mint_per_user = T::MaxMintPerPeriod::get()
                .checked_div(&accounts.into())
                .unwrap_or(0u32.into());

            let reward_per_user = core::cmp::min(T::MaxRewardPerUser::get(), mint_per_user);

            let recipients = NextMintingRewards::<T>::iter()
                .take(accounts.try_into().expect("at least 32bit OS"));
            for (account, _) in recipients {
                T::Currency::deposit_creating(&account, reward_per_user);
                NextMintingRewards::<T>::remove(account);
            }

            let total_minted = mint_per_user * accounts.into();
            Self::deposit_event(Event::Minted(total_minted, accounts));
        }
    }
}
