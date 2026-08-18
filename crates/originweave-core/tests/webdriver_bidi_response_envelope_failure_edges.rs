use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH,
    WebDriverBiDiResponseEnvelopeParseError,
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
fn parser_rejects_non_object_and_truncated_literal_documents() -> Result<(), Box<dyn Error>> {
    for raw in [
        "[]",
        r#"{"type":"success","id":1,"result":{},"x":falsX}"#,
        r#"{"type":"success","id":1,"result":{},"x":nulX}"#,
    ] {
        assert_invalid_json(raw)?;
    }
    Ok(())
}

#[test]
fn parser_rejects_truncated_escape_and_unicode_code_units() -> Result<(), Box<dyn Error>> {
    for raw in [
        r#"{"type":"success","id":1,"result":{},"x":"\"#,
        r#"{"type":"success","id":1,"result":{},"x":"\u12"#,
        r#"{"type":"success","id":1,"result":{},"x":"\uD83D\u"#,
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
