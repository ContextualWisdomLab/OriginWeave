use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
};

#[derive(Default)]
struct RecordingImporter {
    calls: usize,
}

impl VpnSecretImporter for RecordingImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.calls += 1;
        SecretReference::new(format!("secret://wireguard-inline-comment/{}", self.calls))
    }
}

#[test]
fn wg_quick_inline_comments_are_ignored_before_wireguard_validation() -> Result<(), ProfileError> {
    let profile = concat!(
        "[Interface] # exported interface\n",
        "Address=10.0.0.2 # task-local address\n",
        "DNS=1.1.1.1,corp.example # resolver intent\n",
        "PrivateKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= # synthetic key\n",
        "[Peer] # primary peer\n",
        "PublicKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= # synthetic peer key\n",
        "AllowedIPs=0.0.0.0/0 # route intent only\n",
        "Endpoint=vpn.example.com:51820 # destination intent only\n",
    );
    let mut importer = RecordingImporter::default();

    let normalized = import_wireguard_profile(profile, &mut importer)?;

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
