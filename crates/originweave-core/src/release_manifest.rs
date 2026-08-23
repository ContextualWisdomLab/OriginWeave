//! Fail-closed identity binding for release artifacts.
//!
//! The types in this module are deliberately inert metadata contracts. They bind an exact
//! source commit, Chromium revision, release channel, build identity, artifact digests, and
//! release-SBOM identity without granting signing, publication, installation, update, rollback,
//! or release authority.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Maximum number of artifacts admitted by one release manifest.
pub const MAX_RELEASE_ARTIFACTS: usize = 64;
/// Maximum UTF-8 byte length admitted for one canonical artifact leaf name.
pub const MAX_RELEASE_ARTIFACT_NAME_BYTES: usize = 128;
/// Maximum UTF-8 byte length admitted for one Chromium revision token.
pub const MAX_RELEASE_REVISION_BYTES: usize = 128;

/// Buyer-visible release channel bound by a release manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseChannel {
    /// Stable release channel.
    Stable,
    /// Beta release channel.
    Beta,
    /// Development release channel.
    Development,
}

/// Exact build identity retained by one release manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseBuildIdentity {
    rust_toolchain: String,
    dependency_lock_sha256: String,
}

impl ReleaseBuildIdentity {
    /// Construct build identity from the exact repository-pinned Rust toolchain and lock digest.
    ///
    /// The Rust toolchain must match the protected repository baseline exactly; moving aliases
    /// and alternate versions fail closed. The dependency-lock digest must use the exact
    /// `sha256:` prefix followed by 64 lowercase hexadecimal digits. Constructing this value does
    /// not prove reproducibility or authenticate the build environment; it only prevents those
    /// two identity fields from being omitted or represented ambiguously in a release manifest.
    pub fn new(
        rust_toolchain: &str,
        dependency_lock_sha256: &str,
    ) -> Result<Self, ReleaseBuildIdentityError> {
        if !valid_toolchain(rust_toolchain) {
            return Err(ReleaseBuildIdentityError::InvalidRustToolchain);
        }
        if !valid_sha256_digest(dependency_lock_sha256) {
            return Err(ReleaseBuildIdentityError::InvalidDependencyLockDigest);
        }
        Ok(Self {
            rust_toolchain: rust_toolchain.to_owned(),
            dependency_lock_sha256: dependency_lock_sha256.to_owned(),
        })
    }

    /// Return the exact repository-pinned Rust toolchain token.
    #[must_use]
    pub fn rust_toolchain(&self) -> &str {
        &self.rust_toolchain
    }

    /// Return the exact lowercase `sha256:` dependency-lock digest.
    #[must_use]
    pub fn dependency_lock_sha256(&self) -> &str {
        &self.dependency_lock_sha256
    }
}

/// Validation error for release build-identity evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseBuildIdentityError {
    /// Rust toolchain does not match the exact repository-pinned baseline.
    InvalidRustToolchain,
    /// Dependency-lock digest is not a canonical lowercase SHA-256 digest.
    InvalidDependencyLockDigest,
}

impl fmt::Display for ReleaseBuildIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRustToolchain => formatter
                .write_str("release Rust toolchain must match the exact repository-pinned baseline"),
            Self::InvalidDependencyLockDigest => formatter.write_str(
                "release dependency lock digest must be sha256: followed by 64 lowercase hexadecimal digits",
            ),
        }
    }
}

impl Error for ReleaseBuildIdentityError {}

/// One canonical release artifact identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseArtifact {
    name: String,
    sha256_digest: String,
}

impl ReleaseArtifact {
    /// Construct one artifact from a canonical leaf name and lowercase SHA-256 digest.
    ///
    /// The digest must use the exact `sha256:` prefix followed by 64 lowercase hexadecimal
    /// digits. Artifact names are ASCII leaf names and cannot contain path separators,
    /// traversal-like double dots, leading or trailing punctuation, or Windows reserved device
    /// basenames (including those basenames followed by extensions).
    pub fn new(name: &str, sha256_digest: &str) -> Result<Self, ReleaseArtifactError> {
        if !valid_artifact_name(name) {
            return Err(ReleaseArtifactError::InvalidName);
        }
        if !valid_sha256_digest(sha256_digest) {
            return Err(ReleaseArtifactError::InvalidDigest);
        }
        Ok(Self {
            name: name.to_owned(),
            sha256_digest: sha256_digest.to_owned(),
        })
    }

