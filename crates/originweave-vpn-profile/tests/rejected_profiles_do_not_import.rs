use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnProfile, VpnSecret, VpnSecretImporter,
    import_wireguard_profile, parse_ikev2_profile, parse_vpn_profile,
};

#[derive(Default)]
struct RecordingImporter {
    calls: usize,
    fail_on_call: Option<usize>,
}

impl RecordingImporter {
    fn failing_on(call: usize) -> Self {
        Self {
            calls: 0,
            fail_on_call: Some(call),
        }
    }
}

impl VpnSecretImporter for RecordingImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.calls += 1;
        if self.fail_on_call == Some(self.calls) {
            return Err(ProfileError::InvalidSecret);
        }
        SecretReference::new(format!("secret://test/{}", self.calls))
    }
}

fn valid_wireguard_profile() -> &'static str {
    "[Interface]\nAddress=10.0.0.2/32,fd00::2/128\nDNS=1.1.1.1\nMTU=1420\nListenPort=51820\nPrivateKey=raw-private\n[Peer]\nPublicKey=public-one\nPresharedKey=raw-psk\nEndpoint=vpn.example:51820\nAllowedIPs=0.0.0.0/0,::/0\nPersistentKeepalive=25\n"
}

fn valid_ikev2_psk_profile() -> &'static str {
    "[IKEv2]\nServer=vpn.example\nRemoteId=vpn.example\nLocalId=client.example\nAuth=psk\nPsk=raw-psk\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8,fd00::/8\nMobike=true\nFragmentation=false\nDpdSeconds=30\nRekeySeconds=3600\n"
}

fn valid_ikev2_eap_profile() -> &'static str {
    "[IKEv2]\nServer=vpn.example\nAuth=eap\nUsername=alice\nPassword=raw-password\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=0.0.0.0/0\n"
}

#[test]
fn rejected_wireguard_profile_does_not_import_private_key() {
    let mut importer = RecordingImporter::default();
    let profile = "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=raw-private\nPostUp=forbidden\n";

    assert_eq!(
        import_wireguard_profile(profile, &mut importer),
        Err(ProfileError::UnsupportedAuthority)
    );
    assert_eq!(importer.calls, 0);
}

#[test]
fn rejected_ikev2_profile_does_not_import_preshared_key() {
    let mut importer = RecordingImporter::default();
    let profile = "[IKEv2]\nServer=vpn.example\nAuth=psk\nPsk=raw-psk\nProposal=des-md5-modp768\nTrafficSelectors=10.0.0.0/8\n";

    assert_eq!(
        parse_ikev2_profile(profile, &mut importer),
        Err(ProfileError::InvalidValue)
    );
    assert_eq!(importer.calls, 0);
}

#[test]
fn valid_wireguard_profile_imports_only_after_complete_validation() {
    let mut importer = RecordingImporter::default();
    let result = import_wireguard_profile(valid_wireguard_profile(), &mut importer);

    assert!(matches!(
        result,
        Ok(profile)
            if profile.addresses.len() == 2
                && profile.addresses.first().is_some_and(|value| value == "10.0.0.2/32")
                && profile.addresses.get(1).is_some_and(|value| value == "fd00::2/128")
                && profile.dns_servers.len() == 1
                && profile.dns_servers.first().is_some_and(|value| value == "1.1.1.1")
                && profile.mtu == Some(1420)
                && profile.listen_port == Some(51820)
                && profile.peers.len() == 1
                && profile.peers.first().is_some_and(|peer| peer.persistent_keepalive_seconds == Some(25))
    ));
    assert_eq!(importer.calls, 2);
}

#[test]
fn wireguard_external_import_failures_remain_typed_after_validation() {
    let mut first = RecordingImporter::failing_on(1);
    assert_eq!(
        import_wireguard_profile(valid_wireguard_profile(), &mut first),
        Err(ProfileError::SecretImportFailed)
    );
    assert_eq!(first.calls, 1);

    let mut second = RecordingImporter::failing_on(2);
    assert_eq!(
        import_wireguard_profile(valid_wireguard_profile(), &mut second),
        Err(ProfileError::SecretImportFailed)
    );
    assert_eq!(second.calls, 2);
}

