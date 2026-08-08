//! Credential-free immutable evidence for one complete HTTP exchange.

use std::net::SocketAddr;
use std::time::Duration;

use originweave_core::Origin;
use originweave_tls::{NegotiatedAlpn, TlsProtocolVersion};

use crate::disposition::{RedirectMetadata, SafeContentDisposition};
use crate::framing::BodyFraming;
use crate::integrity::IntegrityStatus;
use crate::mime::{MimeMismatch, MimeType, NoSniffStatus, ObservedMimeClassification};
use crate::{ContentCoding, HttpClientPolicy, HttpMethod};

/// A non-sensitive response field name and its value byte count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseFieldEvidence {
    name: String,
    value_byte_count: usize,
}

impl ResponseFieldEvidence {
    pub(crate) fn new(name: String, value_byte_count: usize) -> Self {
        Self {
            name,
            value_byte_count,
        }
    }

    /// Return the normalized lowercase field name.
    #[must_use]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Return the field-value byte count without exposing the value.
    #[must_use]
    pub const fn value_byte_count(&self) -> usize {
        self.value_byte_count
    }
}

/// The complete reviewed resource budgets applied to one exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpResourceBudgets {
    exchange_timeout: Duration,
    max_request_bytes: usize,
    max_status_line_bytes: usize,
    max_header_field_count: usize,
    max_header_name_bytes: usize,
    max_header_value_bytes: usize,
    max_header_section_bytes: usize,
    max_interim_response_count: usize,
    max_chunk_count: usize,
    max_trailer_field_count: usize,
    max_trailer_section_bytes: usize,
    max_encoded_content_bytes: usize,
    max_decoded_content_bytes: usize,
    max_content_expansion_ratio: usize,
}

impl HttpResourceBudgets {
    pub(crate) fn from_policy(policy: &HttpClientPolicy) -> Self {
        Self {
            exchange_timeout: policy.exchange_timeout(),
            max_request_bytes: policy.max_request_bytes(),
            max_status_line_bytes: policy.max_status_line_bytes(),
            max_header_field_count: policy.max_header_field_count(),
            max_header_name_bytes: policy.max_header_name_bytes(),
            max_header_value_bytes: policy.max_header_value_bytes(),
            max_header_section_bytes: policy.max_header_section_bytes(),
            max_interim_response_count: policy.max_interim_response_count(),
            max_chunk_count: policy.max_chunk_count(),
            max_trailer_field_count: policy.max_trailer_field_count(),
            max_trailer_section_bytes: policy.max_trailer_section_bytes(),
            max_encoded_content_bytes: policy.max_encoded_content_bytes(),
            max_decoded_content_bytes: policy.max_decoded_content_bytes(),
            max_content_expansion_ratio: policy.max_content_expansion_ratio(),
        }
    }

    /// Return the total monotonic exchange timeout.
    #[must_use]
    pub const fn exchange_timeout(&self) -> Duration {
        self.exchange_timeout
    }

    /// Return the maximum serialized request bytes.
    #[must_use]
    pub const fn max_request_bytes(&self) -> usize {
        self.max_request_bytes
    }

    /// Return the maximum response status-line bytes.
    #[must_use]
    pub const fn max_status_line_bytes(&self) -> usize {
        self.max_status_line_bytes
    }

    /// Return the maximum response field count.
    #[must_use]
    pub const fn max_header_field_count(&self) -> usize {
        self.max_header_field_count
    }

    /// Return the maximum response field-name bytes.
    #[must_use]
    pub const fn max_header_name_bytes(&self) -> usize {
        self.max_header_name_bytes
    }

    /// Return the maximum response field-value bytes.
    #[must_use]
    pub const fn max_header_value_bytes(&self) -> usize {
        self.max_header_value_bytes
    }

    /// Return the maximum response header-section bytes.
    #[must_use]
    pub const fn max_header_section_bytes(&self) -> usize {
        self.max_header_section_bytes
    }

    /// Return the maximum informational response count.
    #[must_use]
    pub const fn max_interim_response_count(&self) -> usize {
        self.max_interim_response_count
    }

