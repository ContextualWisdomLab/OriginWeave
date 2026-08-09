use std::time::Duration;

use originweave_tls::{
    AlpnRequirement, LeafValidityHorizon, LeafValidityHorizonError, MAX_MINIMUM_LEAF_VALIDITY,
    TlsClientPolicy, TlsError,
};
use rustls::pki_types::UnixTime;

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

#[test]
fn typed_failure_is_usable_in_standard_error_chains_without_a_source() {
    let result = LeafValidityHorizon::new(Duration::from_secs(300)).evaluate(1_000, 1_299);
    assert_eq!(
        result,
        Err(LeafValidityHorizonError::InsufficientRemainingValidity {
            remaining_seconds: 299,
            minimum_seconds: 300,
        })
    );
    let Err(error) = result else {
        return;
    };
    assert_eq!(
        error.to_string(),
        "TLS leaf certificate has 299 seconds remaining; delegated task requires at least 300 seconds"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn tls_policy_defaults_to_zero_leaf_horizon_for_existing_callers() {
    let result = TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(1_000)),
        Duration::from_secs(1),
        Vec::new(),
        AlpnRequirement::Optional,
    );
    assert!(result.is_ok());
    let Ok(policy) = result else {
        return;
    };
    assert_eq!(policy.minimum_leaf_validity(), Duration::ZERO);
}

#[test]
fn tls_policy_bounds_the_configurable_leaf_horizon() {
    assert_eq!(MAX_MINIMUM_LEAF_VALIDITY, Duration::from_secs(604_800));
    let result = TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(1_000)),
        Duration::from_secs(1),
        Vec::new(),
        AlpnRequirement::Optional,
    );
    assert!(result.is_ok());
    let Ok(policy) = result else {
        return;
    };

    let accepted = policy
        .clone()
        .with_minimum_leaf_validity(MAX_MINIMUM_LEAF_VALIDITY);
    assert!(accepted.is_ok());
    let Ok(accepted) = accepted else {
        return;
    };
    assert_eq!(accepted.minimum_leaf_validity(), MAX_MINIMUM_LEAF_VALIDITY);

    let excessive = policy.with_minimum_leaf_validity(
        MAX_MINIMUM_LEAF_VALIDITY.saturating_add(Duration::from_nanos(1)),
    );
    assert!(excessive.is_err());
    let Err(error) = excessive else {
        return;
    };
    match error {
        TlsError::InvalidMinimumLeafValidity {
            minimum_validity,
            maximum_validity,
        } => {
            assert_eq!(
                minimum_validity,
                MAX_MINIMUM_LEAF_VALIDITY.saturating_add(Duration::from_nanos(1))
            );
            assert_eq!(maximum_validity, MAX_MINIMUM_LEAF_VALIDITY);
        }
        other => assert_eq!(
            other.to_string(),
            "TLS minimum leaf validity exceeds the supported product maximum"
        ),
    }
}
