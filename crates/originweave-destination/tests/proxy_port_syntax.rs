use originweave_destination::{ProxyServer, ProxyServerError};

#[test]
fn proxy_server_rejects_non_digit_port_prefixes() {
    for input in [
        "proxy.example:+8080",
        "http://proxy.example:+8080",
        "https://proxy.example:+8443",
        "socks5://proxy.example:+1080",
        "https://[2001:db8::1]:+8443",
    ] {
        assert_eq!(
            ProxyServer::parse(input),
            Err(ProxyServerError::InvalidIdentifier),
            "input={input}",
        );
    }
}
