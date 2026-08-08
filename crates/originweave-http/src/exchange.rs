//! Single-use deadline-bound HTTP exchange orchestration over authenticated TLS.

use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use originweave_tls::{AuthenticatedTlsConnection, NegotiatedAlpn};

use crate::chunked::{
    ChunkParseResult, ChunkedResult, MAX_CHUNK_LINE_BYTES, parse_chunked_body,
};
use crate::content::{ContentCoding, decode_content};
use crate::disposition::{parse_content_disposition, parse_redirect_metadata};
use crate::evidence::{
    AuthenticatedHttpResponse, EvidenceInput, HttpExchangeEvidence, HttpResourceBudgets,
    ResponseFieldEvidence,
};
use crate::field::FieldBlock;
use crate::framing::{BodyFraming, determine_body_framing};
use crate::integrity::{validate_content_digest, validate_representation_digest};
use crate::mime::{classify_mismatch, classify_observed_mime, no_sniff_status, supplied_mime_type};
use crate::request::serialize_request;
use crate::response_head::{FinalHeadParseResult, ResponseHead, parse_final_response_head};
use crate::{
    AlpnHttp11Policy, HttpClientPolicy, HttpError, HttpMethod, HttpRequestTarget, RequestField,
};

const IO_BUFFER_BYTES: usize = 8 * 1_024;

/// A single-use HTTP/1.1 exchange bound to one already-authenticated TLS connection.
///
/// Construction validates the exact authority chain and serializes the complete request before
/// any HTTP bytes are emitted. Executing the plan consumes the authenticated stream, applies the
/// configured monotonic deadline and byte/count budgets, and returns only a complete validated
/// response. The plan never resolves DNS, opens or reconnects a socket, follows redirects, or
/// persists response content.
#[derive(Debug)]
pub struct HttpExchangePlan {
    connection: AuthenticatedTlsConnection,
    method: HttpMethod,
    target: HttpRequestTarget,
    request_bytes: Vec<u8>,
    policy: HttpClientPolicy,
}

impl HttpExchangePlan {
    /// Bind one read-only HTTP request to an authenticated TLS stream.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] when the target origin, current socket peer, inherited TLS evidence,
    /// ALPN evidence, caller fields, or serialized request violates the reviewed HTTP authority.
    pub fn new(
        connection: AuthenticatedTlsConnection,
        method: HttpMethod,
        target: HttpRequestTarget,
        fields: &[RequestField],
        policy: HttpClientPolicy,
    ) -> Result<Self, HttpError> {
        validate_transport_authority(&connection, &target, &policy).and_then(|()| {
            serialize_request(method, &target, fields, &policy).map(|request_bytes| Self {
                connection,
                method,
                target,
                request_bytes,
                policy,
            })
        })
    }

    /// Execute the bounded exchange on the exact authenticated stream.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] for deadline expiry, I/O failure, incomplete or ambiguous framing,
    /// budget exhaustion, integrity failure, unsafe metadata, or failure to restore the socket's
    /// original timeout configuration. No partial response is returned on failure.
    pub fn execute(mut self) -> Result<AuthenticatedHttpResponse, HttpError> {
        let timeout = self.policy.exchange_timeout();
        capture_timeouts(&self.connection).and_then(|(read_timeout, write_timeout)| {
            let started = Instant::now();
            let deadline = started + timeout;
            let result = self.execute_inner(started, deadline);
            let restoration = map_timeout_restoration(restore_timeouts(
                &mut self.connection,
                read_timeout,
                write_timeout,
            ));
            combine_exchange_and_restoration(result, restoration)
        })
    }

