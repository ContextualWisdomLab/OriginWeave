use sha2::{Digest, Sha256};

use crate::{ProvenanceRecord, VerificationResult};

/// Maximum encoded size of the UUID-based WARC record identifier.
pub const MAX_WARC_RECORD_ID_BYTES: usize = 45;
/// Maximum encoded size accepted for a UTC WARC date.
pub const MAX_WARC_DATE_BYTES: usize = 30;
/// Maximum encoded size retained for a WARC content type.
pub const MAX_WARC_CONTENT_TYPE_BYTES: usize = 256;
/// Maximum resource payload retained by one immutable WARC record.
pub const MAX_WARC_PAYLOAD_BYTES: usize = 1_048_576;

/// A validation failure while constructing an immutable WARC resource record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarcResourceRecordError {
    /// The record identifier was not a bounded UUID URN.
    InvalidRecordId,
    /// The date was not a bounded UTC RFC 3339 timestamp.
    InvalidDate,
    /// The content type was empty or contained unsafe whitespace/control input.
    InvalidContentType,
    /// A record field or payload exceeded its retention limit.
    LimitExceeded,
    /// The WARC target URI differed from its provenance source URL.
    TargetUriMismatch,
    /// The source provenance was not independently verified.
    UnverifiedProvenance,
}

/// An immutable, bounded WARC `resource` record over already-authorized bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarcResourceRecord {
    record_id: String,
    warc_date: String,
    target_uri: String,
    content_type: String,
    payload: Vec<u8>,
    block_digest: String,
    provenance: ProvenanceRecord,
}

impl WarcResourceRecord {
    /// Validate and construct one resource record without contacting a live origin.
    pub fn new(
        record_id: &str,
        warc_date: &str,
        target_uri: &str,
        content_type: &str,
        payload: Vec<u8>,
        provenance: ProvenanceRecord,
    ) -> Result<Self, WarcResourceRecordError> {
        if !valid_record_id(record_id) {
            return Err(WarcResourceRecordError::InvalidRecordId);
        }
        if !valid_utc_date(warc_date) {
            return Err(WarcResourceRecordError::InvalidDate);
        }
        if !valid_content_type(content_type) {
            return Err(if content_type.len() > MAX_WARC_CONTENT_TYPE_BYTES {
                WarcResourceRecordError::LimitExceeded
            } else {
                WarcResourceRecordError::InvalidContentType
            });
        }
        if target_uri != provenance.source_url() {
            return Err(WarcResourceRecordError::TargetUriMismatch);
        }
        if provenance.verification_result() != VerificationResult::Verified {
            return Err(WarcResourceRecordError::UnverifiedProvenance);
        }
        if payload.len() > MAX_WARC_PAYLOAD_BYTES {
            return Err(WarcResourceRecordError::LimitExceeded);
        }

        Ok(Self {
            record_id: record_id.to_owned(),
            warc_date: warc_date.to_owned(),
            target_uri: target_uri.to_owned(),
            content_type: content_type.to_owned(),
            block_digest: sha256_digest(&payload),
            payload,
            provenance,
        })
    }

    /// Return the UUID URN used as the WARC record identity.
    #[must_use]
    pub fn record_id(&self) -> &str {
        &self.record_id
    }

    /// Return the normalized UTC capture timestamp.
    #[must_use]
    pub fn warc_date(&self) -> &str {
        &self.warc_date
    }

    /// Return the provenance-bound target URI.
    #[must_use]
    pub fn target_uri(&self) -> &str {
        &self.target_uri
    }

    /// Return the payload media type retained in the WARC record.
    #[must_use]
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    /// Return the immutable resource bytes.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Return the lowercase SHA-256 block digest.
    #[must_use]
    pub fn block_digest(&self) -> &str {
        &self.block_digest
    }

    /// Return the independently verified provenance bound to this record.
    #[must_use]
    pub const fn provenance(&self) -> &ProvenanceRecord {
        &self.provenance
    }

    /// Serialize this bounded resource record as deterministic WARC 1.1 bytes.
    #[must_use]
    pub fn to_warc_bytes(&self) -> Vec<u8> {
        let header = format!(
            "WARC/1.1\r\nWARC-Type: resource\r\nWARC-Record-ID: <{}>\r\nWARC-Date: {}\r\nWARC-Target-URI: {}\r\nContent-Type: {}\r\nWARC-Block-Digest: {}\r\nContent-Length: {}\r\n\r\n",
            self.record_id,
            self.warc_date,
            self.target_uri,
            self.content_type,
            self.block_digest,
            self.payload.len()
        );
        let mut bytes = header.into_bytes();
        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(b"\r\n\r\n");
        bytes
    }
}

fn valid_record_id(record_id: &str) -> bool {
    let bytes = record_id.as_bytes();
    if bytes.len() != MAX_WARC_RECORD_ID_BYTES || !record_id.starts_with("urn:uuid:") {
        return false;
    }
    for (index, byte) in bytes[9..].iter().copied().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn valid_utc_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    if !(20..=MAX_WARC_DATE_BYTES).contains(&bytes.len())
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return false;
    }
    let has_fraction = bytes[19] == b'.';
    if has_fraction {
        if bytes.last() != Some(&b'Z') || bytes.len() < 22 {
            return false;
        }
        let fraction = &bytes[20..bytes.len() - 1];
        if fraction.iter().any(|byte| !byte.is_ascii_digit()) {
            return false;
        }
    } else if bytes.len() != 20 || bytes[19] != b'Z' {
        return false;
    }
    if !bytes[..19]
        .iter()
        .enumerate()
        .all(|(index, byte)| matches!(index, 4 | 7 | 10 | 13 | 16) || byte.is_ascii_digit())
    {
        return false;
    }
    let year = four_digits(bytes[0], bytes[1], bytes[2], bytes[3]);
    let month = two_digits(bytes[5], bytes[6]);
    let day = two_digits(bytes[8], bytes[9]);
    let hour = two_digits(bytes[11], bytes[12]);
    let minute = two_digits(bytes[14], bytes[15]);
    let second = two_digits(bytes[17], bytes[18]);
    valid_calendar_date(year, month, day) && hour < 24 && minute < 60 && second <= 60
}

fn four_digits(first: u8, second: u8, third: u8, fourth: u8) -> u16 {
    u16::from(first - b'0') * 1000
        + u16::from(second - b'0') * 100
        + u16::from(third - b'0') * 10
        + u16::from(fourth - b'0')
}

fn two_digits(high: u8, low: u8) -> u8 {
    (high - b'0') * 10 + (low - b'0')
}

fn valid_calendar_date(year: u16, month: u8, day: u8) -> bool {
    if !(1..=12).contains(&month) {
        return false;
    }
    let days_in_month = if month == 2 {
        if is_leap_year(year) { 29 } else { 28 }
    } else {
        30 + ((month + month / 8) % 2)
    };
    (1..=days_in_month).contains(&day)
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100))
}

fn valid_content_type(content_type: &str) -> bool {
    !content_type.is_empty()
        && content_type.len() <= MAX_WARC_CONTENT_TYPE_BYTES
        && !content_type
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
}

fn sha256_digest(payload: &[u8]) -> String {
    let digest = Sha256::digest(payload);
    let mut encoded = String::from("sha256:");
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}
