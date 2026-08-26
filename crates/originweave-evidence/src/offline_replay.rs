use std::fmt;

use crate::{
    CaptureManifest, CaptureManifestValueBinding, CaptureManifestVerificationError,
    ExtractionSchema, WarcProvBundle, WarcResourceRecord,
};

/// Credential-safe receipt proving one in-memory capture package matched its persisted identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineReplayVerification {
    manifest_digest: String,
    record_count: usize,
    value_count: usize,
}

impl OfflineReplayVerification {
    /// Return the SHA-256 identity of the exact deterministic capture manifest.
    #[must_use]
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    /// Return the number of WARC/PROV record pairs verified by this receipt.
    #[must_use]
    pub const fn record_count(&self) -> usize {
        self.record_count
    }

    /// Return the number of schema-bound structured-value identities verified by this receipt.
    #[must_use]
    pub const fn value_count(&self) -> usize {
        self.value_count
    }
}

/// A fail-closed offline capture-package verification failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineReplayVerificationError {
    /// Persisted manifest bytes were not the exact deterministic serialization expected in memory.
    ManifestBytes(CaptureManifestVerificationError),
    /// Schema, WARC/PROV evidence, or structured-value identity did not match the expected manifest.
    Evidence(CaptureManifestVerificationError),
}

impl fmt::Display for OfflineReplayVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestBytes(error) => write!(
                formatter,
                "offline replay persisted manifest bytes failed verification: {error}"
            ),
            Self::Evidence(error) => write!(
                formatter,
                "offline replay capture evidence failed verification: {error}"
            ),
        }
    }
}

impl std::error::Error for OfflineReplayVerificationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ManifestBytes(error) | Self::Evidence(error) => Some(error),
        }
    }
}

/// Verify one already-materialized capture package without contacting or executing its source.
///
/// Verification first requires `persisted_manifest_bytes` to equal the expected deterministic
/// manifest serialization byte-for-byte. It then reconstructs and verifies the schema-bound
/// WARC/PROV/value identity through [`CaptureManifest::verify_with_warc_values`]. The operation is
/// deliberately in-memory only: it performs no DNS, network, browser, JavaScript, external-reference,
/// secret, persistence, retention, signing, or authorization action and does not establish factual
/// correctness beyond the supplied evidence contracts.
pub fn verify_offline_capture_package(
    expected_manifest: &CaptureManifest,
    persisted_manifest_bytes: &[u8],
    schema: &ExtractionSchema,
    records: &[(&WarcResourceRecord, &WarcProvBundle)],
    values: &[CaptureManifestValueBinding],
) -> Result<OfflineReplayVerification, OfflineReplayVerificationError> {
    expected_manifest
        .verify_serialized_json(persisted_manifest_bytes)
        .map_err(OfflineReplayVerificationError::ManifestBytes)?;
    expected_manifest
        .verify_with_warc_values(schema, records, values)
        .map_err(OfflineReplayVerificationError::Evidence)?;

    Ok(OfflineReplayVerification {
        manifest_digest: expected_manifest.manifest_digest(),
        record_count: expected_manifest.records().len(),
        value_count: expected_manifest.values().len(),
    })
}
