use std::error::Error;

use originweave_core::release_manifest::{
    ReleaseArtifact, ReleaseBuildIdentity, ReleaseChannel, ReleaseManifest, ReleaseSbomBinding,
    ReleaseSbomFormat,
};

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const CHROMIUM_REVISION: &str = "150.0.7871.129";

fn sha256_digest(hex_digit: char) -> String {
    format!("sha256:{}", hex_digit.to_string().repeat(64))
}

#[test]
fn release_sbom_binding_rejects_incomplete_manifest_inventory() -> Result<(), Box<dyn Error>> {
    let manifest = ReleaseManifest::new(
        SOURCE_COMMIT,
        CHROMIUM_REVISION,
        ReleaseChannel::Stable,
        ReleaseBuildIdentity::new("1.97.1", &sha256_digest('9'))?,
        vec![
            ReleaseArtifact::new("originweave-linux-x86_64.tar.zst", &sha256_digest('a'))?,
            ReleaseArtifact::new("originweave-native-host.bin", &sha256_digest('b'))?,
            ReleaseArtifact::new("originweave.spdx.jsonld", &sha256_digest('c'))?,
        ],
    )?;

    let error = ReleaseSbomBinding::new(
        &manifest,
        "originweave.spdx.jsonld",
        ReleaseSbomFormat::Spdx30JsonLd,
        vec!["originweave-linux-x86_64.tar.zst"],
    )
    .expect_err("incomplete release SBOM inventory must fail closed");

    assert_eq!(
        error.to_string(),
        "release SBOM must describe every non-SBOM release artifact"
    );
    Ok(())
}
