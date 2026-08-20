use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
};

const VALID_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

#[derive(Default)]
struct CountingImporter(usize);

impl VpnSecretImporter for CountingImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.0 += 1;
        SecretReference::new(format!("secret://key-shape/{}", self.0))
    }
}

fn profile(private_key: &str, public_key: &str, preshared_key: Option<&str>) -> String {
    let preshared = preshared_key
        .map(|value| format!("PresharedKey={value}\n"))
        .unwrap_or_default();
    format!(
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={private_key}\n[Peer]\nPublicKey={public_key}\n{preshared}AllowedIPs=10.0.0.0/8\n"
    )
}

fn assert_invalid_without_import(private_key: &str, public_key: &str, preshared_key: Option<&str>) {
    let mut importer = CountingImporter::default();
    assert_eq!(
        import_wireguard_profile(
            &profile(private_key, public_key, preshared_key),
            &mut importer,
        ),
        Err(ProfileError::InvalidValue)
    );
    assert_eq!(importer.0, 0, "invalid key reached the caller importer");
}

#[test]
fn wireguard_private_key_must_be_canonical_standard_base64_for_exactly_32_bytes() {
    for invalid in [
        "short",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA_=",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB=",
    ] {
        assert_invalid_without_import(invalid, VALID_KEY, None);
    }
}

#[test]
fn wireguard_public_key_must_be_canonical_standard_base64_for_exactly_32_bytes() {
    for invalid in [
        "short",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA+=",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB=",
    ] {
        assert_invalid_without_import(VALID_KEY, invalid, None);
    }
}

#[test]
fn wireguard_preshared_key_must_be_canonical_standard_base64_for_exactly_32_bytes() {
    for invalid in [
        "short",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/=",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB=",
    ] {
        assert_invalid_without_import(VALID_KEY, VALID_KEY, Some(invalid));
    }
}

#[test]
fn canonical_32_byte_wireguard_keys_remain_accepted() {
    let mut importer = CountingImporter::default();
    let parsed = import_wireguard_profile(
        &profile(VALID_KEY, VALID_KEY, Some(VALID_KEY)),
        &mut importer,
    );
    assert!(
        parsed.is_ok(),
        "canonical WireGuard keys must parse: {parsed:?}"
    );
    assert_eq!(importer.0, 2);
    if let Ok(profile) = parsed {
        assert_eq!(profile.peers[0].public_key, VALID_KEY);
    }
}

#[test]
fn canonical_keys_cover_standard_base64_alphabet_and_padding_endings() {
    let mut keys = vec![format!("+{}A=", "A".repeat(41))];
    keys.extend(['Q', 'g', 'w'].map(|last| format!("{}{}=", "A".repeat(42), last)));

    for key in keys {
        let mut importer = CountingImporter::default();
        let parsed = import_wireguard_profile(&profile(&key, &key, None), &mut importer);

        assert!(parsed.is_ok(), "canonical key must parse: {parsed:?}");
        assert_eq!(importer.0, 1);
    }

    let invalid_key = format!("!{}A=", "A".repeat(41));
    let mut importer = CountingImporter::default();
    assert_eq!(
        import_wireguard_profile(&profile(&invalid_key, &invalid_key, None), &mut importer),
        Err(ProfileError::InvalidValue)
    );
    assert_eq!(importer.0, 0);
}
