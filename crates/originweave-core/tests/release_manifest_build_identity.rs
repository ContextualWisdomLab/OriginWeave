use std::error::Error;

use originweave_core::release_manifest::{ReleaseBuildIdentity, ReleaseBuildIdentityError};

fn sha256_digest(hex_digit: char) -> String {
    format!("sha256:{}", hex_digit.to_string().repeat(64))
}

#[test]
fn release_build_identity_binds_exact_toolchain_and_dependency_lock() -> Result<(), Box<dyn Error>>
{
    let identity = ReleaseBuildIdentity::new("1.97.1", &sha256_digest('a'))?;

    assert_eq!(identity.rust_toolchain(), "1.97.1");
    assert_eq!(identity.dependency_lock_sha256(), sha256_digest('a'));
    Ok(())
}

#[test]
fn release_build_identity_rejects_ambiguous_or_unbounded_evidence() {
    let digest = sha256_digest('b');
    let overlong_toolchain = "a".repeat(65);

    for invalid_toolchain in [
        "",
        "stable",
        "beta",
        "nightly",
        "1.98.0",
        " 1.97.1",
        "1.97.1 ",
        "rust 1.97.1",
        "1.97.1/nightly",
        "1.97.1-µ",
        overlong_toolchain.as_str(),
    ] {
        assert_eq!(
            ReleaseBuildIdentity::new(invalid_toolchain, &digest),
            Err(ReleaseBuildIdentityError::InvalidRustToolchain),
        );
    }

    for invalid_digest in [
        "",
        "sha256:abc",
        "SHA256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
    ] {
        assert_eq!(
            ReleaseBuildIdentity::new("1.97.1", invalid_digest),
            Err(ReleaseBuildIdentityError::InvalidDependencyLockDigest),
        );
    }
}

#[test]
fn release_build_identity_errors_are_standard_source_free_rust_errors() {
    let errors = [
        (
            ReleaseBuildIdentityError::InvalidRustToolchain,
            "release Rust toolchain must match the exact repository-pinned baseline",
        ),
        (
            ReleaseBuildIdentityError::InvalidDependencyLockDigest,
            "release dependency lock digest must be sha256: followed by 64 lowercase hexadecimal digits",
        ),
    ];

    for (error, expected) in errors {
        assert_eq!(error.to_string(), expected);
        assert!(Error::source(&error).is_none());
    }
}