    fn execute_inner(
        &mut self,
        started: Instant,
        deadline: Instant,
    ) -> Result<AuthenticatedHttpResponse, HttpError> {
        write_request(
            &mut self.connection,
            &self.request_bytes,
            deadline,
            self.policy.exchange_timeout(),
        )?;
        let network = read_network_response(
            &mut self.connection,
            self.method,
            &self.policy,
            deadline,
            self.policy.exchange_timeout(),
        )?;
        let timeout = self.policy.exchange_timeout();
        ensure_before_deadline(deadline, timeout).and_then(|()| {
            let content_digest_status = validate_content_digest(
                &network.head.fields,
                &network.trailers,
                &network.encoded_content,
                self.policy.integrity_requirement(),
            )?;
            let has_content_range = !network.head.fields.values("content-range").is_empty();
            let representation_digest_status = validate_representation_digest(
                &network.head.fields,
                &network.trailers,
                &network.encoded_content,
                network.head.status_code,
                has_content_range,
                self.policy.integrity_requirement(),
            )?;
            let decoded = if matches!(network.framing, BodyFraming::NoContent) {
                crate::content::DecodedContent {
                    bytes: Vec::new(),
                    coding: ContentCoding::Identity,
                }
            } else {
                decode_content(&network.encoded_content, &network.head.fields, &self.policy)?
            };
            let supplied_mime = supplied_mime_type(&network.head.fields)?;
            let no_sniff_status = no_sniff_status(&network.head.fields)?;
            let observed_mime = classify_observed_mime(&decoded.bytes, supplied_mime.as_ref());
            let mime_mismatch = classify_mismatch(supplied_mime.as_ref(), &observed_mime);
            let content_disposition =
                parse_content_disposition(&network.head.fields, &observed_mime)?;
            let redirect = parse_redirect_metadata(network.head.status_code, &network.head.fields)?;

            ensure_before_deadline(deadline, timeout).map(|()| {
                let tls_evidence = self.connection.evidence();
                let evidence: HttpExchangeEvidence = EvidenceInput {
                    origin: tls_evidence.origin().clone(),
                    requested_peer: tls_evidence.requested_peer(),
                    observed_peer: tls_evidence.observed_peer(),
                    tls_protocol_version: tls_evidence.protocol_version(),
                    negotiated_alpn: tls_evidence.negotiated_alpn().clone(),
                    method: self.method,
                    target_hash: self.target.target_hash().to_owned(),
                    query_present: self.target.query_present(),
                    path_prefix: self.target.path_prefix().to_owned(),
                    status_code: network.head.status_code,
                    interim_response_count: network.interim_response_count,
                    response_fields: field_evidence(&network.head.fields),
                    body_framing: network.framing,
                    encoded_content_bytes: network.encoded_content.len(),
                    decoded_content_bytes: decoded.bytes.len(),
                    content_coding: decoded.coding,
                    chunk_count: network.chunk_count,
                    trailer_fields: field_evidence(&network.trailers),
                    content_digest_status,
                    representation_digest_status,
                    supplied_mime,
                    observed_mime,
                    no_sniff_status,
                    mime_mismatch,
                    content_disposition,
                    redirect,
                    exchange_duration: started.elapsed(),
                    resource_budgets: HttpResourceBudgets::from_policy(&self.policy),
                }
                .into();
                AuthenticatedHttpResponse {
                    content: decoded.bytes,
                    reason_phrase: network.head.reason_phrase().to_vec(),
                    evidence,
                }
            })
        })
    }
}

#[derive(Debug)]
struct NetworkResult {
    head: ResponseHead,
    encoded_content: Vec<u8>,
    trailers: FieldBlock,
    framing: BodyFraming,
    chunk_count: usize,
    interim_response_count: usize,
}

fn validate_transport_authority(
    connection: &AuthenticatedTlsConnection,
    target: &HttpRequestTarget,
    policy: &HttpClientPolicy,
) -> Result<(), HttpError> {
    let evidence = connection.evidence();
    if target.origin() != evidence.origin() {
        return Err(HttpError::OriginAuthorityMismatch {
            http_origin: target.origin().clone(),
            tls_origin: evidence.origin().clone(),
        });
    }
    peer_inspection(connection.stream().sock.peer_addr())
        .and_then(|current_peer| {
            validate_peer_evidence(
                evidence.requested_peer(),
                evidence.observed_peer(),
                current_peer,
            )
        })
        .and_then(|()| {
            validate_http11_alpn(
                policy.alpn_policy(),
                evidence.negotiated_alpn(),
                evidence.requested_peer(),
                evidence.observed_peer(),
            )
        })
}

