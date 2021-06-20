use crate::{mock::*, *};
use blake2::Blake2b;
use frame_support::{
    assert_noop, assert_ok,
    traits::{OnFinalize, OnInitialize},
};
use merklex::MerkleTree;

#[cfg(test)]
mod register_identity {
    use super::*;

    fn run_to_next_minting() {
        let mint_every_n = <Test as crate::Config>::MintEveryNBlocks::get();

        loop {
            FractalMinting::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
            System::set_block_number(System::block_number() + 1);
            System::on_initialize(System::block_number());
            FractalMinting::on_initialize(System::block_number());

            if System::block_number() % mint_every_n == 0 {
                break;
            }
        }
    }

    fn identity(account: u64) -> Signed<FractalIdentity<u64>> {
        signed_identity(FractalIdentity {
            account,
            fractal_id: account,
            nonce: 0,
        })
    }

    fn signed_identity(id: FractalIdentity<u64>) -> Signed<FractalIdentity<u64>> {
        Signed::with_secret(&fractal_pair(), id)
    }

    fn register_id(id: u64) {
        assert_ok!(FractalMinting::register_identity(
            Origin::signed(id),
            identity(id)
        ));
    }

    fn register_id_mut(id: u64, f: impl FnOnce(&mut FractalIdentity<u64>)) {
        let mut fractal_identity = FractalIdentity {
            account: id,
            fractal_id: id,
            nonce: 0,
        };
        f(&mut fractal_identity);

        assert_ok!(FractalMinting::register_identity(
            Origin::signed(id),
            signed_identity(fractal_identity)
        ));
    }

    fn register_for_minting(id: u64) {
        assert_ok!(FractalMinting::register_for_minting(
            Origin::signed(id),
            None,
            simple_tree().prune_balanced(),
        ));
    }

