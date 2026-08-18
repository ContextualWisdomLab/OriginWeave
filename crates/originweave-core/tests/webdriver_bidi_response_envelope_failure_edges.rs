use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH,
    WebDriverBiDiResponseDocumentAdmissionError, WebDriverBiDiResponseEnvelopeParseError,
};

fn assert_invalid_json(raw: &str) -> Result<(), Box<dyn Error>> {
    let document = BoundedWebDriverBiDiResponseDocument::new(raw)?;
    assert_eq!(
        document.parse_command_response(),
        Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson)
    );
    Ok(())
}

#[test]
fn non_object_response_stops_at_document_admission_before_parser() {
    assert!(matches!(
        BoundedWebDriverBiDiResponseDocument::new("[]"),
        Err(WebDriverBiDiResponseDocumentAdmissionError::InvalidObjectBoundary)
    ));
}

#[test]
fn parser_rejects_truncated_literal_documents() -> Result<(), Box<dyn Error>> {
    for raw in [
        r#"{"type":"success","id":1,"result":{},"x":falsX}"#,
        r#"{"type":"success","id":1,"result":{},"x":nulX}"#,
    ] {
        assert_invalid_json(raw)?;
    }
    Ok(())
}

#[test]
fn parser_rejects_malformed_escape_and_unicode_code_units() -> Result<(), Box<dyn Error>> {
    for raw in [
        r#"{"type":"success","id":1,"result":{},"x":"\q"}"#,
        r#"{"type":"success","id":1,"result":{},"x":"\u12G4"}"#,
        r#"{"type":"success","id":1,"result":{},"x":"\uD83D\u12G4"}"#,
    ] {
        assert_invalid_json(raw)?;
    }
    Ok(())
}

#[test]
fn parser_rejects_nested_object_key_and_colon_faults() -> Result<(), Box<dyn Error>> {
    for raw in [
        r#"{"type":"success","id":1,"result":{},"x":{1:2}}"#,
        r#"{"type":"success","id":1,"result":{},"x":{"a" 1}}"#,
    ] {
        assert_invalid_json(raw)?;
    }
    Ok(())
}

#[test]
fn parser_enforces_depth_budget_for_object_nesting() -> Result<(), Box<dyn Error>> {
    let over_nested = format!(
        "{{\"type\":\"success\",\"id\":1,\"result\":{{\"x\":{}{}}}}}",
        "{\"k\":".repeat(MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH - 1),
        "}".repeat(MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH - 1)
    );
    let document = BoundedWebDriverBiDiResponseDocument::new(&over_nested)?;
    assert_eq!(
        document.parse_command_response(),
        Err(WebDriverBiDiResponseEnvelopeParseError::JsonDepthExceeded)
    );
    Ok(())
}
