use crate::mock::*;
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, OnFinalize, OnInitialize},
};
use num_bigint::BigUint;

#[cfg(test)]
mod register_identity {
    use super::*;

    type Error = crate::Error<Test>;

    const DEFAULT_LOCK: u64 = 100;

    fn run_test(f: impl FnOnce()) {
        new_test_ext().execute_with(|| {
            step_block();
            assert_ok!(FractalStaking::set_lock_period_shares(
                Origin::root(),
                DEFAULT_LOCK,
                10
            ));

            f();

            check_invariants();
        });
    }

    fn step_block() {
        FractalStaking::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        FractalStaking::on_initialize(System::block_number());
    }

    fn check_invariants() {
        let coin_shares: BigUint = crate::pallet::StakedAmounts::<Test>::iter()
            .map(|(_, _, sb)| sb.coin_shares())
            .sum();
        assert_eq!(
            coin_shares,
            crate::pallet::TotalCoinShares::<Test>::get().into(),
            "TotalCoinShares incorrect"
        );

        for (_, _, sb) in crate::pallet::StakedAmounts::<Test>::iter() {
            assert_ne!(sb.balance(), 0);
        }
    }

    fn staked_balance(account: u64) -> u64 {
        crate::StakedAmounts::<Test>::iter_prefix_values(account)
            .map(|sb| sb.balance())
            .sum()
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
        for _ in 0..DEFAULT_LOCK {
            step_block();
        }
    }

