//! Perform one bounded HTTP/1.1 exchange over an authenticated OriginWeave TLS stream.
//!
//! The crate owns no resolver, connector, proxy, pool, cookie jar, filesystem,
//! browser, or model authority. It consumes one authenticated TLS stream,
//! applies strict request and response contracts, and exposes content only
//! after every configured resource and evidence check succeeds.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod chunked;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[path = "tests/chunked_wire_budget_contract.rs"]
mod chunked_wire_budget_contract;
mod content;
#[cfg(test)]
#[path = "tests/coverage_contract.rs"]
mod coverage_contract;
mod disposition;
mod error;
mod evidence;
mod exchange;
#[cfg(test)]
#[path = "tests/exchange_error_contract.rs"]
mod exchange_error_contract;
mod field;
mod framing;
mod integrity;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[path = "tests/integrity_padding_contract.rs"]
mod integrity_padding_contract;
mod mime;
#[cfg(test)]
#[path = "tests/mime_contract.rs"]
mod mime_contract;
mod policy;
#[cfg(test)]
#[path = "tests/reachability_contract.rs"]
mod reachability_contract;
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[path = "tests/reason_phrase_contract.rs"]
mod reason_phrase_contract;
#[cfg(test)]
#[path = "tests/region_contract.rs"]
mod region_contract;
mod request;
mod response_head;
#[cfg(test)]
#[path = "tests/security_contract.rs"]
mod security_contract;
mod target;
#[cfg(test)]
#[path = "tests/trailer_error_contract.rs"]
mod trailer_error_contract;

pub use content::ContentCoding;
pub use disposition::{
    DispositionKind, ExtensionMimeRelation, RedirectMetadata, SafeContentDisposition,
};
pub use error::HttpError;
pub use evidence::{
    AuthenticatedHttpResponse, HttpExchangeEvidence, HttpResourceBudgets, ResponseFieldEvidence,
};
pub use exchange::HttpExchangePlan;
pub use field::RequestField;
pub use framing::BodyFraming;
pub use integrity::{IntegrityAlgorithm, IntegrityStatus};
pub use mime::{
    ContentRiskClass, MIME_CLASSIFIER_VERSION, MimeMismatch, MimeType, NoSniffStatus,
    ObservedMimeClassification,
};
pub use policy::{
    AlpnHttp11Policy, DEFAULT_MAX_CHUNK_COUNT, DEFAULT_MAX_CONTENT_EXPANSION_RATIO,
    DEFAULT_MAX_DECODED_CONTENT_BYTES, DEFAULT_MAX_ENCODED_CONTENT_BYTES,
    DEFAULT_MAX_HEADER_FIELD_COUNT, DEFAULT_MAX_HEADER_NAME_BYTES,
    DEFAULT_MAX_HEADER_SECTION_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES,
    DEFAULT_MAX_INTERIM_RESPONSE_COUNT, DEFAULT_MAX_REQUEST_BYTES, DEFAULT_MAX_STATUS_LINE_BYTES,
    DEFAULT_MAX_TRAILER_FIELD_COUNT, DEFAULT_MAX_TRAILER_SECTION_BYTES, HttpClientPolicy,
    IntegrityRequirement, MAX_HTTP_EXCHANGE_TIMEOUT, MAX_MIME_SNIFF_BYTES, MAX_SAFE_FILENAME_BYTES,
};
pub use request::HttpMethod;
pub use target::HttpRequestTarget;