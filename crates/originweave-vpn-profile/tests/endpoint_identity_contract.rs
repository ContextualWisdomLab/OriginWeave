use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
    parse_ikev2_profile,
};

const VALID_WIREGUARD_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

#[derive(Default)]
struct CountingImporter(usize);

impl VpnSecretImporter for CountingImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.0 += 1;
        SecretReference::new(format!("secret://endpoint-contract/{}", self.0))
    }
}

fn wireguard_profile(endpoint: &str) -> String {
    format!(
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nEndpoint={endpoint}\nAllowedIPs=10.0.0.0/8\n"
    )
}

fn ikev2_profile(server: &str, identity_lines: &str) -> String {
    format!(
        "[IKEv2]\nServer={server}\n{identity_lines}Auth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\n"
    )
}

fn assert_wireguard_invalid_without_import(endpoint: &str) {
    let mut importer = CountingImporter::default();
    assert_eq!(
        import_wireguard_profile(&wireguard_profile(endpoint), &mut importer),
        Err(ProfileError::InvalidValue),
        "invalid WireGuard endpoint was accepted: {endpoint:?}"
    );
    assert_eq!(
        importer.0, 0,
        "invalid endpoint reached the caller importer"
    );
}

fn assert_ikev2_invalid_without_import(server: &str, identity_lines: &str) {
    let mut importer = CountingImporter::default();
    assert_eq!(
        parse_ikev2_profile(&ikev2_profile(server, identity_lines), &mut importer),
        Err(ProfileError::InvalidValue),
        "invalid IKEv2 endpoint or identity was accepted"
    );
    assert_eq!(
        importer.0, 0,
        "invalid IKEv2 metadata reached the caller importer"
    );
}

#[test]
fn wireguard_endpoint_requires_host_and_nonzero_udp_port_before_secret_import() {
    for endpoint in [
        "vpn.example",
        ":51820",
        "vpn.example:",
        "vpn.example:0",
        "vpn.example:65536",
        "vpn.example:+51820",
        "https://vpn.example:51820",
        "[not-ip]:51820",
        "[2001:db8::1]",
        "2001:db8::1:51820",
        "[2001:db8::1]:",
        "[2001:db8::1]:0",
        "[2001:db8::1]:+51820",
        "[2001:db8::1]: 51820",
    ] {
        assert_wireguard_invalid_without_import(endpoint);
    }
}

#[test]
fn wireguard_endpoint_accepts_hostname_ipv4_and_bracketed_ipv6_shapes() {
    for endpoint in [
        "vpn.example:51820",
        "203.0.113.7:51820",
        "[2001:db8::1]:51820",
    ] {
        let mut importer = CountingImporter::default();
        let result = import_wireguard_profile(&wireguard_profile(endpoint), &mut importer);
        assert!(
            result.is_ok(),
            "reviewed endpoint shape should normalize: {endpoint:?}"
        );
        if let Ok(profile) = result {
            assert_eq!(profile.peers[0].endpoint.as_deref(), Some(endpoint));
        }
        assert_eq!(importer.0, 1);
    }
}

#[test]
fn ikev2_server_rejects_url_and_port_authority_before_secret_import() {
    for server in [
        "https://vpn.example",
        "vpn.example:500",
        "[2001:db8::1]:500",
        "vpn example",
    ] {
        assert_ikev2_invalid_without_import(server, "");
    }
}

#[test]
fn gateway_hosts_reject_ambiguous_numeric_ipv4_spellings_before_secret_import() {
    for host in ["2130706433", "127.1", "0177.0.0.1", "0x7f000001"] {
        assert_wireguard_invalid_without_import(&format!("{host}:51820"));
        assert_ikev2_invalid_without_import(host, "");
    }
}

#[test]
fn gateway_hosts_preserve_hex_prefixed_but_nonnumeric_dns_labels() -> Result<(), ProfileError> {
    let host = "0xnothex.example";

    let mut wireguard_importer = CountingImporter::default();
    let wireguard = import_wireguard_profile(
        &wireguard_profile(&format!("{host}:51820")),
        &mut wireguard_importer,
    )?;
    assert_eq!(
        wireguard.peers[0].endpoint.as_deref(),
        Some("0xnothex.example:51820")
    );
    assert_eq!(wireguard_importer.0, 1);

    let mut ikev2_importer = CountingImporter::default();
    let ikev2 = parse_ikev2_profile(&ikev2_profile(host, ""), &mut ikev2_importer)?;
    assert_eq!(ikev2.server, host);
    assert_eq!(ikev2_importer.0, 1);
    Ok(())
}

#[test]
fn ikev2_server_rejects_every_dns_hostname_boundary_before_secret_import() {
    let overlong_host = "a".repeat(254);
    let overlong_label = format!("{}.example", "a".repeat(64));
    for server in [
        overlong_host.as_str(),
        ".example",
        overlong_label.as_str(),
        "-vpn.example",
        "vpn-.example",
        "vpn\u{1}example",
    ] {
        assert_ikev2_invalid_without_import(server, "");
    }
}

#[test]
fn ikev2_identity_rejects_ascii_control_characters_before_secret_import() {
    assert_ikev2_invalid_without_import("vpn.example", "RemoteId=remote\u{1}id\n");
    assert_ikev2_invalid_without_import("vpn.example", "LocalId=local\u{7f}id\n");
}

#[test]
fn ikev2_identity_rejects_unicode_presentation_controls_before_secret_import() {
    for identity_lines in [
        "RemoteId=remote\u{202e}id\n",
        "LocalId=local\u{2066}id\n",
        "RemoteId=remote\u{200b}id\n",
    ] {
        assert_ikev2_invalid_without_import("vpn.example", identity_lines);
    }

    let profile = "[IKEv2]\nServer=vpn.example\nAuth=eap\nUsername=user\u{feff}name\nPassword=p\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8\n";
    let mut importer = CountingImporter::default();
    assert_eq!(
        parse_ikev2_profile(profile, &mut importer),
        Err(ProfileError::InvalidValue)
    );
    assert_eq!(
        importer.0, 0,
        "presentation-controlled username reached caller importer"
    );
}

#[test]
fn ikev2_identity_fields_are_explicitly_bounded_before_secret_import() {
    let overlong_identity = "a".repeat(254);
    assert_ikev2_invalid_without_import("vpn.example", &format!("RemoteId={overlong_identity}\n"));
    assert_ikev2_invalid_without_import("vpn.example", &format!("LocalId={overlong_identity}\n"));

    let profile = format!(
        "[IKEv2]\nServer=vpn.example\nAuth=eap\nUsername={overlong_identity}\nPassword=p\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8\n"
    );
    let mut importer = CountingImporter::default();
    assert_eq!(
        parse_ikev2_profile(&profile, &mut importer),
        Err(ProfileError::InvalidValue)
    );
    assert_eq!(importer.0, 0, "invalid username reached caller importer");
}

#[test]
fn ikev2_optional_negotiation_extensions_default_to_disabled() -> Result<(), ProfileError> {
    let mut importer = CountingImporter::default();
    let profile = parse_ikev2_profile(&ikev2_profile("vpn.example", ""), &mut importer)?;

    assert!(!profile.mobike, "MOBIKE requires explicit profile opt-in");
    assert!(
        !profile.fragmentation,
        "IKEv2 fragmentation requires explicit profile opt-in"
    );
    assert_eq!(importer.0, 1);
    Ok(())
}