    /// Return the canonical artifact leaf name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the canonical lowercase `sha256:` artifact digest.
    #[must_use]
    pub fn sha256_digest(&self) -> &str {
        &self.sha256_digest
    }
}

/// Validation error for one release artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseArtifactError {
    /// The artifact name is not a canonical bounded leaf name.
    InvalidName,
    /// The artifact digest is not a canonical lowercase SHA-256 digest.
    InvalidDigest,
}

impl fmt::Display for ReleaseArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => {
                formatter.write_str("release artifact name is not a canonical bounded leaf name")
            }
            Self::InvalidDigest => formatter.write_str(
                "release artifact digest must be sha256: followed by 64 lowercase hexadecimal digits",
            ),
        }
    }
}

impl Error for ReleaseArtifactError {}

/// SPDX serialization identity admitted for release-SBOM evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseSbomFormat {
    /// SPDX 3.0.1 serialized as JSON-LD.
    Spdx30JsonLd,
}

impl ReleaseSbomFormat {
    /// Return the exact SPDX specification version represented by this format.
    #[must_use]
    pub const fn spdx_specification_version(self) -> &'static str {
        match self {
            Self::Spdx30JsonLd => "3.0.1",
        }
    }
}

/// Inert binding between one release manifest and its declared SPDX SBOM artifact identity.
///
/// This value proves only that the named SBOM and exact described artifact identities are already
/// admitted by the same immutable [`ReleaseManifest`]. It retains the complete bounded release
/// manifest identity and each described artifact digest, so any source, Chromium, channel, build,
/// artifact-inventory, or described-artifact change produces a different binding. It does not
/// parse or validate SPDX content, prove that the SBOM actually describes those artifacts,
/// authenticate any digest, or grant release authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSbomBinding {
    release_manifest: ReleaseManifest,
    format: ReleaseSbomFormat,
    sbom_artifact: ReleaseArtifact,
    described_artifact_names: Vec<String>,
    described_artifacts: Vec<ReleaseArtifact>,
}

impl ReleaseSbomBinding {
    /// Bind a declared SPDX SBOM artifact to exact artifacts in one release manifest.
    ///
    /// The complete bounded release-manifest identity is retained in the binding. The SBOM
    /// artifact name and every described artifact name must match that manifest exactly. At least
    /// one described artifact is required, duplicates fail closed, and exact described artifact
    /// identities plus their public names are sorted deterministically. The release manifest's own
    /// admission bound therefore also bounds this inventory.
    pub fn new<'a, I>(
        manifest: &ReleaseManifest,
        sbom_artifact_name: &str,
        format: ReleaseSbomFormat,
        described_artifact_names: I,
    ) -> Result<Self, ReleaseSbomBindingError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let Some(sbom_artifact) = manifest
            .artifacts()
            .iter()
            .find(|artifact| artifact.name() == sbom_artifact_name)
        else {
            return Err(ReleaseSbomBindingError::UnknownSbomArtifact);
        };

        let mut described_artifacts = Vec::new();
        let mut seen = BTreeSet::new();
        for artifact_name in described_artifact_names {
            let Some(artifact) = manifest
                .artifacts()
                .iter()
                .find(|artifact| artifact.name() == artifact_name)
            else {
                return Err(ReleaseSbomBindingError::UnknownDescribedArtifact);
            };
            if !seen.insert(artifact_name.to_owned()) {
                return Err(ReleaseSbomBindingError::DuplicateDescribedArtifact);
            }
            described_artifacts.push(artifact.clone());
        }
        if described_artifacts.is_empty() {
            return Err(ReleaseSbomBindingError::MissingDescribedArtifacts);
        }
        described_artifacts.sort_by(|left, right| left.name.cmp(&right.name));
        let described_artifact_names = described_artifacts
            .iter()
            .map(|artifact| artifact.name().to_owned())
            .collect();

        Ok(Self {
            release_manifest: manifest.clone(),
            format,
            sbom_artifact: sbom_artifact.clone(),
            described_artifact_names,
            described_artifacts,
        })
    }

    /// Return the declared SPDX serialization format.
    #[must_use]
    pub const fn format(&self) -> ReleaseSbomFormat {
        self.format
    }

    /// Return the exact SPDX specification version represented by the declared format.
    #[must_use]
    pub const fn spdx_specification_version(&self) -> &'static str {
        self.format.spdx_specification_version()
    }

    /// Return the exact manifest-backed SBOM artifact identity.
    #[must_use]
    pub const fn sbom_artifact(&self) -> &ReleaseArtifact {
        &self.sbom_artifact
    }

    /// Return described manifest artifact names in deterministic lexical order.
    #[must_use]
    pub fn described_artifact_names(&self) -> &[String] {
        &self.described_artifact_names
    }
}

