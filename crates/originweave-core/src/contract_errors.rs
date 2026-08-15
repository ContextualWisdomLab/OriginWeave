use std::fmt;

use crate::contracts::{ActionIntentDigestError, ExtensionIdError, OriginError};

impl fmt::Display for OriginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingScheme => "origin must include an explicit scheme",
            Self::UnsupportedScheme => "origin scheme must be HTTPS or loopback HTTP",
            Self::InsecureRemoteOrigin => "remote HTTP origins are not permitted",
            Self::MissingAuthority => "origin authority must not be empty",
            Self::UserInfoNotAllowed => "origin authority must not contain user information",
            Self::PathNotAllowed => "origin must not contain a path, query, or fragment",
            Self::InvalidAuthority => "origin authority is malformed or ambiguous",
            Self::AmbiguousNumericHost => {
                "origin host uses a browser-ambiguous numeric address spelling"
            }
            Self::InvalidPort => "origin port must be a numeric value from 1 through 65535",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for OriginError {}

impl fmt::Display for ActionIntentDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => formatter.write_str(
                "action intent digest must be sha256 followed by 64 lowercase hexadecimal digits",
            ),
        }
    }
}

impl std::error::Error for ActionIntentDigestError {}

impl fmt::Display for ExtensionIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExtensionId => formatter
                .write_str("extension identifier must be 32 lowercase characters from a through p"),
        }
    }
}

impl std::error::Error for ExtensionIdError {}
