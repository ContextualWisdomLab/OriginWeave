use std::io::Write;
use std::time::{Duration, Instant};

use originweave_tls::{AuthenticatedTlsConnection, NegotiatedAlpn, TlsConnectionEvidence};

use crate::error::{HttpError, io_result, timeout_result};
use crate::request::HttpRequest;
use crate::response::{HttpExchangePolicy, HttpResponse, parse_response_until};

/// One complete, non-reusable HTTP/1.1 exchange and its inherited TLS evidence.
#[derive(Debug)]
pub struct HttpExchange {
    response: HttpResponse,
    tls_evidence: TlsConnectionEvidence,
    duration: std::time::Duration,
}

impl HttpExchange {
    /// Execute one bounded request over the already authenticated TLS stream.
    ///
    /// The stream is consumed and never returned for connection reuse. This
    /// method performs no DNS resolution, socket connection, proxy selection,
    /// redirect following, content persistence, or browser authorization.
    pub fn execute(
        connection: AuthenticatedTlsConnection,
        request: &HttpRequest,
        policy: &HttpExchangePolicy,
    ) -> Result<Self, HttpError> {
        if connection.evidence().origin() != request.origin() {
            return Err(HttpError::RequestAuthorityMismatch);
        }
        match connection.evidence().negotiated_alpn() {
            NegotiatedAlpn::Protocol(protocol) if protocol.as_slice() == b"http/1.1" => {}
            NegotiatedAlpn::Protocol(_) => return Err(HttpError::UnexpectedAlpn),
            NegotiatedAlpn::Absent if policy.allow_absent_alpn() => {}
            NegotiatedAlpn::Absent => return Err(HttpError::AbsentAlpnNotPermitted),
        }

        let request_bytes = request.serialize();
        let started = Instant::now();
        let deadline = started + policy.exchange_timeout();
        let (mut stream, tls_evidence) = connection.into_parts();
        let response = timeout_result(stream.sock.read_timeout(), "read timeout inspection")
            .and_then(|original_read_timeout| {
                timeout_result(stream.sock.write_timeout(), "write timeout inspection")
                    .and_then(|original_write_timeout| {
                        remaining(deadline).and_then(|write_remaining| {
                            timeout_result(
                                stream.sock.set_write_timeout(Some(write_remaining)),
                                "write timeout configuration",
                            )
                            .and_then(|()| {
                                io_result(stream.write_all(&request_bytes), "write HTTP request")
                                    .and_then(|()| {
                                        timeout_result(
                                            stream.sock.set_write_timeout(original_write_timeout),
                                            "write timeout cleanup",
                                        )
                                        .and_then(|()| {
                                            remaining(deadline).and_then(|read_remaining| {
                                                timeout_result(
                                                    stream.sock.set_read_timeout(Some(read_remaining)),
                                                    "read timeout configuration",
                                                )
                                                .and_then(|()| {
                                                    parse_response_until(
                                                        &mut stream,
                                                        request.method(),
                                                        policy,
                                                        Some(deadline),
                                                    )
                                                    .and_then(|response| {
                                                        timeout_result(
                                                            stream
                                                                .sock
                                                                .set_read_timeout(original_read_timeout),
                                                            "read timeout cleanup",
                                                        )
                                                        .map(|()| response)
                                                    })
                                                })
                                            })
                                        })
                                    })
                            })
                        })
                    })
            });
        response.map(|response| Self {
            response,
            tls_evidence,
            duration: started.elapsed(),
        })
    }

    /// Return the complete bounded response.
    #[must_use]
    #[inline(never)]
    pub const fn response(&self) -> &HttpResponse {
        &self.response
    }

    /// Return the immutable TLS evidence inherited by this exchange.
    #[must_use]
    #[inline(never)]
    pub const fn tls_evidence(&self) -> &TlsConnectionEvidence {
        &self.tls_evidence
    }

    /// Return the measured monotonic exchange duration.
    #[must_use]
    #[inline(never)]
    pub const fn duration(&self) -> std::time::Duration {
        self.duration
    }
}

fn remaining(deadline: Instant) -> Result<std::time::Duration, HttpError> {
    remaining_at(deadline, Instant::now())
}

fn remaining_at(deadline: Instant, now: Instant) -> Result<Duration, HttpError> {
    let Some(duration) = deadline.checked_duration_since(now) else {
        return Err(HttpError::ExchangeTimedOut);
    };
    if duration.is_zero() {
        Err(HttpError::ExchangeTimedOut)
    } else {
        Ok(duration)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::expect_used)]
mod tests {
    use std::io::{self, Read, Write};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
    use std::sync::Arc;
    use std::thread;

