use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
    parse_ikev2_profile,
};

#[derive(Default)]
struct CountingImporter {
    calls: usize,
}

impl VpnSecretImporter for CountingImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.calls += 1;
        SecretReference::new(format!("secret://network-syntax/{}", self.calls))
    }
}

fn reject_wireguard(profile: &str) {
    let mut importer = CountingImporter::default();
    assert_eq!(
        import_wireguard_profile(profile, &mut importer),
        Err(ProfileError::InvalidValue)
    );
    assert_eq!(importer.calls, 0);
}

fn reject_ikev2(profile: &str) {
    let mut importer = CountingImporter::default();
    assert_eq!(
        parse_ikev2_profile(profile, &mut importer),
        Err(ProfileError::InvalidValue)
    );
    assert_eq!(importer.calls, 0);
}

#[test]
fn wireguard_rejects_invalid_interface_network_syntax_before_secret_import() {
    for profile in [
        "[Interface]\nAddress=999.1.1.1/24\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2/33\nPrivateKey=k",
        "[Interface]\nAddress=2001:db8::2/129\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2/32\nDNS=999.1.1.1\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2\nPrivateKey=k",
    ] {
        reject_wireguard(profile);
    }
}

#[test]
fn wireguard_rejects_invalid_allowed_ip_syntax_before_secret_import() {
    for allowed_ips in [
        "999.0.0.0/8",
        "10.0.0.0/33",
        "2001:db8::/129",
        "10.0.0.0",
    ] {
        let profile = format!(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs={allowed_ips}"
        );
        reject_wireguard(&profile);
    }
}

#[test]
fn ikev2_rejects_invalid_traffic_selector_syntax_before_secret_import() {
    for selector in [
        "999.0.0.0/8",
        "10.0.0.0/33",
        "2001:db8::/129",
        "10.0.0.0",
    ] {
        let profile = format!(
            "[IKEv2]\nServer=vpn.example\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors={selector}"
        );
        reject_ikev2(&profile);
    }
}

#[test]
fn representative_ipv4_and_ipv6_network_syntax_remains_accepted() {
    let wireguard = "[Interface]\nAddress=10.0.0.2/32,fd00::2/128\nDNS=1.1.1.1,2606:4700:4700::1111\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=0.0.0.0/0,::/0\n";
    let mut wireguard_importer = CountingImporter::default();
    assert!(
        import_wireguard_profile(wireguard, &mut wireguard_importer).is_ok(),
        "representative WireGuard network syntax must remain accepted"
    );
    assert_eq!(wireguard_importer.calls, 1);

    let ikev2 = "[IKEv2]\nServer=vpn.example\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8,fd00::/8\n";
    let mut ikev2_importer = CountingImporter::default();
    assert!(
        parse_ikev2_profile(ikev2, &mut ikev2_importer).is_ok(),
        "representative IKEv2 traffic selectors must remain accepted"
    );
    assert_eq!(ikev2_importer.calls, 1);
}
