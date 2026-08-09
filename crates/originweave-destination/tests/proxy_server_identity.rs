use originweave_destination::{ProxyServer, ProxyServerError, ProxyServerScheme};

#[test]
fn chromium_proxy_server_schemes_have_distinct_canonical_identity() {
    let cases = [
        (
            "http://PROXY.EXAMPLE:80",
            "http://proxy.example",
            ProxyServerScheme::Http,
        ),
        (
            "https://PROXY.EXAMPLE:443",
            "https://proxy.example",
            ProxyServerScheme::Https,
        ),
        (
            "socks4://PROXY.EXAMPLE:1080",
            "socks4://proxy.example",
            ProxyServerScheme::Socks4,
        ),
        (
            "socks://PROXY.EXAMPLE:1080",
            "socks5://proxy.example",
            ProxyServerScheme::Socks5,
        ),
        (
            "socks5://PROXY.EXAMPLE:1080",
            "socks5://proxy.example",
            ProxyServerScheme::Socks5,
        ),
        (
            "quic://PROXY.EXAMPLE:443",
            "quic://proxy.example",
            ProxyServerScheme::Quic,
        ),
    ];

    for (input, canonical, scheme) in cases {
        let server = ProxyServer::parse(input).expect("supported proxy server must parse");
        assert_eq!(server.as_str(), canonical);
        assert_eq!(server.scheme(), scheme);
    }
}

#[test]
fn ordinary_http_proxy_is_not_forced_through_web_origin_policy() {
    let server = ProxyServer::parse("http://proxy.example:8080")
        .expect("Chromium-compatible HTTP proxy must be representable");
    assert_eq!(server.as_str(), "http://proxy.example:8080");
    assert_eq!(server.scheme(), ProxyServerScheme::Http);
}

#[test]
fn proxy_server_identity_rejects_ambiguous_or_credential_bearing_values() {
    for input in [
        "ftp://proxy.example:21",
        "http://user:pass@proxy.example:8080",
        "http://proxy.example:0",
        "http://proxy.example/path",
        "http://proxy.example?token=value",
        "http://proxy.example#fragment",
        "http://2130706433:8080",
        "http://0x7f000001:8080",
    ] {
        assert_eq!(
            ProxyServer::parse(input),
            Err(ProxyServerError::InvalidIdentifier),
            "{input} must fail closed",
        );
    }
}
