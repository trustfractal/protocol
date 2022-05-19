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

    use codec::alloc::collections::BTreeMap;
    use frame_support::{
        sp_runtime::traits::Zero,
        traits::{Currency, ExistenceRequirement, Get},
    };
    use frame_system::ensure_signed;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: Currency<Self::AccountId>;
        type DistributeEveryNBlocks: Get<Self::BlockNumber>;
        type StakingLockPeriod: Get<Self::BlockNumber>;

        type DistributionSource: Get<Self::AccountId>;
        type HoldingAccount: Get<Self::AccountId>;
    }

    #[pallet::storage]
    pub type StakedAmounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BalanceOf<T>,
        ValueQuery,
    >;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::metadata(BalanceOf<T> = "Balance")]
    #[pallet::generate_deposit(pub fn deposit_event)]
    pub enum Event<T: Config> {
        Distribution { amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        CannotStakeMoreThanBalance,
        NotEnoughUnlockedStake,
        Internal,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
        pub fn stake(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let address = ensure_signed(origin)?;

            T::Currency::transfer(
                &address,
                &T::HoldingAccount::get(),
                amount,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_| Error::<T>::CannotStakeMoreThanBalance)?;

            StakedAmounts::<T>::insert(
                address,
                <frame_system::Pallet<T>>::block_number() + T::StakingLockPeriod::get(),
                amount,
            );

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(10, 10))]
        pub fn withdraw(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let address = ensure_signed(origin)?;

            let block_number = <frame_system::Pallet<T>>::block_number();
            let unlocked_balances: BTreeMap<_, _> = StakedAmounts::<T>::iter_prefix(&address)
                .filter(|(n, _)| n <= &block_number)
                .collect();

            let total_unlocked: BalanceOf<T> = unlocked_balances
                .values()
                .cloned()
                .fold(BalanceOf::<T>::zero(), |acc, b| acc + b);

            if total_unlocked < amount {
                return Err(Error::<T>::NotEnoughUnlockedStake.into());
            }

            let mut blocks = unlocked_balances.into_keys();
            let mut remaining = amount;
            while remaining > BalanceOf::<T>::zero() {
                let block = blocks.next().expect("Already checked for enough balance");

                StakedAmounts::<T>::mutate(&address, block, |b| {
                    let amount = core::cmp::min(*b, remaining);
                    *b -= amount;
                    remaining -= amount;
                });
            }

            T::Currency::transfer(
                &T::HoldingAccount::get(),
                &address,
                amount,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_| Error::<T>::Internal)?;

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        BalanceOf<T>: core::iter::Sum,
    {
        fn on_finalize(block_number: BlockNumberFor<T>) {
            if block_number % T::DistributeEveryNBlocks::get() != 0u32.into() {
                return;
            }

            let to_distribute = T::Currency::free_balance(&T::DistributionSource::get());
            let total_staked = T::Currency::free_balance(&T::HoldingAccount::get());

            let mut distributed = BalanceOf::<T>::zero();
            for (address, block, _) in StakedAmounts::<T>::iter() {
                StakedAmounts::<T>::mutate(address, block, |b| {
                    let amount = to_distribute * *b / total_staked;
                    *b += amount;
                    distributed += amount;
                });
            }

            T::Currency::deposit_creating(&T::HoldingAccount::get(), distributed);
            T::Currency::make_free_balance_be(
                &T::DistributionSource::get(),
                to_distribute - distributed,
            );

            Self::deposit_event(Event::<T>::Distribution {
                amount: distributed,
            });
        }
    }
}