    use super::*;
    use originweave_core::Origin;
    use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
    use originweave_network::ConnectionPlan;
    use originweave_tls::{
        AlpnRequirement, TlsClientPolicy, TlsHandshakePlan, TrustBundleIdentifier, TrustRootBundle,
    };
    use rcgen::{
        BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer,
        KeyPair, KeyUsagePurpose,
    };
    use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer, UnixTime};
    use rustls::{ServerConfig, ServerConnection, StreamOwned};

    const TRUSTED_TIME_SECONDS: u64 = 1_767_225_600;

    #[test]
    fn covers_deadline_and_socket_error_mapping() {
        let now = Instant::now();
        assert!(matches!(
            remaining_at(now - Duration::from_secs(1), now),
            Err(HttpError::ExchangeTimedOut)
        ));
        assert!(matches!(
            remaining_at(now, now),
            Err(HttpError::ExchangeTimedOut)
        ));
        assert!(matches!(
            remaining_at(now + Duration::from_secs(1), now),
            Ok(value) if value == Duration::from_secs(1)
        ));

        let timeout = timeout_result::<()>(
            Err(io::Error::other("timeout configuration")),
            "test timeout",
        )
        .expect_err("timeout mapping");
        assert!(matches!(timeout, HttpError::TimeoutConfiguration { .. }));
        let io_error =
            io_result::<()>(Err(io::Error::other("I/O")), "test I/O").expect_err("I/O mapping");
        assert!(matches!(io_error, HttpError::Io { .. }));
    }

    #[test]
    fn executes_one_real_exchange_in_the_library_test_target() {
        let mut root_parameters = CertificateParams::new(Vec::new()).expect("root SANs");
        root_parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        root_parameters.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyCertSign,
        ];
        root_parameters
            .distinguished_name
            .push(DnType::CommonName, "OriginWeave unit root");
        let root_key = KeyPair::generate().expect("root key");
        let root_certificate = root_parameters
            .self_signed(&root_key)
            .expect("root certificate");
        let issuer = Issuer::new(root_parameters, root_key);
        let mut leaf_parameters =
            CertificateParams::new(vec!["localhost".to_owned()]).expect("leaf SANs");
        leaf_parameters.not_before = rcgen::date_time_ymd(2025, 1, 1);
        leaf_parameters.not_after = rcgen::date_time_ymd(2030, 1, 1);
        leaf_parameters.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        leaf_parameters.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        let leaf_key = KeyPair::generate().expect("leaf key");
        let leaf_certificate = leaf_parameters
            .signed_by(&leaf_key, &issuer)
            .expect("leaf certificate");

        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .expect("listener");
        let address = listener.local_addr().expect("address");
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let server_builder = ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .expect("TLS versions");
        let mut server_config = server_builder
            .with_no_client_auth()
            .with_single_cert(
                vec![leaf_certificate.der().clone()],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
            )
            .expect("server certificate");
        server_config.alpn_protocols = vec![b"http/1.1".to_vec()];
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("accept");
            let mut connection = ServerConnection::new(Arc::new(server_config)).expect("server");
            connection
                .complete_io(&mut socket)
                .expect("server handshake");
            let mut stream = StreamOwned::new(connection, socket);
            let mut request = [0_u8; 512];
            let read = stream.read(&mut request).expect("request");
            assert!(read > 0);
            stream
                .write_all(b"HTTP/1.1 204 No Content\r\n\r\n")
                .expect("response");
        });

        let origin =
            Origin::parse(&format!("https://localhost:{}", address.port())).expect("origin");
        let snapshot = ResolutionSnapshot::approve(
            origin.clone(),
            [address.ip()],
            &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
        )
        .expect("resolution");
        let tcp = ConnectionPlan::new(&snapshot, address, Duration::from_secs(2), 1)
            .expect("connection plan")
            .connect()
            .expect("TCP connection");
        let roots = TrustRootBundle::new(
            TrustBundleIdentifier::parse("originweave_http_unit:v1").expect("bundle id"),
            vec![root_certificate.der().to_vec()],
        )
        .expect("trust roots");
        let tls_policy = TlsClientPolicy::new(
            UnixTime::since_unix_epoch(Duration::from_secs(TRUSTED_TIME_SECONDS)),
            Duration::from_secs(3),
            vec![b"http/1.1".to_vec()],
            AlpnRequirement::Required,
        )
        .expect("TLS policy");
        let connection = TlsHandshakePlan::new(origin.clone(), tcp, roots, tls_policy)
            .expect("TLS plan")
            .authenticate()
            .expect("TLS authentication");
        let request = HttpRequest::new(
            crate::request::HttpMethod::Get,
            origin,
            crate::request::HttpRequestTarget::parse("/").expect("target"),
        )
        .expect("request");
        let exchange = HttpExchange::execute(
            connection,
            &request,
            &HttpExchangePolicy::new(Duration::from_secs(2), 64).expect("HTTP policy"),
        )
        .expect("HTTP exchange");
        assert_eq!(exchange.response().status_code(), 204);
        server.join().expect("server thread");
    }
}