    #[test]
    fn distributes_to_single_user() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                100_000
            ));

            run_to_distribution();

            assert_eq!(staked_balance(1), 200_000);
        });
    }

    #[test]
    fn distributes_to_reserved_balance() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                100_000
            ));

            run_to_distribution();

            assert_eq!(Balances::free_balance(1), 0);
        });
    }

    #[test]
    fn disallows_staking_more_than_owned() {
        run_test(|| {
            let _ = Balances::deposit_creating(&1, 100_000);
            assert_noop!(
                FractalStaking::stake(Origin::signed(1), DEFAULT_LOCK, 100_001),
                Error::CannotStakeMoreThanBalance
            );
        });
    }

    #[test]
    fn distributes_among_many_users() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                100_000
            ));

            let _ = Balances::deposit_creating(&2, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(2),
                DEFAULT_LOCK,
                100_000
            ));

            run_to_distribution();

            assert_eq!(staked_balance(1), 150_000);
            assert_eq!(staked_balance(2), 150_000);
        });
    }

    #[test]
    fn automatically_withdraws_past_lock_period() {
        run_test(|| {
            let staked_at = System::block_number();
            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                100_000
            ));

            while System::block_number() < (staked_at + DEFAULT_LOCK) {
                step_block();
            }
            run_to_distribution();

            assert_eq!(Balances::free_balance(1), 100_000);
            assert_eq!(staked_balance(1), 0);
        });
    }

    #[test]
    fn only_withdraws_amount_distributed_this_time() {
        run_test(|| {
            let first_stake_at = System::block_number();
            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                50_000
            ));
            run_to_distribution();

            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                50_000
            ));

            while System::block_number() < (first_stake_at + DEFAULT_LOCK) {
                step_block();
            }
            set_distribution_source(20_000);
            run_to_distribution();

            assert_eq!(Balances::free_balance(1), 50_000 + 10_000);
            assert_eq!(staked_balance(1), 50_000 + 10_000);
        });
    }

    #[test]
    fn withdrawn_users_do_not_receive_stake() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                100_000
            ));
            step_past_lock_period();
            run_to_distribution();

            set_distribution_source(100_000);
            let _ = Balances::deposit_creating(&2, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(2),
                DEFAULT_LOCK,
                100_000
            ));

            run_to_distribution();

            assert_eq!(staked_balance(2), 200_000);
        });
    }

    #[test]
    fn distributes_proportionally_to_users() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                100_000
            ));

            let _ = Balances::deposit_creating(&2, 50_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(2),
                DEFAULT_LOCK,
                50_000
            ));

            run_to_distribution();

            assert_eq!(staked_balance(1), 100_000 + 66_666);
            assert_eq!(staked_balance(2), 50_000 + 33_333);
        });
    }

    #[test]
    fn returns_excess_to_distribution_account() {
        run_test(|| {
            set_distribution_source(100_000);

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                100_000
            ));

            let _ = Balances::deposit_creating(&2, 50_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(2),
                DEFAULT_LOCK,
                50_000
            ));

            run_to_distribution();

            assert_eq!(
                Balances::free_balance(&<Test as crate::Config>::DistributionSource::get()),
                1
            );
        });
    }

    #[test]
    fn distributes_more_from_larger_locks() {
        run_test(|| {
            set_distribution_source(100_000);

            assert_ok!(FractalStaking::set_lock_period_shares(
                Origin::root(),
                200,
                20
            ));

            let _ = Balances::deposit_creating(&1, 100_000);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                100_000
            ));

            let _ = Balances::deposit_creating(&2, 100_000);
            assert_ok!(FractalStaking::stake(Origin::signed(2), 200, 100_000));

            run_to_distribution();

            assert_eq!(Balances::reserved_balance(1), 133_333);
            assert_eq!(Balances::reserved_balance(2), 166_666);
        });
    }

    #[test]
    fn disallows_stake_below_minimum() {
        run_test(|| {
            assert_ok!(FractalStaking::set_minimum_stake(Origin::root(), Some(100)));

            assert_noop!(
                FractalStaking::stake(Origin::signed(1), DEFAULT_LOCK, 99),
                Error::AmountBelowMinimum
            );
        });
    }

    #[cfg(test)]
    mod many_in_one_block {
        use super::*;

        #[test]
        fn increments_staked_amounts_with_multiple_in_one_block() {
            run_test(|| {
                let _ = Balances::deposit_creating(&1, 100_000);

                assert_ok!(FractalStaking::stake(
                    Origin::signed(1),
                    DEFAULT_LOCK,
                    50_000
                ));
                assert_ok!(FractalStaking::stake(
                    Origin::signed(1),
                    DEFAULT_LOCK,
                    50_000
                ));

                assert_eq!(staked_balance(1), 100_000);
            });
        }

        #[test]
        fn stakes_with_mismatched_shares() {
            run_test(|| {
                assert_ok!(FractalStaking::set_lock_period_shares(
                    Origin::root(),
                    DEFAULT_LOCK + 1,
                    20
                ));

                let _ = Balances::deposit_creating(&1, 100_000);

                assert_ok!(FractalStaking::stake(
                    Origin::signed(1),
                    DEFAULT_LOCK + 1,
                    50_000
                ));
                step_block();

                assert_ok!(FractalStaking::stake(
                    Origin::signed(1),
                    DEFAULT_LOCK,
                    50_000
                ));

                assert_eq!(staked_balance(1), 100_000);
            });
        }

        #[test]
        fn does_not_overdistribute_to_multiple_stakes() {
            run_test(|| {
                assert_ok!(FractalStaking::set_lock_period_shares(
                    Origin::root(),
                    DEFAULT_LOCK + 1,
                    10
                ));

                let _ = Balances::deposit_creating(&1, 100_000);

                assert_ok!(FractalStaking::stake(
                    Origin::signed(1),
                    DEFAULT_LOCK + 1,
                    50_000
                ));
                step_block();

                assert_ok!(FractalStaking::stake(
                    Origin::signed(1),
                    DEFAULT_LOCK,
                    50_000
                ));

                set_distribution_source(100_000);
                run_to_distribution();

                assert_eq!(staked_balance(1), 200_000);
            });
        }
    }

    #[test]
    fn does_not_fail_on_overflow() {
        run_test(|| {
            set_distribution_source(100_000);

            let balance = u64::MAX - 100_000;
            let _ = Balances::deposit_creating(&1, balance);
            assert_ok!(FractalStaking::stake(
                Origin::signed(1),
                DEFAULT_LOCK,
                balance
            ));

            run_to_distribution();

            assert_eq!(staked_balance(1), u64::MAX);
        });
    }
}