    fn register_for_minting_dataset(id: u64, dataset: &[&'static str]) {
        assert_ok!(FractalMinting::register_for_minting(
            Origin::signed(id),
            None,
            MerkleTree::from_iter(dataset).expect("dataset with at least one element"),
        ));
    }

    fn simple_tree() -> MerkleTree<Blake2b> {
        MerkleTree::from_iter(&["test", "values"]).unwrap()
    }

    #[test]
    fn receives_portion_of_minting_after_block() {
        new_test_ext().execute_with(|| {
            register_id(1);
            register_for_minting(1);

            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[test]
    fn gets_no_reward_until_block() {
        new_test_ext().execute_with(|| {
            run_to_next_minting();

            register_id(1);
            register_for_minting(1);
            assert_eq!(Balances::free_balance(&1), 0);

            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[test]
    fn only_receives_for_immediate_minting() {
        new_test_ext().execute_with(|| {
            register_id(1);
            register_for_minting(1);

            run_to_next_minting();
            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[test]
    fn divides_max_minting_between_users() {
        new_test_ext().execute_with(|| {
            let users = 5;

            for id in 1..=users {
                register_id(id);
                register_for_minting(id);
            }

            run_to_next_minting();

            for id in 1..=users {
                assert_eq!(
                    Balances::free_balance(&id),
                    <Test as crate::Config>::MaxMintPerPeriod::get() / users
                );
            }
        });
    }

    #[test]
    fn multiple_registrations_only_one_mint() {
        new_test_ext().execute_with(|| {
            register_id(1);
            register_for_minting_dataset(1, &["1"]);
            register_for_minting_dataset(1, &["1", "2"]);
            register_for_minting_dataset(1, &["1", "2", "3"]);

            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[test]
    fn errors_with_invalid_fractal_signature() {
        new_test_ext().execute_with(|| {
            use codec::{Decode, Encode};
            let bad_signature = vec![42; 64];
            let message = vec![42];

            let signed_message = bad_signature
                .into_iter()
                .chain(message.encode())
                .collect::<Vec<_>>();
            let identity = Signed::decode(&mut signed_message.as_ref()).unwrap();

            assert_noop!(
                FractalMinting::register_identity(Origin::signed(1), identity),
                Error::<Test>::InvalidIdentitySignature
            );
        });
    }

    #[test]
    fn errors_when_fractal_identity_does_not_match_origin_account() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                FractalMinting::register_identity(Origin::signed(1), identity(2)),
                Error::<Test>::MismatchedFractalIdentity
            );
        });
    }

    #[test]
    fn later_fractal_id_replaces_previous() {
        new_test_ext().execute_with(|| {
            register_id_mut(1, |id| {
                id.fractal_id = 42;
            });
            register_for_minting(1);

            register_id_mut(2, |id| {
                id.fractal_id = 42;
                id.nonce = 1;
            });

            run_to_next_minting();

            assert_eq!(Balances::free_balance(&1), 0);
        });
    }

    #[test]
    fn greater_nonce_overrides_lesser() {
        new_test_ext().execute_with(|| {
            register_id_mut(1, |id| {
                id.fractal_id = 42;
                id.nonce = 1;
            });
            register_for_minting(1);

            assert_noop!(
                FractalMinting::register_identity(
                    Origin::signed(2),
                    signed_identity(FractalIdentity {
                        account: 2,
                        fractal_id: 42,
                        nonce: 0,
                    })
                ),
                Error::<Test>::NonIncreasingNonce
            );

            run_to_next_minting();

            assert_eq!(Balances::free_balance(&2), 0);
            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[test]
    fn overridden_account_disallows_minting() {
        new_test_ext().execute_with(|| {
            register_id_mut(1, |id| {
                id.fractal_id = 42;
            });
            register_id_mut(2, |id| {
                id.fractal_id = 42;
                id.nonce = 1;
            });

            assert_noop!(
                FractalMinting::register_for_minting(
                    Origin::signed(1),
                    None,
                    simple_tree().prune_balanced()
                ),
                Error::<Test>::NoIdentityRegistered
            );
        });
    }

    #[test]
    fn greater_nonce_overrides_lesser_in_subsequent_minting() {
        new_test_ext().execute_with(|| {
            register_id_mut(1, |id| {
                id.fractal_id = 42;
                id.nonce = 1;
            });

            run_to_next_minting();

            assert_noop!(
                FractalMinting::register_identity(
                    Origin::signed(2),
                    signed_identity(FractalIdentity {
                        account: 2,
                        fractal_id: 42,
                        nonce: 0,
                    })
                ),
                Error::<Test>::NonIncreasingNonce
            );
        });
    }

    #[test]
    fn minting_requires_identity() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                FractalMinting::register_for_minting(
                    Origin::signed(1),
                    None,
                    simple_tree().prune_balanced()
                ),
                Error::<Test>::NoIdentityRegistered
            );
        });
    }

    #[test]
    fn second_identity_to_same_account_gets_double() {
        new_test_ext().execute_with(|| {
            register_id_mut(1, |id| {
                id.fractal_id = 42;
            });
            register_id_mut(1, |id| {
                id.fractal_id = 43;
            });

            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                Some(42),
                simple_tree().prune_balanced()
            ));
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                Some(43),
                simple_tree().prune_balanced()
            ));
            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                2 * <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[test]
    fn second_identity_to_same_account_overridden() {
        new_test_ext().execute_with(|| {
            register_id_mut(1, |id| {
                id.fractal_id = 42;
            });
            register_id_mut(1, |id| {
                id.fractal_id = 43;
            });
            register_id_mut(2, |id| {
                id.fractal_id = 43;
                id.nonce = 1;
            });

            register_for_minting(1);
            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[test]
    fn second_identity_after_registered_for_minting() {
        new_test_ext().execute_with(|| {
            register_id_mut(1, |id| {
                id.fractal_id = 42;
            });
            register_id_mut(1, |id| {
                id.fractal_id = 43;
            });
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                Some(42),
                simple_tree().prune_balanced()
            ));
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                Some(43),
                simple_tree().prune_balanced()
            ));

            register_id_mut(2, |id| {
                id.fractal_id = 43;
                id.nonce = 1;
            });

            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[cfg(test)]
    mod extension_proofs {
        use super::*;

        #[test]
        fn second_proof_does_not_extend_initial_proof() {
            new_test_ext().execute_with(|| {
                register_id(1);
                assert_ok!(FractalMinting::register_for_minting(
                    Origin::signed(1),
                    None,
                    simple_tree().prune_balanced()
                ));
                assert_noop!(
                    FractalMinting::register_for_minting(
                        Origin::signed(1),
                        None,
                        simple_tree().prune_balanced()
                    ),
                    Error::<Test>::ExtensionDoesNotExtendExistingDataset
                );
            });
        }

        #[test]
        fn multiple_identities_requires_specifying_identity() {
            new_test_ext().execute_with(|| {
                register_id_mut(1, |id| {
                    id.fractal_id = 42;
                });
                register_id_mut(1, |id| {
                    id.fractal_id = 43;
                });

                assert_noop!(
                    FractalMinting::register_for_minting(
                        Origin::signed(1),
                        None,
                        simple_tree().prune_balanced()
                    ),
                    Error::<Test>::MustSpecifyFractalIdWithMultipleIds
                );
            });
        }

        #[test]
        fn provided_identity_not_registered() {
            new_test_ext().execute_with(|| {
                register_id_mut(1, |id| {
                    id.fractal_id = 42;
                });

                assert_noop!(
                    FractalMinting::register_for_minting(
                        Origin::signed(1),
                        Some(43),
                        simple_tree().prune_balanced()
                    ),
                    Error::<Test>::FractalIdNotRegisteredToAccount
                );
            });
        }
    }
}
