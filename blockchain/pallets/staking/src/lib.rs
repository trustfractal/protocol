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
    pub type MinimumStake<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    pub type TotalCoinShares<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type StakedAmounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BTreeMap<u32, BalanceOf<T>>,
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
        Distribution {
            amount: BalanceOf<T>,
        },
        Unstaked {
            amount: BalanceOf<T>,
            who: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        CannotStakeMoreThanBalance,
        NotEnoughUnlockedStake,
        UnknownLockPeriod,
        AmountBelowMinimum,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(0, 1),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn set_minimum_stake(
            origin: OriginFor<T>,
            min: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            MinimumStake::<T>::set(min);

            Ok(())
        }

        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(0, 1),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn set_lock_period_shares(
            origin: OriginFor<T>,
            #[pallet::compact] blocks: BlockNumberFor<T>,
            #[pallet::compact] shares: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            LockPeriodShares::<T>::insert(blocks, shares);

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(4, 2))]
        pub fn stake(
            origin: OriginFor<T>,
            #[pallet::compact] lock_period: BlockNumberFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let address = ensure_signed(origin)?;

            if let Some(min) = MinimumStake::<T>::get() {
                if amount < min {
                    return Err(Error::<T>::AmountBelowMinimum.into());
                }
            }

            let shares =
                LockPeriodShares::<T>::get(lock_period).ok_or(Error::<T>::UnknownLockPeriod)?;

            T::Currency::reserve(&address, amount)
                .map_err(|_| Error::<T>::CannotStakeMoreThanBalance)?;

            StakedAmounts::<T>::mutate(
                address,
                <frame_system::Pallet<T>>::block_number() + lock_period,
                |map| {
                    *map.entry(shares).or_insert_with(BalanceOf::<T>::zero) += amount;
                },
            );
            TotalCoinShares::<T>::mutate(|b| *b += amount * shares.into());

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn staked_balance(account: T::AccountId) -> BalanceOf<T>
        where
            BalanceOf<T>: core::iter::Sum,
        {
            StakedAmounts::<T>::iter_prefix_values(account)
                .map(|map| map.values().cloned().sum())
                .sum()
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
                let amount = StakedAmounts::<T>::mutate(&address, block, |map| {
                    let mut amount = BalanceOf::<T>::zero();

                    for (shares, b) in map.iter_mut() {
                        let shares = (*shares).into();

                        amount += to_distribute * *b * shares / total_staked;
                        *b += amount;

                        distributed += amount;
                        distributed_shares += amount * shares;
                    }

                    amount
                });

                T::Currency::transfer(
                    &T::DistributionSource::get(),
                    &address,
                    amount,
                    ExistenceRequirement::AllowDeath,
                )
                .expect("distributing based on balance");
                T::Currency::reserve(&address, amount).expect("just deposited to this account");

                if block <= block_number {
                    let map = StakedAmounts::<T>::take(&address, &block);

                    let balance = map
                        .into_iter()
                        .inspect(|&(shares, balance)| {
                            TotalCoinShares::<T>::mutate(|b| *b -= balance * shares.into());
                        })
                        .map(|(_, b)| b)
                        .sum();
                    T::Currency::unreserve(&address, balance);

                    Self::deposit_event(Event::<T>::Unstaked {
                        amount: balance,
                        who: address,
                    });
                }
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
