use std::error::Error;

use originweave_core::{ExtensionId, ExtensionIdError};

fn assert_standard_error<T: Error>() {}

#[test]
fn extension_id_error_exposes_a_stable_standard_error_contract() {
    assert_standard_error::<ExtensionIdError>();

    let result = ExtensionId::parse("invalid");
    assert_eq!(result, Err(ExtensionIdError::InvalidExtensionId));

    let rendered = result.as_ref().err().map(ToString::to_string);
    assert_eq!(
        rendered.as_deref(),
        Some("extension identifier must be exactly 32 lowercase characters from a through p")
    );
    let source = result.as_ref().err().and_then(|error| error.source());
    assert!(source.is_none());
}
