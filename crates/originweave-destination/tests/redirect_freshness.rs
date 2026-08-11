#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
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

#[test]
fn redirect_rejects_expired_resolution_authority_without_advancing_chain() {
    let initial = origin("https://start.example");
    let target = origin("https://target.example");
    let approved_at = Duration::from_secs(10);
    let validity = Duration::from_secs(2);
    let resolution = FreshResolutionSnapshot::approve(
        target.clone(),
        [IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
        &DestinationPolicy::public_web(),
        approved_at,
        validity,
    )
    .expect("fresh public resolution must be approved");
    let grants = BTreeSet::from([target.clone()]);
    let mut guard = RedirectGuard::new(initial.clone(), digest(INITIAL_DIGEST), 2)
        .expect("redirect guard must be valid");

    assert_eq!(
        guard.authorize_redirect(
            target,
            digest(TARGET_DIGEST),
            &resolution,
            approved_at + validity,
            &grants,
        ),
        Err(RedirectError::ResolutionFreshnessDenied {
            error: DestinationError::ResolutionApprovalExpired {
                valid_until: approved_at + validity,
                current_time: approved_at + validity,
            },
        })
    );
    assert_eq!(guard.current_origin(), &initial);
    assert_eq!(guard.hop_count(), 0);
}
