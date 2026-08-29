use std::{net::TcpListener, thread, time::Duration};

use originweave_core::WebDriverBiDiWebSocketEndpoint;
use originweave_network::{WebDriverBiDiTcpConnectionError, WebDriverBiDiTcpConnectionPlan};

const SESSION_ID: &str = "01234567-89ab-cdef-0123-456789abcdef";

fn connect_target(endpoint: &str) -> originweave_core::WebDriverBiDiWebSocketConnectTarget {
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

    let target = correlated.into_explicit_connect_target();
    assert!(target.is_ok(), "{target:?}");
    let Ok(target) = target else {
        unreachable!("asserted literal loopback target")
    };
    target
}

#[test]
fn exact_loopback_target_opens_one_verified_bidi_tcp_stream() {
    let listener = TcpListener::bind(("127.0.0.1", 0));
    assert!(listener.is_ok(), "{listener:?}");
    let Ok(listener) = listener else {
        return;
    };
    let local_addr = listener.local_addr();
    assert!(local_addr.is_ok(), "{local_addr:?}");
    let Ok(local_addr) = local_addr else {
        return;
    };

    let server = thread::spawn(move || listener.accept().map(|_| ()));
    let endpoint = format!("ws://{local_addr}/session/{SESSION_ID}");
    let target = connect_target(&endpoint);
    let plan = WebDriverBiDiTcpConnectionPlan::new(target, Duration::from_secs(1), 1);
    assert!(plan.is_ok(), "{plan:?}");
    let Ok(plan) = plan else {
        return;
    };

    let connection = plan.connect();
    assert!(connection.is_ok(), "{connection:?}");
    let Ok(connection) = connection else {
        return;
    };

    assert_eq!(connection.verified_peer().socket_addr(), local_addr);
    assert!(!connection.verified_peer().requires_tls());
    assert_eq!(connection.verified_peer().session_id(), SESSION_ID);
    assert_eq!(connection.attempt_number(), 1);
    assert_eq!(connection.connect_timeout(), Duration::from_secs(1));

    let (stream, evidence) = connection.into_parts();
    assert_eq!(stream.peer_addr().ok(), Some(local_addr));
    assert_eq!(evidence.verified_peer().socket_addr(), local_addr);
    assert!(!evidence.verified_peer().requires_tls());
    assert_eq!(evidence.verified_peer().session_id(), SESSION_ID);
    assert_eq!(evidence.attempt_number(), 1);
    assert_eq!(evidence.connect_timeout(), Duration::from_secs(1));

    let server_result = server.join();
    assert!(server_result.is_ok(), "{server_result:?}");
    if let Ok(accept_result) = server_result {
        assert!(accept_result.is_ok(), "{accept_result:?}");
    }
}

#[test]
fn bidi_tcp_plan_rejects_invalid_retry_settings_before_io() {
    let endpoint = format!("wss://127.0.0.1:9443/session/{SESSION_ID}");
    let zero_timeout =
        WebDriverBiDiTcpConnectionPlan::new(connect_target(&endpoint), Duration::ZERO, 1);
    assert!(matches!(
        zero_timeout,
        Err(WebDriverBiDiTcpConnectionError::InvalidConnectTimeout { .. })
    ));

    let zero_attempts =
        WebDriverBiDiTcpConnectionPlan::new(connect_target(&endpoint), Duration::from_secs(1), 0);
    assert!(matches!(
        zero_attempts,
        Err(WebDriverBiDiTcpConnectionError::InvalidAttemptCount { .. })
    ));
}
