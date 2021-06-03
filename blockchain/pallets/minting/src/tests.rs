use crate::mock::*;
use frame_support::{
    assert_ok,
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

    #[test]
    fn receives_portion_of_minting_after_block() {
        new_test_ext().execute_with(|| {
            assert_ok!(FractalMinting::register_for_minting(Origin::signed(1)));

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
            assert_ok!(FractalMinting::register_for_minting(Origin::signed(1)));

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
                assert_ok!(FractalMinting::register_for_minting(Origin::signed(id)));
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
            assert_ok!(FractalMinting::register_for_minting(Origin::signed(1)));
            assert_ok!(FractalMinting::register_for_minting(Origin::signed(1)));
            assert_ok!(FractalMinting::register_for_minting(Origin::signed(1)));

            run_to_next_minting();

            assert_eq!(
                Balances::free_balance(&1),
                <Test as crate::Config>::MaxRewardPerUser::get()
            );
        });
    }

    // TODO(shelbyd): Add unit tests for event emitting.
}
