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

    use codec::{Decode, Encode};
    use core::convert::TryInto;
    use frame_support::{
        dispatch::Vec,
        traits::{Currency, Get},
        weights::Weight,
    };
    use frame_system::ensure_signed;
    use sp_core::sr25519::{Public, Signature};
    use sp_runtime::{
        traits::{CheckedDiv, Verify},
        AnySignature,
    };

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    pub struct Signed<T: Decode> {
        signature: AnySignature,
        encoded: Vec<u8>,
        _value: core::marker::PhantomData<T>,
    }

    impl<T: Decode> Signed<T> {
        pub fn new(signature: Signature, encoded: Vec<u8>) -> Self {
            Signed {
                signature: signature.into(),
                encoded,
                _value: core::marker::PhantomData,
            }
        }

        #[cfg(feature = "std")]
        pub fn with_secret(pair: &sp_core::sr25519::Pair, value: T) -> Signed<T>
        where
            T: Encode,
        {
            use sp_core::Pair;

            let signature = pair.sign(&value.encode()).into();
            Signed {
                signature,
                encoded: value.encode(),
                _value: core::marker::PhantomData,
            }
        }

        pub fn verify_against(&self, public: &Public) -> Option<T> {
            let verified = self.signature.verify(self.encoded.as_slice(), public);
            if !verified {
                return None;
            }
            let decoded = T::decode(&mut self.encoded.as_ref()).ok()?;
            Some(decoded)
        }
    }

    pub type FractalId = u64;
    pub type Nonce = u32;

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    pub struct FractalIdentity<A> {
        pub account: A,
        pub fractal_id: FractalId,
        #[codec(compact)]
        pub nonce: Nonce,
    }

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
    pub type IdNonces<T: Config> = StorageMap<_, Blake2_128Concat, FractalId, Nonce, ValueQuery>;

    #[pallet::storage]
    pub type FractalPublicKey<T: Config> = StorageValue<_, Public, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {
        pub fractal_public_key: Public,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            FractalPublicKey::<T>::put(self.fractal_public_key);
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
        InvalidIdentitySignature,
        MismatchedFractalIdentity,
        LesserNonce,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register the origin for minting in the next minting period.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 2))]
        pub fn register_for_minting(
            origin: OriginFor<T>,
            identity: Signed<FractalIdentity<T::AccountId>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let identity = identity
                .verify_against(&FractalPublicKey::<T>::get())
                .ok_or(Error::<T>::InvalidIdentitySignature)?;

            ensure!(
                identity.account == who,
                Error::<T>::MismatchedFractalIdentity
            );

            match IdNonces::<T>::try_get(identity.fractal_id) {
                // Include this branch to not issue a storage write in the common case that the
                // nonces match exactly.
                Ok(nonce) if identity.nonce == nonce => {}

                Ok(nonce) if identity.nonce < nonce => {
                    return Err(Error::<T>::LesserNonce)?;
                }
                Ok(_) | Err(()) => {
                    IdNonces::<T>::insert(identity.fractal_id, identity.nonce);
                }
            }

            NextMintingRewards::<T>::insert(identity.fractal_id, who);

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
            for (id, account) in recipients {
                T::Currency::deposit_creating(&account, reward_per_user);
                NextMintingRewards::<T>::remove(id);
            }

            let total_minted = mint_per_user * accounts.into();
            Self::deposit_event(Event::Minted(total_minted, accounts));
        }
    }
}
