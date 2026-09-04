use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
};

const VALID_WIREGUARD_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

#[derive(Default)]
struct RecordingImporter {
    calls: usize,
}

impl VpnSecretImporter for RecordingImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.calls += 1;
        SecretReference::new(format!("secret://wireguard-repeat/{}", self.calls))
    }
}

#[test]
fn wireguard_interface_accumulates_repeated_address_and_dns_fields() -> Result<(), ProfileError> {
    let profile = format!(
        "[Interface]\nAddress=10.0.0.2\nAddress=fd00::2/128\nDNS=1.1.1.1\nDNS=2606:4700:4700::1111\nPrivateKey={VALID_WIREGUARD_KEY}\n"
    );
    let mut importer = RecordingImporter::default();

    let normalized = import_wireguard_profile(&profile, &mut importer)?;

    assert_eq!(
        normalized.addresses,
        vec!["10.0.0.2/32".to_owned(), "fd00::2/128".to_owned()]
    );
    assert_eq!(
        normalized.dns_servers,
        vec!["1.1.1.1".to_owned(), "2606:4700:4700::1111".to_owned()]
    );
    assert_eq!(normalized.dns_search_domains, Vec::<String>::new());
    assert_eq!(importer.calls, 1);
    Ok(())
}

#[test]
fn wireguard_dns_preserves_search_domains_separately_from_dns_servers() -> Result<(), ProfileError>
{
    let profile = format!(
        "[Interface]\nAddress=10.0.0.2/32\nDNS=1.1.1.1,corp.example\nDNS=fd00::53,svc.corp.example\nPrivateKey={VALID_WIREGUARD_KEY}\n"
    );
    let mut importer = RecordingImporter::default();

    let normalized = import_wireguard_profile(&profile, &mut importer)?;

    assert_eq!(
        normalized.dns_servers,
        vec!["1.1.1.1".to_owned(), "fd00::53".to_owned()]
    );
    assert_eq!(
        normalized.dns_search_domains,
        vec!["corp.example".to_owned(), "svc.corp.example".to_owned()]
    );
    assert_eq!(importer.calls, 1);
    Ok(())
}

#[test]
fn wireguard_dns_rejects_ambiguous_numeric_ipv4_spellings_before_secret_import() {
    for dns_value in ["999.1.1.1", "192.168.1", "0177.0.0.1", "0x7f000001"] {
        let profile = format!(
            "[Interface]\nAddress=10.0.0.2/32\nDNS={dns_value}\nPrivateKey={VALID_WIREGUARD_KEY}\n"
        );
        let mut importer = RecordingImporter::default();
        assert_eq!(
            import_wireguard_profile(&profile, &mut importer),
            Err(ProfileError::InvalidValue),
            "ambiguous numeric DNS value was accepted: {dns_value:?}"
        );
        assert_eq!(
            importer.calls, 0,
            "ambiguous numeric DNS value reached caller importer"
        );
    }
}

#[test]
fn repeated_wireguard_addresses_share_the_global_list_bound_before_import() {
    let mut profile = String::from("[Interface]\n");
    for index in 0..257 {
        profile.push_str(&format!(
            "Address=10.{}.{}.{}\n",
            index / 65_536,
            (index / 256) % 256,
            index % 256
        ));
    }
    profile.push_str(&format!("PrivateKey={VALID_WIREGUARD_KEY}\n"));
    let mut importer = RecordingImporter::default();

    assert_eq!(
        import_wireguard_profile(&profile, &mut importer),
        Err(ProfileError::TooManyItems)
    );
    assert_eq!(importer.calls, 0);
}

#[test]
fn repeated_wireguard_dns_fields_share_the_global_list_bound_before_import() {
    let mut profile = String::from("[Interface]\n");
    for _ in 0..257 {
        profile.push_str("DNS=1.1.1.1\n");
    }
    profile.push_str(&format!("PrivateKey={VALID_WIREGUARD_KEY}\n"));
    let mut importer = RecordingImporter::default();

    assert_eq!(
        import_wireguard_profile(&profile, &mut importer),
        Err(ProfileError::TooManyItems)
    );
    assert_eq!(importer.calls, 0);
}