#[test]
fn wireguard_structure_and_bounds_fail_before_external_import() {
    for profile in [
        "# comment only",
        "PrivateKey=k",
        "[Peer]\nPublicKey=p\nAllowedIPs=10.0.0.0/8",
        "[Interface]\n[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k",
        "[Interface]\nbroken",
        "[Interface]\nAddress=\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\nUnknown=x",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=10.0.0.0/8\nUnknown=x",
    ] {
        let mut importer = RecordingImporter::default();
        assert!(import_wireguard_profile(profile, &mut importer).is_err());
        assert_eq!(importer.calls, 0);
    }

    let mut importer = RecordingImporter::default();
    assert_eq!(
        import_wireguard_profile("", &mut importer),
        Err(ProfileError::UnsupportedProfile)
    );
    assert_eq!(importer.calls, 0);

    let oversized = "x".repeat(65_537);
    assert_eq!(
        import_wireguard_profile(&oversized, &mut importer),
        Err(ProfileError::ProfileTooLarge)
    );
    assert_eq!(importer.calls, 0);
}

#[test]
fn wireguard_duplicate_and_numeric_variants_cover_each_storage_type() {
    for profile in [
        "[Interface]\nAddress=10.0.0.2/32\nAddress=10.0.0.3/32\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\nPrivateKey=q",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\nMTU=1400\nMTU=1500",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\nListenPort=nope",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=10.0.0.0/8\nPersistentKeepalive=nope",
        "[Interface]\nAddress=10.0.0.2/32,,10.0.0.3/32\nPrivateKey=k",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nPublicKey=q\nAllowedIPs=10.0.0.0/8",
    ] {
        let mut importer = RecordingImporter::default();
        assert!(import_wireguard_profile(profile, &mut importer).is_err());
        assert_eq!(importer.calls, 0);
    }
}

#[test]
fn wireguard_exercises_both_peer_limit_rejection_boundaries() {
    for peer_count in [65usize, 66usize] {
        let mut profile = String::from("[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n");
        for _ in 0..peer_count {
            profile.push_str("[Peer]\nPublicKey=p\nAllowedIPs=10.0.0.0/8\n");
        }
        let mut importer = RecordingImporter::default();
        assert_eq!(
            import_wireguard_profile(&profile, &mut importer),
            Err(ProfileError::TooManyItems)
        );
        assert_eq!(importer.calls, 0);
    }
}

#[test]
fn wireguard_rejects_list_and_secret_resource_overflow_before_external_import() {
    let list = std::iter::repeat_n("10.0.0.2/32", 257)
        .collect::<Vec<_>>()
        .join(",");
    let profile = format!("[Interface]\nAddress={list}\nPrivateKey=k");
    let mut importer = RecordingImporter::default();
    assert_eq!(
        import_wireguard_profile(&profile, &mut importer),
        Err(ProfileError::TooManyItems)
    );
    assert_eq!(importer.calls, 0);

    let secret = "x".repeat(4_097);
    let profile = format!("[Interface]\nAddress=10.0.0.2/32\nPrivateKey={secret}");
    assert_eq!(
        import_wireguard_profile(&profile, &mut importer),
        Err(ProfileError::InvalidSecret)
    );
    assert_eq!(importer.calls, 0);
}

#[test]
fn valid_ikev2_profiles_import_only_after_complete_validation() {
    let mut psk_importer = RecordingImporter::default();
    assert!(matches!(
        parse_ikev2_profile(valid_ikev2_psk_profile(), &mut psk_importer),
        Ok(profile)
            if profile.dpd_seconds == 30
                && profile.rekey_seconds == 3600
                && profile.mobike
                && !profile.fragmentation
    ));
    assert_eq!(psk_importer.calls, 1);

    let mut eap_importer = RecordingImporter::default();
    assert!(matches!(
        parse_ikev2_profile(valid_ikev2_eap_profile(), &mut eap_importer),
        Ok(profile) if profile.mobike && profile.fragmentation
    ));
    assert_eq!(eap_importer.calls, 1);
}

