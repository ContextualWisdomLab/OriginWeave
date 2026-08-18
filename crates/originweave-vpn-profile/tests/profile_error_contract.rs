use originweave_vpn_profile::{
    ProfileError, SecretReference, MAX_SECRET_REFERENCE_BYTES,
};

fn assert_standard_error<T: std::error::Error + Send + Sync + 'static>() {}

#[test]
fn profile_error_is_a_standard_error_with_stable_nonsecret_display() {
    assert_standard_error::<ProfileError>();

    let cases = [
        (
            ProfileError::ProfileTooLarge,
            "profile exceeds the bounded size",
        ),
        (
            ProfileError::UnsupportedProfile,
            "profile format is unsupported",
        ),
        (
            ProfileError::MalformedLine,
            "profile contains a malformed line",
        ),
        (
            ProfileError::UnsupportedAuthority,
            "profile requests unsupported authority",
        ),
        (ProfileError::DuplicateField, "profile field is duplicated"),
        (
            ProfileError::MissingField,
            "profile is missing a required field",
        ),
        (ProfileError::InvalidValue, "profile field value is invalid"),
        (
            ProfileError::TooManyItems,
            "profile contains too many items",
        ),
        (ProfileError::InvalidSecret, "profile secret is invalid"),
        (
            ProfileError::InvalidSecretReference,
            "secret reference is invalid",
        ),
        (ProfileError::SecretImportFailed, "secret import failed"),
        (ProfileError::SecretCleanupFailed, "secret cleanup failed"),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn secret_reference_rejects_log_injection_and_ambiguous_whitespace() {
    for reference in [
        "",
        "secret://tenant/item\nforged-log-line",
        "secret://tenant/item\tforged-column",
        "secret://tenant/item\u{1b}forged-terminal-sequence",
        "\u{00ad}secret://tenant/item",
        "\u{061c}secret://tenant/item",
        "\u{200b}secret://tenant/item",
        "\u{200f}secret://tenant/item",
        "\u{202a}secret://tenant/item",
        "\u{202e}secret://tenant/item",
        "\u{2060}secret://tenant/item",
        "\u{206f}secret://tenant/item",
        "\u{feff}secret://tenant/item",
        " secret://tenant/item",
        "secret://tenant/item ",
        "secret://tenant/item id",
    ] {
        assert_eq!(
            SecretReference::new(reference),
            Err(ProfileError::InvalidSecretReference)
        );
    }

    let oversized_reference = "x".repeat(MAX_SECRET_REFERENCE_BYTES + 1);
    assert_eq!(
        SecretReference::new(oversized_reference.as_str()),
        Err(ProfileError::InvalidSecretReference)
    );

    for reference in [
        String::new(),
        "x".repeat(MAX_SECRET_REFERENCE_BYTES + 1),
        "secret://tenant/owned item".to_owned(),
        "secret://tenant/item\u{1b}owned-string".to_owned(),
        "\u{00ad}secret://tenant/owned-string".to_owned(),
    ] {
        assert_eq!(
            SecretReference::new(reference),
            Err(ProfileError::InvalidSecretReference)
        );
    }

    for reference in ["secret://tenant/item", "secret://tenant/키"] {
        assert_eq!(
            SecretReference::new(reference).map(|value| value.as_str().to_owned()),
            Ok(reference.to_owned())
        );
    }

    let owned_reference = "secret://tenant/owned-item".to_owned();
    assert_eq!(
        SecretReference::new(owned_reference.clone()).map(|value| value.as_str().to_owned()),
        Ok(owned_reference)
    );
}
