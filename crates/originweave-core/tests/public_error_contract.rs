use std::error::Error;

use originweave_core::{ActionIntentDigestError, ExtensionIdError, OriginError};

fn assert_standard_error<T: Error + Send + Sync + 'static>() {}

#[test]
fn public_validation_errors_are_standard_errors() {
    assert_standard_error::<OriginError>();
    assert_standard_error::<ActionIntentDigestError>();
    assert_standard_error::<ExtensionIdError>();
}

#[test]
fn public_validation_errors_have_stable_operator_messages() {
    assert_eq!(
        OriginError::MissingScheme.to_string(),
        "origin must include an explicit scheme"
    );
    assert_eq!(
        OriginError::UnsupportedScheme.to_string(),
        "origin scheme must be HTTPS or loopback HTTP"
    );
    assert_eq!(
        OriginError::InsecureRemoteOrigin.to_string(),
        "remote HTTP origins are not permitted"
    );
    assert_eq!(
        OriginError::MissingAuthority.to_string(),
        "origin authority must not be empty"
    );
    assert_eq!(
        OriginError::UserInfoNotAllowed.to_string(),
        "origin authority must not contain user information"
    );
    assert_eq!(
        OriginError::PathNotAllowed.to_string(),
        "origin must not contain a path, query, or fragment"
    );
    assert_eq!(
        OriginError::InvalidAuthority.to_string(),
        "origin authority is malformed or ambiguous"
    );
    assert_eq!(
        OriginError::AmbiguousNumericHost.to_string(),
        "origin host uses a browser-ambiguous numeric address spelling"
    );
    assert_eq!(
        OriginError::InvalidPort.to_string(),
        "origin port must be a numeric value from 1 through 65535"
    );
    assert_eq!(
        ActionIntentDigestError::InvalidFormat.to_string(),
        "action intent digest must be sha256: followed by 64 lowercase hexadecimal digits"
    );
    assert_eq!(
        ExtensionIdError::InvalidExtensionId.to_string(),
        "extension identifier must be 32 lowercase characters from a through p"
    );
}
