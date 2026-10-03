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

    for invalid in ["", "contains space", "한글", "slash/value", "---"] {
        assert!(matches!(
            TrustBundleIdentifier::parse(invalid),
            Err(TlsError::InvalidTrustBundleIdentifier)
        ));
    }
    assert!(matches!(
        TrustBundleIdentifier::parse(&"a".repeat(129)),
        Err(TlsError::InvalidTrustBundleIdentifier)
    ));

    let exact_maximum = "a".repeat(128);
    assert_eq!(
        TrustBundleIdentifier::parse(&exact_maximum)
            .expect("128-byte identifier is the accepted maximum")
            .as_str(),
        exact_maximum
    );
    let full_alphabet = "Az09._:-az09._:-";
    assert_eq!(
        TrustBundleIdentifier::parse(full_alphabet)
            .expect("every admitted identifier byte is accepted")
            .as_str(),
        full_alphabet
    );
}

#[test]
fn trust_root_bundle_is_nonempty_bounded_deduplicated_and_hashed() {
    let root = root_der();
    let bundle = TrustRootBundle::new(
        TrustBundleIdentifier::parse("test_roots:v1").expect("identifier"),
        vec![root.clone(), root.clone()],
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
            std::iter::repeat_n(root.clone(), MAX_TRUST_ROOT_COUNT + 1).collect(),
        ),
        Err(TlsError::InvalidTrustRootCount { .. })
    ));
    assert!(matches!(
        TrustRootBundle::new(
            TrustBundleIdentifier::parse("large:v1").expect("identifier"),
            vec![vec![0_u8; MAX_TRUST_ROOT_BYTES + 1]],
        ),
        Err(TlsError::InvalidTrustRootBytes { .. })
    ));
    let exact_maximum_bytes = TrustRootBundle::new(
        TrustBundleIdentifier::parse("at_limit:v1").expect("identifier"),
        vec![vec![0_u8; MAX_TRUST_ROOT_BYTES]],
    );
    assert!(
        matches!(exact_maximum_bytes, Err(TlsError::InvalidTrustRoot { .. })),
        "exactly MAX_TRUST_ROOT_BYTES must pass the byte-count gate and fail later at DER parsing",
    );
    assert!(matches!(
        TrustRootBundle::new(
            TrustBundleIdentifier::parse("malformed:v1").expect("identifier"),
            vec![vec![1_u8, 2, 3]],
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
        vec![b"h2".to_vec(), b"http/1.1".to_vec()],
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
    assert_eq!(policy.minimum_leaf_validity(), Duration::ZERO);

    let horizon = Duration::from_secs(3_600);
    let policy_with_horizon = policy
        .with_minimum_leaf_validity(horizon)
        .expect("valid delegated-task leaf horizon");
    assert_eq!(policy_with_horizon.minimum_leaf_validity(), horizon);

    assert!(matches!(
        TlsClientPolicy::new(
            trusted_time,
            Duration::from_secs(1),
            Vec::new(),
            AlpnRequirement::Required,
        ),
        Err(TlsError::InvalidAlpnCount {
            protocol_count: 0,
            ..
        })
    ));

    for timeout in [
        Duration::ZERO,
        MAX_TLS_HANDSHAKE_TIMEOUT + Duration::from_nanos(1),
    ] {
        assert!(matches!(
            TlsClientPolicy::new(
                trusted_time,
                timeout,
                vec![b"h2".to_vec()],
                AlpnRequirement::Optional,
            ),
            Err(TlsError::InvalidHandshakeTimeout { .. })
        ));
    }

    assert!(matches!(
        policy_with_horizon.with_minimum_leaf_validity(
            originweave_tls::MAX_MINIMUM_LEAF_VALIDITY + Duration::from_nanos(1),
        ),
        Err(TlsError::InvalidMinimumLeafValidity { .. })
    ));

    let cumulative_overflow: Vec<Vec<u8>> = (0_u8..5)
        .map(|index| vec![b'a' + index; MAX_ALPN_PROTOCOL_LENGTH])
        .collect();
    let invalid_alpn_sets = [
        vec![Vec::new()],
        vec![b"h2".to_vec(), b"h2".to_vec()],
        std::iter::repeat_n(b"h2".to_vec(), MAX_ALPN_PROTOCOL_COUNT + 1).collect(),
        vec![vec![b'a'; MAX_ALPN_PROTOCOL_LENGTH + 1]],
        vec![vec![b'a'; MAX_ALPN_TOTAL_BYTES + 1]],
        cumulative_overflow,
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
fn tls_policy_accepts_every_exact_maximum_bound() {
    let trusted_time = UnixTime::since_unix_epoch(Duration::from_secs(1_800_000_000));

    let boundary_timeout = TlsClientPolicy::new(
        trusted_time,
        MAX_TLS_HANDSHAKE_TIMEOUT,
        vec![b"h2".to_vec()],
        AlpnRequirement::Required,
    )
    .expect("maximum handshake timeout is accepted");
    assert_eq!(
        boundary_timeout.handshake_timeout(),
        MAX_TLS_HANDSHAKE_TIMEOUT
    );

    let optional_without_alpn = TlsClientPolicy::new(
        trusted_time,
        Duration::from_secs(1),
        Vec::new(),
        AlpnRequirement::Optional,
    )
    .expect("optional ALPN admits an empty allow-list");
    assert!(optional_without_alpn.alpn_protocols().is_empty());

    let maximum_protocol_count: Vec<Vec<u8>> = (0..MAX_ALPN_PROTOCOL_COUNT)
        .map(|index| format!("p{index}").into_bytes())
        .collect();
    TlsClientPolicy::new(
        trusted_time,
        Duration::from_secs(1),
        maximum_protocol_count,
        AlpnRequirement::Required,
    )
    .expect("maximum distinct ALPN protocol count is accepted");

    TlsClientPolicy::new(
        trusted_time,
        Duration::from_secs(1),
        vec![vec![b'a'; MAX_ALPN_PROTOCOL_LENGTH]],
        AlpnRequirement::Required,
    )
    .expect("maximum ALPN protocol length is accepted");

    let mut maximum_total_bytes = vec![
        vec![b'a'; MAX_ALPN_PROTOCOL_LENGTH],
        vec![b'b'; MAX_ALPN_PROTOCOL_LENGTH],
        vec![b'c'; MAX_ALPN_PROTOCOL_LENGTH],
        vec![b'd'; MAX_ALPN_PROTOCOL_LENGTH],
    ];
    maximum_total_bytes.push(vec![
        b'e';
        MAX_ALPN_TOTAL_BYTES - 4 * MAX_ALPN_PROTOCOL_LENGTH
    ]);
    TlsClientPolicy::new(
        trusted_time,
        Duration::from_secs(1),
        maximum_total_bytes,
        AlpnRequirement::Required,
    )
    .expect("exact maximum ALPN total bytes is accepted");

    let boundary_horizon = boundary_timeout
        .with_minimum_leaf_validity(originweave_tls::MAX_MINIMUM_LEAF_VALIDITY)
        .expect("maximum leaf validity horizon is accepted");
    assert_eq!(
        boundary_horizon.minimum_leaf_validity(),
        originweave_tls::MAX_MINIMUM_LEAF_VALIDITY
    );

    let root = root_der();
    let duplicate_bundle = TrustRootBundle::new(
        TrustBundleIdentifier::parse("duplicate_roots:v1").expect("identifier"),
        std::iter::repeat_n(root, 256).collect(),
    )
    .expect("256 input roots are accepted before canonical deduplication");
    assert_eq!(duplicate_bundle.root_count(), 1);

    let roots: Vec<Vec<u8>> = (0..256)
        .map(|index| {
            rcgen::generate_simple_self_signed(vec![format!("root-{index}.example")])
                .expect("distinct test root generation")
                .cert
                .der()
                .to_vec()
        })
        .collect();
    assert_eq!(
        roots
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        256
    );
    let encoded_bytes: usize = roots.iter().map(Vec::len).sum();
    let boundary_bundle = TrustRootBundle::new(
        TrustBundleIdentifier::parse("boundary_roots:v1").expect("identifier"),
        roots.clone(),
    )
    .expect("256 distinct roots are retained at the literal count limit");
    assert_eq!(boundary_bundle.root_count(), 256);
    assert_eq!(boundary_bundle.encoded_byte_count(), encoded_bytes);

    let mut reversed_roots = roots.clone();
    reversed_roots.reverse();
    let reversed_bundle = TrustRootBundle::new(
        TrustBundleIdentifier::parse("boundary_roots:v1").expect("identifier"),
        reversed_roots,
    )
    .expect("reversed root input retains the same canonical bundle");
    assert_eq!(reversed_bundle.root_count(), 256);
    assert_eq!(reversed_bundle.encoded_byte_count(), encoded_bytes);
    assert_eq!(reversed_bundle.bundle_hash(), boundary_bundle.bundle_hash());

    let mut oversized_roots = roots;
    oversized_roots.push(root_der());
    assert!(matches!(
        TrustRootBundle::new(
            TrustBundleIdentifier::parse("overflow_roots:v1").expect("identifier"),
            oversized_roots,
        ),
        Err(TlsError::InvalidTrustRootCount {
            root_count: 257,
            maximum_count: 256,
        })
    ));
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

#[test]
fn explicitly_constructed_reference_identities_validate_fail_closed() {
    let origin = Origin::parse("https://example.com").expect("canonical HTTPS origin");

    TlsReferenceIdentity::Dns("example.com".to_owned())
        .validate_syntax(&origin)
        .expect("valid DNS identity");
    TlsReferenceIdentity::Ip(IpAddr::V4(Ipv4Addr::LOCALHOST))
        .validate_syntax(&origin)
        .expect("valid IP identity");

    for invalid_dns_identity in ["contains space", "127.0.0.1", "::1"] {
        assert!(matches!(
            TlsReferenceIdentity::Dns(invalid_dns_identity.to_owned()).validate_syntax(&origin),
            Err(TlsError::InvalidReferenceIdentity { .. })
        ));
    }
}
