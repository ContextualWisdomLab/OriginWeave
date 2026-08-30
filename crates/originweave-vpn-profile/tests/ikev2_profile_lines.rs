use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, parse_ikev2_profile,
};

#[derive(Default)]
struct CountingImporter {
    calls: usize,
}

impl VpnSecretImporter for CountingImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.calls += 1;
        SecretReference::new(format!("secret://ikev2-lines/{}", self.calls))
    }
}

#[test]
fn comments_and_blank_lines_do_not_change_ikev2_secret_import() {
    let profile = "# managed profile\n\n[IKEv2]\n\nServer=s\n# authentication\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\n";
    let mut importer = CountingImporter::default();
    let parsed = parse_ikev2_profile(profile, &mut importer);
    assert!(parsed.is_ok(), "commented IKEv2 profile must parse: {parsed:?}");
    assert_eq!(importer.calls, 1);
}
