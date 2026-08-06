#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_tls::{
    AlpnRequirement, MAX_ALPN_PROTOCOL_COUNT, MAX_ALPN_PROTOCOL_LENGTH, MAX_ALPN_TOTAL_BYTES,
    MAX_TLS_HANDSHAKE_TIMEOUT, MAX_TRUST_ROOT_BYTES, MAX_TRUST_ROOT_COUNT, TlsClientPolicy,
    TlsError, TlsReferenceIdentity, TrustBundleIdentifier, TrustRootBundle,
};
use rustls::pki_types::UnixTime;

fn root_der() -> Vec<u8> {
    rcgen::generate_simple_self_signed(vec!["root.example".to_owned()])
        .expect("test root generation")
        .cert
        .der()
        .to_vec()
}

#[test]
fn trust_bundle_identifier_is_bounded_and_ascii() {
    let identifier =
        TrustBundleIdentifier::parse("enterprise_roots:v1").expect("valid trust bundle identifier");
    assert_eq!(identifier.as_str(), "enterprise_roots:v1");

    for invalid in ["", "contains space", "한글", "slash/value"] {
        assert!(matches!(
            TrustBundleIdentifier::parse(invalid),
            Err(TlsError::InvalidTrustBundleIdentifier)
        ));
    }
    assert!(matches!(
        TrustBundleIdentifier::parse(&"a".repeat(129)),
        Err(TlsError::InvalidTrustBundleIdentifier)
    ));
}

#[test]
fn trust_root_bundle_is_nonempty_bounded_deduplicated_and_hashed() {
    let root = root_der();
    let bundle = TrustRootBundle::new(
        TrustBundleIdentifier::parse("test_roots:v1").expect("identifier"),
        [root.clone(), root.clone()],
    )
    .expect("valid roots");

    assert_eq!(bundle.root_count(), 1);
    assert_eq!(bundle.encoded_byte_count(), root.len());
    assert_eq!(bundle.identifier().as_str(), "test_roots:v1");
    assert!(bundle.bundle_hash().starts_with("sha256:"));
    assert_eq!(bundle.bundle_hash().len(), 71);

    assert!(matches!(
        TrustRootBundle::new(
            TrustBundleIdentifier::parse("empty:v1").expect("identifier"),
            Vec::<Vec<u8>>::new(),
        ),
        Err(TlsError::InvalidTrustRootCount { root_count: 0, .. })
    ));
    assert!(matches!(
        TrustRootBundle::new(
            TrustBundleIdentifier::parse("many:v1").expect("identifier"),
            std::iter::repeat(root.clone()).take(MAX_TRUST_ROOT_COUNT + 1),
        ),
        Err(TlsError::InvalidTrustRootCount { .. })
    ));
    assert!(matches!(
        TrustRootBundle::new(
            TrustBundleIdentifier::parse("large:v1").expect("identifier"),
            [vec![0_u8; MAX_TRUST_ROOT_BYTES + 1]],
        ),
        Err(TlsError::InvalidTrustRootBytes { .. })
    ));
    assert!(matches!(
        TrustRootBundle::new(
            TrustBundleIdentifier::parse("malformed:v1").expect("identifier"),
            [vec![1_u8, 2, 3]],
        ),
        Err(TlsError::InvalidTrustRoot { .. })
    ));
}

#[test]
fn tls_policy_bounds_timeouts_and_alpn() {
    let trusted_time = UnixTime::since_unix_epoch(Duration::from_secs(1_800_000_000));
    let policy = TlsClientPolicy::new(
        trusted_time,
        Duration::from_secs(3),
        [b"h2".to_vec(), b"http/1.1".to_vec()],
        AlpnRequirement::Required,
    )
    .expect("valid TLS policy");

    assert_eq!(policy.trusted_time(), trusted_time);
    assert_eq!(policy.handshake_timeout(), Duration::from_secs(3));
    assert_eq!(
        policy.alpn_protocols(),
        [b"h2".as_slice(), b"http/1.1".as_slice()]
    );
    assert_eq!(policy.alpn_requirement(), AlpnRequirement::Required);

    for timeout in [
        Duration::ZERO,
        MAX_TLS_HANDSHAKE_TIMEOUT + Duration::from_nanos(1),
    ] {
        assert!(matches!(
            TlsClientPolicy::new(
                trusted_time,
                timeout,
                [b"h2".to_vec()],
                AlpnRequirement::Optional,
            ),
            Err(TlsError::InvalidHandshakeTimeout { .. })
        ));
    }

    let invalid_alpn_sets = [
        vec![Vec::new()],
        vec![b"h2".to_vec(), b"h2".to_vec()],
        std::iter::repeat(b"h2".to_vec())
            .take(MAX_ALPN_PROTOCOL_COUNT + 1)
            .collect(),
        vec![vec![b'a'; MAX_ALPN_PROTOCOL_LENGTH + 1]],
        vec![vec![b'a'; MAX_ALPN_TOTAL_BYTES + 1]],
    ];
    for alpn in invalid_alpn_sets {
        assert!(
            TlsClientPolicy::new(
                trusted_time,
                Duration::from_secs(1),
                alpn,
                AlpnRequirement::Optional,
            )
            .is_err()
        );
    }
}

#[test]
fn canonical_https_origins_produce_dns_or_ip_reference_identities() {
    let cases = [
        (
            "https://example.com",
            TlsReferenceIdentity::Dns("example.com".to_owned()),
        ),
        (
            "https://localhost:8443",
            TlsReferenceIdentity::Dns("localhost".to_owned()),
        ),
        (
            "https://127.0.0.1",
            TlsReferenceIdentity::Ip(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        ),
        (
            "https://[::1]:8443",
            TlsReferenceIdentity::Ip(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        ),
    ];

    for (input, expected) in cases {
        let origin = Origin::parse(input).expect("canonical origin");
        assert_eq!(
            TlsReferenceIdentity::from_origin(&origin).expect("TLS identity"),
            expected
        );
    }

    let http = Origin::parse("http://localhost").expect("loopback HTTP origin");
    assert!(matches!(
        TlsReferenceIdentity::from_origin(&http),
        Err(TlsError::OriginRequiresHttps { .. })
    ));
}