fn peer_inspection(result: io::Result<SocketAddr>) -> Result<SocketAddr, HttpError> {
    match result {
        Ok(current_peer) => Ok(current_peer),
        Err(_error) => Err(HttpError::InvalidTransportEvidence),
    }
}

fn validate_peer_evidence(
    requested_peer: SocketAddr,
    observed_peer: SocketAddr,
    current_peer: SocketAddr,
) -> Result<(), HttpError> {
    if current_peer != observed_peer || requested_peer != observed_peer {
        Err(HttpError::InvalidTransportEvidence)
    } else {
        Ok(())
    }
}

fn validate_http11_alpn(
    policy: AlpnHttp11Policy,
    negotiated_alpn: &NegotiatedAlpn,
    requested_peer: SocketAddr,
    observed_peer: SocketAddr,
) -> Result<(), HttpError> {
    match (policy, negotiated_alpn) {
        (_, NegotiatedAlpn::Protocol(protocol)) if protocol.as_slice() == b"http/1.1" => Ok(()),
        (AlpnHttp11Policy::PermitAbsentForManagedLoopback, NegotiatedAlpn::Absent)
            if requested_peer.ip().is_loopback() && observed_peer.ip().is_loopback() =>
        {
            Ok(())
        }
        _other => Err(HttpError::UnexpectedAlpn),
    }
}

fn write_request(
    connection: &mut AuthenticatedTlsConnection,
    request: &[u8],
    deadline: Instant,
    timeout: Duration,
) -> Result<(), HttpError> {
    let mut written = 0_usize;
    while written < request.len() {
        let step = set_write_deadline(connection, deadline, timeout)
            .and_then(|()| {
                classify_write_result(connection.stream_mut().write(&request[written..]), timeout)
            })
            .and_then(|byte_count| ensure_before_deadline(deadline, timeout).map(|()| byte_count));
        let byte_count = step?;
        written = written.saturating_add(byte_count);
    }
    set_write_deadline(connection, deadline, timeout)
        .and_then(|()| classify_unit_io_result(connection.stream_mut().flush(), timeout))
        .and_then(|()| ensure_before_deadline(deadline, timeout))
}

fn classify_write_result(result: io::Result<usize>, timeout: Duration) -> Result<usize, HttpError> {
    match result {
        Ok(0) => Err(http_io_error(io::Error::new(
            io::ErrorKind::WriteZero,
            "TLS stream wrote zero request bytes",
        ))),
        Ok(byte_count) => Ok(byte_count),
        Err(error) => Err(classify_io_error(error, timeout)),
    }
}

fn classify_unit_io_result(result: io::Result<()>, timeout: Duration) -> Result<(), HttpError> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(classify_io_error(error, timeout)),
    }
}

