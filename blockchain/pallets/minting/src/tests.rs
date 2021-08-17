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

    fn register_id_account(id: u64, account: u64) {
        assert_ok!(FractalMinting::register_identity(
            Origin::signed(123),
            id,
            account,
        ));
    }

    fn register_for_minting(account: u64) {
        assert_ok!(FractalMinting::register_for_minting(
            Origin::signed(account),
            None,
            simple_tree().prune_balanced(),
        ));
    }

    fn register_for_minting_dataset(account: u64, dataset: &[&'static str]) {
        assert_ok!(FractalMinting::register_for_minting(
            Origin::signed(account),
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
            register_id_account(1, 1);
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

            register_id_account(1, 1);
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
            register_id_account(1, 1);
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
                register_id_account(id, id);
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
            register_id_account(1, 1);
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
    fn new_registration_clears_dataset() {
        new_test_ext().execute_with(|| {
            register_id_account(1, 1);
            register_for_minting_dataset(1, &["1"]);
            register_id_account(1, 2);
            register_for_minting_dataset(2, &["2"]);

            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&2),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[test]
    fn errors_with_invalid_fractal_signature() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                FractalMinting::register_identity(Origin::signed(124), 1, 2),
                Error::<Test>::MustBeFractal
            );
        });
    }

    #[test]
    fn later_fractal_id_replaces_previous() {
        new_test_ext().execute_with(|| {
            register_id_account(42, 1);
            register_for_minting(1);

            register_id_account(42, 2);

            run_to_next_minting();

            assert_eq!(Balances::free_balance(&1), 0);
        });
    }

    #[test]
    fn overridden_account_disallows_minting() {
        new_test_ext().execute_with(|| {
            register_id_account(42, 1);
            register_id_account(42, 2);

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
            register_id_account(42, 1);
            register_id_account(43, 1);

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
            register_id_account(42, 1);
            register_id_account(43, 1);
            register_id_account(43, 2);

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
            register_id_account(42, 1);
            register_id_account(43, 1);
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

            register_id_account(43, 2);

            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    #[cfg(test)]
    mod extension_proofs {
        use frame_support::dispatch::PostDispatchInfo;

        use super::*;

        #[test]
        fn second_proof_does_not_extend_initial_proof() {
            new_test_ext().execute_with(|| {
                register_id_account(1, 1);
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
                register_id_account(42, 1);
                register_id_account(43, 1);

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
                register_id_account(42, 1);

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

        #[test]
        fn register_for_minting_is_free_for_first_call() {
            use frame_support::pallet_prelude::Pays;

            fn gen_tree(r: &[&str]) -> MerkleTree<Blake2b> {
                MerkleTree::from_iter(r).unwrap()
            }

            new_test_ext().execute_with(|| {
                register_id_account(42, 1);

                // first call is free, actual_weight set to 0
                let tree_0 = gen_tree(&["a", "b"]);
                assert_eq!(
                    FractalMinting::register_for_minting(Origin::signed(1), Some(42), tree_0),
                    Ok(PostDispatchInfo {
                        actual_weight: Some(0),
                        pays_fee: Pays::Yes
                    })
                );

                // second is charged, actual_weith None indicates annotation
                // weight is applied
                let tree_1 = gen_tree(&["a", "b", "c"]);
                assert_eq!(
                    FractalMinting::register_for_minting(Origin::signed(1), Some(42), tree_1),
                    Ok(PostDispatchInfo {
                        actual_weight: None,
                        pays_fee: Pays::Yes
                    })
                );
            });
        }
    }
}
