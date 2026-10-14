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
        SecretReference::new(format!("secret://wireguard-address/{}", self.calls))
    }
}

#[test]
fn wireguard_interface_addresses_accept_standard_host_shorthand() -> Result<(), ProfileError> {
    let profile = "[Interface]\nAddress=10.0.0.2,fd00::2\nPrivateKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n";
    let mut importer = RecordingImporter::default();

    let normalized = import_wireguard_profile(profile, &mut importer)?;

    assert_eq!(
        normalized.addresses,
        vec!["10.0.0.2/32".to_owned(), "fd00::2/128".to_owned()]
    );
    assert_eq!(importer.calls, 1);
    Ok(())
}
