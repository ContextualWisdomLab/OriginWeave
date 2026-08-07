use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use originweave_tls::{AuthenticatedTlsConnection, NegotiatedAlpn};
use rustls::{ClientConnection, StreamOwned};

use crate::evidence::EvidenceInput;
use crate::framing::determine_body_framing;
use crate::request::serialize_request;
use crate::response_head::parse_response_head;
use crate::{
    AlpnHttp11Policy, AuthenticatedHttpResponse, BodyFraming, HttpClientPolicy, HttpError,
    HttpMethod, HttpRequestTarget, RequestField,
};

type TlsStream = StreamOwned<ClientConnection, TcpStream>;

/// A single-use authority for one bounded HTTP/1.1 exchange.
#[derive(Debug)]
pub struct HttpExchangePlan {
    connection: AuthenticatedTlsConnection,
    method: HttpMethod,
    target: HttpRequestTarget,
    policy: HttpClientPolicy,
    request_bytes: Vec<u8>,
}

impl HttpExchangePlan {
    /// Validate one HTTP exchange plan before emitting application bytes.
    pub fn new(
        connection: AuthenticatedTlsConnection,
        method: HttpMethod,
        target: HttpRequestTarget,
        fields: impl IntoIterator<Item = RequestField>,
        policy: HttpClientPolicy,
    ) -> Result<Self, HttpError> {
        let tls_evidence = connection.evidence();
        if target.origin() != tls_evidence.origin() {
            return Err(HttpError::OriginAuthorityMismatch);
        }
        if tls_evidence.requested_peer() != tls_evidence.observed_peer() {
            return Err(HttpError::TlsPeerEvidenceMismatch);
        }
        validate_alpn(
            tls_evidence.negotiated_alpn(),
            tls_evidence.observed_peer(),
            policy.alpn_policy(),
        )?;
        let fields = fields.into_iter().collect::<Vec<_>>();
        let request_bytes = serialize_request(method, &target, &fields, &policy)?;
        Ok(Self {
            connection,
            method,
            target,
            policy,
            request_bytes,
        })
    }

    /// Perform the one-shot exchange on the already authenticated TLS stream.
    pub fn exchange(self) -> Result<AuthenticatedHttpResponse, HttpError> {
        let Self {
            connection,
            method,
            target,
            policy,
            request_bytes,
        } = self;
        let (mut stream, tls_evidence) = connection.into_parts();
        let started_at = Instant::now();
        let deadline = started_at + policy.exchange_timeout();

        write_request(
            &mut stream,
            request_bytes.as_slice(),
            deadline,
            policy.exchange_timeout(),
        )?;
        let (head_bytes, body_prefix) = read_response_head(
            &mut stream,
            &policy,
            deadline,
            policy.exchange_timeout(),
        )?;
        let head = parse_response_head(head_bytes.as_slice(), &policy)?;
        let framing = determine_body_framing(method, head.status_code, &head.fields, &policy)?;
        let content = read_response_content(
            &mut stream,
            framing,
            body_prefix,
            &policy,
            deadline,
            policy.exchange_timeout(),
        )?;
        enforce_deadline(deadline, policy.exchange_timeout())?;
        let evidence = EvidenceInput {
            origin: tls_evidence.origin().clone(),
            requested_peer: tls_evidence.requested_peer(),
            observed_peer: tls_evidence.observed_peer(),
            method,
            target_hash: target.target_hash().to_owned(),
            status_code: head.status_code,
            response_field_names: head.fields.into_iter().map(|field| field.name).collect(),
            body_framing: framing,
            encoded_content_bytes: content.len(),
            exchange_duration: started_at.elapsed(),
            exchange_timeout: policy.exchange_timeout(),
        }
        .into();
        Ok(AuthenticatedHttpResponse { content, evidence })
    }
}

