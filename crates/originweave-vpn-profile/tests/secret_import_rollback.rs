use originweave_vpn_profile::{
    ProfileError, SecretReference, VpnSecret, VpnSecretImporter, import_wireguard_profile,
    parse_ikev2_profile,
};

const VALID_WIREGUARD_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

#[derive(Default)]
struct TransactionalImporter {
    import_calls: usize,
    fail_on_import: Option<usize>,
    fail_cleanup: bool,
    stored: Vec<String>,
    discard_calls: Vec<String>,
}

impl TransactionalImporter {
    fn failing_on_import(call: usize) -> Self {
        Self {
            fail_on_import: Some(call),
            ..Self::default()
        }
    }

    fn with_cleanup_failure(mut self) -> Self {
        self.fail_cleanup = true;
        self
    }
}

impl VpnSecretImporter for TransactionalImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        self.import_calls += 1;
        if self.fail_on_import == Some(self.import_calls) {
            return Err(ProfileError::InvalidSecret);
        }
        let reference = format!("secret://rollback/{}", self.import_calls);
        self.stored.push(reference.clone());
        SecretReference::new(reference)
    }

    fn discard_secret(&mut self, reference: &SecretReference) -> Result<(), ProfileError> {
        self.discard_calls.push(reference.as_str().to_owned());
        if self.fail_cleanup {
            return Err(ProfileError::SecretImportFailed);
        }
        self.stored.retain(|stored| stored != reference.as_str());
        Ok(())
    }
}

fn three_secret_wireguard_profile() -> String {
    format!(
        "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n\
         [Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nPresharedKey={VALID_WIREGUARD_KEY}\nAllowedIPs=10.1.0.0/16\n\
         [Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nPresharedKey={VALID_WIREGUARD_KEY}\nAllowedIPs=10.2.0.0/16\n"
    )
}

#[test]
fn later_import_failure_rolls_back_every_successful_secret_in_reverse_order() {
    let mut importer = TransactionalImporter::failing_on_import(3);

    assert_eq!(
        import_wireguard_profile(&three_secret_wireguard_profile(), &mut importer),
        Err(ProfileError::SecretImportFailed)
    );
    assert_eq!(importer.import_calls, 3);
    assert_eq!(
        importer.discard_calls,
        vec![
            "secret://rollback/2".to_owned(),
            "secret://rollback/1".to_owned()
        ]
    );
    assert!(importer.stored.is_empty());
}

#[test]
fn cleanup_failure_is_typed_and_does_not_stop_remaining_cleanup_attempts() {
    let mut importer = TransactionalImporter::failing_on_import(3).with_cleanup_failure();

    assert_eq!(
        import_wireguard_profile(&three_secret_wireguard_profile(), &mut importer),
        Err(ProfileError::SecretCleanupFailed)
    );
    assert_eq!(importer.import_calls, 3);
    assert_eq!(
        importer.discard_calls,
        vec![
            "secret://rollback/2".to_owned(),
            "secret://rollback/1".to_owned()
        ]
    );
    assert_eq!(importer.stored.len(), 2);
}

#[test]
fn first_import_failure_needs_no_cleanup() {
    let mut importer = TransactionalImporter::failing_on_import(1);

    assert_eq!(
        import_wireguard_profile(&three_secret_wireguard_profile(), &mut importer),
        Err(ProfileError::SecretImportFailed)
    );
    assert_eq!(importer.import_calls, 1);
    assert!(importer.discard_calls.is_empty());
    assert!(importer.stored.is_empty());
}

#[test]
fn ikev2_first_import_failure_needs_no_cleanup() {
    let mut importer = TransactionalImporter::failing_on_import(1);
    let profile = "[IKEv2]\nServer=vpn.example\nAuth=psk\nPsk=secret\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8";

    assert_eq!(
        parse_ikev2_profile(profile, &mut importer),
        Err(ProfileError::SecretImportFailed)
    );
    assert_eq!(importer.import_calls, 1);
    assert!(importer.discard_calls.is_empty());
    assert!(importer.stored.is_empty());
}

#[test]
fn ikev2_success_imports_exactly_one_secret_per_authentication_variant() {
    for profile in [
        "[IKEv2]\nServer=vpn.example\nAuth=psk\nPsk=secret\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
        "[IKEv2]\nServer=vpn.example\nAuth=eap\nUsername=alice\nPassword=raw-password\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=0.0.0.0/0",
    ] {
        let mut importer = TransactionalImporter::default();

        assert!(parse_ikev2_profile(profile, &mut importer).is_ok());
        assert_eq!(importer.import_calls, 1);
        assert_eq!(importer.stored.len(), 1);
        assert!(importer.discard_calls.is_empty());
    }
}
