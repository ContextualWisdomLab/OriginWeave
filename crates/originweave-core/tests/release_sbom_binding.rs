use std::error::Error;

use originweave_core::release_manifest::{
    ReleaseArtifact, ReleaseBuildIdentity, ReleaseChannel, ReleaseManifest, ReleaseSbomBinding,
    ReleaseSbomBindingError, ReleaseSbomFormat,
};

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const CHROMIUM_REVISION: &str = "150.0.7871.129";

fn sha256_digest(hex_digit: char) -> String {
    format!("sha256:{}", hex_digit.to_string().repeat(64))
}

fn release_manifest() -> Result<ReleaseManifest, Box<dyn Error>> {
    ReleaseManifest::new(
        SOURCE_COMMIT,
        CHROMIUM_REVISION,
        ReleaseChannel::Stable,
        ReleaseBuildIdentity::new("1.97.1", &sha256_digest('9'))?,
        vec![
            ReleaseArtifact::new("originweave-linux-x86_64.tar.zst", &sha256_digest('a'))?,
            ReleaseArtifact::new("originweave-native-host.bin", &sha256_digest('b'))?,
            ReleaseArtifact::new("originweave.spdx.jsonld", &sha256_digest('c'))?,
        ],
    )
    .map_err(Into::into)
}

#[test]
fn spdx_sbom_binding_requires_manifest_backed_sbom_and_described_artifacts()
-> Result<(), Box<dyn Error>> {
    let manifest = release_manifest()?;
    let binding = ReleaseSbomBinding::new(
        &manifest,
        "originweave.spdx.jsonld",
        ReleaseSbomFormat::Spdx30JsonLd,
        vec![
            "originweave-native-host.bin",
            "originweave-linux-x86_64.tar.zst",
        ],
    )?;

    assert_eq!(binding.format(), ReleaseSbomFormat::Spdx30JsonLd);
    assert_eq!(binding.spdx_specification_version(), "3.0.1");
    assert_eq!(binding.sbom_artifact().name(), "originweave.spdx.jsonld");
    assert_eq!(binding.sbom_artifact().sha256_digest(), sha256_digest('c'));
    assert_eq!(
        binding.described_artifact_names(),
        [
            "originweave-linux-x86_64.tar.zst",
            "originweave-native-host.bin",
        ]
    );
    Ok(())
}

#[test]
fn spdx_sbom_binding_fails_closed_on_missing_or_ambiguous_manifest_identity()
-> Result<(), Box<dyn Error>> {
    let manifest = release_manifest()?;

    assert_eq!(
        ReleaseSbomBinding::new(
            &manifest,
            "missing.spdx.jsonld",
            ReleaseSbomFormat::Spdx30JsonLd,
            vec!["originweave-linux-x86_64.tar.zst"],
        ),
        Err(ReleaseSbomBindingError::UnknownSbomArtifact)
    );
    assert_eq!(
        ReleaseSbomBinding::new(
            &manifest,
            "originweave.spdx.jsonld",
            ReleaseSbomFormat::Spdx30JsonLd,
            Vec::<&str>::new(),
        ),
        Err(ReleaseSbomBindingError::MissingDescribedArtifacts)
    );
    assert_eq!(
        ReleaseSbomBinding::new(
            &manifest,
            "originweave.spdx.jsonld",
            ReleaseSbomFormat::Spdx30JsonLd,
            vec!["not-in-release.bin"],
        ),
        Err(ReleaseSbomBindingError::UnknownDescribedArtifact)
    );
    assert_eq!(
        ReleaseSbomBinding::new(
            &manifest,
            "originweave.spdx.jsonld",
            ReleaseSbomFormat::Spdx30JsonLd,
            vec![
                "originweave-linux-x86_64.tar.zst",
                "originweave-linux-x86_64.tar.zst",
            ],
        ),
        Err(ReleaseSbomBindingError::DuplicateDescribedArtifact)
    );
    assert_eq!(
        ReleaseSbomBinding::new(
            &manifest,
            "originweave.spdx.jsonld",
            ReleaseSbomFormat::Spdx30JsonLd,
            vec!["ORIGINWEAVE-LINUX-X86_64.TAR.ZST"],
        ),
        Err(ReleaseSbomBindingError::UnknownDescribedArtifact)
    );
    Ok(())
}

#[test]
fn spdx_sbom_binding_errors_are_standard_source_free_rust_errors() {
    let errors = [
        (
            ReleaseSbomBindingError::UnknownSbomArtifact,
            "release SBOM artifact is not present in the bound release manifest",
        ),
        (
            ReleaseSbomBindingError::MissingDescribedArtifacts,
            "release SBOM must describe at least one release artifact",
        ),
        (
            ReleaseSbomBindingError::UnknownDescribedArtifact,
            "release SBOM describes an artifact that is not present in the bound release manifest",
        ),
        (
            ReleaseSbomBindingError::DuplicateDescribedArtifact,
            "release SBOM repeats a described release artifact",
        ),
    ];

    for (error, expected) in errors {
        assert_eq!(error.to_string(), expected);
        assert!(Error::source(&error).is_none());
    }
}