fn read_network_response(
    connection: &mut AuthenticatedTlsConnection,
    method: HttpMethod,
    policy: &HttpClientPolicy,
    deadline: Instant,
    timeout: Duration,
) -> Result<NetworkResult, HttpError> {
    let (head, interim_response_count, body_prefix) =
        read_final_head(connection, policy, deadline, timeout)?;
    let framing = determine_body_framing(
        method,
        head.status_code,
        &head.fields,
        policy.max_encoded_content_bytes(),
    )?;
    match framing {
        BodyFraming::NoContent => {
            if !body_prefix.is_empty() {
                return Err(HttpError::UnexpectedResponseBytes {
                    byte_count: body_prefix.len(),
                });
            }
            Ok(NetworkResult {
                head,
                encoded_content: Vec::new(),
                trailers: FieldBlock::default(),
                framing,
                chunk_count: 0,
                interim_response_count,
            })
        }
        BodyFraming::ContentLength(expected) => {
            let encoded_content =
                read_exact_content(connection, body_prefix, expected, deadline, timeout)?;
            Ok(NetworkResult {
                head,
                encoded_content,
                trailers: FieldBlock::default(),
                framing,
                chunk_count: 0,
                interim_response_count,
            })
        }
        BodyFraming::Chunked => {
            let result = read_chunked_body(connection, body_prefix, policy, deadline, timeout)?;
            Ok(NetworkResult {
                head,
                encoded_content: result.content,
                trailers: result.trailers,
                framing,
                chunk_count: result.chunk_count,
                interim_response_count,
            })
        }
        BodyFraming::CloseDelimited => {
            let encoded_content = read_to_clean_eof_bounded(
                connection,
                body_prefix,
                policy.max_encoded_content_bytes(),
                deadline,
                timeout,
            )?;
            Ok(NetworkResult {
                head,
                encoded_content,
                trailers: FieldBlock::default(),
                framing,
                chunk_count: 0,
                interim_response_count,
            })
        }
    }
}

fn read_final_head(
    connection: &mut AuthenticatedTlsConnection,
    policy: &HttpClientPolicy,
    deadline: Instant,
    timeout: Duration,
) -> Result<(ResponseHead, usize, Vec<u8>), HttpError> {
    let mut buffer = Vec::new();
    loop {
        match parse_final_response_head(&buffer, policy)? {
            FinalHeadParseResult::Complete {
                head,
                consumed,
                interim_response_count,
            } => {
                let body_prefix = buffer.split_off(consumed);
                return Ok((head, interim_response_count, body_prefix));
            }
            FinalHeadParseResult::Incomplete => {}
        }
        let mut scratch = [0_u8; IO_BUFFER_BYTES];
        let byte_count = read_with_deadline(connection, &mut scratch, deadline, timeout)?;
        if byte_count == 0 {
            return Err(HttpError::IncompleteResponse);
        }
        buffer.extend_from_slice(&scratch[..byte_count]);
    }
}

fn read_exact_content(
    connection: &mut AuthenticatedTlsConnection,
    mut output: Vec<u8>,
    expected: usize,
    deadline: Instant,
    timeout: Duration,
) -> Result<Vec<u8>, HttpError> {
    if output.len() > expected {
        return Err(HttpError::UnexpectedResponseBytes {
            byte_count: output.len() - expected,
        });
    }
    while output.len() < expected {
        let remaining = expected - output.len();
        let mut scratch = [0_u8; IO_BUFFER_BYTES];
        let read_limit = remaining.min(scratch.len());
        let byte_count =
            read_with_deadline(connection, &mut scratch[..read_limit], deadline, timeout)?;
        if byte_count == 0 {
            return Err(HttpError::IncompleteResponse);
        }
        output.extend_from_slice(&scratch[..byte_count]);
    }
    Ok(output)
}

fn read_chunked_body(
    connection: &mut AuthenticatedTlsConnection,
    mut wire: Vec<u8>,
    policy: &HttpClientPolicy,
    deadline: Instant,
    timeout: Duration,
) -> Result<ChunkedResult, HttpError> {
    let maximum_wire_bytes = maximum_chunked_wire_bytes(policy);
    extend_chunked_wire(&mut wire, &[], maximum_wire_bytes)?;
    loop {
        match parse_chunked_body(&wire, policy)? {
            ChunkParseResult::Complete(result) => {
                if result.consumed != wire.len() {
                    return Err(HttpError::UnexpectedResponseBytes {
                        byte_count: wire.len() - result.consumed,
                    });
                }
                return Ok(result);
            }
            ChunkParseResult::Incomplete => {}
        }

        let remaining_capacity = maximum_wire_bytes.saturating_sub(wire.len());
        let mut scratch = [0_u8; IO_BUFFER_BYTES];
        let read_limit = remaining_capacity.saturating_add(1).min(scratch.len());
        let byte_count = require_read_progress(read_with_deadline(
            connection,
            &mut scratch[..read_limit],
            deadline,
            timeout,
        )?)?;
        extend_chunked_wire(
            &mut wire,
            &scratch[..byte_count],
            maximum_wire_bytes,
        )?;
    }
}

