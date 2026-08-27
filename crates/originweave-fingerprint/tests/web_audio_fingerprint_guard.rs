//! Web Audio fingerprint-blocking contracts for OriginWeave privacy profiles.
//!
//! These tests are intentionally added before production code. They define a
//! default-deny policy, exact-origin exceptions, bounded configuration, and a
//! deterministic pre-document guard script that blocks Web Audio constructors
//! before page JavaScript can create a silent fingerprint graph.
#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_fingerprint::{
    WebAudioDecision, WebAudioFingerprintPolicy, WebAudioPolicyError,
};

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must satisfy the shared origin contract")
}

#[test]
fn default_policy_blocks_web_audio_fingerprinting_with_audit_reason() {
    let policy = WebAudioFingerprintPolicy::default();
    let decision = policy.decision(&origin("https://shop.example"));

    assert_eq!(decision, WebAudioDecision::BlockFingerprinting);
    assert!(decision.blocks_fingerprinting());
    assert_eq!(
        decision.reason_code(),
        Some("web_audio_fingerprinting_no_explicit_origin_grant")
    );
    assert_eq!(policy.allowed_origin_count(), 0);
}

#[test]
fn explicit_grant_is_exact_origin_scoped() {
    let policy = WebAudioFingerprintPolicy::new(vec![origin("https://shop.example")])
        .expect("one valid grant must fit the bounded policy");

    let allowed = policy.decision(&origin("https://shop.example:443"));
    assert_eq!(allowed, WebAudioDecision::AllowExplicitOrigin);
    assert!(!allowed.blocks_fingerprinting());
    assert_eq!(allowed.reason_code(), None);

    assert_eq!(
        policy.decision(&origin("https://cdn.shop.example")),
        WebAudioDecision::BlockFingerprinting
    );
    assert_eq!(
        policy.decision(&origin("https://shop.example:8443")),
        WebAudioDecision::BlockFingerprinting
    );
}

#[test]
fn duplicate_grants_collapse_to_one_canonical_origin() {
    let policy = WebAudioFingerprintPolicy::new(vec![
        origin("https://shop.example"),
        origin("https://shop.example:443"),
    ])
    .expect("canonical duplicate grants must remain bounded");

    assert_eq!(policy.allowed_origin_count(), 1);
}

#[test]
fn allowlist_rejects_more_than_the_bounded_unique_origin_count() {
    let grants = (0..129)
        .map(|index| origin(&format!("https://site-{index}.example")))
        .collect::<Vec<_>>();

    assert_eq!(
        WebAudioFingerprintPolicy::new(grants),
        Err(WebAudioPolicyError::TooManyAllowedOrigins {
            maximum: 128,
            actual: 129,
        })
    );
}

#[test]
fn rendered_guard_is_deterministic_and_contains_only_canonical_grants() {
    let policy = WebAudioFingerprintPolicy::new(vec![
        origin("https://z.example"),
        origin("https://a.example:443"),
    ])
    .expect("two exact grants must fit the bounded policy");

    let first = policy.render_guard_script();
    let second = policy.render_guard_script();
    assert_eq!(first, second);
    assert!(!first.contains("ORIGINWEAVE_ALLOWED_WEB_AUDIO_ORIGINS"));
    assert!(first.contains("\"https://a.example\""));
    assert!(first.contains("\"https://z.example\""));
    assert!(
        first.find("https://a.example").expect("first origin must be rendered")
            < first
                .find("https://z.example")
                .expect("second origin must be rendered")
    );
}

#[test]
fn rendered_guard_blocks_every_web_audio_construction_entrypoint() {
    let script = WebAudioFingerprintPolicy::default().render_guard_script();

    for constructor in [
        "AudioContext",
        "webkitAudioContext",
        "OfflineAudioContext",
        "webkitOfflineAudioContext",
        "AudioWorkletNode",
    ] {
        assert!(script.contains(constructor), "missing {constructor}");
    }
    assert!(script.contains("NotAllowedError"));
    assert!(script.contains("document_start"));
}

#[test]
fn policy_error_formats_a_stable_operator_message() {
    let error = WebAudioPolicyError::TooManyAllowedOrigins {
        maximum: 128,
        actual: 129,
    };

    assert_eq!(
        error.to_string(),
        "web audio allowlist contains 129 unique origins; maximum is 128"
    );
}
