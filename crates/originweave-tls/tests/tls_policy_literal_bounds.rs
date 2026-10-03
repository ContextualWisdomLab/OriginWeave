#![allow(clippy::expect_used)]

use std::time::Duration;

use originweave_tls::{AlpnRequirement, TlsClientPolicy, TlsError};
use rustls::pki_types::UnixTime;

fn policy(timeout: Duration, protocols: Vec<Vec<u8>>) -> Result<TlsClientPolicy, TlsError> {
    TlsClientPolicy::new(
        UnixTime::since_unix_epoch(Duration::from_secs(1_800_000_000)),
        timeout,
        protocols,
        AlpnRequirement::Required,
    )
}

#[test]
fn literal_handshake_deadline_and_leaf_horizon_preserve_exact_limits() {
    let maximum = policy(Duration::from_secs(30), vec![b"h2".to_vec()])
        .expect("the documented 30-second handshake ceiling is admitted");
    assert_eq!(maximum.handshake_timeout(), Duration::from_secs(30));
    let excessive_timeout = Duration::from_secs(30) + Duration::from_nanos(1);
    assert!(matches!(
        policy(excessive_timeout, vec![b"h2".to_vec()]),
        Err(TlsError::InvalidHandshakeTimeout { timeout, maximum_timeout })
            if timeout == excessive_timeout && maximum_timeout == Duration::from_secs(30)
    ));
    let horizon = maximum
        .with_minimum_leaf_validity(Duration::from_secs(604_800))
        .expect("the documented seven-day leaf horizon is admitted");
    assert_eq!(
        horizon.minimum_leaf_validity(),
        Duration::from_secs(604_800)
    );
    let excessive_horizon = Duration::from_secs(604_800) + Duration::from_nanos(1);
    assert!(matches!(
        horizon.with_minimum_leaf_validity(excessive_horizon),
        Err(TlsError::InvalidMinimumLeafValidity { minimum_validity, maximum_validity })
            if minimum_validity == excessive_horizon && maximum_validity == Duration::from_secs(604_800)
    ));
}

#[test]
fn literal_alpn_count_length_and_total_limits_preserve_every_identifier() {
    let timeout = Duration::from_secs(1);
    let eight: Vec<Vec<u8>> = (0..8)
        .map(|index| format!("p{index}").into_bytes())
        .collect();
    let count_policy = policy(timeout, eight.clone()).expect("eight distinct ALPN identifiers");
    assert_eq!(
        count_policy.alpn_protocols(),
        eight.iter().map(Vec::as_slice).collect::<Vec<_>>()
    );
    let mut nine = eight;
    nine.push(b"p8".to_vec());
    assert!(matches!(
        policy(timeout, nine),
        Err(TlsError::InvalidAlpnCount {
            protocol_count: 9,
            maximum_count: 8
        })
    ));

    let longest = vec![0x80; 255];
    let length_policy = policy(timeout, vec![longest.clone()])
        .expect("255 opaque ALPN bytes, with no ASCII reinterpretation");
    assert_eq!(length_policy.alpn_protocols(), [longest.as_slice()]);
    assert!(matches!(
        policy(timeout, vec![vec![0x80; 256]]),
        Err(TlsError::InvalidAlpnIdentifier {
            protocol_index: 0,
            protocol_length: 256,
            maximum_length: 255,
        })
    ));

    let mut exact_total: Vec<Vec<u8>> = (b'a'..=b'd').map(|value| vec![value; 255]).collect();
    exact_total.push(vec![b'e'; 4]);
    assert_eq!(exact_total.iter().map(Vec::len).sum::<usize>(), 1_024);
    let total_policy = policy(timeout, exact_total.clone()).expect("1,024 total ALPN bytes");
    assert_eq!(
        total_policy.alpn_protocols(),
        exact_total.iter().map(Vec::as_slice).collect::<Vec<_>>()
    );
    exact_total[4].push(b'e');
    assert!(matches!(
        policy(timeout, exact_total),
        Err(TlsError::InvalidAlpnBytes {
            byte_count: 1_025,
            maximum_bytes: 1_024
        })
    ));
}
