#![allow(clippy::expect_used)]

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
fn ordinary_http_and_ipv6_proxies_are_not_forced_through_web_origin_policy() {
    let http = ProxyServer::parse("http://proxy.example:8080")
        .expect("Chromium-compatible HTTP proxy must be representable");
    assert_eq!(http.as_str(), "http://proxy.example:8080");
    assert_eq!(http.scheme(), ProxyServerScheme::Http);

    let no_port = ProxyServer::parse("http://proxy.example")
        .expect("default-port HTTP proxy must be representable");
    assert_eq!(no_port.as_str(), "http://proxy.example");

    let ipv6 = ProxyServer::parse("socks5://[2001:db8::1]:1080")
        .expect("canonical IPv6 proxy must be representable");
    assert_eq!(ipv6.as_str(), "socks5://[2001:db8::1]");

    let ipv6_nondefault = ProxyServer::parse("https://[2001:db8::1]:8443")
        .expect("non-default IPv6 proxy port must be preserved");
    assert_eq!(ipv6_nondefault.as_str(), "https://[2001:db8::1]:8443");
}

#[test]
fn proxy_server_identity_rejects_ambiguous_or_credential_bearing_values() {
    for input in [
        " proxy://host",
        "http://proxy.exa\nmple:8080",
        "http://proxy.exa\u{2003}mple:8080",
        "proxy.example:8080",
        "ftp://proxy.example:21",
        "http://",
        "http://user:pass@proxy.example:8080",
        "http://proxy.example:0",
        "http://proxy.example:not-a-port",
        "http://proxy.example/path",
        "http://proxy.example?token=value",
        "http://proxy.example#fragment",
        "http://2130706433:8080",
        "http://0x7f000001:8080",
        "http://[::1",
        "http://[::1]oops",
    ] {
        assert_eq!(
            ProxyServer::parse(input),
            Err(ProxyServerError::InvalidIdentifier),
            "{input} must fail closed",
        );
    }
}

#[test]
fn proxy_server_error_has_a_deterministic_standard_error_contract() {
    let error = ProxyServerError::InvalidIdentifier;
    assert_eq!(error.to_string(), "invalid proxy server identifier");
    let standard: &dyn std::error::Error = &error;
    assert!(standard.source().is_none());
}
