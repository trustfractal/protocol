use crate::{mock::*, *};
use frame_support::{
    assert_noop, assert_ok,
    traits::{OnFinalize, OnInitialize},
};

#[cfg(test)]
mod register_for_minting {
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

    #[test]
    fn receives_portion_of_minting_after_block() {
        new_test_ext().execute_with(|| {
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                identity(1)
            ));

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

            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                identity(1)
            ));
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
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                identity(1)
            ));

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
                assert_ok!(FractalMinting::register_for_minting(
                    Origin::signed(id),
                    identity(id)
                ));
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
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                identity(1)
            ));
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                identity(1)
            ));
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                identity(1)
            ));

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

            let signed_message = bad_signature.into_iter()
                .chain(message.encode())
                .collect::<Vec<_>>();
            let identity = Signed::decode(&mut signed_message.as_ref()).unwrap();

            assert_noop!(
                FractalMinting::register_for_minting(Origin::signed(1), identity),
                Error::<Test>::InvalidIdentitySignature
            );
        });
    }

    #[test]
    fn errors_when_fractal_identity_does_not_match_origin_account() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                FractalMinting::register_for_minting(Origin::signed(1), identity(2)),
                Error::<Test>::MismatchedFractalIdentity
            );
        });
    }

    #[test]
    fn later_fractal_id_replaces_previous() {
        new_test_ext().execute_with(|| {
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                signed_identity(FractalIdentity {
                    account: 1,
                    fractal_id: 42,
                    nonce: 0,
                })
            ));
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(2),
                signed_identity(FractalIdentity {
                    account: 2,
                    fractal_id: 42,
                    nonce: 0,
                })
            ));

            run_to_next_minting();

            assert_eq!(Balances::free_balance(&1), 0);
        });
    }

    #[test]
    fn greater_nonce_overrides_lesser() {
        new_test_ext().execute_with(|| {
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                signed_identity(FractalIdentity {
                    account: 1,
                    fractal_id: 42,
                    nonce: 1,
                })
            ));
            assert_noop!(
                FractalMinting::register_for_minting(
                    Origin::signed(2),
                    signed_identity(FractalIdentity {
                        account: 2,
                        fractal_id: 42,
                        nonce: 0,
                    })
                ),
                Error::<Test>::LesserNonce
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
    fn greater_nonce_overrides_lesser_in_subsequent_minting() {
        new_test_ext().execute_with(|| {
            assert_ok!(FractalMinting::register_for_minting(
                Origin::signed(1),
                signed_identity(FractalIdentity {
                    account: 1,
                    fractal_id: 42,
                    nonce: 1,
                })
            ));

            run_to_next_minting();

            assert_noop!(
                FractalMinting::register_for_minting(
                    Origin::signed(2),
                    signed_identity(FractalIdentity {
                        account: 2,
                        fractal_id: 42,
                        nonce: 0,
                    })
                ),
                Error::<Test>::LesserNonce
            );
        });
    }

    // TODO(shelbyd): Add unit tests for event emitting.
}
