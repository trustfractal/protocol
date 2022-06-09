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
    use core::{convert::TryFrom, ops::Add};
    use frame_support::traits::{Currency, ExistenceRequirement, Get, ReservableCurrency};
    use frame_system::ensure_signed;
    use num_bigint::BigUint;

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
    pub type TotalCoinShares<T: Config> = StorageValue<_, EncodableBigUint, ValueQuery>;

    #[pallet::storage]
    pub type StakedAmounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        BlockNumberFor<T>,
        ShareBalance<BalanceOf<T>>,
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

    #[derive(Default, Decode, Encode, Debug, Clone)]
    pub struct EncodableBigUint {
        digits: Vec<u32>,
    }

    impl From<BigUint> for EncodableBigUint {
        fn from(n: BigUint) -> Self {
            Self {
                digits: n.to_u32_digits(),
            }
        }
    }

    impl Into<BigUint> for EncodableBigUint {
        fn into(self) -> BigUint {
            BigUint::new(self.digits)
        }
    }

    #[derive(Default, Decode, Encode)]
    pub struct ShareBalance<B> {
        map: BTreeMap<u32, B>,
    }

    impl<B> ShareBalance<B>
    where
        B: Default + Copy + sp_arithmetic::traits::AtLeast32BitUnsigned,
    {
        pub fn coin_shares(&self) -> BigUint
        where
            BigUint: Add<B, Output = BigUint> + From<B>,
        {
            self.map.iter().map(|(&s, &b)| BigUint::from(b) * s).sum()
        }

        pub fn balance(&self) -> B {
            self.map
                .values()
                .cloned()
                .fold(B::default(), |acc, b| acc + b)
        }

        fn distribute(&mut self, to_distribute: B, total_staked: &BigUint) -> B
        where
            BigUint: From<B>,
            B: TryFrom<BigUint>,
        {
            let mut total = B::default();

            for (&shares, b) in self.map.iter_mut() {
                let big_amount =
                    BigUint::from(to_distribute) * BigUint::from(*b) * shares / total_staked;
                let this_amount = B::try_from(big_amount)
                    .ok() // Silence error since it's not Debug
                    .expect("should never distribute more than B can handle");

                *b += this_amount;
                total += this_amount;
            }

            total
        }

        fn increment(&mut self, shares: u32, amount: B) {
            *self.map.entry(shares).or_default() += amount;
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        BigUint: From<BalanceOf<T>>,
    {
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
                |sb| sb.increment(shares, amount),
            );
            TotalCoinShares::<T>::mutate(|b| {
                let current_value: BigUint = b.clone().into();
                let increment = BigUint::from(amount) * shares;
                *b = EncodableBigUint::from(current_value + increment)
            });

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        BigUint: From<BalanceOf<T>> + Add<BalanceOf<T>, Output = BigUint>,
        BalanceOf<T>: TryFrom<BigUint>,
    {
        fn on_finalize(current_block: BlockNumberFor<T>) {
            if current_block % T::DistributeEveryNBlocks::get() != 0u32.into() {
                return;
            }

            let to_distribute = T::Currency::free_balance(&T::DistributionSource::get());
            let total_staked = TotalCoinShares::<T>::get().into();

            let mut new_coin_shares = BigUint::default();

            StakedAmounts::<T>::translate(|addr, unstake_at, mut sb: ShareBalance<_>| {
                let amt_to_this = sb.distribute(to_distribute, &total_staked);

                T::Currency::transfer(
                    &T::DistributionSource::get(),
                    &addr,
                    amt_to_this,
                    ExistenceRequirement::AllowDeath,
                )
                .expect("distributing based on balance");

                if unstake_at <= current_block {
                    T::Currency::unreserve(&addr, sb.balance() - amt_to_this);

                    Self::deposit_event(Event::<T>::Unstaked {
                        amount: sb.balance(),
                        who: addr,
                    });
                    return None;
                }

                T::Currency::reserve(&addr, amt_to_this).expect("just deposited to this account");
                new_coin_shares += sb.coin_shares();

                Some(sb)
            });

            TotalCoinShares::<T>::set(new_coin_shares.into());

            Self::deposit_event(Event::<T>::Distribution {
                amount: to_distribute - T::Currency::free_balance(&T::DistributionSource::get()),
            });
        }
    }
}
