use crate::{mock::*, *};
use frame_support::{
    assert_noop, assert_ok,
    traits::{tokens::currency::Currency, OnFinalize, OnInitialize},
};

#[cfg(test)]
mod token_distribution {
    use super::*;

    const FIRST_MINTING_TOTAL: u64 = 484_923;
    const SECOND_MINTING_TOTAL: u64 = 484_363;
    const TEST_PURPOSE: u8 = 13;

    fn run_test(f: impl FnOnce()) {
        new_test_ext().execute_with(|| {
            step_block();
            f();
        });
    }

    fn step_block() {
        FractalTokenDistribution::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        FractalTokenDistribution::on_initialize(System::block_number());
    }

    #[cfg(test)]
    mod per_block {
        use super::*;

        #[test]
        fn distributes_to_single_address() {
            run_test(|| {
                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(42),
                    1
                ));
                step_block();

                assert_eq!(Balances::free_balance(&42), FIRST_MINTING_TOTAL);
            });
        }

        #[test]
        fn distributes_among_many_addresses() {
            run_test(|| {
                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(42),
                    1
                ));
                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(43),
                    1
                ));
                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(44),
                    1
                ));

                step_block();

                assert_eq!(Balances::free_balance(&42), FIRST_MINTING_TOTAL / 3);
                assert_eq!(Balances::free_balance(&43), FIRST_MINTING_TOTAL / 3);
                assert_eq!(Balances::free_balance(&44), FIRST_MINTING_TOTAL / 3);
            });
        }

        #[test]
        fn requires_root() {
            run_test(|| {
                assert_noop!(
                    FractalTokenDistribution::set_weight(
                        Origin::signed(1),
                        Destination::Address(42),
                        1
                    ),
                    sp_runtime::traits::BadOrigin
                );
            });
        }

        #[cfg(test)]
        mod purpose {
            use super::*;

            #[test]
            fn distributes_to_single_purpose() {
                run_test(|| {
                    assert_ok!(FractalTokenDistribution::set_weight(
                        Origin::root(),
                        Destination::Purpose(TEST_PURPOSE),
                        1
                    ));
                    step_block();

                    assert_eq!(
                        FractalTokenDistribution::take_from(TEST_PURPOSE),
                        FIRST_MINTING_TOTAL
                    );
                });
            }

            #[test]
            fn accounts_for_amount_in_distribution() {
                run_test(|| {
                    assert_ok!(FractalTokenDistribution::set_weight(
                        Origin::root(),
                        Destination::Purpose(TEST_PURPOSE),
                        1
                    ));
                    step_block();

                    assert_ok!(FractalTokenDistribution::set_weight(
                        Origin::root(),
                        Destination::Address(42),
                        1
                    ));
                    step_block();

                    assert_eq!(Balances::free_balance(42), SECOND_MINTING_TOTAL / 2);
                });
            }

            #[test]
            fn take_leaves_amount_as_zero() {
                run_test(|| {
                    assert_ok!(FractalTokenDistribution::set_weight(
                        Origin::root(),
                        Destination::Purpose(TEST_PURPOSE),
                        1
                    ));
                    step_block();

                    FractalTokenDistribution::take_from(TEST_PURPOSE);
                    assert_eq!(FractalTokenDistribution::take_from(TEST_PURPOSE), 0);
                });
            }

            #[test]
            fn return_after_take() {
                run_test(|| {
                    assert_ok!(FractalTokenDistribution::set_weight(
                        Origin::root(),
                        Destination::Purpose(TEST_PURPOSE),
                        1
                    ));
                    step_block();

                    FractalTokenDistribution::take_from(TEST_PURPOSE);
                    FractalTokenDistribution::return_to(TEST_PURPOSE, 100);

                    assert_eq!(FractalTokenDistribution::take_from(TEST_PURPOSE), 100);
                });
            }
        }

        #[test]
        fn distributes_additional_based_on_already_distributed() {
            run_test(|| {
                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(42),
                    1
                ));
                step_block();
                let first = Balances::free_balance(&42);

                step_block();
                assert_eq!(Balances::free_balance(&42) - first, SECOND_MINTING_TOTAL);
            });
        }

        #[test]
        fn accounts_for_genesis_issuance() {
            run_test(|| {
                let _ = Balances::deposit_creating(&43, 100_000);
                assert_ok!(FractalTokenDistribution::increment_artificially_issued(
                    Origin::root(),
                    Balances::total_issuance()
                ));

                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(42),
                    1
                ));
                step_block();

                assert_eq!(Balances::free_balance(&42), FIRST_MINTING_TOTAL);
            });
        }

        #[test]
        fn minting_on_demand() {
            run_test(|| {
                assert_ok!(FractalTokenDistribution::mint(Origin::root(), 42, 123456));

                assert_eq!(Balances::free_balance(&42), 123456);
            });
        }

        #[test]
        fn minting_on_demand_decrements_issuance() {
            run_test(|| {
                assert_ok!(FractalTokenDistribution::mint(Origin::root(), 43, 120000));

                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(42),
                    1
                ));
                step_block();

                assert_eq!(Balances::free_balance(&42), FIRST_MINTING_TOTAL);
            });
        }

        #[test]
        fn issues_more_to_offset() {
            run_test(|| {
                let _ = Balances::deposit_creating(&43, 100_000);
                assert_ok!(FractalTokenDistribution::increment_artificially_issued(
                    Origin::root(),
                    120_000
                ));

                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(42),
                    1
                ));
                step_block();

                assert_eq!(Balances::free_balance(&42), FIRST_MINTING_TOTAL + 20_000);
            });
        }

        #[test]
        fn issues_less_to_offset() {
            run_test(|| {
                let _ = Balances::deposit_creating(&43, 1_000_000);
                assert_ok!(FractalTokenDistribution::increment_artificially_issued(
                    Origin::root(),
                    20_000
                ));

                assert_ok!(FractalTokenDistribution::set_weight(
                    Origin::root(),
                    Destination::Address(42),
                    1
                ));
                step_block();

                assert_eq!(Balances::free_balance(&42), 0);
            });
        }
    }
}
