use std::error::Error as _;

use originweave_core::ActionIntentDigestError;

#[test]
fn action_intent_digest_error_exposes_standard_error_contract() {
    let error = ActionIntentDigestError::InvalidFormat;
    assert_eq!(
        error.to_string(),
        "action intent digest must be sha256: followed by 64 lowercase hexadecimal digits"
    );
    assert!(error.source().is_none());
}
