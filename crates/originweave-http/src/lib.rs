//! Perform one bounded HTTP/1.1 exchange over an authenticated OriginWeave TLS stream.
//!
//! The crate owns no resolver, connector, proxy, pool, cookie jar, filesystem,
//! browser, or model authority. It consumes one authenticated TLS stream,
//! applies strict request and response contracts, and will expose content only
//! after every configured resource and evidence check succeeds.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod field;
mod policy;
mod request;
mod target;

#[cfg(test)]
mod framing;
#[cfg(test)]
mod response_head;

pub use error::HttpError;
pub use field::RequestField;
pub use policy::{
    AlpnHttp11Policy, DEFAULT_MAX_CHUNK_COUNT, DEFAULT_MAX_CONTENT_EXPANSION_RATIO,
    DEFAULT_MAX_DECODED_CONTENT_BYTES, DEFAULT_MAX_ENCODED_CONTENT_BYTES,
    DEFAULT_MAX_HEADER_FIELD_COUNT, DEFAULT_MAX_HEADER_NAME_BYTES,
    DEFAULT_MAX_HEADER_SECTION_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES,
    DEFAULT_MAX_INTERIM_RESPONSE_COUNT, DEFAULT_MAX_REQUEST_BYTES,
    DEFAULT_MAX_STATUS_LINE_BYTES, DEFAULT_MAX_TRAILER_FIELD_COUNT,
    DEFAULT_MAX_TRAILER_SECTION_BYTES, HttpClientPolicy, IntegrityRequirement,
    MAX_HTTP_EXCHANGE_TIMEOUT, MAX_MIME_SNIFF_BYTES, MAX_SAFE_FILENAME_BYTES,
};
pub use request::HttpMethod;
pub use target::HttpRequestTarget;
