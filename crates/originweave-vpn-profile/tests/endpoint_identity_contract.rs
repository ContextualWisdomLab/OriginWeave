use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
    parse_ikev2_profile,
};

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
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nEndpoint={endpoint}\nAllowedIPs=10.0.0.0/8\n"
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
        "vpn.example:0",
        "vpn.example:65536",
        "https://vpn.example:51820",
        "[2001:db8::1]",
        "2001:db8::1:51820",
        "[2001:db8::1]:0",
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
        let profile = import_wireguard_profile(&wireguard_profile(endpoint), &mut importer)
            .expect("reviewed endpoint shape should normalize");
        assert_eq!(profile.peers[0].endpoint.as_deref(), Some(endpoint));
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
fn ikev2_identity_rejects_ascii_control_characters_before_secret_import() {
    assert_ikev2_invalid_without_import("vpn.example", "RemoteId=remote\u{1}id\n");
    assert_ikev2_invalid_without_import("vpn.example", "LocalId=local\u{7f}id\n");
}
