use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, WebDriverBiDiResponseEnvelopeParseError,
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
fn parser_rejects_top_level_separator_trailing_document_and_missing_colon_faults()
-> Result<(), Box<dyn Error>> {
    for raw in [
        "{\"type\":\"success\" \"id\":1,\"result\":{}}",
        "{\"type\" \"success\",\"id\":1,\"result\":{}}",
        "{\"type\":\"success\",\"id\":1,\"result\":{}} {}",
    ] {
        assert_invalid_json(raw)?;
    }

    let empty = BoundedWebDriverBiDiResponseDocument::new("{}")?;
    assert_eq!(
        empty.parse_command_response(),
        Err(WebDriverBiDiResponseEnvelopeParseError::MissingResponseType)
    );
    Ok(())
}

#[test]
fn parser_rejects_malformed_nested_object_array_string_and_literal_values()
-> Result<(), Box<dyn Error>> {
    for raw in [
        "{\"type\":\"success\",\"id\":1,\"result\":{\"a\":1 \"b\":2}}",
        "{\"type\":\"success\",\"id\":1,\"result\":{},\"x\":[1 2]}",
        "{\"type\":\"success\",\"id\":1,\"result\":{},\"x\":tru}",
        "{\"type\":\"success\",\"id\":1,\"result\":{},\"x\":-}",
        "{\"type\":\"success\",\"id\":1,\"result\":{},\"x\":1.}",
        "{\"type\":\"success\",\"id\":1,\"result\":{},\"x\":1e}",
        "{\"type\":\"success\",\"id\":1,\"result\":{},\"x\":\"unterminated}",
    ] {
        assert_invalid_json(raw)?;
    }

    let raw_control = "{\"type\":\"success\",\"id\":1,\"result\":{},\"x\":\"bad\u{0001}text\"}";
    assert_invalid_json(raw_control)?;
    Ok(())
}

#[test]
fn parser_accepts_complete_json_escape_number_and_nested_container_forms()
-> Result<(), Box<dyn Error>> {
    let raw = concat!(
        r#"{"type":"success","id":1,"result":{"a":1,"b":2},"esc":""#,
        r#"\"\\\/\b\f\n\r\t","zero":0,"signed_exponent":1e+2,"nested":[1,2,{"ok":true}]}"#,
    );
    let parsed = BoundedWebDriverBiDiResponseDocument::new(raw)?.parse_command_response()?;
    assert_eq!(parsed.response_id(), Some(1));
    Ok(())
}

#[test]
fn parser_accepts_all_utf8_widths_from_json_unicode_escapes() -> Result<(), Box<dyn Error>> {
    for raw in [
        r#"{"type":"success","id":1,"result":{},"text":"\u0041"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\u00E9"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\u263A"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\uD83D\uDE00"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\u00AF"}"#,
    ] {
        let parsed = BoundedWebDriverBiDiResponseDocument::new(raw)?.parse_command_response()?;
        assert_eq!(parsed.response_id(), Some(1));
    }
    Ok(())
}

#[test]
fn parser_rejects_invalid_unicode_escape_sequences() -> Result<(), Box<dyn Error>> {
    for raw in [
        r#"{"type":"success","id":1,"result":{},"text":"\uD83D"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\uD83D\x"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\uD83D\u0041"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\uDE00"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\u12"}"#,
        r#"{"type":"success","id":1,"result":{},"text":"\u00G0"}"#,
    ] {
        assert_invalid_json(raw)?;
    }
    Ok(())
}

#[test]
fn parser_rejects_integer_overflow_even_before_protocol_range_validation()
-> Result<(), Box<dyn Error>> {
    let raw = "{\"type\":\"success\",\"id\":18446744073709551616,\"result\":{}}";
    let document = BoundedWebDriverBiDiResponseDocument::new(raw)?;
    assert_eq!(
        document.parse_command_response(),
        Err(WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId)
    );
    Ok(())
}
