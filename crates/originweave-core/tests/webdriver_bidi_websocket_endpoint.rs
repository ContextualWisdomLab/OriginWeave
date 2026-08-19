use std::error::Error;

use originweave_core::{
    MAX_WEBDRIVER_BIDI_WEBSOCKET_ENDPOINT_BYTES, WebDriverBiDiWebSocketEndpoint,
    WebDriverBiDiWebSocketEndpointAdmissionError,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";

#[test]
fn canonical_loopback_session_endpoints_are_admitted_without_granting_authority() {
    let ipv4 = WebDriverBiDiWebSocketEndpoint::new(&format!(
        "ws://127.0.0.1:9515/session/{SESSION_ID}"
    ))
    .unwrap();
    assert!(!ipv4.is_secure());
    assert_eq!(ipv4.host(), "127.0.0.1");
    assert_eq!(ipv4.port(), 9515);
    assert_eq!(ipv4.session_id(), SESSION_ID);
    assert_eq!(
        ipv4.as_str(),
        format!("ws://127.0.0.1:9515/session/{SESSION_ID}")
    );

    let localhost = WebDriverBiDiWebSocketEndpoint::new(&format!(
        "ws://localhost:4444/session/{SESSION_ID}"
    ))
    .unwrap();
    assert_eq!(localhost.host(), "localhost");

    let ipv6 = WebDriverBiDiWebSocketEndpoint::new(&format!(
        "wss://[::1]:9222/session/{SESSION_ID}"
    ))
    .unwrap();
    assert!(ipv6.is_secure());
    assert_eq!(ipv6.host(), "::1");
    assert_eq!(ipv6.port(), 9222);
}

#[test]
fn remote_or_ambiguous_authorities_fail_closed() {
    for endpoint in [
        format!("ws://example.com:9515/session/{SESSION_ID}"),
        format!("ws://192.0.2.1:9515/session/{SESSION_ID}"),
        format!("ws://[2001:db8::1]:9515/session/{SESSION_ID}"),
    ] {
        assert_eq!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint).unwrap_err(),
            WebDriverBiDiWebSocketEndpointAdmissionError::NonLoopbackHost
        );
    }

    for endpoint in [
        format!("ws://user@localhost:9515/session/{SESSION_ID}"),
        format!("ws://localhost/session/{SESSION_ID}"),
        format!("ws://::1:9515/session/{SESSION_ID}"),
        format!("ws://[::1]9515/session/{SESSION_ID}"),
    ] {
        assert_eq!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint).unwrap_err(),
            WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority
        );
    }
}

#[test]
fn port_and_session_resource_are_canonical_and_bounded() {
    for endpoint in [
        format!("ws://localhost:0/session/{SESSION_ID}"),
        format!("ws://localhost:09515/session/{SESSION_ID}"),
        format!("ws://localhost:65536/session/{SESSION_ID}"),
        format!("ws://localhost:+9515/session/{SESSION_ID}"),
    ] {
        assert_eq!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint).unwrap_err(),
            WebDriverBiDiWebSocketEndpointAdmissionError::InvalidPort
        );
    }

    for endpoint in [
        format!("ws://localhost:9515/other/{SESSION_ID}"),
        format!("ws://localhost:9515/session/{SESSION_ID}/extra"),
        "ws://localhost:9515/session/".to_owned(),
        "ws://localhost:9515".to_owned(),
    ] {
        assert_eq!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint).unwrap_err(),
            WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionResource
        );
    }

    for session_id in [
        "01234567-89ab-cdef-0123-456789abcdeF",
        "0123456789ab-cdef-0123-456789abcdef",
        "01234567-89ab-cdef-0123-456789abcdeg",
    ] {
        assert_eq!(
            WebDriverBiDiWebSocketEndpoint::new(&format!(
                "ws://localhost:9515/session/{session_id}"
            ))
            .unwrap_err(),
            WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionId
        );
    }
}

#[test]
fn endpoint_text_rejects_noncanonical_or_unbounded_inputs_before_transport_use() {
    assert_eq!(
        WebDriverBiDiWebSocketEndpoint::new("").unwrap_err(),
        WebDriverBiDiWebSocketEndpointAdmissionError::EmptyEndpoint
    );
    assert_eq!(
        WebDriverBiDiWebSocketEndpoint::new(&format!(
            "http://localhost:9515/session/{SESSION_ID}"
        ))
        .unwrap_err(),
        WebDriverBiDiWebSocketEndpointAdmissionError::InvalidScheme
    );
    assert_eq!(
        WebDriverBiDiWebSocketEndpoint::new(&format!(
            "ws://local host:9515/session/{SESSION_ID}"
        ))
        .unwrap_err(),
        WebDriverBiDiWebSocketEndpointAdmissionError::InvalidEndpointText
    );
    assert_eq!(
        WebDriverBiDiWebSocketEndpoint::new(&format!(
            "ws://localhost:9515/session/{SESSION_ID}?token=secret"
        ))
        .unwrap_err(),
        WebDriverBiDiWebSocketEndpointAdmissionError::QueryOrFragmentForbidden
    );
    assert_eq!(
        WebDriverBiDiWebSocketEndpoint::new(&format!(
            "ws://localhost:9515/session/{SESSION_ID}#fragment"
        ))
        .unwrap_err(),
        WebDriverBiDiWebSocketEndpointAdmissionError::QueryOrFragmentForbidden
    );

    let oversized = "x".repeat(MAX_WEBDRIVER_BIDI_WEBSOCKET_ENDPOINT_BYTES + 1);
    assert_eq!(
        WebDriverBiDiWebSocketEndpoint::new(&oversized).unwrap_err(),
        WebDriverBiDiWebSocketEndpointAdmissionError::EndpointTooLong
    );
}

#[test]
fn endpoint_error_contract_is_deterministic_and_source_free() {
    let errors = [
        WebDriverBiDiWebSocketEndpointAdmissionError::EmptyEndpoint,
        WebDriverBiDiWebSocketEndpointAdmissionError::EndpointTooLong,
        WebDriverBiDiWebSocketEndpointAdmissionError::InvalidEndpointText,
        WebDriverBiDiWebSocketEndpointAdmissionError::InvalidScheme,
        WebDriverBiDiWebSocketEndpointAdmissionError::QueryOrFragmentForbidden,
        WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority,
        WebDriverBiDiWebSocketEndpointAdmissionError::NonLoopbackHost,
        WebDriverBiDiWebSocketEndpointAdmissionError::InvalidPort,
        WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionResource,
        WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionId,
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }
}