fn validate_alpn(
    negotiated: &NegotiatedAlpn,
    peer: SocketAddr,
    policy: AlpnHttp11Policy,
) -> Result<(), HttpError> {
    match (negotiated, policy) {
        (NegotiatedAlpn::Protocol(protocol), _) if protocol.as_slice() == b"http/1.1" => Ok(()),
        (NegotiatedAlpn::Absent, AlpnHttp11Policy::PermitAbsentForManagedLoopback)
            if peer.ip().is_loopback() =>
        {
            Ok(())
        }
        _ => Err(HttpError::UnexpectedAlpn),
    }
}

fn write_request(
    stream: &mut TlsStream,
    request: &[u8],
    deadline: Instant,
    timeout: Duration,
) -> Result<(), HttpError> {
    let mut written = 0_usize;
    while written < request.len() {
        let remaining = remaining_time(deadline, timeout)?;
        update_timeout(stream.sock.set_write_timeout(Some(remaining)), "write timeout")?;
        let count = stream
            .write(request.get(written..).unwrap_or_default())
            .map_err(|source| classify_io(source, timeout, "request write"))?;
        enforce_deadline(deadline, timeout)?;
        if count == 0 {
            return Err(HttpError::Io {
                operation: "request write",
                source: io::Error::new(io::ErrorKind::WriteZero, "HTTP request write made no progress"),
            });
        }
        written = written.saturating_add(count);
    }
    let remaining = remaining_time(deadline, timeout)?;
    update_timeout(stream.sock.set_write_timeout(Some(remaining)), "flush timeout")?;
    stream
        .flush()
        .map_err(|source| classify_io(source, timeout, "request flush"))?;
    enforce_deadline(deadline, timeout)
}

fn read_response_head(
    stream: &mut TlsStream,
    policy: &HttpClientPolicy,
    deadline: Instant,
    timeout: Duration,
) -> Result<(Vec<u8>, Vec<u8>), HttpError> {
    let mut buffer = Vec::new();
    let mut scratch = [0_u8; 8_192];
    loop {
        if let Some(end) = find_head_end(buffer.as_slice()) {
            if end > policy.max_header_section_bytes() {
                return Err(HttpError::HeaderSectionTooLarge {
                    byte_count: end,
                    maximum_bytes: policy.max_header_section_bytes(),
                });
            }
            let body = buffer.split_off(end);
            return Ok((buffer, body));
        }
        if buffer.len() > policy.max_header_section_bytes() {
            return Err(HttpError::HeaderSectionTooLarge {
                byte_count: buffer.len(),
                maximum_bytes: policy.max_header_section_bytes(),
            });
        }
        let count = read_once(stream, &mut scratch, deadline, timeout, "response head read")?;
        if count == 0 {
            return Err(HttpError::IncompleteResponse);
        }
        buffer.extend_from_slice(scratch.get(..count).unwrap_or_default());
    }
}

fn read_response_content(
    stream: &mut TlsStream,
    framing: BodyFraming,
    mut prefix: Vec<u8>,
    policy: &HttpClientPolicy,
    deadline: Instant,
    timeout: Duration,
) -> Result<Vec<u8>, HttpError> {
    match framing {
        BodyFraming::NoContent => {
            if prefix.is_empty() {
                Ok(Vec::new())
            } else {
                Err(HttpError::InvalidResponseField)
            }
        }
        BodyFraming::ContentLength(length) => {
            let expected = usize::try_from(length).map_err(|_error| HttpError::ContentTooLarge {
                byte_count: usize::MAX,
                maximum_bytes: policy.max_encoded_content_bytes(),
            })?;
            if prefix.len() > expected {
                prefix.truncate(expected);
            }
            let mut scratch = [0_u8; 8_192];
            while prefix.len() < expected {
                let remaining_content = expected.saturating_sub(prefix.len());
                let read_limit = std::cmp::min(remaining_content, scratch.len());
                let count = read_once(
                    stream,
                    scratch.get_mut(..read_limit).unwrap_or_default(),
                    deadline,
                    timeout,
                    "response content read",
                )?;
                if count == 0 {
                    return Err(HttpError::IncompleteResponse);
                }
                prefix.extend_from_slice(scratch.get(..count).unwrap_or_default());
                if prefix.len() > policy.max_encoded_content_bytes() {
                    return Err(HttpError::ContentTooLarge {
                        byte_count: prefix.len(),
                        maximum_bytes: policy.max_encoded_content_bytes(),
                    });
                }
            }
            Ok(prefix)
        }
    }
}

