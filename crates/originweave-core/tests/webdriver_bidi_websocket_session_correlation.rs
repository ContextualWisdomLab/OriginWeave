use std::error::Error;

use originweave_core::{
    WebDriverBiDiWebSocketEndpoint, WebDriverBiDiWebSocketEndpointCorrelationError,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";
const OTHER_SESSION_ID: &str = "11234567-89ab-cdef-0123-456789abcdef";

fn endpoint() -> WebDriverBiDiWebSocketEndpoint {
    let result = WebDriverBiDiWebSocketEndpoint::new(&format!(
        "ws://127.0.0.1:9515/session/{SESSION_ID}"
    ));
    assert!(result.is_ok(), "{result:?}");
    let Ok(endpoint) = result else {
        unreachable!("asserted valid endpoint")
    };
    endpoint
}

#[test]
fn exact_session_identity_correlation_preserves_bounded_endpoint_metadata() {
    let result = endpoint().correlate_session_id(SESSION_ID);
    assert!(result.is_ok(), "{result:?}");
    let Ok(correlated) = result else {
        return;
    };

    assert_eq!(
        correlated.as_str(),
        format!("ws://127.0.0.1:9515/session/{SESSION_ID}")
    );
    assert!(!correlated.is_secure());
    assert_eq!(correlated.host(), "127.0.0.1");
    assert_eq!(correlated.port(), 9515);
    assert_eq!(correlated.session_id(), SESSION_ID);
}

#[test]
fn a_different_canonical_session_identity_fails_closed() {
    assert!(matches!(
        endpoint().correlate_session_id(OTHER_SESSION_ID),
        Err(WebDriverBiDiWebSocketEndpointCorrelationError::SessionIdMismatch)
    ));
}

#[test]
fn malformed_expected_session_identity_is_rejected_before_comparison() {
    for expected in [
        "",
        "01234567-89ab-cdef-0123-456789abcdeF",
        "0123456789ab-cdef-0123-456789abcdef",
        "01234567-89ab-cdef-0123-456789abcdeg",
        "01234567_89ab-cdef-0123-456789abcdef",
    ] {
        assert!(matches!(
            endpoint().correlate_session_id(expected),
            Err(WebDriverBiDiWebSocketEndpointCorrelationError::InvalidExpectedSessionId)
        ));
    }
}

#[test]
fn session_correlation_errors_are_deterministic_and_source_free() {
    for error in [
        WebDriverBiDiWebSocketEndpointCorrelationError::InvalidExpectedSessionId,
        WebDriverBiDiWebSocketEndpointCorrelationError::SessionIdMismatch,
    ] {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }
}
