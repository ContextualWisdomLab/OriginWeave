use std::{error::Error, net::SocketAddr};

use originweave_core::{
    CorrelatedWebDriverBiDiWebSocketEndpoint, WebDriverBiDiWebSocketConnectTargetError,
    WebDriverBiDiWebSocketEndpoint,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";

fn correlated(endpoint: &str) -> CorrelatedWebDriverBiDiWebSocketEndpoint {
    let admitted = WebDriverBiDiWebSocketEndpoint::new(endpoint);
    assert!(admitted.is_ok(), "{admitted:?}");
    let Ok(admitted) = admitted else {
        unreachable!("asserted valid endpoint")
    };
    let correlated = admitted.correlate_session_id(SESSION_ID);
    assert!(correlated.is_ok(), "{correlated:?}");
    let Ok(correlated) = correlated else {
        unreachable!("asserted correlated endpoint")
    };
    correlated
}

#[test]
fn explicit_ipv4_loopback_becomes_exact_no_dns_connect_target() {
    let endpoint = format!("ws://127.0.0.1:9515/session/{SESSION_ID}");
    let result = correlated(&endpoint).into_explicit_connect_target();
    assert!(result.is_ok(), "{result:?}");
    let Ok(target) = result else {
        return;
    };

    assert_eq!(
        target.socket_addr(),
        SocketAddr::from(([127, 0, 0, 1], 9515))
    );
    assert!(!target.requires_tls());
    assert_eq!(target.session_id(), SESSION_ID);
}

#[test]
fn explicit_ipv6_loopback_preserves_exact_destination_and_tls_requirement() {
    let endpoint = format!("wss://[::1]:9443/session/{SESSION_ID}");
    let result = correlated(&endpoint).into_explicit_connect_target();
    assert!(result.is_ok(), "{result:?}");
    let Ok(target) = result else {
        return;
    };

    assert_eq!(
        target.socket_addr(),
        SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 9443))
    );
    assert!(target.requires_tls());
    assert_eq!(target.session_id(), SESSION_ID);
}

#[test]
fn localhost_name_never_silently_inherits_ambient_dns_authority() {
    let endpoint = format!("ws://localhost:9515/session/{SESSION_ID}");
    assert!(matches!(
        correlated(&endpoint).into_explicit_connect_target(),
        Err(WebDriverBiDiWebSocketConnectTargetError::NameResolutionRequired)
    ));
}

#[test]
fn connect_target_errors_are_deterministic_and_source_free() {
    let error = WebDriverBiDiWebSocketConnectTargetError::NameResolutionRequired;
    assert_eq!(
        error.to_string(),
        "WebDriver BiDi WebSocket endpoint requires explicit trusted name resolution"
    );
    assert!(error.source().is_none());
}
