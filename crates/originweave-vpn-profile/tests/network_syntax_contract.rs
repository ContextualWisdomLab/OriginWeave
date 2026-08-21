use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
    parse_ikev2_profile,
};

const VALID_WIREGUARD_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

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
        "[Interface]\nAddress=10.0.0.2/+32\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2/032\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2/32\nDNS=999.1.1.1\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2/32\nDNS=bad_domain\nPrivateKey=k",
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
        "not-an-ip",
        "10.0.0.0/+8",
        "10.0.0.0/08",
    ] {
        let profile = format!(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nAllowedIPs={allowed_ips}"
        );
        reject_wireguard(&profile);
    }
}

#[test]
fn wireguard_prefixless_allowed_ips_normalize_to_host_routes() {
    let profile = format!(
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nAllowedIPs=192.0.2.7,2001:db8::7\n"
    );
    let mut importer = CountingImporter::default();
    let normalized = import_wireguard_profile(&profile, &mut importer);

    assert!(matches!(
        normalized,
        Ok(profile)
            if profile.peers.first().is_some_and(|peer| {
                peer.allowed_ips == ["192.0.2.7/32", "2001:db8::7/128"]
            })
    ));
    assert_eq!(importer.calls, 1);
}

#[test]
fn wireguard_rejects_noncanonical_decimal_scalars_before_secret_import() {
    for profile in [
        format!(
            "[Interface]\nAddress=10.0.0.2/32\nMTU=01420\nPrivateKey={VALID_WIREGUARD_KEY}\n"
        ),
        format!(
            "[Interface]\nAddress=10.0.0.2/32\nListenPort=051820\nPrivateKey={VALID_WIREGUARD_KEY}\n"
        ),
        format!(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nEndpoint=vpn.example:051820\nAllowedIPs=10.0.0.0/8\n"
        ),
        format!(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nAllowedIPs=10.0.0.0/8\nPersistentKeepalive=025\n"
        ),
    ] {
        reject_wireguard(&profile);
    }
}

#[test]
fn ikev2_rejects_noncanonical_decimal_timers_before_secret_import() {
    for timer in ["DpdSeconds=030", "RekeySeconds=03600"] {
        let profile = format!(
            "[IKEv2]\nServer=vpn.example\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\n{timer}\n"
        );
        reject_ikev2(&profile);
    }
}

#[test]
fn ikev2_rejects_invalid_traffic_selector_syntax_before_secret_import() {
    for selector in [
        "999.0.0.0/8",
        "10.0.0.0/33",
        "2001:db8::/129",
        "10.0.0.0",
        "10.0.0.0/+8",
        "10.0.0.0/08",
    ] {
        let profile = format!(
            "[IKEv2]\nServer=vpn.example\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors={selector}"
        );
        reject_ikev2(&profile);
    }
}

#[test]
fn representative_ipv4_and_ipv6_network_syntax_remains_accepted() {
    let wireguard = format!(
        "[Interface]\nAddress=10.0.0.2/32,fd00::2/128\nDNS=1.1.1.1,2606:4700:4700::1111\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nAllowedIPs=0.0.0.0/0,::/0\n"
    );
    let mut wireguard_importer = CountingImporter::default();
    assert!(
        import_wireguard_profile(&wireguard, &mut wireguard_importer).is_ok(),
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
