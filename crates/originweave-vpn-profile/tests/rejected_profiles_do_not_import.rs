use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
    parse_ikev2_profile,
};

#[derive(Default)]
struct RecordingImporter {
    calls: usize,
}

impl VpnSecretImporter for RecordingImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.calls += 1;
        SecretReference::new(format!("secret://test/{}", self.calls))
    }
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