    /// Return the maximum chunk count.
    #[must_use]
    pub const fn max_chunk_count(&self) -> usize {
        self.max_chunk_count
    }

    /// Return the maximum trailer field count.
    #[must_use]
    pub const fn max_trailer_field_count(&self) -> usize {
        self.max_trailer_field_count
    }

    /// Return the maximum trailer-section bytes.
    #[must_use]
    pub const fn max_trailer_section_bytes(&self) -> usize {
        self.max_trailer_section_bytes
    }

    /// Return the maximum encoded response-content bytes.
    #[must_use]
    pub const fn max_encoded_content_bytes(&self) -> usize {
        self.max_encoded_content_bytes
    }

    /// Return the maximum decoded response-content bytes.
    #[must_use]
    pub const fn max_decoded_content_bytes(&self) -> usize {
        self.max_decoded_content_bytes
    }

    /// Return the maximum decoded-to-encoded expansion ratio.
    #[must_use]
    pub const fn max_content_expansion_ratio(&self) -> usize {
        self.max_content_expansion_ratio
    }
}

/// Credential-free evidence for one complete authenticated HTTP/1.1 exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpExchangeEvidence {
    origin: Origin,
    requested_peer: SocketAddr,
    observed_peer: SocketAddr,
    tls_protocol_version: TlsProtocolVersion,
    negotiated_alpn: NegotiatedAlpn,
    method: HttpMethod,
    target_hash: String,
    query_present: bool,
    path_prefix: String,
    status_code: u16,
    interim_response_count: usize,
    response_fields: Vec<ResponseFieldEvidence>,
    body_framing: BodyFraming,
    encoded_content_bytes: usize,
    decoded_content_bytes: usize,
    content_coding: ContentCoding,
    chunk_count: usize,
    trailer_fields: Vec<ResponseFieldEvidence>,
    content_digest_status: IntegrityStatus,
    representation_digest_status: IntegrityStatus,
    supplied_mime: Option<MimeType>,
    observed_mime: ObservedMimeClassification,
    no_sniff_status: NoSniffStatus,
    mime_mismatch: MimeMismatch,
    content_disposition: Option<SafeContentDisposition>,
    redirect: Option<RedirectMetadata>,
    response_complete: bool,
    exchange_duration: Duration,
    resource_budgets: HttpResourceBudgets,
}

impl HttpExchangeEvidence {
    /// Return the authenticated canonical origin.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the exact requested TCP peer inherited from TLS evidence.
    #[must_use]
    pub const fn requested_peer(&self) -> SocketAddr {
        self.requested_peer
    }

    /// Return the exact observed TCP peer inherited from TLS evidence.
    #[must_use]
    pub const fn observed_peer(&self) -> SocketAddr {
        self.observed_peer
    }

    /// Return the authenticated TLS protocol version.
    #[must_use]
    pub const fn tls_protocol_version(&self) -> TlsProtocolVersion {
        self.tls_protocol_version
    }

    /// Return the TLS ALPN evidence used by the HTTP authority.
    #[must_use]
    pub const fn negotiated_alpn(&self) -> &NegotiatedAlpn {
        &self.negotiated_alpn
    }

    /// Return the read-only request method.
    #[must_use]
    pub const fn method(&self) -> HttpMethod {
        self.method
    }

    /// Return the SHA-256 identifier of the exact request target.
    #[must_use]
    pub const fn target_hash(&self) -> &str {
        self.target_hash.as_str()
    }

    /// Return whether the exact request target included a query component.
    #[must_use]
    pub const fn query_present(&self) -> bool {
        self.query_present
    }

    /// Return the bounded request path prefix retained without query values.
    #[must_use]
    pub const fn path_prefix(&self) -> &str {
        self.path_prefix.as_str()
    }

    /// Return the final HTTP status code.
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Return the number of informational responses consumed first.
    #[must_use]
    pub const fn interim_response_count(&self) -> usize {
        self.interim_response_count
    }