/// Validation error for release-SBOM identity binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseSbomBindingError {
    /// The named SBOM artifact is absent from the release manifest.
    UnknownSbomArtifact,
    /// No release artifacts were declared as described by the SBOM.
    MissingDescribedArtifacts,
    /// A described artifact name is absent from the release manifest.
    UnknownDescribedArtifact,
    /// A described artifact name was repeated.
    DuplicateDescribedArtifact,
}

impl fmt::Display for ReleaseSbomBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSbomArtifact => formatter
                .write_str("release SBOM artifact is not present in the bound release manifest"),
            Self::MissingDescribedArtifacts => {
                formatter.write_str("release SBOM must describe at least one release artifact")
            }
            Self::UnknownDescribedArtifact => formatter.write_str(
                "release SBOM describes an artifact that is not present in the bound release manifest",
            ),
            Self::DuplicateDescribedArtifact => {
                formatter.write_str("release SBOM repeats a described release artifact")
            }
        }
    }
}

impl Error for ReleaseSbomBindingError {}

/// Deterministic, bounded identity manifest for one OriginWeave release candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseManifest {
    source_commit: String,
    chromium_revision: String,
    channel: ReleaseChannel,
    build_identity: ReleaseBuildIdentity,
    artifacts: Vec<ReleaseArtifact>,
}

impl ReleaseManifest {
    /// Construct an inert release manifest from exact identity evidence.
    ///
    /// Source identity is a full 40-digit lowercase Git commit SHA. Chromium revision is a
    /// bounded canonical ASCII token, Rust toolchain identity is the exact repository pin, and
    /// dependency-lock identity is a canonical lowercase SHA-256 digest. Artifact names must be
    /// unique under ASCII case folding so one manifest cannot bind two names that collide on a
    /// case-insensitive target filesystem; original spelling is preserved and artifacts are
    /// sorted deterministically before storage. Constructing this value does not authenticate any
    /// artifact, prove reproducibility, or authorize release or installation.
    pub fn new<I>(
        source_commit: &str,
        chromium_revision: &str,
        channel: ReleaseChannel,
        build_identity: ReleaseBuildIdentity,
        artifacts: I,
    ) -> Result<Self, ReleaseManifestError>
    where
        I: IntoIterator<Item = ReleaseArtifact>,
    {
        if !valid_source_commit(source_commit) {
            return Err(ReleaseManifestError::InvalidSourceCommit);
        }
        if !valid_revision(chromium_revision) {
            return Err(ReleaseManifestError::InvalidChromiumRevision);
        }

        let mut admitted = Vec::new();
        let mut artifact_names = BTreeSet::new();
        for artifact in artifacts {
            if admitted.len() >= MAX_RELEASE_ARTIFACTS {
                return Err(ReleaseManifestError::TooManyArtifacts);
            }
            if !artifact_names.insert(artifact.name.to_ascii_lowercase()) {
                return Err(ReleaseManifestError::DuplicateArtifactName);
            }
            admitted.push(artifact);
        }
        if admitted.is_empty() {
            return Err(ReleaseManifestError::MissingArtifacts);
        }
        admitted.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(Self {
            source_commit: source_commit.to_owned(),
            chromium_revision: chromium_revision.to_owned(),
            channel,
            build_identity,
            artifacts: admitted,
        })
    }