pub(crate) fn maximum_chunked_wire_bytes(policy: &HttpClientPolicy) -> usize {
    let per_chunk_overhead = MAX_CHUNK_LINE_BYTES.saturating_add(4);
    policy
        .max_encoded_content_bytes()
        .saturating_add(policy.max_chunk_count().saturating_mul(per_chunk_overhead))
        .saturating_add(policy.max_trailer_section_bytes())
        .saturating_add(MAX_CHUNK_LINE_BYTES)
        .saturating_add(4)
}

pub(crate) fn extend_chunked_wire(
    wire: &mut Vec<u8>,
    bytes: &[u8],
    maximum: usize,
) -> Result<(), HttpError> {
    let next_len = wire.len().saturating_add(bytes.len());
    if next_len > maximum {
        return Err(HttpError::EncodedContentTooLarge {
            byte_count: u64::try_from(next_len).unwrap_or(u64::MAX),
            maximum_bytes: maximum,
        });
    }
    wire.extend_from_slice(bytes);
    Ok(())
}

fn read_to_clean_eof_bounded(
    connection: &mut AuthenticatedTlsConnection,
    mut output: Vec<u8>,
    maximum: usize,
    deadline: Instant,
    timeout: Duration,
) -> Result<Vec<u8>, HttpError> {
    if output.len() > maximum {
        return Err(HttpError::EncodedContentTooLarge {
            byte_count: u64::try_from(output.len()).unwrap_or(u64::MAX),
            maximum_bytes: maximum,
        });
    }
    loop {
        let remaining_capacity = maximum.saturating_sub(output.len());
        let mut scratch = [0_u8; IO_BUFFER_BYTES];
        let read_limit = if remaining_capacity == 0 {
            1
        } else {
            remaining_capacity.min(scratch.len())
        };
        let byte_count =
            read_with_deadline(connection, &mut scratch[..read_limit], deadline, timeout)?;
        if byte_count == 0 {
            return Ok(output);
        }
        if remaining_capacity == 0 {
            return Err(HttpError::EncodedContentTooLarge {
                byte_count: u64::try_from(maximum.saturating_add(byte_count)).unwrap_or(u64::MAX),
                maximum_bytes: maximum,
            });
        }
        output.extend_from_slice(&scratch[..byte_count]);
    }
}

fn read_with_deadline(
    connection: &mut AuthenticatedTlsConnection,
    output: &mut [u8],
    deadline: Instant,
    timeout: Duration,
) -> Result<usize, HttpError> {
    set_read_deadline(connection, deadline, timeout).and_then(|()| {
        let result = classify_read_result(connection.stream_mut().read(output), timeout);
        ensure_before_deadline(deadline, timeout).and(result)
    })
}

fn require_read_progress(byte_count: usize) -> Result<usize, HttpError> {
    if byte_count == 0 {
        Err(HttpError::IncompleteResponse)
    } else {
        Ok(byte_count)
    }
}

fn classify_read_result(result: io::Result<usize>, timeout: Duration) -> Result<usize, HttpError> {
    match result {
        Ok(byte_count) => Ok(byte_count),
        Err(error) => Err(classify_read_error(error, timeout)),
    }
}

fn set_read_deadline(
    connection: &mut AuthenticatedTlsConnection,
    deadline: Instant,
    timeout: Duration,
) -> Result<(), HttpError> {
    remaining_duration(deadline, timeout).and_then(|remaining| {
        map_timeout_update(
            connection
                .stream_mut()
                .sock
                .set_read_timeout(Some(remaining)),
        )
    })
}

