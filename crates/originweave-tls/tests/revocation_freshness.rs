use std::error::Error as _;

use originweave_tls::{RevocationMaterialFreshness, RevocationMaterialFreshnessError};

#[test]
fn revocation_material_freshness_uses_a_half_open_verified_window() {
    let freshness = RevocationMaterialFreshness::new(1_000, 1_100);
    assert!(freshness.is_ok());

    if let Ok(freshness) = freshness {
        assert_eq!(freshness.this_update_unix_seconds(), 1_000);
        assert_eq!(freshness.next_update_unix_seconds(), 1_100);
        assert_eq!(freshness.evaluate(1_000), Ok(()));
        assert_eq!(freshness.evaluate(1_099), Ok(()));
        assert_eq!(
            freshness.evaluate(999),
            Err(RevocationMaterialFreshnessError::NotYetValid {
                trusted_time_unix_seconds: 999,
                this_update_unix_seconds: 1_000,
            })
        );
        assert_eq!(
            freshness.evaluate(1_100),
            Err(RevocationMaterialFreshnessError::Expired {
                trusted_time_unix_seconds: 1_100,
                next_update_unix_seconds: 1_100,
            })
        );
    }
}

#[test]
fn revocation_material_freshness_rejects_empty_or_reversed_windows() {
    for (this_update, next_update) in [(1_000, 1_000), (1_001, 1_000)] {
        assert_eq!(
            RevocationMaterialFreshness::new(this_update, next_update),
            Err(RevocationMaterialFreshnessError::InvalidWindow {
                this_update_unix_seconds: this_update,
                next_update_unix_seconds: next_update,
            })
        );
    }
}

#[test]
fn revocation_freshness_errors_are_stable_and_source_free() {
    let invalid = RevocationMaterialFreshnessError::InvalidWindow {
        this_update_unix_seconds: 1_000,
        next_update_unix_seconds: 1_000,
    };
    let future = RevocationMaterialFreshnessError::NotYetValid {
        trusted_time_unix_seconds: 999,
        this_update_unix_seconds: 1_000,
    };
    let stale = RevocationMaterialFreshnessError::Expired {
        trusted_time_unix_seconds: 1_100,
        next_update_unix_seconds: 1_100,
    };

    assert_eq!(
        invalid.to_string(),
        "revocation material window is invalid: thisUpdate 1000 must be before nextUpdate 1000"
    );
    assert_eq!(
        future.to_string(),
        "revocation material is not usable at trusted time 999; thisUpdate is 1000"
    );
    assert_eq!(
        stale.to_string(),
        "revocation material is stale at trusted time 1100; nextUpdate is 1100"
    );

    for error in [invalid, future, stale] {
        assert!(error.source().is_none());
    }
}
