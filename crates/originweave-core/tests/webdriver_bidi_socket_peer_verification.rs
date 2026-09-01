use std::{error::Error, net::SocketAddr};

use originweave_core::{
    CorrelatedWebDriverBiDiWebSocketEndpoint, WebDriverBiDiSocketPeerVerificationError,
    WebDriverBiDiWebSocketEndpoint,
};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";

fn connect_target(endpoint: &str) -> originweave_core::WebDriverBiDiWebSocketConnectTarget {
    let admitted = WebDriverBiDiWebSocketEndpoint::new(endpoint);
    assert!(admitted.is_ok(), "{admitted:?}");
    let Ok(admitted) = admitted else {
        unreachable!("asserted valid endpoint")
    };

    let correlated: Result<CorrelatedWebDriverBiDiWebSocketEndpoint, _> =
        admitted.correlate_session_id(SESSION_ID);
    assert!(correlated.is_ok(), "{correlated:?}");
    let Ok(correlated) = correlated else {
        unreachable!("asserted correlated endpoint")
    };

    let target = correlated.into_explicit_connect_target();
    assert!(target.is_ok(), "{target:?}");
    let Ok(target) = target else {
        unreachable!("asserted literal loopback target")
    };
    target
}

#[test]
fn exact_connected_peer_becomes_verified_transport_metadata() {
    let endpoint = format!("wss://127.0.0.1:9443/session/{SESSION_ID}");
    let target = connect_target(&endpoint);
    let peer = SocketAddr::from(([127, 0, 0, 1], 9443));

    let verified = target.verify_connected_peer(peer);
    assert!(verified.is_ok(), "{verified:?}");
    let Ok(verified) = verified else {
        return;
    };

    assert_eq!(verified.socket_addr(), peer);
    assert!(verified.requires_tls());
    assert_eq!(verified.session_id(), SESSION_ID);
}

#[test]
fn connected_peer_with_wrong_port_fails_closed() {
    let endpoint = format!("ws://127.0.0.1:9515/session/{SESSION_ID}");
    let target = connect_target(&endpoint);
    let actual = SocketAddr::from(([127, 0, 0, 1], 9516));

    let result = target.verify_connected_peer(actual);
    assert_eq!(
        result,
        Err(WebDriverBiDiSocketPeerVerificationError::PeerMismatch {
            expected: SocketAddr::from(([127, 0, 0, 1], 9515)),
            actual,
        })
    );
}

#[test]
fn connected_peer_with_different_address_fails_closed() {
    let endpoint = format!("ws://[::1]:9515/session/{SESSION_ID}");
    let target = connect_target(&endpoint);
    let actual = SocketAddr::from(([127, 0, 0, 1], 9515));

    let result = target.verify_connected_peer(actual);
    assert!(matches!(
        result,
        Err(WebDriverBiDiSocketPeerVerificationError::PeerMismatch { .. })
    ));
}

#[test]
fn non_loopback_observed_peer_cannot_inherit_approved_loopback_authority() {
    let endpoint = format!("ws://127.0.0.1:9515/session/{SESSION_ID}");
    let target = connect_target(&endpoint);
    let actual = SocketAddr::from(([192, 0, 2, 10], 9515));

    let result = target.verify_connected_peer(actual);
    assert!(matches!(
        result,
        Err(WebDriverBiDiSocketPeerVerificationError::PeerMismatch { .. })
    ));
}

#[test]
fn peer_mismatch_error_is_deterministic_and_source_free() {
    let endpoint = format!("ws://127.0.0.1:9515/session/{SESSION_ID}");
    let target = connect_target(&endpoint);
    let actual = SocketAddr::from(([127, 0, 0, 1], 9516));

    let result = target.verify_connected_peer(actual);
    let Err(error) = result else {
        return;
    };

    assert_eq!(
        error.to_string(),
        "connected WebDriver BiDi socket peer does not match the approved destination"
    );
    assert!(error.source().is_none());
}
