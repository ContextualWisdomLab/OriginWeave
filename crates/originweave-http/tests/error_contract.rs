use std::error::Error as _;
use std::io;
use std::time::Duration;

use originweave_core::Origin;
use originweave_http::HttpError;

fn origin(host: &str) -> Origin {
    Origin::parse(host).expect("test origin")
}

#[test]
fn every_public_http_error_has_a_nonempty_operator_message() {
    let errors = vec![
        HttpError::InvalidExchangeTimeout {
            timeout: Duration::ZERO,
            maximum_timeout: Duration::from_secs(1),
        },
        HttpError::InvalidPolicyLimit {
            limit_name: "max_request_bytes",
            value: 0,
            maximum: 1,
        },
        HttpError::InvalidExpansionRatio {
            ratio: 0,
            maximum_ratio: 4,
        },
        HttpError::InvalidRequestTarget,
        HttpError::InvalidPercentEncoding { byte_index: 3 },
        HttpError::RequestTargetTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::InvalidRequestFieldName,
        HttpError::InvalidRequestFieldValue,
        HttpError::RequestFieldNameTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::RequestFieldValueTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::ForbiddenRequestField {
            field_name: "authorization".to_owned(),
        },
        HttpError::DuplicateRequestField {
            field_name: "accept".to_owned(),
        },
        HttpError::ExcessiveRequestFieldCount {
            field_count: 2,
            maximum_count: 1,
        },
        HttpError::RequestTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::OriginAuthorityMismatch {
            http_origin: origin("https://example.test"),
            tls_origin: origin("https://other.test"),
        },
        HttpError::InvalidTransportEvidence,
        HttpError::UnexpectedResponseBytes { byte_count: 1 },
        HttpError::UnexpectedAlpn,
        HttpError::InvalidResponseStatusLine,
        HttpError::UnsupportedHttpVersion,
        HttpError::StatusLineTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::InvalidResponseLineEnding,
        HttpError::HeaderSectionTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::ExcessiveResponseFieldCount {
            field_count: 2,
            maximum_count: 1,
        },
        HttpError::InvalidResponseFieldName,
        HttpError::InvalidResponseFieldValue,
        HttpError::ResponseFieldNameTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::ResponseFieldValueTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::ObsoleteFieldFolding,
        HttpError::ExcessiveInterimResponses {
            response_count: 2,
            maximum_count: 1,
        },
        HttpError::SwitchingProtocolsUnsupported,
        HttpError::TransferEncodingWithContentLength,
        HttpError::UnsupportedTransferCoding,
        HttpError::InvalidContentLength,
        HttpError::ConflictingContentLength,
        HttpError::EncodedContentTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::MalformedChunkedBody,
        HttpError::ChunkLineTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::ExcessiveChunkCount {
            chunk_count: 2,
            maximum_count: 1,
        },
        HttpError::InvalidTrailerSection,
        HttpError::ExcessiveTrailerFieldCount {
            field_count: 2,
            maximum_count: 1,
        },
        HttpError::TrailerSectionTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::IncompleteResponse,
        HttpError::UnsupportedContentCoding,
        HttpError::ContentDecodingFailed {
            source: io::Error::new(io::ErrorKind::InvalidData, "decode"),
        },
        HttpError::DecodedContentTooLarge {
            byte_count: 2,
            maximum_bytes: 1,
        },
        HttpError::ContentExpansionRatioExceeded {
            decoded_bytes: 8,
            encoded_bytes: 1,
            maximum_ratio: 4,
        },
        HttpError::InvalidDigestField,
        HttpError::DigestMismatch {
            algorithm: "sha-256",
        },
        HttpError::SupportedDigestRequired,
        HttpError::InvalidMimeType,
        HttpError::InvalidContentDisposition,
        HttpError::InvalidRedirectMetadata,
        HttpError::HttpExchangeTimedOut {
            timeout: Duration::from_secs(1),
        },
        HttpError::HttpExchangeIoFailed {
            source: io::Error::new(io::ErrorKind::ConnectionReset, "exchange"),
        },
        HttpError::TimeoutRestorationFailed {
            source: io::Error::other("restore"),
        },
    ];

    for error in errors {
        assert!(!error.to_string().is_empty(), "{error:?}");
    }
}

#[test]
fn http_error_sources_are_exposed_only_for_wrapped_io_failures() {
    for error in [
        HttpError::ContentDecodingFailed {
            source: io::Error::new(io::ErrorKind::InvalidData, "decode"),
        },
        HttpError::HttpExchangeIoFailed {
            source: io::Error::new(io::ErrorKind::ConnectionReset, "exchange"),
        },
        HttpError::TimeoutRestorationFailed {
            source: io::Error::other("restore"),
        },
    ] {
        assert!(error.source().is_some());
    }

    assert!(HttpError::InvalidRequestTarget.source().is_none());
}
