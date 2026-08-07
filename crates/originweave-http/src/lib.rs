//! Perform one bounded HTTP/1.1 exchange over an authenticated OriginWeave TLS stream.
//!
//! The first vertical slice supports `GET` and `HEAD` with strict request
//! construction and either no response content or an explicit `Content-Length`.
//! It consumes an existing authenticated TLS stream and contains no DNS,
//! connect, reconnect, proxy, redirect-following, filesystem, Chromium, or
//! model authority.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod evidence;
mod exchange;
mod field;
mod framing;
mod policy;
mod request;
mod response_head;
mod target;

pub use error::HttpError;
pub use evidence::{AuthenticatedHttpResponse, BodyFraming, HttpExchangeEvidence};
pub use exchange::HttpExchangePlan;
pub use field::RequestField;
pub use policy::{
    AlpnHttp11Policy, DEFAULT_MAX_ENCODED_CONTENT_BYTES, DEFAULT_MAX_HEADER_FIELD_COUNT,
    DEFAULT_MAX_HEADER_NAME_BYTES, DEFAULT_MAX_HEADER_SECTION_BYTES, DEFAULT_MAX_HEADER_VALUE_BYTES,
    DEFAULT_MAX_REQUEST_BYTES, DEFAULT_MAX_STATUS_LINE_BYTES, HttpClientPolicy,
    MAX_HTTP_EXCHANGE_TIMEOUT,
};
pub use request::HttpMethod;
pub use target::HttpRequestTarget;
