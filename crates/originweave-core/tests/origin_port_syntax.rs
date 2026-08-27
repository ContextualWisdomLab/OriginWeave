use originweave_core::{Origin, OriginError};

#[test]
fn origin_rejects_non_digit_port_prefixes() {
    for input in [
        "https://example.com:+443",
        "https://example.com:+8443",
        "http://localhost:+80",
        "http://127.0.0.1:+8080",
        "https://[2001:db8::1]:+443",
    ] {
        assert_eq!(
            Origin::parse(input),
            Err(OriginError::InvalidPort),
            "input={input}"
        );
    }
}
