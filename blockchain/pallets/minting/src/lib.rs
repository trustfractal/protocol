#![cfg_attr(not(feature = "std"), no_std)]

use core::convert::TryInto;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch,
    traits::{Currency, Get},
    weights::Weight,
};
use frame_system::ensure_signed;
use sp_runtime::traits::CheckedDiv;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    type Currency: Currency<Self::AccountId>;

    type MaxRewardPerUser: Get<BalanceOf<Self>>;
    type MaxMintPerPeriod: Get<BalanceOf<Self>>;

    type MintEveryNBlocks: Get<Self::BlockNumber>;
}

decl_storage! {
    trait Store for Module<T: Config> as FractalMintingStorage {
        pub NextMintingRewards get(fn next_minting_rewards):
            map hasher(blake2_128_concat) T::AccountId => ();
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
    {
        /// Some amount of balance was minted among the number of provided accounts.
        /// [amount, number_of_accounts]
        Minted(Balance, u32),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn register_for_minting(origin) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            NextMintingRewards::<T>::insert(who, ());

            Ok(())
        }

        fn on_initialize(_block_number: T::BlockNumber) -> Weight {
            Weight::default()
        }

        fn on_finalize(block_number: T::BlockNumber) {
            if block_number % T::MintEveryNBlocks::get() != 0u32.into() {
                return;
            }

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
            Self::deposit_event(RawEvent::Minted(total_minted, accounts));
        }
    }
}
