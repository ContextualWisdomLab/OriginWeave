use std::error::Error as _;

use originweave_core::ExtensionIdError;

#[test]
fn extension_id_errors_have_a_stable_standard_error_contract() {
    let error = ExtensionIdError::InvalidExtensionId;
    assert_eq!(
        error.to_string(),
        "extension identifier must be 32 lowercase characters from a through p"
    );
    assert!(error.source().is_none());
    fn assert_standard_error_contract<E: std::error::Error + Send + Sync + 'static>() {}
    assert_standard_error_contract::<ExtensionIdError>();
}
