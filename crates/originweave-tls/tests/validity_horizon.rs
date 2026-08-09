use std::time::Duration;

use originweave_tls::{LeafValidityHorizon, LeafValidityHorizonError};

#[test]
fn zero_horizon_accepts_a_certificate_valid_at_the_trusted_second() {
    let horizon = LeafValidityHorizon::new(Duration::ZERO);
    assert_eq!(horizon.minimum_remaining(), Duration::ZERO);
    assert_eq!(horizon.evaluate(1_000, 1_000), Ok(()));
}

#[test]
fn exact_remaining_horizon_is_accepted() {
    let horizon = LeafValidityHorizon::new(Duration::from_secs(300));
    assert_eq!(horizon.evaluate(1_000, 1_300), Ok(()));
}

#[test]
fn one_second_short_fails_closed_with_a_typed_budget_error() {
    let horizon = LeafValidityHorizon::new(Duration::from_secs(300));
    assert_eq!(
        horizon.evaluate(1_000, 1_299),
        Err(LeafValidityHorizonError::InsufficientRemainingValidity {
            remaining_seconds: 299,
            minimum_seconds: 300,
        })
    );
}

#[test]
fn expired_or_pre_epoch_not_after_never_underflows() {
    let horizon = LeafValidityHorizon::new(Duration::from_secs(1));
    for not_after in [-1, 999] {
        assert_eq!(
            horizon.evaluate(1_000, not_after),
            Err(LeafValidityHorizonError::InsufficientRemainingValidity {
                remaining_seconds: 0,
                minimum_seconds: 1,
            })
        );
    }
}

#[test]
fn fractional_horizons_round_up_to_the_next_certificate_second() {
    let horizon = LeafValidityHorizon::new(Duration::new(1, 1));
    assert_eq!(
        horizon.evaluate(1_000, 1_001),
        Err(LeafValidityHorizonError::InsufficientRemainingValidity {
            remaining_seconds: 1,
            minimum_seconds: 2,
        })
    );
    assert_eq!(horizon.evaluate(1_000, 1_002), Ok(()));
}

#[test]
fn extreme_horizon_uses_saturating_seconds_instead_of_wrapping() {
    let horizon = LeafValidityHorizon::new(Duration::MAX);
    assert_eq!(
        horizon.evaluate(0, i64::MAX),
        Err(LeafValidityHorizonError::InsufficientRemainingValidity {
            remaining_seconds: i64::MAX as u64,
            minimum_seconds: u64::MAX,
        })
    );
}
