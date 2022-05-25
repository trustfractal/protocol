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
        traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
    };
    use frame_system::ensure_signed;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type DistributeEveryNBlocks: Get<Self::BlockNumber>;

        type DistributionSource: Get<Self::AccountId>;
    }

    #[pallet::storage]
    pub type TotalCoinShares<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type StakedAmounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        BlockNumberFor<T>,
        (BalanceOf<T>, u32),
        ValueQuery,
    >;

    #[pallet::storage]
    pub type LockPeriodShares<T: Config> =
        StorageMap<_, Blake2_128Concat, BlockNumberFor<T>, u32, OptionQuery>;

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
        UnknownLockPeriod,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
        pub fn set_lock_period_shares(
            origin: OriginFor<T>,
            #[pallet::compact] blocks: BlockNumberFor<T>,
            #[pallet::compact] shares: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            LockPeriodShares::<T>::insert(blocks, shares);

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
        pub fn stake(
            origin: OriginFor<T>,
            #[pallet::compact] lock_period: BlockNumberFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let address = ensure_signed(origin)?;

            let shares =
                LockPeriodShares::<T>::get(lock_period).ok_or(Error::<T>::UnknownLockPeriod)?;

            T::Currency::reserve(&address, amount)
                .map_err(|_| Error::<T>::CannotStakeMoreThanBalance)?;

            StakedAmounts::<T>::insert(
                address,
                <frame_system::Pallet<T>>::block_number() + lock_period,
                (amount, shares),
            );
            TotalCoinShares::<T>::mutate(|b| *b += amount * shares.into());

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(10, 10))]
        pub fn withdraw(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let address = ensure_signed(origin)?;

            let zero_balance = BalanceOf::<T>::zero();

            let block_number = <frame_system::Pallet<T>>::block_number();
            let unlocked_balances: BTreeMap<_, _> = StakedAmounts::<T>::iter_prefix(&address)
                .filter(|(n, _)| n <= &block_number)
                .collect();

            let total_unlocked: BalanceOf<T> = unlocked_balances
                .values()
                .fold(zero_balance, |acc, (b, _)| acc + *b);

            if total_unlocked < amount {
                return Err(Error::<T>::NotEnoughUnlockedStake.into());
            }

            let mut remaining = amount;
            let mut withdrawn_shares = zero_balance;

            let mut blocks = unlocked_balances.into_keys();
            while remaining > zero_balance {
                let block = blocks.next().expect("Already checked for enough balance");

                let should_remove = StakedAmounts::<T>::mutate(&address, block, |(b, shares)| {
                    let amount = core::cmp::min(*b, remaining);
                    *b -= amount;
                    remaining -= amount;
                    withdrawn_shares += amount * (*shares).into();

                    *b == zero_balance
                });

                if should_remove {
                    StakedAmounts::<T>::remove(&address, block);
                }
            }

            T::Currency::unreserve(&address, amount);
            TotalCoinShares::<T>::mutate(|b| *b -= withdrawn_shares);

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
            let total_staked = TotalCoinShares::<T>::get();

            let mut distributed = BalanceOf::<T>::zero();
            let mut distributed_shares = BalanceOf::<T>::zero();
            for (address, block, _) in StakedAmounts::<T>::iter() {
                let amount = StakedAmounts::<T>::mutate(&address, block, |(b, shares)| {
                    let amount = to_distribute * *b * (*shares).into() / total_staked;
                    *b += amount;

                    distributed_shares += amount * (*shares).into();

                    amount
                });

                distributed += amount;

                T::Currency::transfer(
                    &T::DistributionSource::get(),
                    &address,
                    amount,
                    ExistenceRequirement::AllowDeath,
                )
                .expect("distributing based on balance");
                T::Currency::reserve(&address, amount).expect("just deposited to this account");
            }

            TotalCoinShares::<T>::mutate(|b| *b += distributed_shares);
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
