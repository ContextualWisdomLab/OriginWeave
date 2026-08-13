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
        SecretReference::new(format!("secret://coverage/{}", self.calls))
    }
}

fn reject_wireguard(profile: &str, expected: ProfileError) {
    let mut importer = CountingImporter::default();
    assert_eq!(import_wireguard_profile(profile, &mut importer), Err(expected));
    assert_eq!(importer.calls, 0);
}

fn reject_ikev2(profile: &str, expected: ProfileError) {
    let mut importer = CountingImporter::default();
    assert_eq!(parse_ikev2_profile(profile, &mut importer), Err(expected));
    assert_eq!(importer.calls, 0);
}

#[test]
fn wireguard_duplicate_singletons_fail_before_external_secret_import() {
    for profile in [
        "[Interface]\nAddress=a\nDNS=1.1.1.1\nDNS=8.8.8.8\nPrivateKey=k",
        "[Interface]\nAddress=a\nMTU=1400\nMTU=1500\nPrivateKey=k",
        "[Interface]\nAddress=a\nListenPort=51820\nListenPort=51821\nPrivateKey=k",
        "[Interface]\nAddress=a\nPrivateKey=k\nPrivateKey=q",
        "[Interface]\nAddress=a\nPrivateKey=k\n[Peer]\nPublicKey=p\nPresharedKey=s\nPresharedKey=t\nAllowedIPs=a",
        "[Interface]\nAddress=a\nPrivateKey=k\n[Peer]\nPublicKey=p\nEndpoint=vpn.example:51820\nEndpoint=vpn.example:51821\nAllowedIPs=a",
        "[Interface]\nAddress=a\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=a\nAllowedIPs=b",
        "[Interface]\nAddress=a\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=a\nPersistentKeepalive=25\nPersistentKeepalive=30",
    ] {
        reject_wireguard(profile, ProfileError::DuplicateField);
    }
}

#[test]
fn wireguard_peer_flush_and_list_failures_are_covered_without_import_side_effects() {
    reject_wireguard(
        "[Interface]\nAddress=a\nPrivateKey=k\n[Peer]\nAllowedIPs=a\n[Peer]\nPublicKey=p\nAllowedIPs=a",
        ProfileError::MissingField,
    );
    reject_wireguard(
        "[Interface]\nAddress=a\nPrivateKey=k\n[Peer]\nPublicKey=p\n[Peer]\nPublicKey=q\nAllowedIPs=a",
        ProfileError::MissingField,
    );
    reject_wireguard(
        "[Interface]\nAddress=a\nDNS=1.1.1.1,,8.8.8.8\nPrivateKey=k",
        ProfileError::InvalidValue,
    );
    reject_wireguard(
        "[Interface]\nAddress=a\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=a,,b",
        ProfileError::InvalidValue,
    );
}

#[test]
fn wireguard_comments_blank_lines_and_optional_peer_fields_remain_parseable() {
    let profile = "# synthetic fixture\n\n[Interface]\nAddress=a\nPrivateKey=k\n\n# peer\n[Peer]\nPublicKey=p\nAllowedIPs=a\n";
    let mut importer = CountingImporter::default();
    let parsed = import_wireguard_profile(profile, &mut importer).expect("synthetic profile must parse");
    assert_eq!(parsed.addresses, vec!["a"]);
    assert!(parsed.dns_servers.is_empty());
    assert_eq!(parsed.mtu, None);
    assert_eq!(parsed.listen_port, None);
    assert_eq!(parsed.peers.len(), 1);
    assert_eq!(importer.calls, 1);
}

#[test]
fn ikev2_duplicate_singletons_fail_before_external_secret_import() {
    for profile in [
        "[IKEv2]\nServer=s\nRemoteId=r\nRemoteId=q\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nLocalId=l\nLocalId=q\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nAuth=psk\nAuth=eap\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nUsername=v\nPassword=p\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nPsk=q\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nPassword=p\nPassword=q\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a\nTrafficSelectors=b",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a\nFragmentation=true\nFragmentation=false",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a\nRekeySeconds=3600\nRekeySeconds=7200",
    ] {
        reject_ikev2(profile, ProfileError::DuplicateField);
    }
}

#[test]
fn ikev2_missing_required_fields_and_list_failures_are_covered() {
    for profile in [
        "[IKEv2]\nServer=s\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nTrafficSelectors=a",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384",
    ] {
        reject_ikev2(profile, ProfileError::MissingField);
    }
    reject_ikev2(
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a,,b",
        ProfileError::InvalidValue,
    );
}

#[test]
fn ikev2_section_and_authentication_conflicts_take_both_fail_closed_paths() {
    reject_ikev2(
        "[IKEv2]\n[Other]\nServer=s",
        ProfileError::MalformedLine,
    );
    reject_ikev2(
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nPassword=p\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        ProfileError::InvalidValue,
    );
    reject_ikev2(
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nPassword=p\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=a",
        ProfileError::InvalidValue,
    );
}
