use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnProfile, VpnSecret, VpnSecretImporter,
    import_wireguard_profile, parse_ikev2_profile, parse_vpn_profile,
};

const COMMENTED_WIREGUARD_PROFILE: &str = concat!(
    "[Interface] # exported interface\n",
    "Address=10.0.0.2 # task-local address\n",
    "DNS=1.1.1.1,corp.example # resolver intent\n",
    "PrivateKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= # synthetic key\n",
    "[Peer] # primary peer\n",
    "PublicKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= # synthetic peer key\n",
    "AllowedIPs=0.0.0.0/0 # route intent only\n",
    "Endpoint=vpn.example.com:51820 # destination intent only\n",
);

#[derive(Default)]
struct RecordingImporter {
    calls: usize,
    saw_ike_hash_secret: bool,
}

impl VpnSecretImporter for RecordingImporter {
    fn import_secret(&mut self, secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.calls += 1;
        if matches!(secret, VpnSecret::Ikev2PresharedKey("raw#psk")) {
            self.saw_ike_hash_secret = true;
        }
        SecretReference::new(format!("secret://wireguard-inline-comment/{}", self.calls))
    }
}

#[test]
fn wg_quick_inline_comments_are_ignored_before_wireguard_validation() -> Result<(), ProfileError> {
    let mut importer = RecordingImporter::default();

    let normalized = import_wireguard_profile(COMMENTED_WIREGUARD_PROFILE, &mut importer)?;

    assert_eq!(normalized.addresses, vec!["10.0.0.2/32".to_owned()]);
    assert_eq!(normalized.dns_servers, vec!["1.1.1.1".to_owned()]);
    assert_eq!(
        normalized.dns_search_domains,
        vec!["corp.example".to_owned()]
    );
    assert_eq!(normalized.peers.len(), 1);
    assert_eq!(
        normalized.peers[0].endpoint.as_deref(),
        Some("vpn.example.com:51820")
    );
    assert_eq!(
        normalized.peers[0].allowed_ips,
        vec!["0.0.0.0/0".to_owned()]
    );
    assert_eq!(importer.calls, 1);
    Ok(())
}

#[test]
fn generic_dispatch_recognizes_a_commented_wg_quick_section_header() -> Result<(), ProfileError> {
    let mut importer = RecordingImporter::default();

    let normalized = parse_vpn_profile(COMMENTED_WIREGUARD_PROFILE, &mut importer)?;

    assert!(matches!(normalized, VpnProfile::WireGuard(_)));
    assert_eq!(importer.calls, 1);
    Ok(())
}

#[test]
fn provider_neutral_ikev2_preserves_hash_characters_inside_secret_values()
-> Result<(), ProfileError> {
    let profile = concat!(
        "[IKEv2]\n",
        "Server=vpn.example\n",
        "Auth=psk\n",
        "Psk=raw#psk\n",
        "Proposal=aes256gcm16-prfsha384-ecp384\n",
        "TrafficSelectors=10.0.0.0/8\n",
    );
    let mut importer = RecordingImporter::default();

    parse_ikev2_profile(profile, &mut importer)?;

    assert_eq!(importer.calls, 1);
    assert!(importer.saw_ike_hash_secret);
    Ok(())
}
