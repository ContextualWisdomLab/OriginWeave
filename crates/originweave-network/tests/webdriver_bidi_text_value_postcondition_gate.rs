#[path = "support/type_text_intent.rs"]
mod type_text_intent;

use std::{error::Error, io};

use originweave_network::{
    WebDriverBiDiCommandKind, WebDriverBiDiTextValuePostconditionError,
    verify_webdriver_bidi_text_value_postcondition,
};

#[test]
fn exact_match_is_the_only_successful_text_postcondition() -> Result<(), Box<dyn Error>> {
    let (acknowledged_intent, response, mut correlation) =
        type_text_intent::acknowledged_type_text_intent_and_observation(
            42,
            "expected",
            70,
            br#"{"type":"success","id":70,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"expected"}}}"#,
        )?;

    let verified = verify_webdriver_bidi_text_value_postcondition(
        &response,
        acknowledged_intent,
        &mut correlation,
    )?;

    assert_eq!(verified.type_text_command_id(), 42);
    assert_eq!(verified.command_id(), 70);
    assert_eq!(verified.observed_text_bytes(), "expected".len());
    assert_eq!(correlation.outstanding_count(), 0);
    assert!(!format!("{verified:?}").contains("expected"));
    Ok(())
}

#[test]
fn mismatch_is_typed_failure_after_consuming_its_exact_response() -> Result<(), Box<dyn Error>> {
    let (acknowledged_intent, response, mut correlation) =
        type_text_intent::acknowledged_type_text_intent_and_observation(
            43,
            "expected",
            71,
            br#"{"type":"success","id":71,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"unexpected"}}}"#,
        )?;

    let Err(error) = verify_webdriver_bidi_text_value_postcondition(
        &response,
        acknowledged_intent,
        &mut correlation,
    ) else {
        return Err(io::Error::other(
            "a mismatched page value must not be returned as successful postcondition evidence",
        )
        .into());
    };

    assert!(matches!(
        &error,
        WebDriverBiDiTextValuePostconditionError::PostconditionMismatch {
            type_text_command_id: 43,
            command_id: 71,
            observed_text_bytes: 10,
        }
    ));
    assert_eq!(correlation.outstanding_count(), 0);
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-value postcondition did not match the acknowledged typed-input intent"
    );
    assert!(error.source().is_none());
    let debug = format!("{error:?}");
    assert!(!debug.contains("expected"));
    assert!(!debug.contains("unexpected"));
    Ok(())
}

#[test]
fn malformed_observation_stays_a_typed_source_error_without_consuming_state()
-> Result<(), Box<dyn Error>> {
    let (acknowledged_intent, response, mut correlation) =
        type_text_intent::acknowledged_type_text_intent_and_observation(
            44,
            "expected",
            72,
            b"not-json",
        )?;

    let Err(error) = verify_webdriver_bidi_text_value_postcondition(
        &response,
        acknowledged_intent,
        &mut correlation,
    ) else {
        return Err(io::Error::other("malformed observation must fail closed").into());
    };

    assert!(matches!(
        &error,
        WebDriverBiDiTextValuePostconditionError::Observation { .. }
    ));
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi text-value postcondition observation failed"
    );
    assert!(error.source().is_some());
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}

#[test]
fn unrelated_outstanding_command_cannot_certify_text_postcondition() -> Result<(), Box<dyn Error>> {
    let (acknowledged_intent, response, mut correlation) =
        type_text_intent::acknowledged_type_text_intent_and_registered_response(
            45,
            "expected",
            73,
            WebDriverBiDiCommandKind::SessionStatus,
            br#"{"type":"success","id":73,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"expected"}}}"#,
        )?;

    let Err(error) = verify_webdriver_bidi_text_value_postcondition(
        &response,
        acknowledged_intent,
        &mut correlation,
    ) else {
        return Err(io::Error::other(
            "an unrelated outstanding command id must not certify a text-value postcondition",
        )
        .into());
    };

    assert!(matches!(
        &error,
        WebDriverBiDiTextValuePostconditionError::Observation { .. }
    ));
    assert_eq!(correlation.outstanding_count(), 1);
    Ok(())
}