fn read_once(
    stream: &mut TlsStream,
    output: &mut [u8],
    deadline: Instant,
    timeout: Duration,
    operation: &'static str,
) -> Result<usize, HttpError> {
    let remaining = remaining_time(deadline, timeout)?;
    update_timeout(stream.sock.set_read_timeout(Some(remaining)), "read timeout")?;
    let count = stream
        .read(output)
        .map_err(|source| classify_io(source, timeout, operation))?;
    enforce_deadline(deadline, timeout)?;
    Ok(count)
}

fn find_head_end(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn remaining_time(deadline: Instant, timeout: Duration) -> Result<Duration, HttpError> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        Err(HttpError::ExchangeTimedOut { timeout })
    } else {
        Ok(remaining)
    }
}

fn enforce_deadline(deadline: Instant, timeout: Duration) -> Result<(), HttpError> {
    remaining_time(deadline, timeout).map(|_remaining| ())
}

fn update_timeout(result: io::Result<()>, operation: &'static str) -> Result<(), HttpError> {
    result.map_err(|source| HttpError::Io { operation, source })
}

fn classify_io(source: io::Error, timeout: Duration, operation: &'static str) -> HttpError {
    if matches!(
        source.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    ) {
        HttpError::ExchangeTimedOut { timeout }
    } else {
        HttpError::Io { operation, source }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::net::{IpAddr, Ipv4Addr};

    use super::*;

    #[test]
    fn alpn_policy_is_exact_and_loopback_exception_is_narrow() {
        let loopback = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 443);
        let public = SocketAddr::from(([203, 0, 113, 1], 443));
        validate_alpn(
            &NegotiatedAlpn::Protocol(b"http/1.1".to_vec()),
            public,
            AlpnHttp11Policy::RequireHttp11,
        )
        .expect("HTTP/1.1 ALPN");
        validate_alpn(
            &NegotiatedAlpn::Absent,
            loopback,
            AlpnHttp11Policy::PermitAbsentForManagedLoopback,
        )
        .expect("managed loopback absence");
        for (alpn, peer, policy) in [
            (
                NegotiatedAlpn::Protocol(b"h2".to_vec()),
                public,
                AlpnHttp11Policy::RequireHttp11,
            ),
            (
                NegotiatedAlpn::Absent,
                public,
                AlpnHttp11Policy::PermitAbsentForManagedLoopback,
            ),
            (
                NegotiatedAlpn::Absent,
                loopback,
                AlpnHttp11Policy::RequireHttp11,
            ),
        ] {
            assert!(matches!(
                validate_alpn(&alpn, peer, policy),
                Err(HttpError::UnexpectedAlpn)
            ));
        }
    }

    #[test]
    fn head_boundary_and_deadline_helpers_are_total() {
        assert_eq!(find_head_end(b"HTTP/1.1 200 OK\r\n\r\nbody"), Some(19));
        assert_eq!(find_head_end(b"HTTP/1.1 200 OK\r\n"), None);
        let timeout = Duration::from_secs(1);
        assert!(remaining_time(Instant::now() + timeout, timeout).is_ok());
        assert!(matches!(
            remaining_time(Instant::now(), timeout),
            Err(HttpError::ExchangeTimedOut { .. })
        ));
        assert!(matches!(
            classify_io(io::Error::from(io::ErrorKind::TimedOut), timeout, "read"),
            HttpError::ExchangeTimedOut { .. }
        ));
        assert!(matches!(
            classify_io(io::Error::from(io::ErrorKind::ConnectionReset), timeout, "read"),
            HttpError::Io { .. }
        ));
        update_timeout(Ok(()), "timeout").expect("timeout update");
        assert!(matches!(
            update_timeout(Err(io::Error::other("failure")), "timeout"),
            Err(HttpError::Io { .. })
        ));
    }
}
