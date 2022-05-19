use crate::mock::*;
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, OnFinalize, OnInitialize},
};

#[cfg(test)]
mod register_identity {
    use super::*;

    type Error = crate::Error<Test>;

    fn run_test(f: impl FnOnce()) {
        new_test_ext().execute_with(|| {
            step_block();

            f();
        });
    }

    fn step_block() {
        FractalStaking::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        FractalStaking::on_initialize(System::block_number());
    }

    fn run_to_distribution() {
        let distribute_every_n = <Test as crate::Config>::DistributeEveryNBlocks::get();

        loop {
            step_block();

            if System::block_number() % distribute_every_n == 1 {
                break;
            }
        }
    }

    fn set_distribution_source(amount: u64) {
        Balances::make_free_balance_be(&<Test as crate::Config>::DistributionSource::get(), amount);
    }

    fn step_past_lock_period() {
        for _ in 0..<Test as crate::Config>::StakingLockPeriod::get() {
            step_block();
        }
    }

    #[test]
    fn distributes_to_single_user() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(1), 100_000));

            run_to_distribution();

            assert_ok!(FractalStaking::withdraw(Origin::signed(1), 200_000));
            assert_eq!(Balances::free_balance(1), 200_000);
        });
    }

    #[test]
    fn disallows_staking_more_than_owned() {
        run_test(|| {
            let _ = Balances::deposit_creating(&1, 100_000);
            assert_noop!(
                FractalStaking::stake(Origin::signed(1), 100_001),
                Error::CannotStakeMoreThanBalance
            );
        });
    }

    #[test]
    fn disallows_withdrawing_more_than_staked() {
        run_test(|| {
            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(1), 100_000));

            step_past_lock_period();

            assert_noop!(
                FractalStaking::withdraw(Origin::signed(1), 100_001),
                Error::NotEnoughUnlockedStake
            );
        });
    }

    #[test]
    fn disallows_double_withdraw() {
        run_test(|| {
            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(1), 100_000));

            step_past_lock_period();

            assert_ok!(FractalStaking::withdraw(Origin::signed(1), 50_000),);
            assert_noop!(
                FractalStaking::withdraw(Origin::signed(1), 50_001),
                Error::NotEnoughUnlockedStake
            );
        });
    }

    #[test]
    fn distributes_among_many_users() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(1), 100_000));

            let _ = Balances::deposit_creating(&2, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(2), 100_000));

            run_to_distribution();

            assert_noop!(
                FractalStaking::withdraw(Origin::signed(1), 150_001),
                Error::NotEnoughUnlockedStake
            );
            assert_noop!(
                FractalStaking::withdraw(Origin::signed(2), 150_001),
                Error::NotEnoughUnlockedStake
            );
        });
    }

    #[test]
    fn withdrawn_users_do_not_receive_stake() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(1), 100_000));
            step_past_lock_period();
            assert_ok!(FractalStaking::withdraw(Origin::signed(1), 100_000));

            let _ = Balances::deposit_creating(&2, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(2), 100_000));

            run_to_distribution();

            assert_ok!(FractalStaking::withdraw(Origin::signed(2), 200_000));
        });
    }

    #[test]
    fn distributes_proportionally_to_users() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(1), 100_000));

            let _ = Balances::deposit_creating(&2, 50_000);
            assert_ok!(FractalStaking::stake(Origin::signed(2), 50_000));

            run_to_distribution();

            assert_noop!(
                FractalStaking::withdraw(Origin::signed(1), 166_667),
                Error::NotEnoughUnlockedStake
            );
            assert_noop!(
                FractalStaking::withdraw(Origin::signed(2), 133_334),
                Error::NotEnoughUnlockedStake
            );
        });
    }

    #[test]
    fn returns_excess_to_distribution_account() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(1), 100_000));

            let _ = Balances::deposit_creating(&2, 50_000);
            assert_ok!(FractalStaking::stake(Origin::signed(2), 50_000));

            run_to_distribution();

            assert_eq!(
                Balances::free_balance(&<Test as crate::Config>::DistributionSource::get()),
                1
            );
        });
    }

    #[test]
    fn disallows_withdrawal_immediately() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(1), 100_000));
            assert_noop!(
                FractalStaking::withdraw(Origin::signed(1), 1),
                Error::NotEnoughUnlockedStake
            );
        });
    }
}