fn set_write_deadline(
    connection: &mut AuthenticatedTlsConnection,
    deadline: Instant,
    timeout: Duration,
) -> Result<(), HttpError> {
    remaining_duration(deadline, timeout).and_then(|remaining| {
        map_timeout_update(
            connection
                .stream_mut()
                .sock
                .set_write_timeout(Some(remaining)),
        )
    })
}

fn remaining_duration(deadline: Instant, timeout: Duration) -> Result<Duration, HttpError> {
    remaining_duration_at(deadline, Instant::now(), timeout)
}

fn remaining_duration_at(
    deadline: Instant,
    now: Instant,
    timeout: Duration,
) -> Result<Duration, HttpError> {
    deadline
        .checked_duration_since(now)
        .filter(|remaining| !remaining.is_zero())
        .ok_or(HttpError::HttpExchangeTimedOut { timeout })
}

fn ensure_before_deadline(deadline: Instant, timeout: Duration) -> Result<(), HttpError> {
    ensure_before_deadline_at(deadline, Instant::now(), timeout)
}

fn ensure_before_deadline_at(
    deadline: Instant,
    now: Instant,
    timeout: Duration,
) -> Result<(), HttpError> {
    if now >= deadline {
        Err(HttpError::HttpExchangeTimedOut { timeout })
    } else {
        Ok(())
    }
}

fn classify_read_error(error: io::Error, timeout: Duration) -> HttpError {
    if matches!(
        error.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    ) {
        HttpError::HttpExchangeTimedOut { timeout }
    } else if error.kind() == io::ErrorKind::UnexpectedEof {
        HttpError::IncompleteResponse
    } else {
        http_io_error(error)
    }
}

fn classify_io_error(error: io::Error, timeout: Duration) -> HttpError {
    if matches!(
        error.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    ) {
        HttpError::HttpExchangeTimedOut { timeout }
    } else {
        http_io_error(error)
    }
}

fn http_io_error(source: io::Error) -> HttpError {
    HttpError::HttpExchangeIoFailed { source }
}

fn capture_timeouts(
    connection: &AuthenticatedTlsConnection,
) -> Result<(Option<Duration>, Option<Duration>), HttpError> {
    map_timeout_query(connection.stream().sock.read_timeout()).and_then(|read_timeout| {
        map_timeout_query(connection.stream().sock.write_timeout())
            .map(|write_timeout| (read_timeout, write_timeout))
    })
}

fn map_timeout_query(result: io::Result<Option<Duration>>) -> Result<Option<Duration>, HttpError> {
    match result {
        Ok(timeout) => Ok(timeout),
        Err(source) => Err(http_io_error(source)),
    }
}

fn map_timeout_update(result: io::Result<()>) -> Result<(), HttpError> {
    match result {
        Ok(()) => Ok(()),
        Err(source) => Err(http_io_error(source)),
    }
}

fn map_timeout_restoration(result: io::Result<()>) -> Result<(), HttpError> {
    match result {
        Ok(()) => Ok(()),
        Err(source) => Err(HttpError::TimeoutRestorationFailed { source }),
    }
}

pub(crate) fn combine_exchange_and_restoration<T>(
    exchange: Result<T, HttpError>,
    restoration: Result<(), HttpError>,
) -> Result<T, HttpError> {
    match exchange {
        Ok(value) => restoration.map(|()| value),
        Err(error) => Err(error),
    }
}

fn restore_timeouts(
    connection: &mut AuthenticatedTlsConnection,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
) -> io::Result<()> {
    connection
        .stream_mut()
        .sock
        .set_read_timeout(read_timeout)
        .and_then(|()| {
            connection
                .stream_mut()
                .sock
                .set_write_timeout(write_timeout)
        })
}