#[test]
fn ikev2_external_import_failures_are_typed_after_validation() {
    let mut psk = RecordingImporter::failing_on(1);
    assert_eq!(
        parse_ikev2_profile(valid_ikev2_psk_profile(), &mut psk),
        Err(ProfileError::SecretImportFailed)
    );
    assert_eq!(psk.calls, 1);

    let mut password = RecordingImporter::failing_on(1);
    assert_eq!(
        parse_ikev2_profile(valid_ikev2_eap_profile(), &mut password),
        Err(ProfileError::SecretImportFailed)
    );
    assert_eq!(password.calls, 1);
}

#[test]
fn ikev2_structure_authority_and_duplicate_variants_fail_before_import() {
    for profile in [
        "# comment only",
        "Server=s",
        "[Other]\nServer=s",
        "[IKEv2]\n[Other]",
        "[IKEv2]\n[IKEv2]",
        "[IKEv2]\nbroken",
        "[IKEv2]\nServer=",
        "[IKEv2]\nServer=s\nServer=t\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nMobike=true\nMobike=false",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nDpdSeconds=30\nDpdSeconds=31",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nExec=x",
    ] {
        let mut importer = RecordingImporter::default();
        assert!(parse_ikev2_profile(profile, &mut importer).is_err());
        assert_eq!(importer.calls, 0);
    }
}

#[test]
fn ikev2_authentication_conflicts_and_timer_short_circuit_paths_fail_closed() {
    for profile in [
        "[IKEv2]\nServer=s\nAuth=unknown\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nUsername=u\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nPassword=p\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nPassword=p\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=des-md5-modp768\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nMobike=yes",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nDpdSeconds=0",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nRekeySeconds=299",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nDpdSeconds=400\nRekeySeconds=400",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nDpdSeconds=nope",
    ] {
        let mut importer = RecordingImporter::default();
        assert!(parse_ikev2_profile(profile, &mut importer).is_err());
        assert_eq!(importer.calls, 0);
    }
}

#[test]
fn top_level_detection_dispatches_without_ambient_import_authority() {
    let mut wg = RecordingImporter::default();
    assert!(matches!(
        parse_vpn_profile(valid_wireguard_profile(), &mut wg),
        Ok(VpnProfile::WireGuard(_))
    ));
    assert_eq!(wg.calls, 2);

    let mut ike = RecordingImporter::default();
    assert!(matches!(
        parse_vpn_profile(valid_ikev2_psk_profile(), &mut ike),
        Ok(VpnProfile::Ikev2(_))
    ));
    assert_eq!(ike.calls, 1);

    for profile in ["[Other]\nA=B", "", "# comment only"] {
        let mut importer = RecordingImporter::default();
        assert_eq!(
            parse_vpn_profile(profile, &mut importer),
            Err(ProfileError::UnsupportedProfile)
        );
        assert_eq!(importer.calls, 0);
    }

    let mut importer = RecordingImporter::default();
    assert_eq!(
        parse_vpn_profile(&"x".repeat(65_537), &mut importer),
        Err(ProfileError::ProfileTooLarge)
    );
    assert_eq!(importer.calls, 0);
}

#[test]
fn string_owned_secret_references_cover_valid_and_invalid_bounds() {
    assert_eq!(
        SecretReference::new(String::new()),
        Err(ProfileError::InvalidSecretReference)
    );
    assert_eq!(
        SecretReference::new("x".repeat(513)),
        Err(ProfileError::InvalidSecretReference)
    );
    assert!(matches!(
        SecretReference::new("secret://integration".to_owned()),
        Ok(reference) if reference.as_str() == "secret://integration"
    ));
}