    /// Return the exact lowercase source commit bound by this manifest.
    #[must_use]
    pub fn source_commit(&self) -> &str {
        &self.source_commit
    }

    /// Return the canonical Chromium revision token bound by this manifest.
    #[must_use]
    pub fn chromium_revision(&self) -> &str {
        &self.chromium_revision
    }

    /// Return the release channel bound by this manifest.
    #[must_use]
    pub const fn channel(&self) -> ReleaseChannel {
        self.channel
    }

    /// Return the exact build identity bound by this manifest.
    #[must_use]
    pub const fn build_identity(&self) -> &ReleaseBuildIdentity {
        &self.build_identity
    }

    /// Return artifacts sorted deterministically by canonical name.
    #[must_use]
    pub fn artifacts(&self) -> &[ReleaseArtifact] {
        &self.artifacts
    }
}

/// Validation error for release-manifest identity evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseManifestError {
    /// Source commit is not a non-null full lowercase 40-hex Git object identity.
    InvalidSourceCommit,
    /// Chromium revision is not a canonical bounded release token.
    InvalidChromiumRevision,
    /// No release artifacts were supplied.
    MissingArtifacts,
    /// Artifact inventory exceeds the bounded release-manifest limit.
    TooManyArtifacts,
    /// Artifact inventory repeats an ASCII-case-folded artifact name.
    DuplicateArtifactName,
}

impl fmt::Display for ReleaseManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSourceCommit => formatter.write_str(
                "release source commit must be a non-null 40-digit lowercase Git object identity",
            ),
            Self::InvalidChromiumRevision => {
                formatter.write_str("Chromium revision must be a canonical bounded release token")
            }
            Self::MissingArtifacts => {
                formatter.write_str("release manifest must contain at least one artifact")
            }
            Self::TooManyArtifacts => {
                formatter.write_str("release manifest exceeds the artifact-count limit")
            }
            Self::DuplicateArtifactName => {
                formatter.write_str("release manifest contains a duplicate artifact name")
            }
        }
    }
}

impl Error for ReleaseManifestError {}

fn valid_artifact_name(name: &str) -> bool {
    if name.is_empty()
        || name.len() > MAX_RELEASE_ARTIFACT_NAME_BYTES
        || !name.is_ascii()
        || name.contains("..")
        || windows_reserved_device_basename(name)
    {
        return false;
    }
    let bytes = name.as_bytes();
    bytes[0].is_ascii_alphanumeric()
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'-'))
}

fn windows_reserved_device_basename(name: &str) -> bool {
    let basename = match name.find('.') {
        Some(dot_index) => &name[..dot_index],
        None => name,
    };

    if basename.eq_ignore_ascii_case("CON")
        || basename.eq_ignore_ascii_case("PRN")
        || basename.eq_ignore_ascii_case("AUX")
        || basename.eq_ignore_ascii_case("NUL")
    {
        return true;
    }

    let bytes = basename.as_bytes();
    bytes.len() == 4
        && (basename[..3].eq_ignore_ascii_case("COM") || basename[..3].eq_ignore_ascii_case("LPT"))
        && bytes[3].is_ascii_digit()
}

fn valid_sha256_digest(digest: &str) -> bool {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn valid_source_commit(source_commit: &str) -> bool {
    source_commit.len() == 40
        && source_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        && source_commit.bytes().any(|byte| byte != b'0')
}

fn valid_revision(revision: &str) -> bool {
    if revision.is_empty() || revision.len() > MAX_RELEASE_REVISION_BYTES || !revision.is_ascii() {
        return false;
    }
    let bytes = revision.as_bytes();
    bytes[0].is_ascii_alphanumeric()
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
        && bytes.iter().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'-' | b'+' | b':' | b'@')
        })
}

fn valid_toolchain(toolchain: &str) -> bool {
    toolchain == "1.97.1"
}
