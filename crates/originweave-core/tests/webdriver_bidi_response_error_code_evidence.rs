use std::error::Error;

use originweave_core::{BoundedWebDriverBiDiResponseDocument, WebDriverBiDiErrorCode};

#[test]
fn parsed_error_envelope_retains_typed_error_code() -> Result<(), Box<dyn Error>> {
    for (raw_code, expected) in [
        ("invalid argument", WebDriverBiDiErrorCode::InvalidArgument),
        (
            "no such client window",
            WebDriverBiDiErrorCode::NoSuchClientWindow,
        ),
        (
            "unavailable network data",
            WebDriverBiDiErrorCode::UnavailableNetworkData,
        ),
    ] {
        let raw = format!(
            "{{\"type\":\"error\",\"id\":7,\"error\":\"{raw_code}\",\"message\":\"remote failure\"}}"
        );
        let parsed = BoundedWebDriverBiDiResponseDocument::new(&raw)?.parse_command_response()?;
        assert_eq!(parsed.error_code(), Some(expected));
    }
    Ok(())
}

#[test]
fn parsed_success_envelope_has_no_error_code() -> Result<(), Box<dyn Error>> {
    let parsed =
        BoundedWebDriverBiDiResponseDocument::new("{\"type\":\"success\",\"id\":7,\"result\":{}}")?
            .parse_command_response()?;

    assert_eq!(parsed.error_code(), None);
    Ok(())
}
