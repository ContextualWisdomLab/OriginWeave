use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, MAX_WEBDRIVER_BIDI_COMMAND_ID,
    MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH, MAX_WEBDRIVER_BIDI_RESPONSE_TOP_LEVEL_FIELDS,
    ParsedWebDriverBiDiCommandResponseEnvelope, WebDriverBiDiCommandResponseKind,
    WebDriverBiDiResponseEnvelopeParseError,
};

#[test]
fn parser_classifies_exact_success_and_nullable_error_envelopes() -> Result<(), Box<dyn Error>> {
    let success_raw = " {\"type\":\"success\",\"id\":42,\"result\":{\"nodes\":[]}}\r\n";
    let success =
        BoundedWebDriverBiDiResponseDocument::new(success_raw)?.parse_command_response()?;
    assert_eq!(success.kind(), WebDriverBiDiCommandResponseKind::Success);
    assert_eq!(success.response_id(), Some(42));
    assert_eq!(success.as_str(), success_raw);

    let error_raw = "{\"type\":\"error\",\"id\":null,\"error\":\"invalid argument\",\"message\":\"bad request\"}";
    let error = BoundedWebDriverBiDiResponseDocument::new(error_raw)?.parse_command_response()?;
    assert_eq!(error.kind(), WebDriverBiDiCommandResponseKind::Error);
    assert_eq!(error.response_id(), None);
    assert_eq!(error.as_str(), error_raw);
    Ok(())
}

#[test]
fn parser_accepts_extensible_fields_only_when_the_complete_json_is_valid()
-> Result<(), Box<dyn Error>> {
    let raw = concat!(
        "{\"vendor\":{\"nested\":[true,false,null,{\"text\":\"a\\\\b\\\"c\\u263a\"}]},",
        "\"id\":7,\"result\":{},\"type\":\"success\"}"
    );
    let parsed = BoundedWebDriverBiDiResponseDocument::new(raw)?.parse_command_response()?;
    assert_eq!(parsed.response_id(), Some(7));

    for malformed in [
        "{\"type\":\"success\",\"id\":7,\"result\":{},}",
        "{\"type\":\"success\",\"id\":7,\"result\":{},\"x\":01}",
        "{\"type\":\"success\",\"id\":7,\"result\":{},\"x\":\"\\q\"}",
        "{\"type\":\"success\",\"id\":7,\"result\":{},\"x\":[1,]}",
    ] {
        let document = BoundedWebDriverBiDiResponseDocument::new(malformed)?;
        assert_eq!(
            document.parse_command_response(),
            Err(WebDriverBiDiResponseEnvelopeParseError::InvalidJson)
        );
    }
    Ok(())
}

#[test]
fn parser_rejects_missing_duplicate_or_unexpected_response_discriminators()
-> Result<(), Box<dyn Error>> {
    for (raw, expected) in [
        (
            "{\"id\":1,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::MissingResponseType,
        ),
        (
            "{\"type\":\"event\",\"id\":1,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::UnexpectedResponseType,
        ),
        (
            "{\"type\":\"success\",\"type\":\"error\",\"id\":1,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::DuplicateTopLevelField,
        ),
        (
            "{\"type\":\"success\",\"id\":1,\"id\":1,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::DuplicateTopLevelField,
        ),
    ] {
        let document = BoundedWebDriverBiDiResponseDocument::new(raw)?;
        assert_eq!(document.parse_command_response(), Err(expected));
    }
    Ok(())
}

#[test]
fn parser_requires_a_present_protocol_range_id_and_success_result() -> Result<(), Box<dyn Error>> {
    for (raw, expected) in [
        (
            "{\"type\":\"success\",\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::MissingResponseId,
        ),
        (
            "{\"type\":\"error\",\"error\":\"invalid argument\",\"message\":\"bad\"}",
            WebDriverBiDiResponseEnvelopeParseError::MissingResponseId,
        ),
        (
            "{\"type\":\"success\",\"id\":null,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId,
        ),
        (
            "{\"type\":\"success\",\"id\":-1,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId,
        ),
        (
            "{\"type\":\"success\",\"id\":1.0,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId,
        ),
        (
            "{\"type\":\"success\",\"id\":1e0,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId,
        ),
        (
            "{\"type\":\"success\",\"id\":9007199254740992,\"result\":{}}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId,
        ),
        (
            "{\"type\":\"success\",\"id\":1}",
            WebDriverBiDiResponseEnvelopeParseError::MissingRequiredPayload,
        ),
        (
            "{\"type\":\"success\",\"id\":1,\"result\":[]}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidRequiredPayloadType,
        ),
    ] {
        let document = BoundedWebDriverBiDiResponseDocument::new(raw)?;
        assert_eq!(document.parse_command_response(), Err(expected));
    }

    let maximum =
        format!("{{\"type\":\"success\",\"id\":{MAX_WEBDRIVER_BIDI_COMMAND_ID},\"result\":{{}}}}");
    assert_eq!(
        BoundedWebDriverBiDiResponseDocument::new(&maximum)?
            .parse_command_response()?
            .response_id(),
        Some(MAX_WEBDRIVER_BIDI_COMMAND_ID)
    );
    Ok(())
}