    /// Return ordered response field names and byte counts.
    #[must_use]
    pub const fn response_fields(&self) -> &[ResponseFieldEvidence] {
        self.response_fields.as_slice()
    }

    /// Return the selected HTTP body framing.
    #[must_use]
    pub const fn body_framing(&self) -> BodyFraming {
        self.body_framing
    }

    /// Return the transfer-decoded, content-encoded byte count.
    #[must_use]
    pub const fn encoded_content_bytes(&self) -> usize {
        self.encoded_content_bytes
    }

    /// Return the content-decoded byte count exposed to the caller.
    #[must_use]
    pub const fn decoded_content_bytes(&self) -> usize {
        self.decoded_content_bytes
    }

    /// Return the selected content coding.
    #[must_use]
    pub const fn content_coding(&self) -> ContentCoding {
        self.content_coding
    }

    /// Return the number of parsed chunks including the zero chunk.
    #[must_use]
    pub const fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    /// Return ordered trailer field names and byte counts.
    #[must_use]
    pub const fn trailer_fields(&self) -> &[ResponseFieldEvidence] {
        self.trailer_fields.as_slice()
    }

    /// Return the RFC 9530 content-digest status.
    #[must_use]
    pub const fn content_digest_status(&self) -> &IntegrityStatus {
        &self.content_digest_status
    }

    /// Return the RFC 9530 representation-digest status.
    #[must_use]
    pub const fn representation_digest_status(&self) -> &IntegrityStatus {
        &self.representation_digest_status
    }

    /// Return the supplied Content-Type metadata, when valid and present.
    #[must_use]
    pub const fn supplied_mime(&self) -> Option<&MimeType> {
        self.supplied_mime.as_ref()
    }

    /// Return the bounded observed MIME classification.
    #[must_use]
    pub const fn observed_mime(&self) -> &ObservedMimeClassification {
        &self.observed_mime
    }

    /// Return the explicit no-sniff status.
    #[must_use]
    pub const fn no_sniff_status(&self) -> NoSniffStatus {
        self.no_sniff_status
    }

    /// Return the supplied-versus-observed MIME relation.
    #[must_use]
    pub const fn mime_mismatch(&self) -> MimeMismatch {
        self.mime_mismatch
    }

    /// Return safe Content-Disposition metadata, when present.
    #[must_use]
    pub const fn content_disposition(&self) -> Option<&SafeContentDisposition> {
        self.content_disposition.as_ref()
    }

    /// Return redirect metadata without following the target.
    #[must_use]
    pub const fn redirect(&self) -> Option<&RedirectMetadata> {
        self.redirect.as_ref()
    }

    /// Return whether every framing and validation step completed.
    #[must_use]
    pub const fn response_complete(&self) -> bool {
        self.response_complete
    }

    /// Return measured monotonic exchange duration.
    #[must_use]
    pub const fn exchange_duration(&self) -> Duration {
        self.exchange_duration
    }

    /// Return every reviewed resource budget applied to the exchange.
    #[must_use]
    pub const fn resource_budgets(&self) -> &HttpResourceBudgets {
        &self.resource_budgets
    }
}

pub(crate) struct EvidenceInput {
    pub(crate) origin: Origin,
    pub(crate) requested_peer: SocketAddr,
    pub(crate) observed_peer: SocketAddr,
    pub(crate) tls_protocol_version: TlsProtocolVersion,
    pub(crate) negotiated_alpn: NegotiatedAlpn,
    pub(crate) method: HttpMethod,
    pub(crate) target_hash: String,
    pub(crate) query_present: bool,
    pub(crate) path_prefix: String,
    pub(crate) status_code: u16,
    pub(crate) interim_response_count: usize,
    pub(crate) response_fields: Vec<ResponseFieldEvidence>,
    pub(crate) body_framing: BodyFraming,
    pub(crate) encoded_content_bytes: usize,
    pub(crate) decoded_content_bytes: usize,
    pub(crate) content_coding: ContentCoding,
    pub(crate) chunk_count: usize,
    pub(crate) trailer_fields: Vec<ResponseFieldEvidence>,
    pub(crate) content_digest_status: IntegrityStatus,
    pub(crate) representation_digest_status: IntegrityStatus,
    pub(crate) supplied_mime: Option<MimeType>,
    pub(crate) observed_mime: ObservedMimeClassification,
    pub(crate) no_sniff_status: NoSniffStatus,
    pub(crate) mime_mismatch: MimeMismatch,
    pub(crate) content_disposition: Option<SafeContentDisposition>,
    pub(crate) redirect: Option<RedirectMetadata>,
    pub(crate) exchange_duration: Duration,
    pub(crate) resource_budgets: HttpResourceBudgets,
}

