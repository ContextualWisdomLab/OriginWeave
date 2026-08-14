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
        SecretReference::new(format!("secret://coverage/{}", self.0))
    }
}

fn canonical_wireguard(profile: &str) -> String {
    profile.replace("<wg-key>", VALID_WIREGUARD_KEY)
}

fn assert_wg_error(profile: &str, expected: ProfileError) {
    let mut importer = CountingImporter::default();
    assert_eq!(
        import_wireguard_profile(profile, &mut importer),
        Err(expected)
    );
    assert_eq!(importer.0, 0);
}

fn assert_ike_error(profile: &str, expected: ProfileError) {
    let mut importer = CountingImporter::default();
    assert_eq!(parse_ikev2_profile(profile, &mut importer), Err(expected));
    assert_eq!(importer.0, 0);
}

#[test]
fn semantic_network_errors_cover_every_ip_and_prefix_boundary_before_import() {
    for address in [
        "10.0.0.2",
        "not-an-ip/24",
        "10.0.0.2/not-a-prefix",
        "10.0.0.2/33",
        "fd00::2/129",
    ] {
        assert_wg_error(
            &format!("[Interface]\nAddress={address}\nPrivateKey=k\n"),
            ProfileError::InvalidValue,
        );
    }

    assert_wg_error(
        "[Interface]\nAddress=10.0.0.2/32\nDNS=not-an-ip\nPrivateKey=k\n",
        ProfileError::InvalidValue,
    );
    assert_wg_error(
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=10.0.0.0/not-a-prefix\n",
        ProfileError::InvalidValue,
    );
    assert_ike_error(
        "[IKEv2]\nServer=vpn.example\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=not-an-ip/8\n",
        ProfileError::InvalidValue,
    );
}

#[test]
fn wireguard_duplicate_variants_cover_every_singleton_storage_type() {
    for profile in [
        "[Interface]\nAddress=10.0.0.2/32\nDNS=1.1.1.1\nDNS=8.8.8.8\nPrivateKey=<wg-key>\n",
        "[Interface]\nAddress=10.0.0.2/32\nMTU=1400\nMTU=1500\nPrivateKey=<wg-key>\n",
        "[Interface]\nAddress=10.0.0.2/32\nListenPort=51820\nListenPort=51821\nPrivateKey=<wg-key>\n",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\nPrivateKey=<wg-key>\n",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8\n",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nPresharedKey=<wg-key>\nPresharedKey=<wg-key>\nAllowedIPs=10.0.0.0/8\n",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nEndpoint=vpn.example:51820\nEndpoint=vpn2.example:51820\nAllowedIPs=10.0.0.0/8\n",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8\nAllowedIPs=192.0.2.0/24\n",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8\nPersistentKeepalive=10\nPersistentKeepalive=20\n",
    ] {
        assert_wg_error(&canonical_wireguard(profile), ProfileError::DuplicateField);
    }
}

#[test]
fn wireguard_peer_flush_and_required_field_edges_are_covered() {
    for profile in [
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nAllowedIPs=10.0.0.0/8\n[Peer]\nPublicKey=q\nAllowedIPs=192.0.2.0/24\n",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\n[Peer]\nPublicKey=q\nAllowedIPs=192.0.2.0/24\n",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nAllowedIPs=10.0.0.0/8\n",
        "[Interface]\nPrivateKey=k\n",
        "[Interface]\nAddress=10.0.0.2/32\n",
        "[Interface]\n=x\nAddress=10.0.0.2/32\nPrivateKey=k\n",
    ] {
        let mut importer = CountingImporter::default();
        assert!(import_wireguard_profile(profile, &mut importer).is_err());
        assert_eq!(importer.0, 0);
    }
}

#[test]
fn wireguard_post_key_validation_errors_remain_fail_closed() {
    for (profile, expected) in [
        (
            canonical_wireguard(
                "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8\nUnknown=x\n",
            ),
            ProfileError::UnsupportedAuthority,
        ),
        (
            canonical_wireguard(
                "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nAllowedIPs=10.0.0.0/8\n",
            ),
            ProfileError::MissingField,
        ),
        (
            canonical_wireguard("[Interface]\nPrivateKey=<wg-key>\n"),
            ProfileError::MissingField,
        ),
    ] {
        assert_wg_error(&profile, expected);
    }
}

#[test]
fn wireguard_scalar_syntax_errors_fail_before_import() {
    for profile in [
        "[Interface]\nAddress=10.0.0.2/32\nMTU=not-a-number\nPrivateKey=k\n".to_owned(),
        "[Interface]\nAddress=10.0.0.2/32\nListenPort=not-a-number\nPrivateKey=k\n".to_owned(),
        canonical_wireguard(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPersistentKeepalive=not-a-number\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8\n",
        ),
    ] {
        assert_wg_error(&profile, ProfileError::InvalidValue);
    }
}

#[test]
fn wireguard_forbidden_interface_authority_variants_are_exhaustive() {
    for key in ["PreDown", "PostDown", "SaveConfig", "Table", "Unknown"] {
        let profile = format!(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n{key}=x\n"
        );
        assert_wg_error(&profile, ProfileError::UnsupportedAuthority);
    }
}

#[test]
fn ikev2_duplicate_variants_cover_every_singleton_storage_type() {
    let suffix =
        "Auth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\n";
    for prefix in [
        "Server=a\nServer=b\n",
        "Server=a\nRemoteId=a\nRemoteId=b\n",
        "Server=a\nLocalId=a\nLocalId=b\n",
        "Server=a\nAuth=psk\nAuth=psk\n",
    ] {
        assert_ike_error(
            &format!("[IKEv2]\n{prefix}{suffix}"),
            ProfileError::DuplicateField,
        );
    }

    for profile in [
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nUsername=v\nPassword=p\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=a\nPsk=b\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nPassword=a\nPassword=b\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nTrafficSelectors=192.0.2.0/24\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nMobike=true\nMobike=false\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nFragmentation=true\nFragmentation=false\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nDpdSeconds=30\nDpdSeconds=31\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nRekeySeconds=3600\nRekeySeconds=7200\n",
    ] {
        assert_ike_error(profile, ProfileError::DuplicateField);
    }
}

#[test]
fn ikev2_missing_and_cross_field_edges_fail_before_import() {
    for profile in [
        "[IKEv2]\nServer=s\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=psk\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=eap\nPassword=p\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nTrafficSelectors=10.0.0.0/8\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\n",
    ] {
        let mut importer = CountingImporter::default();
        assert!(parse_ikev2_profile(profile, &mut importer).is_err());
        assert_eq!(importer.0, 0);
    }
}

#[test]
fn ikev2_field_specific_scalar_syntax_errors_fail_before_import() {
    for profile in [
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nFragmentation=yes\n",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nRekeySeconds=nope\n",
    ] {
        assert_ike_error(profile, ProfileError::InvalidValue);
    }
}
