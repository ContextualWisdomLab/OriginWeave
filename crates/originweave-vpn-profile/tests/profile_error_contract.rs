use originweave_vpn_profile::ProfileError;

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
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(std::error::Error::source(&error).is_none());
    }
}
