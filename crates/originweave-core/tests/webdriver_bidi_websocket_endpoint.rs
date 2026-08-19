use std::error::Error;

use originweave_core::{
    MAX_WEBDRIVER_BIDI_WEBSOCKET_ENDPOINT_BYTES, WebDriverBiDiWebSocketEndpoint,
    WebDriverBiDiWebSocketEndpointAdmissionError,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";

#[test]
fn canonical_loopback_session_endpoints_are_admitted_without_granting_authority() {
    let ipv4_result =
        WebDriverBiDiWebSocketEndpoint::new(&format!("ws://127.0.0.1:9515/session/{SESSION_ID}"));
    assert!(ipv4_result.is_ok(), "{ipv4_result:?}");
    let Ok(ipv4) = ipv4_result else {
        return;
    };
    assert!(!ipv4.is_secure());
    assert_eq!(ipv4.host(), "127.0.0.1");
    assert_eq!(ipv4.port(), 9515);
    assert_eq!(ipv4.session_id(), SESSION_ID);
    assert_eq!(
        ipv4.as_str(),
        format!("ws://127.0.0.1:9515/session/{SESSION_ID}")
    );

    let localhost_result =
        WebDriverBiDiWebSocketEndpoint::new(&format!("ws://localhost:4444/session/{SESSION_ID}"));
    assert!(localhost_result.is_ok(), "{localhost_result:?}");
    let Ok(localhost) = localhost_result else {
        return;
    };
    assert_eq!(localhost.host(), "localhost");

    let ipv6_result =
        WebDriverBiDiWebSocketEndpoint::new(&format!("wss://[::1]:9222/session/{SESSION_ID}"));
    assert!(ipv6_result.is_ok(), "{ipv6_result:?}");
    let Ok(ipv6) = ipv6_result else {
        return;
    };
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
        assert!(matches!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint),
            Err(WebDriverBiDiWebSocketEndpointAdmissionError::NonLoopbackHost)
        ));
    }

    for endpoint in [
        format!("ws://user@localhost:9515/session/{SESSION_ID}"),
        format!("ws://localhost/session/{SESSION_ID}"),
        format!("ws://::1:9515/session/{SESSION_ID}"),
        format!("ws://[::1]9515/session/{SESSION_ID}"),
        format!("ws://[::zz]:9515/session/{SESSION_ID}"),
        format!("ws://:9515/session/{SESSION_ID}"),
    ] {
        assert!(matches!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint),
            Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority)
        ));
    }
}

#[test]
fn malformed_loopback_authority_edge_cases_fail_closed() {
    for endpoint in [
        format!("ws://[::1:9515/session/{SESSION_ID}"),
        format!("ws://[::1]:/session/{SESSION_ID}"),
        format!("ws://[0:0:0:0:0:0:0:1]:9515/session/{SESSION_ID}"),
        format!("ws://localhost:/session/{SESSION_ID}"),
        format!("ws://local_host:9515/session/{SESSION_ID}"),
    ] {
        assert!(matches!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint),
            Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority)
        ));
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
        assert!(matches!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint),
            Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidPort)
        ));
    }

    for endpoint in [
        format!("ws://localhost:9515/other/{SESSION_ID}"),
        format!("ws://localhost:9515/session/{SESSION_ID}/extra"),
        "ws://localhost:9515/session/".to_owned(),
        "ws://localhost:9515".to_owned(),
    ] {
        assert!(matches!(
            WebDriverBiDiWebSocketEndpoint::new(&endpoint),
            Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionResource)
        ));
    }

    for session_id in [
        "01234567-89ab-cdef-0123-456789abcdeF",
        "0123456789ab-cdef-0123-456789abcdef",
        "01234567-89ab-cdef-0123-456789abcdeg",
        "01234567_89ab-cdef-0123-456789abcdef",
    ] {
        assert!(matches!(
            WebDriverBiDiWebSocketEndpoint::new(&format!(
                "ws://localhost:9515/session/{session_id}"
            )),
            Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionId)
        ));
    }
}

#[test]
fn endpoint_text_rejects_noncanonical_or_unbounded_inputs_before_transport_use() {
    assert!(matches!(
        WebDriverBiDiWebSocketEndpoint::new(""),
        Err(WebDriverBiDiWebSocketEndpointAdmissionError::EmptyEndpoint)
    ));
    assert!(matches!(
        WebDriverBiDiWebSocketEndpoint::new(&format!("http://localhost:9515/session/{SESSION_ID}")),
        Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidScheme)
    ));
    assert!(matches!(
        WebDriverBiDiWebSocketEndpoint::new(&format!("ws://local host:9515/session/{SESSION_ID}")),
        Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidEndpointText)
    ));
    assert!(matches!(
        WebDriverBiDiWebSocketEndpoint::new(&format!("ws://locálhost:9515/session/{SESSION_ID}")),
        Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidEndpointText)
    ));
    assert!(matches!(
        WebDriverBiDiWebSocketEndpoint::new(&format!(
            "ws://localhost:9515/session/{SESSION_ID}?token=secret"
        )),
        Err(WebDriverBiDiWebSocketEndpointAdmissionError::QueryOrFragmentForbidden)
    ));
    assert!(matches!(
        WebDriverBiDiWebSocketEndpoint::new(&format!(
            "ws://localhost:9515/session/{SESSION_ID}#fragment"
        )),
        Err(WebDriverBiDiWebSocketEndpointAdmissionError::QueryOrFragmentForbidden)
    ));

    let oversized = "x".repeat(MAX_WEBDRIVER_BIDI_WEBSOCKET_ENDPOINT_BYTES + 1);
    assert!(matches!(
        WebDriverBiDiWebSocketEndpoint::new(&oversized),
        Err(WebDriverBiDiWebSocketEndpointAdmissionError::EndpointTooLong)
    ));
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
