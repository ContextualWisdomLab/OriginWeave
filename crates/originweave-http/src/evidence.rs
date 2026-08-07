use std::net::SocketAddr;
use std::time::Duration;

use originweave_core::Origin;

use crate::HttpMethod;

/// Framing decision admitted by the fixed-length HTTP/1.1 vertical slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyFraming {
    /// HTTP semantics suppress response content.
    NoContent,
    /// The response declared one unambiguous content length.
    ContentLength(u64),
}

/// Credential-free evidence for one complete bounded HTTP exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpExchangeEvidence {
    origin: Origin,
    requested_peer: SocketAddr,
    observed_peer: SocketAddr,
    method: HttpMethod,
    target_hash: String,
    status_code: u16,
    response_field_names: Vec<String>,
    body_framing: BodyFraming,
    encoded_content_bytes: usize,
    exchange_duration: Duration,
    exchange_timeout: Duration,
    response_complete: bool,
}

impl HttpExchangeEvidence {
    /// Return the authenticated canonical origin inherited from TLS evidence.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the requested TCP peer inherited from TLS evidence.
    #[must_use]
    pub const fn requested_peer(&self) -> SocketAddr {
        self.requested_peer
    }

    /// Return the observed operating-system TCP peer inherited from TLS evidence.
    #[must_use]
    pub const fn observed_peer(&self) -> SocketAddr {
        self.observed_peer
    }

    /// Return the admitted request method.
    #[must_use]
    pub const fn method(&self) -> HttpMethod {
        self.method
    }

    /// Return the domain-separated request-target hash without query disclosure.
    #[must_use]
    pub const fn target_hash(&self) -> &str {
        self.target_hash.as_str()
    }

    /// Return the HTTP status code.
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Return ordered lowercase response field names without values.
    #[must_use]
    pub const fn response_field_names(&self) -> &[String] {
        self.response_field_names.as_slice()
    }

    /// Return the fail-closed body framing decision.
    #[must_use]
    pub const fn body_framing(&self) -> BodyFraming {
        self.body_framing
    }

    /// Return the complete encoded content byte count.
    #[must_use]
    pub const fn encoded_content_bytes(&self) -> usize {
        self.encoded_content_bytes
    }

    /// Return the measured monotonic exchange duration.
    #[must_use]
    pub const fn exchange_duration(&self) -> Duration {
        self.exchange_duration
    }

    /// Return the configured total exchange timeout.
    #[must_use]
    pub const fn exchange_timeout(&self) -> Duration {
        self.exchange_timeout
    }

    /// Return whether framing and all configured budgets completed successfully.
    #[must_use]
    pub const fn response_complete(&self) -> bool {
        self.response_complete
    }
}

pub(crate) struct EvidenceInput {
    pub(crate) origin: Origin,
    pub(crate) requested_peer: SocketAddr,
    pub(crate) observed_peer: SocketAddr,
    pub(crate) method: HttpMethod,
    pub(crate) target_hash: String,
    pub(crate) status_code: u16,
    pub(crate) response_field_names: Vec<String>,
    pub(crate) body_framing: BodyFraming,
    pub(crate) encoded_content_bytes: usize,
    pub(crate) exchange_duration: Duration,
    pub(crate) exchange_timeout: Duration,
}

impl From<EvidenceInput> for HttpExchangeEvidence {
    fn from(input: EvidenceInput) -> Self {
        Self {
            origin: input.origin,
            requested_peer: input.requested_peer,
            observed_peer: input.observed_peer,
            method: input.method,
            target_hash: input.target_hash,
            status_code: input.status_code,
            response_field_names: input.response_field_names,
            body_framing: input.body_framing,
            encoded_content_bytes: input.encoded_content_bytes,
            exchange_duration: input.exchange_duration,
            exchange_timeout: input.exchange_timeout,
            response_complete: true,
        }
    }
}

/// One complete bounded HTTP response and immutable credential-free evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedHttpResponse {
    pub(crate) content: Vec<u8>,
    pub(crate) evidence: HttpExchangeEvidence,
}

impl AuthenticatedHttpResponse {
    /// Borrow the complete response content.
    #[must_use]
    pub const fn content(&self) -> &[u8] {
        self.content.as_slice()
    }

    /// Borrow immutable exchange evidence.
    #[must_use]
    pub const fn evidence(&self) -> &HttpExchangeEvidence {
        &self.evidence
    }

    /// Consume the response into content and evidence.
    #[must_use]
    pub fn into_parts(self) -> (Vec<u8>, HttpExchangeEvidence) {
        (self.content, self.evidence)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn evidence_input_marks_only_complete_responses_successful() {
        let origin = Origin::parse("https://example.com").expect("origin");
        let peer = "127.0.0.1:443".parse().expect("peer");
        let evidence: HttpExchangeEvidence = EvidenceInput {
            origin: origin.clone(),
            requested_peer: peer,
            observed_peer: peer,
            method: HttpMethod::Get,
            target_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_owned(),
            status_code: 200,
            response_field_names: vec!["content-length".to_owned()],
            body_framing: BodyFraming::ContentLength(3),
            encoded_content_bytes: 3,
            exchange_duration: Duration::from_millis(1),
            exchange_timeout: Duration::from_secs(1),
        }
        .into();
        assert_eq!(evidence.origin(), &origin);
        assert_eq!(evidence.requested_peer(), peer);
        assert_eq!(evidence.observed_peer(), peer);
        assert_eq!(evidence.method(), HttpMethod::Get);
        assert_eq!(evidence.status_code(), 200);
        assert_eq!(evidence.response_field_names(), ["content-length"]);
        assert_eq!(evidence.body_framing(), BodyFraming::ContentLength(3));
        assert_eq!(evidence.encoded_content_bytes(), 3);
        assert!(evidence.response_complete());
    }
}
