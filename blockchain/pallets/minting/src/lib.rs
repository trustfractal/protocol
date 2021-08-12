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

    use blake2::Blake2b;

    use core::convert::TryInto;
    use frame_support::{
        dispatch::Vec,
        traits::{Currency, Get},
        weights::Weight,
    };
    use frame_system::ensure_signed;
    use merklex::MerkleTree;
    use sp_runtime::traits::CheckedDiv;

    pub type FractalId = u64;

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
        StorageMap<_, Blake2_128Concat, FractalId, T::AccountId, ValueQuery>;

    #[pallet::storage]
    pub type AccountIds<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        FractalId,
        (),
        ValueQuery,
    >;

    #[pallet::storage]
    pub type IdToAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, FractalId, T::AccountId, ValueQuery>;

    #[pallet::storage]
    pub type IdDatasets<T: Config> =
        StorageMap<_, Blake2_128Concat, FractalId, MerkleTree<Blake2b>, OptionQuery>;

    #[pallet::storage]
    pub type FractalAuthoritativeAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub fractal_authoritative_account: T::AccountId,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                fractal_authoritative_account: T::AccountId::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T>
    where
        T::AccountId: Clone,
    {
        fn build(&self) {
            FractalAuthoritativeAccount::<T>::put(self.fractal_authoritative_account.clone());
        }
    }

    #[pallet::event]
    #[pallet::metadata(BalanceOf<T> = "Balance")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Some amount of balance was minted among the number of provided accounts.
        /// [amount, number_of_accounts]
        Minted(BalanceOf<T>, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoIdentityRegistered,
        ExtensionDoesNotExtendExistingDataset,
        MustSpecifyFractalIdWithMultipleIds,
        FractalIdNotRegisteredToAccount,
        MustBeFractal,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(2, 4),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn register_identity(
            origin: OriginFor<T>,
            fractal_id: FractalId,
            account: T::AccountId,
        ) -> DispatchResult
        where
            T::AccountId: Clone,
        {
            let should_be_fractal = ensure_signed(origin)?;
            ensure!(
                should_be_fractal == FractalAuthoritativeAccount::<T>::get(),
                Error::<T>::MustBeFractal
            );

            if let Ok(account) = IdToAccount::<T>::try_get(fractal_id) {
                AccountIds::<T>::remove(account, fractal_id);
            }
            NextMintingRewards::<T>::remove(fractal_id);

            IdToAccount::<T>::insert(fractal_id, account.clone());
            AccountIds::<T>::insert(account, fractal_id, ());

            Ok(())
        }

        /// Register to receive minting in the next period.
        // TODO(shelbyd): Charge users transaction fees if this isn't their first registration.
        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(2, 2),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn register_for_minting(
            origin: OriginFor<T>,
            identity: Option<FractalId>,
            extension_proof: MerkleTree<Blake2b>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let id = match identity {
                Some(id) => {
                    ensure!(
                        AccountIds::<T>::contains_key(&who, &id),
                        Error::<T>::FractalIdNotRegisteredToAccount
                    );
                    id
                }
                None => {
                    let mut ids = AccountIds::<T>::iter_prefix(&who);

                    match (ids.next(), ids.next()) {
                        (None, _) => return Err(Error::<T>::NoIdentityRegistered.into()),
                        (Some((id, ())), None) => id,
                        (Some(_), Some(_)) => {
                            return Err(Error::<T>::MustSpecifyFractalIdWithMultipleIds.into());
                        }
                    }
                }
            };

            if let Some(existing) = IdDatasets::<T>::get(id) {
                ensure!(
                    extension_proof.strict_extends(&existing),
                    Error::<T>::ExtensionDoesNotExtendExistingDataset
                );
            }

            IdDatasets::<T>::insert(id, extension_proof);
            NextMintingRewards::<T>::insert(id, who);

            Ok(())
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

            let accounts = NextMintingRewards::<T>::iter().collect::<Vec<_>>();

            let accounts_count: u32 = accounts.len().try_into().unwrap_or(core::u32::MAX);

            let mint_per_user = T::MaxMintPerPeriod::get()
                .checked_div(&accounts_count.into())
                .unwrap_or_else(|| 0u32.into());

            let reward_per_user = core::cmp::min(T::MaxRewardPerUser::get(), mint_per_user);

            let recipients = accounts
                .iter()
                .take(accounts_count.try_into().expect("at least 32bit OS"));
            for (id, account) in recipients {
                T::Currency::deposit_creating(account, reward_per_user);
                NextMintingRewards::<T>::remove(id);
            }

            let total_minted = reward_per_user * accounts_count.into();
            Self::deposit_event(Event::Minted(total_minted, accounts_count));
        }
    }
}
