use std::error::Error;

use originweave_core::release_manifest::{
    MAX_RELEASE_ARTIFACT_NAME_BYTES, MAX_RELEASE_ARTIFACTS, MAX_RELEASE_REVISION_BYTES,
    ReleaseArtifact, ReleaseArtifactError, ReleaseChannel, ReleaseManifest, ReleaseManifestError,
};

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const CHROMIUM_REVISION: &str = "150.0.7871.129";

fn sha256_digest(hex_digit: char) -> String {
    format!("sha256:{}", hex_digit.to_string().repeat(64))
}

#[test]
fn release_manifest_binds_and_canonicalizes_exact_artifact_evidence() -> Result<(), Box<dyn Error>>
{
    let runtime = ReleaseArtifact::new("originweave-linux-x86_64.tar.zst", &sha256_digest('a'))?;
    let sbom = ReleaseArtifact::new("originweave.spdx.json", &sha256_digest('b'))?;

    let manifest = ReleaseManifest::new(
        SOURCE_COMMIT,
        CHROMIUM_REVISION,
        ReleaseChannel::Stable,
        [sbom, runtime],
    )?;

    assert_eq!(manifest.source_commit(), SOURCE_COMMIT);
    assert_eq!(manifest.chromium_revision(), CHROMIUM_REVISION);
    assert_eq!(manifest.channel(), ReleaseChannel::Stable);
    assert_eq!(manifest.artifacts().len(), 2);
    assert_eq!(
        manifest.artifacts()[0].name(),
        "originweave-linux-x86_64.tar.zst"
    );
    assert_eq!(manifest.artifacts()[0].sha256_digest(), sha256_digest('a'));
    assert_eq!(manifest.artifacts()[1].name(), "originweave.spdx.json");
    assert_eq!(manifest.artifacts()[1].sha256_digest(), sha256_digest('b'));
    Ok(())
}

#[test]
fn release_artifact_rejects_ambiguous_names_and_noncanonical_digests() {
    let valid_digest = sha256_digest('c');
    let overlong_name = "a".repeat(MAX_RELEASE_ARTIFACT_NAME_BYTES + 1);

    for invalid_name in [
        "",
        ".hidden",
        "artifact/child.bin",
        "artifact\\child.bin",
        "artifact..bin",
        "artifact-.bin-",
        overlong_name.as_str(),
    ] {
        assert_eq!(
            ReleaseArtifact::new(invalid_name, &valid_digest),
            Err(ReleaseArtifactError::InvalidName)
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
            ReleaseArtifact::new("originweave.bin", invalid_digest),
            Err(ReleaseArtifactError::InvalidDigest)
        );
    }
}

#[test]
fn release_manifest_rejects_invalid_identity_missing_duplicate_and_unbounded_evidence()
-> Result<(), Box<dyn Error>> {
    let artifact = ReleaseArtifact::new("originweave.bin", &sha256_digest('d'))?;

    for invalid_commit in [
        "",
        "0123456789abcdef0123456789abcdef0123456",
        "0123456789ABCDEF0123456789ABCDEF01234567",
        "g123456789abcdef0123456789abcdef01234567",
    ] {
        assert_eq!(
            ReleaseManifest::new(
                invalid_commit,
                CHROMIUM_REVISION,
                ReleaseChannel::Development,
                [artifact.clone()],
            ),
            Err(ReleaseManifestError::InvalidSourceCommit)
        );
    }

    let overlong_revision = "a".repeat(MAX_RELEASE_REVISION_BYTES + 1);
    for invalid_revision in [
        "",
        " chromium-150",
        "chromium 150",
        "chromium/150",
        "chromium-150-",
        overlong_revision.as_str(),
    ] {
        assert_eq!(
            ReleaseManifest::new(
                SOURCE_COMMIT,
                invalid_revision,
                ReleaseChannel::Beta,
                [artifact.clone()],
            ),
            Err(ReleaseManifestError::InvalidChromiumRevision)
        );
    }

    assert_eq!(
        ReleaseManifest::new(
            SOURCE_COMMIT,
            CHROMIUM_REVISION,
            ReleaseChannel::Stable,
            std::iter::empty(),
        ),
        Err(ReleaseManifestError::MissingArtifacts)
    );

    let duplicate = ReleaseArtifact::new("originweave.bin", &sha256_digest('e'))?;
    assert_eq!(
        ReleaseManifest::new(
            SOURCE_COMMIT,
            CHROMIUM_REVISION,
            ReleaseChannel::Stable,
            [artifact.clone(), duplicate],
        ),
        Err(ReleaseManifestError::DuplicateArtifactName)
    );

    let mut too_many = Vec::new();
    for index in 0..=MAX_RELEASE_ARTIFACTS {
        too_many.push(ReleaseArtifact::new(
            &format!("artifact-{index}.bin"),
            &sha256_digest('f'),
        )?);
    }
    assert_eq!(
        ReleaseManifest::new(
            SOURCE_COMMIT,
            CHROMIUM_REVISION,
            ReleaseChannel::Stable,
            too_many,
        ),
        Err(ReleaseManifestError::TooManyArtifacts)
    );
    Ok(())
}

#[test]
fn release_manifest_errors_are_standard_source_free_rust_errors() {
    let artifact_errors = [
        (
            ReleaseArtifactError::InvalidName,
            "release artifact name is not a canonical bounded leaf name",
        ),
        (
            ReleaseArtifactError::InvalidDigest,
            "release artifact digest must be sha256: followed by 64 lowercase hexadecimal digits",
        ),
    ];
    for (error, expected) in artifact_errors {
        assert_eq!(error.to_string(), expected);
        assert!(Error::source(&error).is_none());
    }

    let manifest_errors = [
        (
            ReleaseManifestError::InvalidSourceCommit,
            "release source commit must be exactly 40 lowercase hexadecimal digits",
        ),
        (
            ReleaseManifestError::InvalidChromiumRevision,
            "Chromium revision must be a canonical bounded release token",
        ),
        (
            ReleaseManifestError::MissingArtifacts,
            "release manifest must contain at least one artifact",
        ),
        (
            ReleaseManifestError::TooManyArtifacts,
            "release manifest exceeds the artifact-count limit",
        ),
        (
            ReleaseManifestError::DuplicateArtifactName,
            "release manifest contains a duplicate artifact name",
        ),
    ];
    for (error, expected) in manifest_errors {
        assert_eq!(error.to_string(), expected);
        assert!(Error::source(&error).is_none());
    }
}
