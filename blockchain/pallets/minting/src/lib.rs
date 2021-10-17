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

    use blake2::Blake2b;

    use core::convert::TryInto;
    use frame_support::{
        dispatch::Vec,
        traits::{Currency, Get, Imbalance},
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

        type TotalIssuance: Get<BalanceOf<Self>>;
        type IssuanceHalfLife: Get<Self::BlockNumber>;
        type IssuanceCompleteAt: Get<Self::BlockNumber>;
        type MaxRewardPerUser: Get<BalanceOf<Self>>;

        type MintEveryNBlocks: Get<Self::BlockNumber>;

        type ExcessMintingReceiver: Get<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type TotalAlreadyMinted<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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
    pub type AccountIdDatasets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        FractalId,
        MerkleTree<Blake2b>,
        OptionQuery,
    >;

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
        /// [total, per_user, number_of_accounts, excess]
        Minted {
            total: BalanceOf<T>,
            per_user: BalanceOf<T>,
            number_of_accounts: u32,
            excess: BalanceOf<T>,
        },
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
            10_000 + T::DbWeight::get().reads_writes(1, 1),
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

            AccountIds::<T>::insert(account, fractal_id, ());

            Ok(())
        }

        /// Register to receive minting in the next period.
        // TODO(shelbyd): Charge users transaction fees if this isn't their first registration.
        #[pallet::weight((
            10_000 + T::DbWeight::get().reads_writes(3, 2),
            DispatchClass::Normal,
            Pays::No
        ))]
        pub fn register_for_minting(
            origin: OriginFor<T>,
            identity: Option<FractalId>,
            extension_proof: MerkleTree<Blake2b>,
        ) -> DispatchResultWithPostInfo {
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

            let id_datasets_entry = AccountIdDatasets::<T>::get(who.clone(), id);
            if let Some(existing) = &id_datasets_entry {
                ensure!(
                    extension_proof.strict_extends(existing),
                    Error::<T>::ExtensionDoesNotExtendExistingDataset
                );
            }

            AccountIdDatasets::<T>::insert(who.clone(), id, extension_proof);
            NextMintingRewards::<T>::insert(id, who);

            Ok(match id_datasets_entry {
                Some(_) => Pays::Yes.into(),
                None => Pays::No.into(),
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        BalanceOf<T>: num_traits::PrimInt,
        BlockNumberFor<T>: num_traits::PrimInt,
    {
        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            Weight::default()
        }

        fn on_finalize(block_number: BlockNumberFor<T>) {
            let is_mint_block = block_number % T::MintEveryNBlocks::get() == 0u32.into()
                && block_number != 0u32.into();
            if !is_mint_block {
                return;
            }

            let accounts = NextMintingRewards::<T>::iter().collect::<Vec<_>>();
            let accounts_count: u32 = accounts.len().try_into().unwrap_or(core::u32::MAX);

            let (mint_per_user, total_minted) = Self::minting_amount(block_number, accounts_count);

            let recipients = accounts
                .iter()
                .take(accounts_count.try_into().expect("at least 32bit OS"));
            let removed = recipients.inspect(|(id, _)| NextMintingRewards::<T>::remove(id));

            // Dropping the Imbalance resolves it.
            let _imbalance = removed
                .map(|(_, account)| T::Currency::deposit_creating(account, mint_per_user))
                .fold(
                    <T::Currency as Currency<_>>::PositiveImbalance::zero(),
                    |acc, v| acc.merge(v),
                );

            let distributed_to_users = mint_per_user * accounts_count.into();
            let unclaimed = total_minted - distributed_to_users;
            T::Currency::deposit_creating(&T::ExcessMintingReceiver::get(), unclaimed);

            TotalAlreadyMinted::<T>::set(TotalAlreadyMinted::<T>::get() + total_minted);

            Self::deposit_event(Event::Minted {
                total: total_minted,
                per_user: mint_per_user,
                number_of_accounts: accounts_count,
                excess: unclaimed,
            });
        }
    }

    impl<T: Config> Pallet<T>
    where
        BalanceOf<T>: num_traits::PrimInt,
        BlockNumberFor<T>: num_traits::PrimInt,
    {
        fn minting_amount(
            block_number: BlockNumberFor<T>,
            accounts_count: u32,
        ) -> (BalanceOf<T>, BalanceOf<T>) {
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
            let total_after_block = issuance.total_issued_by(block_number);
            let to_issue = total_after_block - TotalAlreadyMinted::<T>::get();

            let even_mint_per_user = to_issue
                .checked_div(&accounts_count.into())
                .unwrap_or_else(|| 0u32.into());

            let mint_per_user = core::cmp::min(T::MaxRewardPerUser::get(), even_mint_per_user);

            (mint_per_user, to_issue)
        }
    }
}
