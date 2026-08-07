//! Perform one bounded HTTP/1.1 exchange over an authenticated OriginWeave TLS stream.
//!
//! The crate owns no resolver, connector, proxy, pool, cookie jar, filesystem,
//! browser, or model authority. It consumes one authenticated TLS stream,
//! applies strict request and response contracts, and exposes content only
//! after every configured resource and evidence check succeeds.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod chunked;
mod content;
#[cfg(test)]
mod coverage_contract;
mod disposition;
mod error;
mod evidence;
mod exchange;
mod field;
mod framing;
mod integrity;
mod mime;
mod policy;
mod request;
mod response_head;
mod target;

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
