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
    /// The persisted WARC/PROV byte-pair inventory did not match the typed record inventory.
    PersistedRecordCountMismatch,
    /// Persisted deterministic WARC bytes differed from the typed record at this zero-based index.
    WarcBytes {
        /// Zero-based record index whose persisted WARC bytes failed exact verification.
        record_index: usize,
    },
    /// Persisted deterministic PROV JSON-LD bytes differed from the typed bundle at this index.
    ProvBytes {
        /// Zero-based record index whose persisted PROV JSON-LD bytes failed exact verification.
        record_index: usize,
    },
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
            Self::PersistedRecordCountMismatch => formatter.write_str(
                "offline replay persisted WARC/PROV record count does not match typed evidence",
            ),
            Self::WarcBytes { record_index } => write!(
                formatter,
                "offline replay persisted WARC bytes failed verification at record {record_index}"
            ),
            Self::ProvBytes { record_index } => write!(
                formatter,
                "offline replay persisted PROV bytes failed verification at record {record_index}"
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
            Self::PersistedRecordCountMismatch
            | Self::WarcBytes { .. }
            | Self::ProvBytes { .. } => None,
        }
    }
}

fn verification_receipt(expected_manifest: &CaptureManifest) -> OfflineReplayVerification {
    OfflineReplayVerification {
        manifest_digest: expected_manifest.manifest_digest(),
        record_count: expected_manifest.records().len(),
        value_count: expected_manifest.values().len(),
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

    Ok(verification_receipt(expected_manifest))
}

/// Verify exact persisted manifest, WARC, and PROV bytes against one typed offline capture package.
///
/// `persisted_record_bytes` must contain exactly one `(WARC bytes, PROV JSON-LD bytes)` pair for
/// each typed WARC/PROV pair in `records`, in the same canonical order. The persisted bytes are
/// compared byte-for-byte with [`WarcResourceRecord::to_warc_bytes`] and
/// [`WarcProvBundle::to_json_ld`] before the manifest's typed schema/WARC/PROV/value identity is
/// revalidated. This closes the gap between reconstructing trusted in-memory objects and verifying
/// the exact artifacts a buyer retained for offline replay. It performs no parsing, execution,
/// network access, persistence mutation, signing, retention decision, or authority escalation.
pub fn verify_persisted_offline_capture_package(
    expected_manifest: &CaptureManifest,
    persisted_manifest_bytes: &[u8],
    schema: &ExtractionSchema,
    records: &[(&WarcResourceRecord, &WarcProvBundle)],
    persisted_record_bytes: &[(&[u8], &[u8])],
    values: &[CaptureManifestValueBinding],
) -> Result<OfflineReplayVerification, OfflineReplayVerificationError> {
    expected_manifest
        .verify_serialized_json(persisted_manifest_bytes)
        .map_err(OfflineReplayVerificationError::ManifestBytes)?;

    if persisted_record_bytes.len() != records.len() {
        return Err(OfflineReplayVerificationError::PersistedRecordCountMismatch);
    }

    for (record_index, ((record, bundle), (persisted_warc, persisted_prov))) in records
        .iter()
        .zip(persisted_record_bytes.iter())
        .enumerate()
    {
        if record.to_warc_bytes().as_slice() != *persisted_warc {
            return Err(OfflineReplayVerificationError::WarcBytes { record_index });
        }
        if bundle.to_json_ld().as_bytes() != *persisted_prov {
            return Err(OfflineReplayVerificationError::ProvBytes { record_index });
        }
    }

    expected_manifest
        .verify_with_warc_values(schema, records, values)
        .map_err(OfflineReplayVerificationError::Evidence)?;

    Ok(verification_receipt(expected_manifest))
}
