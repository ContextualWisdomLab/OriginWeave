#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use super::{ConnectionPlan, NetworkError};
use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};

/// Approve one canonical loopback address under an explicit test-only destination policy.
fn approved_loopback(ip: IpAddr, origin: &str) -> ResolutionSnapshot {
    ResolutionSnapshot::approve(
        Origin::parse(origin).expect("canonical loopback origin"),
        [ip],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("explicitly approved loopback snapshot")
}

/// Validate literal timeout admission and typed adjacent-overflow rejection without opening a socket.
#[test]
fn literal_thirty_second_deadline_admits_exact_input_and_rejects_adjacent_overflow() {
    for (ip, origin) in [
        (IpAddr::V4(Ipv4Addr::LOCALHOST), "http://localhost"),
        (IpAddr::V6(Ipv6Addr::LOCALHOST), "http://[::1]"),
    ] {
        let snapshot = approved_loopback(ip, origin);
        let socket = SocketAddr::new(ip, 80);
        for deadline in [Duration::from_nanos(1), Duration::from_secs(30)] {
            ConnectionPlan::new(&snapshot, socket, deadline, 1)
                .expect("literal minimum and maximum deadlines are admitted without connecting");
        }
        for deadline in [
            Duration::ZERO,
            Duration::from_secs(30) + Duration::from_nanos(1),
        ] {
            let error = ConnectionPlan::new(&snapshot, socket, deadline, 1)
                .expect_err("zero and literal adjacent overflow must be rejected");
            assert!(matches!(
                error,
                NetworkError::InvalidConnectTimeout { connect_timeout, maximum_timeout }
                    if connect_timeout == deadline && maximum_timeout == Duration::from_secs(30)
            ));
        }
    }
}

/// Validate every literal attempt budget and typed overflow rejection without executing retries.
#[test]
fn literal_four_attempt_budget_admits_every_count_and_rejects_adjacent_overflow() {
    for (ip, origin) in [
        (IpAddr::V4(Ipv4Addr::LOCALHOST), "http://localhost"),
        (IpAddr::V6(Ipv6Addr::LOCALHOST), "http://[::1]"),
    ] {
        let snapshot = approved_loopback(ip, origin);
        let socket = SocketAddr::new(ip, 80);
        for attempts in 1..=4 {
            ConnectionPlan::new(&snapshot, socket, Duration::from_nanos(1), attempts)
                .expect("every literal admitted attempt count validates without connecting");
        }
        for attempts in [0, 5, u8::MAX] {
            let error = ConnectionPlan::new(&snapshot, socket, Duration::from_nanos(1), attempts)
                .expect_err("zero, adjacent overflow, and largest byte must be rejected");
            assert!(matches!(
                error,
                NetworkError::InvalidAttemptCount { attempt_count, maximum_attempts }
                    if attempt_count == attempts && maximum_attempts == 4
            ));
        }
    }
}
