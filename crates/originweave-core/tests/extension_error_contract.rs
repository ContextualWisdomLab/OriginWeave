use std::error::Error;

use originweave_core::{ExtensionId, ExtensionIdError};

fn assert_standard_error<T: Error>() {}

#[test]
fn extension_id_error_exposes_a_stable_standard_error_contract() {
    assert_standard_error::<ExtensionIdError>();

    let error = ExtensionId::parse("invalid")
        .expect_err("non-canonical Chromium extension identifiers must fail closed");
    assert_eq!(
        error.to_string(),
        "extension identifier must be exactly 32 lowercase characters from a through p"
    );
    assert!(error.source().is_none());
}
