use std::error::Error;

use originweave_core::{BoundedWebDriverBiDiResponseDocument, WebDriverBiDiErrorCode};

const CURRENT_WEBDRIVER_BIDI_ERROR_CODES: &[(&str, WebDriverBiDiErrorCode)] = &[
    ("invalid argument", WebDriverBiDiErrorCode::InvalidArgument),
    ("invalid selector", WebDriverBiDiErrorCode::InvalidSelector),
    ("invalid session id", WebDriverBiDiErrorCode::InvalidSessionId),
    ("invalid web extension", WebDriverBiDiErrorCode::InvalidWebExtension),
    (
        "move target out of bounds",
        WebDriverBiDiErrorCode::MoveTargetOutOfBounds,
    ),
    ("no such alert", WebDriverBiDiErrorCode::NoSuchAlert),
    (
        "no such client window",
        WebDriverBiDiErrorCode::NoSuchClientWindow,
    ),
    (
        "no such network collector",
        WebDriverBiDiErrorCode::NoSuchNetworkCollector,
    ),
    ("no such element", WebDriverBiDiErrorCode::NoSuchElement),
    ("no such frame", WebDriverBiDiErrorCode::NoSuchFrame),
    ("no such handle", WebDriverBiDiErrorCode::NoSuchHandle),
    (
        "no such history entry",
        WebDriverBiDiErrorCode::NoSuchHistoryEntry,
    ),
    ("no such intercept", WebDriverBiDiErrorCode::NoSuchIntercept),
    (
        "no such network data",
        WebDriverBiDiErrorCode::NoSuchNetworkData,
    ),
    ("no such node", WebDriverBiDiErrorCode::NoSuchNode),
    ("no such request", WebDriverBiDiErrorCode::NoSuchRequest),
    ("no such screencast", WebDriverBiDiErrorCode::NoSuchScreencast),
    ("no such script", WebDriverBiDiErrorCode::NoSuchScript),
    (
        "no such storage partition",
        WebDriverBiDiErrorCode::NoSuchStoragePartition,
    ),
    (
        "no such user context",
        WebDriverBiDiErrorCode::NoSuchUserContext,
    ),
    (
        "no such web extension",
        WebDriverBiDiErrorCode::NoSuchWebExtension,
    ),
    (
        "session not created",
        WebDriverBiDiErrorCode::SessionNotCreated,
    ),
    (
        "unable to capture screen",
        WebDriverBiDiErrorCode::UnableToCaptureScreen,
    ),
    (
        "unable to close browser",
        WebDriverBiDiErrorCode::UnableToCloseBrowser,
    ),
    ("unable to set cookie", WebDriverBiDiErrorCode::UnableToSetCookie),
    (
        "unable to set file input",
        WebDriverBiDiErrorCode::UnableToSetFileInput,
    ),
    (
        "unavailable network data",
        WebDriverBiDiErrorCode::UnavailableNetworkData,
    ),
    (
        "underspecified storage partition",
        WebDriverBiDiErrorCode::UnderspecifiedStoragePartition,
    ),
    ("unknown command", WebDriverBiDiErrorCode::UnknownCommand),
    ("unknown error", WebDriverBiDiErrorCode::UnknownError),
    (
        "unsupported operation",
        WebDriverBiDiErrorCode::UnsupportedOperation,
    ),
];

#[test]
fn parser_retains_every_current_webdriver_bidi_error_code() -> Result<(), Box<dyn Error>> {
    for &(raw_code, expected) in CURRENT_WEBDRIVER_BIDI_ERROR_CODES {
        let raw = format!(
            "{{\"type\":\"error\",\"id\":7,\"error\":\"{raw_code}\",\"message\":\"browser rejected command\"}}"
        );
        let parsed = BoundedWebDriverBiDiResponseDocument::new(&raw)?.parse_command_response()?;
        assert_eq!(
            parsed.error_code(),
            Some(expected),
            "current WebDriver BiDi error code must retain its typed mapping: {raw_code}"
        );
    }
    Ok(())
}

#[test]
fn parser_rejects_unknown_webdriver_bidi_error_code() -> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(
        "{\"type\":\"error\",\"id\":7,\"error\":\"made up browser failure\",\"message\":\"untrusted adapter text\"}",
    )?;

    let error = match document.parse_command_response() {
        Ok(_) => {
            return Err(std::io::Error::other(
                "unknown WebDriver BiDi error code was unexpectedly accepted",
            )
            .into());
        }
        Err(error) => error,
    };
    assert!(!error.to_string().is_empty());
    assert!(error.source().is_none());
    Ok(())
}