impl From<EvidenceInput> for HttpExchangeEvidence {
    fn from(input: EvidenceInput) -> Self {
        Self {
            origin: input.origin,
            requested_peer: input.requested_peer,
            observed_peer: input.observed_peer,
            tls_protocol_version: input.tls_protocol_version,
            negotiated_alpn: input.negotiated_alpn,
            method: input.method,
            target_hash: input.target_hash,
            query_present: input.query_present,
            path_prefix: input.path_prefix,
            status_code: input.status_code,
            interim_response_count: input.interim_response_count,
            response_fields: input.response_fields,
            body_framing: input.body_framing,
            encoded_content_bytes: input.encoded_content_bytes,
            decoded_content_bytes: input.decoded_content_bytes,
            content_coding: input.content_coding,
            chunk_count: input.chunk_count,
            trailer_fields: input.trailer_fields,
            content_digest_status: input.content_digest_status,
            representation_digest_status: input.representation_digest_status,
            supplied_mime: input.supplied_mime,
            observed_mime: input.observed_mime,
            no_sniff_status: input.no_sniff_status,
            mime_mismatch: input.mime_mismatch,
            content_disposition: input.content_disposition,
            redirect: input.redirect,
            response_complete: true,
            exchange_duration: input.exchange_duration,
            resource_budgets: input.resource_budgets,
        }
    }
}

/// One complete decoded HTTP response and its immutable evidence.
#[derive(Debug)]
pub struct AuthenticatedHttpResponse {
    pub(crate) content: Vec<u8>,
    pub(crate) reason_phrase: Vec<u8>,
    pub(crate) evidence: HttpExchangeEvidence,
}

impl AuthenticatedHttpResponse {
    /// Borrow the complete decoded response content.
    #[must_use]
    pub const fn content(&self) -> &[u8] {
        self.content.as_slice()
    }

    /// Borrow the exact untrusted HTTP/1.1 reason-phrase octets for diagnostics.
    ///
    /// The reason phrase never drives HTTP semantics and is deliberately excluded from the
    /// credential-free evidence record because servers can place arbitrary octets in it.
    #[must_use]
    pub const fn reason_phrase(&self) -> &[u8] {
        self.reason_phrase.as_slice()
    }

    /// Borrow the immutable credential-free exchange evidence.
    #[must_use]
    pub const fn evidence(&self) -> &HttpExchangeEvidence {
        &self.evidence
    }

    /// Borrow safe redirect metadata without following the target.
    #[must_use]
    pub const fn redirect(&self) -> Option<&RedirectMetadata> {
        self.evidence.redirect()
    }

    /// Borrow the supplied MIME metadata, when present.
    #[must_use]
    pub const fn supplied_mime(&self) -> Option<&MimeType> {
        self.evidence.supplied_mime()
    }

    /// Borrow the bounded observed MIME classification.
    #[must_use]
    pub const fn observed_mime(&self) -> &ObservedMimeClassification {
        self.evidence.observed_mime()
    }

    /// Borrow safe Content-Disposition metadata, when present.
    #[must_use]
    pub const fn disposition(&self) -> Option<&SafeContentDisposition> {
        self.evidence.content_disposition()
    }

    /// Consume the response and return content, reason phrase, and immutable evidence.
    #[must_use]
    pub fn into_parts(self) -> (Vec<u8>, Vec<u8>, HttpExchangeEvidence) {
        (self.content, self.reason_phrase, self.evidence)
    }
}
