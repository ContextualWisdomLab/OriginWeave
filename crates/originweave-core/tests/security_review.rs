#![allow(clippy::expect_used)]

use originweave_core::{
    ActionIntentDigest, ActionIntentDigestError, ActionKind, ActionRequest, ApprovalScope,
    InstructionSource, Origin, OriginError, SecretDelivery,
};

const INTENT_A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const INTENT_B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[test]
fn origin_rejects_browser_special_numeric_hosts() {
    for input in [
        "https://127.1",
        "https://2130706433",
        "https://0x7f000001",
        "https://0177.0.0.1",
        "https://1.2.3.04",
        "https://example.127",
        "https://0x",
        "https://1.2.3.0x",
    ] {
        assert_eq!(
            Origin::parse(input),
            Err(OriginError::AmbiguousNumericHost),
            "input={input}"
        );
    }

    assert_eq!(
        Origin::parse("https://127.0.0.1")
            .expect("canonical dotted-decimal IPv4")
            .as_str(),
        "https://127.0.0.1"
    );
}

#[test]
fn action_intent_digest_accepts_only_prefixed_lowercase_sha256() {
    let digest = ActionIntentDigest::parse(INTENT_A).expect("valid action intent digest");
    assert_eq!(digest.as_str(), INTENT_A);

    for invalid in [
        "",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "sha256:short",
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
    ] {
        assert_eq!(
            ActionIntentDigest::parse(invalid),
            Err(ActionIntentDigestError::InvalidFormat),
            "invalid={invalid}"
        );
    }
}

#[test]
fn approval_and_request_are_bound_to_the_complete_intent_digest() {
    let origin = Origin::parse("https://shop.example").expect("origin");
    let digest_a = ActionIntentDigest::parse(INTENT_A).expect("intent A");
    let digest_b = ActionIntentDigest::parse(INTENT_B).expect("intent B");
    let scope = ApprovalScope::new(ActionKind::Purchase, origin.clone(), digest_a.clone());
    let request = ActionRequest::new(
        ActionKind::Purchase,
        origin.clone(),
        origin,
        InstructionSource::User,
        SecretDelivery::None,
        digest_b,
    );

    assert_eq!(scope.intent_digest(), &digest_a);
    assert_eq!(request.intent_digest().as_str(), INTENT_B);
    assert_ne!(scope.intent_digest(), request.intent_digest());
}