fn field_evidence(fields: &FieldBlock) -> Vec<ResponseFieldEvidence> {
    fields
        .iter()
        .map(|field| ResponseFieldEvidence::new(field.name().to_owned(), field.value().len()))
        .collect()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::expect_used)]

    use std::mem::discriminant;

    use super::*;

    fn socket(octets: [u8; 4]) -> SocketAddr {
        SocketAddr::from((octets, 443))
    }

    fn assert_variant(actual: &HttpError, expected: &HttpError) {
        assert_eq!(discriminant(actual), discriminant(expected));
    }

    fn io_failure() -> HttpError {
        HttpError::HttpExchangeIoFailed {
            source: io::Error::other("expected HTTP I/O failure"),
        }
    }

    #[test]
    fn timeout_result_mappers_are_total_and_fail_closed() {
        let timeout = Some(Duration::from_secs(1));
        assert_eq!(
            map_timeout_query(Ok(timeout)).expect("timeout query"),
            timeout
        );
        let query_error =
            map_timeout_query(Err(io::Error::other("query"))).expect_err("query failure");
        assert_variant(&query_error, &io_failure());

        map_timeout_update(Ok(())).expect("timeout update");
        let update_error =
            map_timeout_update(Err(io::Error::other("update"))).expect_err("update failure");
        assert_variant(&update_error, &io_failure());

        map_timeout_restoration(Ok(())).expect("timeout restoration");
        let restoration_error = map_timeout_restoration(Err(io::Error::other("restore")))
            .expect_err("restoration failure");
        assert_variant(
            &restoration_error,
            &HttpError::TimeoutRestorationFailed {
                source: io::Error::other("expected"),
            },
        );
    }

    #[test]
    fn peer_evidence_and_inspection_are_exact() {
        let loopback = socket([127, 0, 0, 1]);
        let other = socket([127, 0, 0, 2]);
        assert_eq!(peer_inspection(Ok(loopback)).expect("peer"), loopback);
        let inspection_error =
            peer_inspection(Err(io::Error::other("peer"))).expect_err("peer failure");
        assert_variant(&inspection_error, &HttpError::InvalidTransportEvidence);

        validate_peer_evidence(loopback, loopback, loopback).expect("consistent peer evidence");
        for (requested, observed, current) in [
            (loopback, loopback, other),
            (other, loopback, loopback),
            (other, loopback, other),
        ] {
            let error = validate_peer_evidence(requested, observed, current)
                .expect_err("peer mismatch must fail");
            assert_variant(&error, &HttpError::InvalidTransportEvidence);
        }
    }

    #[test]
    fn http11_alpn_policy_covers_protocol_and_managed_loopback_paths() {
        let loopback = socket([127, 0, 0, 1]);
        let public = socket([203, 0, 113, 1]);
        validate_http11_alpn(
            AlpnHttp11Policy::RequireHttp11,
            &NegotiatedAlpn::Protocol(b"http/1.1".to_vec()),
            public,
            public,
        )
        .expect("HTTP/1.1 ALPN");
        let wrong_protocol = validate_http11_alpn(
            AlpnHttp11Policy::RequireHttp11,
            &NegotiatedAlpn::Protocol(b"h2".to_vec()),
            public,
            public,
        )
        .expect_err("wrong protocol");
        assert_variant(&wrong_protocol, &HttpError::UnexpectedAlpn);

        validate_http11_alpn(
            AlpnHttp11Policy::PermitAbsentForManagedLoopback,
            &NegotiatedAlpn::Absent,
            loopback,
            loopback,
        )
        .expect("managed loopback absence");
        for (policy, requested, observed) in [
            (AlpnHttp11Policy::RequireHttp11, loopback, loopback),
            (
                AlpnHttp11Policy::PermitAbsentForManagedLoopback,
                public,
                loopback,
            ),
            (
                AlpnHttp11Policy::PermitAbsentForManagedLoopback,
                loopback,
                public,
            ),
        ] {
            let error = validate_http11_alpn(policy, &NegotiatedAlpn::Absent, requested, observed)
                .expect_err("unauthorized ALPN absence");
            assert_variant(&error, &HttpError::UnexpectedAlpn);
        }
    }

    #[test]
    fn write_and_unit_io_results_preserve_progress_and_classify_failures() {
        let timeout = Duration::from_secs(1);
        assert_eq!(
            classify_write_result(Ok(7), timeout).expect("write progress"),
            7
        );
        let zero = classify_write_result(Ok(0), timeout).expect_err("zero write");
        assert_variant(&zero, &io_failure());
        for kind in [io::ErrorKind::TimedOut, io::ErrorKind::WouldBlock] {
            let error = classify_write_result(Err(io::Error::from(kind)), timeout)
                .expect_err("write timeout");
            assert_variant(&error, &HttpError::HttpExchangeTimedOut { timeout });
        }
        let write_failure = classify_write_result(
            Err(io::Error::from(io::ErrorKind::ConnectionReset)),
            timeout,
        )
        .expect_err("write failure");
        assert_variant(&write_failure, &io_failure());

        classify_unit_io_result(Ok(()), timeout).expect("unit I/O success");
        let unit_timeout =
            classify_unit_io_result(Err(io::Error::from(io::ErrorKind::TimedOut)), timeout)
                .expect_err("unit I/O timeout");
        assert_variant(&unit_timeout, &HttpError::HttpExchangeTimedOut { timeout });
        let unit_failure =
            classify_unit_io_result(Err(io::Error::from(io::ErrorKind::BrokenPipe)), timeout)
                .expect_err("unit I/O failure");
        assert_variant(&unit_failure, &io_failure());
    }

    #[test]
    fn deadline_helpers_cover_future_equal_and_elapsed_instants() {
        let timeout = Duration::from_secs(1);
        let now = Instant::now();
        let deadline = now + timeout;
        assert_eq!(
            remaining_duration_at(deadline, now, timeout).expect("remaining time"),
            timeout
        );
        ensure_before_deadline_at(deadline, now, timeout).expect("future deadline");
        for observed in [deadline, deadline + Duration::from_nanos(1)] {
            let remaining_error = remaining_duration_at(deadline, observed, timeout)
                .expect_err("elapsed remaining time");
            assert_variant(
                &remaining_error,
                &HttpError::HttpExchangeTimedOut { timeout },
            );
            let deadline_error = ensure_before_deadline_at(deadline, observed, timeout)
                .expect_err("elapsed deadline");
            assert_variant(
                &deadline_error,
                &HttpError::HttpExchangeTimedOut { timeout },
            );
        }
    }

    #[test]
    fn required_response_reads_reject_clean_eof_and_preserve_progress() {
        let eof = require_read_progress(0).expect_err("clean EOF");
        assert_variant(&eof, &HttpError::IncompleteResponse);
        assert_eq!(require_read_progress(3).expect("read progress"), 3);
    }

    #[test]
    fn read_and_general_io_classification_is_complete() {
        let timeout = Duration::from_secs(1);
        assert_eq!(
            classify_read_result(Ok(3), timeout).expect("read success"),
            3
        );
        for kind in [io::ErrorKind::TimedOut, io::ErrorKind::WouldBlock] {
            let error = classify_read_result(Err(io::Error::from(kind)), timeout)
                .expect_err("read timeout");
            assert_variant(&error, &HttpError::HttpExchangeTimedOut { timeout });
            assert_variant(
                &classify_io_error(io::Error::from(kind), timeout),
                &HttpError::HttpExchangeTimedOut { timeout },
            );
        }
        let eof = classify_read_result(Err(io::Error::from(io::ErrorKind::UnexpectedEof)), timeout)
            .expect_err("read EOF");
        assert_variant(&eof, &HttpError::IncompleteResponse);
        let ordinary = classify_read_result(
            Err(io::Error::from(io::ErrorKind::ConnectionReset)),
            timeout,
        )
        .expect_err("read failure");
        assert_variant(&ordinary, &io_failure());
        assert_variant(
            &classify_io_error(io::Error::from(io::ErrorKind::BrokenPipe), timeout),
            &io_failure(),
        );
    }
}
