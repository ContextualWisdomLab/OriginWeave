#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr};

use originweave_core::Origin;
use originweave_destination::{
    AddressClass, DestinationPolicy, MAX_REDIRECT_HOPS, RedirectError, RedirectGuard,
    RedirectTargetDigest, RedirectTargetDigestError, ResolutionSnapshot,
};

const DIGEST_A: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DIGEST_C: &str =
    "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must parse")
}

fn digest(value: &str) -> RedirectTargetDigest {
    RedirectTargetDigest::parse(value).expect("test digest must parse")
}

fn public_resolution(target: &Origin, address: [u8; 4]) -> ResolutionSnapshot {
    ResolutionSnapshot::approve(
        target.clone(),
        [IpAddr::V4(Ipv4Addr::from(address))],
        &DestinationPolicy::public_web(),
    )
    .expect("public resolution")
}

#[test]
fn redirect_target_digest_is_strict_and_canonical() {
    let valid = digest(DIGEST_A);
    assert_eq!(valid.as_str(), DIGEST_A);

    for invalid in [
        "",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "sha256:short",
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        assert_eq!(
            RedirectTargetDigest::parse(invalid),
            Err(RedirectTargetDigestError::InvalidFormat),
            "invalid={invalid}"
        );
    }
}

#[test]
fn redirect_guard_validates_bounds_and_reports_state() {
    let initial = origin("https://start.example");
    assert_eq!(
        RedirectGuard::new(initial.clone(), digest(DIGEST_A), 0),
        Err(RedirectError::InvalidMaximumHops { maximum_hops: 0 })
    );
    assert_eq!(
        RedirectGuard::new(
            initial.clone(),
            digest(DIGEST_A),
            MAX_REDIRECT_HOPS + 1,
        ),
        Err(RedirectError::InvalidMaximumHops {
            maximum_hops: MAX_REDIRECT_HOPS + 1,
        })
    );

    let guard = RedirectGuard::new(initial.clone(), digest(DIGEST_A), 2)
        .expect("valid redirect bound");
    assert_eq!(guard.current_origin(), &initial);
    assert_eq!(guard.hop_count(), 0);
    assert_eq!(guard.maximum_hops(), 2);
}

#[test]
fn every_redirect_hop_reauthorizes_origin_resolution_and_evidence() {
    let initial = origin("https://start.example");
    let target = origin("https://target.example");
    let resolution = public_resolution(&target, [8, 8, 8, 8]);
    let grants = BTreeSet::from([target.clone()]);
    let mut guard = RedirectGuard::new(initial.clone(), digest(DIGEST_A), 3)
        .expect("redirect guard");

    let first = guard
        .authorize_redirect(target.clone(), digest(DIGEST_B), &resolution, &grants)
        .expect("authorized cross-origin redirect");
    assert_eq!(first.hop_number(), 1);
    assert_eq!(first.source_origin(), &initial);
    assert_eq!(first.target_origin(), &target);
    assert_eq!(first.target_digest().as_str(), DIGEST_B);
    assert_eq!(first.approved_address_count(), 1);
    assert_eq!(guard.current_origin(), &target);
    assert_eq!(guard.hop_count(), 1);

    let second = guard
        .authorize_redirect(target.clone(), digest(DIGEST_C), &resolution, &grants)
        .expect("same-origin redirect still has a distinct target digest");
    assert_eq!(second.hop_number(), 2);
    assert_eq!(second.source_origin(), &target);
    assert_eq!(second.target_origin(), &target);
    assert_eq!(guard.hop_count(), 2);
}

#[test]
fn redirect_guard_fails_closed_for_missing_authority_and_mismatched_resolution() {
    let initial = origin("https://start.example");
    let target = origin("https://target.example");
    let other = origin("https://other.example");
    let target_resolution = public_resolution(&target, [8, 8, 8, 8]);
    let other_resolution = public_resolution(&other, [1, 1, 1, 1]);

    let mut ungranted = RedirectGuard::new(initial.clone(), digest(DIGEST_A), 2)
        .expect("redirect guard");
    assert_eq!(
        ungranted.authorize_redirect(
            target.clone(),
            digest(DIGEST_B),
            &target_resolution,
            &BTreeSet::new(),
        ),
        Err(RedirectError::OriginNotGranted {
            origin: target.clone(),
        })
    );

    let mut mismatched = RedirectGuard::new(initial, digest(DIGEST_A), 2)
        .expect("redirect guard");
    assert_eq!(
        mismatched.authorize_redirect(
            target.clone(),
            digest(DIGEST_B),
            &other_resolution,
            &BTreeSet::from([target.clone()]),
        ),
        Err(RedirectError::ResolutionOriginMismatch {
            target_origin: target,
            resolution_origin: other,
        })
    );
}

#[test]
fn redirect_guard_rejects_https_downgrade_cycles_and_excess_hops() {
    let secure = origin("https://secure.example");
    let loopback = origin("http://localhost");
    let loopback_policy = DestinationPolicy::from_allowed_classes([AddressClass::Loopback]);
    let loopback_resolution = ResolutionSnapshot::approve(
        loopback.clone(),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &loopback_policy,
    )
    .expect("managed loopback resolution");
    let mut downgrade = RedirectGuard::new(secure.clone(), digest(DIGEST_A), 2)
        .expect("redirect guard");
    assert_eq!(
        downgrade.authorize_redirect(
            loopback.clone(),
            digest(DIGEST_B),
            &loopback_resolution,
            &BTreeSet::from([loopback.clone()]),
        ),
        Err(RedirectError::InsecureSchemeDowngrade {
            source_origin: secure,
            target_origin: loopback.clone(),
        })
    );

    let target = origin("https://target.example");
    let resolution = public_resolution(&target, [8, 8, 4, 4]);
    let grants = BTreeSet::from([target.clone()]);
    let mut cycle = RedirectGuard::new(origin("https://start.example"), digest(DIGEST_A), 2)
        .expect("redirect guard");
    assert_eq!(
        cycle.authorize_redirect(target.clone(), digest(DIGEST_A), &resolution, &grants),
        Err(RedirectError::RedirectCycle {
            target_digest: digest(DIGEST_A),
        })
    );

    let mut limited = RedirectGuard::new(origin("https://start.example"), digest(DIGEST_A), 1)
        .expect("redirect guard");
    limited
        .authorize_redirect(target.clone(), digest(DIGEST_B), &resolution, &grants)
        .expect("first and only redirect");
    assert_eq!(
        limited.authorize_redirect(target, digest(DIGEST_C), &resolution, &grants),
        Err(RedirectError::RedirectLimitExceeded)
    );
}

#[test]
fn explicitly_managed_http_loopback_redirects_do_not_trigger_downgrade_logic() {
    let initial = origin("http://localhost");
    let target = origin("http://localhost:8080");
    let policy = DestinationPolicy::from_allowed_classes([AddressClass::Loopback]);
    let resolution = ResolutionSnapshot::approve(
        target.clone(),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &policy,
    )
    .expect("loopback resolution");
    let grants = BTreeSet::from([target.clone()]);
    let mut guard = RedirectGuard::new(initial, digest(DIGEST_A), 1)
        .expect("redirect guard");

    let evidence = guard
        .authorize_redirect(target.clone(), digest(DIGEST_B), &resolution, &grants)
        .expect("explicit loopback redirect");
    assert_eq!(evidence.target_origin(), &target);
}