#[test]
fn parser_requires_error_code_message_and_string_stacktrace() -> Result<(), Box<dyn Error>> {
    for (raw, expected) in [
        (
            "{\"type\":\"error\",\"id\":1,\"message\":\"bad\"}",
            WebDriverBiDiResponseEnvelopeParseError::MissingRequiredPayload,
        ),
        (
            "{\"type\":\"error\",\"id\":1,\"error\":\"invalid argument\"}",
            WebDriverBiDiResponseEnvelopeParseError::MissingRequiredPayload,
        ),
        (
            "{\"type\":\"error\",\"id\":1,\"error\":1,\"message\":\"bad\"}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidRequiredPayloadType,
        ),
        (
            "{\"type\":\"error\",\"id\":1,\"error\":\"invalid argument\",\"message\":false}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidRequiredPayloadType,
        ),
        (
            "{\"type\":\"error\",\"id\":1,\"error\":\"invalid argument\",\"message\":\"bad\",\"stacktrace\":[]}",
            WebDriverBiDiResponseEnvelopeParseError::InvalidRequiredPayloadType,
        ),
    ] {
        let document = BoundedWebDriverBiDiResponseDocument::new(raw)?;
        assert_eq!(document.parse_command_response(), Err(expected));
    }

    let valid = BoundedWebDriverBiDiResponseDocument::new(
        "{\"type\":\"error\",\"id\":7,\"error\":\"invalid argument\",\"message\":\"bad\",\"stacktrace\":\"frame\"}",
    )?
    .parse_command_response()?;
    assert_eq!(valid.response_id(), Some(7));
    Ok(())
}

#[test]
fn parser_enforces_top_level_field_and_json_depth_budgets() -> Result<(), Box<dyn Error>> {
    let mut fields = vec![
        "\"type\":\"success\"".to_owned(),
        "\"id\":1".to_owned(),
        "\"result\":{}".to_owned(),
    ];
    while fields.len() < MAX_WEBDRIVER_BIDI_RESPONSE_TOP_LEVEL_FIELDS {
        fields.push(format!("\"x{}\":null", fields.len()));
    }
    let exact_fields = format!("{{{}}}", fields.join(","));
    assert!(
        BoundedWebDriverBiDiResponseDocument::new(&exact_fields)?
            .parse_command_response()
            .is_ok()
    );
    fields.push("\"overflow\":null".to_owned());
    let over_fields = format!("{{{}}}", fields.join(","));
    assert_eq!(
        BoundedWebDriverBiDiResponseDocument::new(&over_fields)?.parse_command_response(),
        Err(WebDriverBiDiResponseEnvelopeParseError::TopLevelFieldCountExceeded)
    );

    let exact_nested = format!(
        "{{\"type\":\"success\",\"id\":1,\"result\":{{\"x\":{}{}}}}}",
        "[".repeat(MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH - 2),
        "]".repeat(MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH - 2)
    );
    assert!(
        BoundedWebDriverBiDiResponseDocument::new(&exact_nested)?
            .parse_command_response()
            .is_ok()
    );

    let over_nested = format!(
        "{{\"type\":\"success\",\"id\":1,\"result\":{{\"x\":{}{}}}}}",
        "[".repeat(MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH - 1),
        "]".repeat(MAX_WEBDRIVER_BIDI_RESPONSE_JSON_DEPTH - 1)
    );
    assert_eq!(
        BoundedWebDriverBiDiResponseDocument::new(&over_nested)?.parse_command_response(),
        Err(WebDriverBiDiResponseEnvelopeParseError::JsonDepthExceeded)
    );
    Ok(())
}

#[test]
fn parser_normalizes_escaped_top_level_names_before_duplicate_detection()
-> Result<(), Box<dyn Error>> {
    let duplicate = BoundedWebDriverBiDiResponseDocument::new(
        "{\"type\":\"success\",\"\\u0069d\":1,\"id\":1,\"result\":{}}",
    )?;
    assert_eq!(
        duplicate.parse_command_response(),
        Err(WebDriverBiDiResponseEnvelopeParseError::DuplicateTopLevelField)
    );

    let unicode_extension = BoundedWebDriverBiDiResponseDocument::new(
        "{\"type\":\"success\",\"id\":1,\"result\":{},\"메타\":\"값\"}",
    )?
    .parse_command_response()?;
    assert_eq!(unicode_extension.response_id(), Some(1));
    Ok(())
}

#[test]
fn response_envelope_parse_errors_are_deterministic_and_source_free() {
    for error in [
        WebDriverBiDiResponseEnvelopeParseError::InvalidJson,
        WebDriverBiDiResponseEnvelopeParseError::JsonDepthExceeded,
        WebDriverBiDiResponseEnvelopeParseError::TopLevelFieldCountExceeded,
        WebDriverBiDiResponseEnvelopeParseError::DuplicateTopLevelField,
        WebDriverBiDiResponseEnvelopeParseError::MissingResponseType,
        WebDriverBiDiResponseEnvelopeParseError::UnexpectedResponseType,
        WebDriverBiDiResponseEnvelopeParseError::MissingResponseId,
        WebDriverBiDiResponseEnvelopeParseError::InvalidResponseId,
        WebDriverBiDiResponseEnvelopeParseError::MissingRequiredPayload,
        WebDriverBiDiResponseEnvelopeParseError::InvalidRequiredPayloadType,
    ] {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }
}

fn _parsed_type_is_public(_parsed: ParsedWebDriverBiDiCommandResponseEnvelope) {}
