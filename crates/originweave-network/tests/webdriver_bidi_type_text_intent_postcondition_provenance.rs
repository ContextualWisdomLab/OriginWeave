#[path = "support/text_observation.rs"]
mod text_observation;
#[path = "support/type_text_intent.rs"]
mod type_text_intent;

use std::{error::Error, io};

use originweave_network::{
    WebDriverBiDiCommandCorrelation, WebDriverBiDiTextValuePostconditionError,
    verify_webdriver_bidi_text_value_postcondition,
};

#[test]
fn substituted_expected_text_cannot_certify_a_different_authorized_typed_input()
-> Result<(), Box<dyn Error>> {
    let acknowledged_intent =
        type_text_intent::acknowledged_type_text_intent(42, "authorized-value")?;

    let mut correlation = WebDriverBiDiCommandCorrelation::new();
    let observation = text_observation::receive_command_responses(
        &[br#"{"type":"success","id":70,"result":{"type":"success","realm":"realm-1","result":{"type":"string","value":"substituted-value"}}}"#],
        70,
        &mut correlation,
    )?
    .remove(0);

    let substituted = verify_webdriver_bidi_text_value_postcondition(
        &observation,
        acknowledged_intent,
        &mut correlation,
    );
    let Err(error) = substituted else {
        return Err(io::Error::other(
            "a value chosen only at verification time must not certify a different authorized typed-input intent",
        )
        .into());
    };
    assert!(matches!(
        error,
        WebDriverBiDiTextValuePostconditionError::PostconditionMismatch {
            type_text_command_id: 42,
            command_id: 70,
            observed_text_bytes: 17,
        }
    ));
    assert_eq!(correlation.outstanding_count(), 0);
    Ok(())
}
