use std::error::Error;

use originweave_core::BoundedWebDriverBiDiResponseDocument;

const CURRENT_WEBDRIVER_BIDI_ERROR_CODES: &[&str] = &[
    "invalid argument",
    "invalid selector",
    "invalid session id",
    "invalid web extension",
    "move target out of bounds",
    "no such alert",
    "no such client window",
    "no such network collector",
    "no such element",
    "no such frame",
    "no such handle",
    "no such history entry",
    "no such intercept",
    "no such network data",
    "no such node",
    "no such request",
    "no such screencast",
    "no such script",
    "no such storage partition",
    "no such user context",
    "no such web extension",
    "session not created",
    "unable to capture screen",
    "unable to close browser",
    "unable to set cookie",
    "unable to set file input",
    "unavailable network data",
    "underspecified storage partition",
    "unknown command",
    "unknown error",
    "unsupported operation",
];

#[test]
fn parser_accepts_current_webdriver_bidi_error_code_vocabulary() -> Result<(), Box<dyn Error>> {
    for error_code in CURRENT_WEBDRIVER_BIDI_ERROR_CODES {
        let raw = format!(
            "{{\"type\":\"error\",\"id\":7,\"error\":\"{error_code}\",\"message\":\"browser rejected command\"}}"
        );
        let parsed = BoundedWebDriverBiDiResponseDocument::new(&raw)?.parse_command_response();
        assert!(
            parsed.is_ok(),
            "current WebDriver BiDi error code must remain admissible: {error_code}"
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
