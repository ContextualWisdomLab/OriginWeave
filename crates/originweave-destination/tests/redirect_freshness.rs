#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{
    DestinationError, DestinationPolicy, FreshResolutionSnapshot, RedirectError, RedirectGuard,
    RedirectTargetDigest,
};

const INITIAL_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TARGET_DIGEST: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must parse")
}

fn digest(value: &str) -> RedirectTargetDigest {
    RedirectTargetDigest::parse(value).expect("test digest must parse")
}

fn fresh_resolution(
    target: &Origin,
    approved_at: Duration,
    validity: Duration,
) -> FreshResolutionSnapshot {
    FreshResolutionSnapshot::approve(
        target.clone(),
        [IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
        &DestinationPolicy::public_web(),
        approved_at,
        validity,
    )
    .expect("fresh public resolution must be approved")
}

#[test]
fn redirect_rejects_expired_resolution_authority_without_advancing_chain() {
    let initial = origin("https://start.example");
    let target = origin("https://target.example");
    let approved_at = Duration::from_secs(10);
    let validity = Duration::from_secs(2);
    let current_time = approved_at + validity;
    let resolution = fresh_resolution(&target, approved_at, validity);
    let grants = BTreeSet::from([target.clone()]);
    let mut guard = RedirectGuard::new(initial.clone(), digest(INITIAL_DIGEST), 2)
        .expect("redirect guard must be valid");

    let error = guard
        .authorize_redirect(
            target,
            digest(TARGET_DIGEST),
            &resolution,
            current_time,
            &grants,
        )
        .expect_err("exclusive freshness deadline must reject redirect");
    assert_eq!(
        error,
        RedirectError::ResolutionFreshnessDenied {
            error: DestinationError::ResolutionApprovalExpired {
                valid_until: current_time,
                current_time,
            },
        }
    );
    assert_eq!(
        error.to_string(),
        "redirect resolution freshness denied: resolution approval expired at 12s; current time is 12s"
    );
    let standard: &dyn Error = &error;
    assert_eq!(
        standard
            .source()
            .expect("freshness wrapper must preserve source")
            .to_string(),
        "resolution approval expired at 12s; current time is 12s"
    );
    assert_eq!(guard.current_origin(), &initial);
    assert_eq!(guard.hop_count(), 0);
}

#[test]
fn redirect_rejects_resolution_use_before_approval_without_advancing_chain() {
    let initial = origin("https://start.example");
    let target = origin("https://target.example");
    let approved_at = Duration::from_secs(10);
    let current_time = Duration::from_secs(9);
    let resolution = fresh_resolution(&target, approved_at, Duration::from_secs(2));
    let grants = BTreeSet::from([target.clone()]);
    let mut guard = RedirectGuard::new(initial.clone(), digest(INITIAL_DIGEST), 2)
        .expect("redirect guard must be valid");

    assert_eq!(
        guard.authorize_redirect(
            target,
            digest(TARGET_DIGEST),
            &resolution,
            current_time,
            &grants,
        ),
        Err(RedirectError::ResolutionFreshnessDenied {
            error: DestinationError::ResolutionUseBeforeApproval {
                approved_at,
                current_time,
            },
        })
    );
    assert_eq!(guard.current_origin(), &initial);
    assert_eq!(guard.hop_count(), 0);
}
