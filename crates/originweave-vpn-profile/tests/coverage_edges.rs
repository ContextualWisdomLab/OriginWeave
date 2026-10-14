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
        SecretReference::new(format!("secret://coverage/{}", self.calls))
    }
}

fn canonical_wireguard(profile: &str) -> String {
    profile.replace("<wg-key>", VALID_WIREGUARD_KEY)
}

fn reject_wireguard(profile: &str, expected: ProfileError) {
    let mut importer = CountingImporter::default();
    assert_eq!(
        import_wireguard_profile(profile, &mut importer),
        Err(expected)
    );
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
        "[Interface]\nAddress=10.0.0.2/32\nMTU=1400\nMTU=1500\nPrivateKey=<wg-key>",
        "[Interface]\nAddress=10.0.0.2/32\nListenPort=51820\nListenPort=51821\nPrivateKey=<wg-key>",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\nPrivateKey=<wg-key>",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nPresharedKey=<wg-key>\nPresharedKey=<wg-key>\nAllowedIPs=10.0.0.0/8",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nEndpoint=vpn.example:51820\nEndpoint=vpn.example:51821\nAllowedIPs=10.0.0.0/8",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8\nAllowedIPs=192.168.0.0/16",
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8\nPersistentKeepalive=25\nPersistentKeepalive=30",
    ] {
        reject_wireguard(&canonical_wireguard(profile), ProfileError::DuplicateField);
    }
}

#[test]
fn wireguard_peer_flush_and_list_failures_are_covered_without_import_side_effects() {
    reject_wireguard(
        &canonical_wireguard(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nAllowedIPs=10.0.0.0/8\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8",
        ),
        ProfileError::MissingField,
    );
    reject_wireguard(
        &canonical_wireguard(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8",
        ),
        ProfileError::MissingField,
    );
    reject_wireguard(
        &canonical_wireguard(
            "[Interface]\nAddress=10.0.0.2/32\nDNS=1.1.1.1,,8.8.8.8\nPrivateKey=<wg-key>",
        ),
        ProfileError::InvalidValue,
    );
    reject_wireguard(
        &canonical_wireguard(
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8,,192.168.0.0/16",
        ),
        ProfileError::InvalidValue,
    );
    reject_wireguard(
        &canonical_wireguard("[Interface]\n=x\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>"),
        ProfileError::InvalidValue,
    );
}

#[test]
fn wireguard_comments_blank_lines_and_optional_peer_fields_remain_parseable() {
    let profile = canonical_wireguard(
        "# synthetic fixture\n\n[Interface]\nAddress=10.0.0.2/32\nPrivateKey=<wg-key>\n\n# peer\n[Peer]\nPublicKey=<wg-key>\nAllowedIPs=10.0.0.0/8\n",
    );
    let mut importer = CountingImporter::default();
    let parsed = import_wireguard_profile(&profile, &mut importer);
    assert!(parsed.is_ok(), "synthetic profile must parse: {parsed:?}");
    assert_eq!(importer.calls, 1);
}

#[test]
fn ikev2_duplicate_singletons_fail_before_external_secret_import() {
    for profile in [
        "[IKEv2]\nServer=s\nRemoteId=r\nRemoteId=q\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nLocalId=l\nLocalId=q\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nAuth=eap\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nUsername=v\nPassword=p\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nPsk=q\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nPassword=p\nPassword=q\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nTrafficSelectors=192.168.0.0/16",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nFragmentation=true\nFragmentation=false",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nRekeySeconds=3600\nRekeySeconds=7200",
    ] {
        reject_ikev2(profile, ProfileError::DuplicateField);
    }
}

#[test]
fn ikev2_missing_required_fields_and_list_failures_are_covered() {
    for profile in [
        "[IKEv2]\nServer=s\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384",
    ] {
        reject_ikev2(profile, ProfileError::MissingField);
    }
    reject_ikev2(
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8,,192.168.0.0/16",
        ProfileError::InvalidValue,
    );
    reject_ikev2(&"x".repeat(65_537), ProfileError::ProfileTooLarge);
}

#[test]
fn ikev2_section_and_authentication_conflicts_take_both_fail_closed_paths() {
    reject_ikev2("[IKEv2]\n[Other]\nServer=s", ProfileError::MalformedLine);
    reject_ikev2(
        "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nPassword=p\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        ProfileError::InvalidValue,
    );
    reject_ikev2(
        "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nPassword=p\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        ProfileError::InvalidValue,
    );
}
