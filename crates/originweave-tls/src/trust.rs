use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;
use sha2::{Digest, Sha256};

use crate::TlsError;

/// The largest number of trust-root certificates accepted in one bundle.
pub const MAX_TRUST_ROOT_COUNT: usize = 256;

/// The largest encoded input size accepted for one trust-root bundle.
pub const MAX_TRUST_ROOT_BYTES: usize = 2 * 1024 * 1024;

/// A bounded policy label for one immutable trust-root bundle.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrustBundleIdentifier(String);

impl TrustBundleIdentifier {
    /// Parse a 1–128 byte ASCII trust-bundle policy identifier.
    pub fn parse(input: &str) -> Result<Self, TlsError> {
        if input.is_empty()
            || input.len() > 128
            || !input.bytes().any(|byte| byte.is_ascii_alphanumeric())
            || !input.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
            })
        {
            return Err(TlsError::InvalidTrustBundleIdentifier);
        }
        Ok(Self(input.to_owned()))
    }

    /// Return the validated policy identifier.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// An immutable, bounded, canonical set of WebPKI trust roots.
#[derive(Debug, Clone)]
pub struct TrustRootBundle {
    identifier: TrustBundleIdentifier,
    certificates: Vec<CertificateDer<'static>>,
    root_store: RootCertStore,
    bundle_hash: String,
    encoded_byte_count: usize,
}

impl TrustRootBundle {
    /// Validate, canonicalize, hash, and load explicit trust-root certificates.
    pub fn new(
        identifier: TrustBundleIdentifier,
        certificate_der: Vec<Vec<u8>>,
    ) -> Result<Self, TlsError> {
        if certificate_der.is_empty() || certificate_der.len() > MAX_TRUST_ROOT_COUNT {
            return Err(TlsError::InvalidTrustRootCount {
                root_count: certificate_der.len(),
                maximum_count: MAX_TRUST_ROOT_COUNT,
            });
        }
        let input_byte_count = certificate_der.iter().fold(0_usize, |total, certificate| {
            total.saturating_add(certificate.len())
        });
        if input_byte_count > MAX_TRUST_ROOT_BYTES {
            return Err(TlsError::InvalidTrustRootBytes {
                byte_count: input_byte_count,
                maximum_bytes: MAX_TRUST_ROOT_BYTES,
            });
        }

        let mut canonical = certificate_der;
        canonical.sort();
        canonical.dedup();
        let encoded_byte_count = canonical.iter().map(Vec::len).sum();
        let bundle_hash = bundle_hash(&canonical);
        let certificates: Vec<CertificateDer<'static>> =
            canonical.into_iter().map(CertificateDer::from).collect();
        let mut root_store = RootCertStore::empty();
        for (root_index, certificate) in certificates.iter().cloned().enumerate() {
            root_store
                .add(certificate)
                .map_err(|source| TlsError::InvalidTrustRoot { root_index, source })?;
        }

        Ok(Self {
            identifier,
            certificates,
            root_store,
            bundle_hash,
            encoded_byte_count,
        })
    }

    /// Return the immutable bundle policy identifier.
    #[must_use]
    pub const fn identifier(&self) -> &TrustBundleIdentifier {
        &self.identifier
    }

    /// Return the canonical trust-root SHA-256 identifier.
    #[must_use]
    pub const fn bundle_hash(&self) -> &str {
        self.bundle_hash.as_str()
    }

    /// Return the canonical distinct root count.
    #[must_use]
    pub const fn root_count(&self) -> usize {
        self.certificates.len()
    }

    /// Return the canonical distinct encoded root bytes.
    #[must_use]
    pub const fn encoded_byte_count(&self) -> usize {
        self.encoded_byte_count
    }

    pub(crate) fn into_parts(self) -> (TrustBundleIdentifier, RootCertStore, String, usize, usize) {
        (
            self.identifier,
            self.root_store,
            self.bundle_hash,
            self.certificates.len(),
            self.encoded_byte_count,
        )
    }
}

fn bundle_hash(certificates: &[Vec<u8>]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"originweave-trust-root-bundle-v1\0");
    hasher.update(certificates.len().to_be_bytes());
    for certificate in certificates {
        hasher.update(certificate.len().to_be_bytes());
        hasher.update(certificate);
    }
    sha256_identifier(hasher.finalize().as_slice())
}

pub(crate) fn sha256_identifier(bytes: &[u8]) -> String {
    let mut identifier = String::with_capacity(71);
    identifier.push_str("sha256:");
    for byte in bytes {
        use std::fmt::Write;
        let _result = write!(identifier, "{byte:02x}");
    }
    identifier
}
