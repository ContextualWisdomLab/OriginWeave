#![allow(clippy::expect_used)]

use crate::chunked::{ChunkParseResult, parse_chunked_body};
use crate::disposition::{parse_content_disposition, parse_redirect_metadata};
use crate::field::{FieldBlock, FieldLine};
use crate::integrity::{validate_content_digest, validate_representation_digest};
use crate::mime::{MimeType, classify_observed_mime, no_sniff_status, supplied_mime_type};
use crate::response_head::{FinalHeadParseResult, parse_final_response_head, parse_response_head};
use crate::{ContentRiskClass, HttpClientPolicy, HttpError, IntegrityRequirement};

fn fields(entries: &[(&str, &[u8])]) -> FieldBlock {
    FieldBlock::new(
        entries
            .iter()
            .map(|(name, value)| {
                FieldLine::new(name.as_bytes(), value, 16_384, 32_768).expect("test field")
            })
            .collect(),
    )
}

fn observed(content: &[u8]) -> crate::ObservedMimeClassification {
    classify_observed_mime(content, None)
}

#[test]
fn disposition_rejects_ambiguous_and_malformed_metadata_boundaries() {
    assert!(matches!(
        parse_content_disposition(
            &fields(&[
                ("content-disposition", b"inline"),
                ("content-disposition", b"attachment"),
            ]),
            &observed(b"plain"),
        ),
        Err(HttpError::InvalidContentDisposition)
    ));

    for value in [
        b"attachment; filename".as_slice(),
        b"attachment; =x",
        b"attachment; file name=x",
        b"attachment; filename=",
        b"attachment; filename=\"\"",
        b"attachment; filename=\".\"",
        b"attachment; filename=\"..\"",
        br#"attachment; filename="trailing ""#,
        b"attachment; filename=AUX.txt",
        b"attachment; filename=COM1.txt",
        b"attachment; filename*=UTF-8",
        b"attachment; filename*=UTF-8'",
        b"attachment; filename*=UTF-8''%FF",
        b"attachment; filename*=UTF-8''bad%",
    ] {
        assert!(matches!(
            parse_content_disposition(
                &fields(&[("content-disposition", value)]),
                &observed(b"plain"),
            ),
            Err(HttpError::InvalidContentDisposition)
        ));
    }

    let raw_non_ascii = b"attachment; filename*=UTF-8''\xff";
    assert!(matches!(
        parse_content_disposition(
            &fields(&[("content-disposition", raw_non_ascii)]),
            &observed(b"plain"),
        ),
        Err(HttpError::InvalidContentDisposition)
    ));
}

#[test]
fn redirect_metadata_rejects_oversized_non_utf8_and_invalid_authorities() {
    let oversized = vec![b'a'; 8_193];
    for value in [
        b"https:///missing-host".as_slice(),
        b"ftp://example.com/path",
        b"https://example.com/a\tb",
        b"https://example.com/path#fragment",
        b"relative/path",
    ] {
        assert!(matches!(
            parse_redirect_metadata(308, &fields(&[("location", value)])),
            Err(HttpError::InvalidRedirectMetadata)
        ));
    }
    assert!(matches!(
        parse_redirect_metadata(301, &fields(&[("location", oversized.as_slice())])),
        Err(HttpError::InvalidRedirectMetadata)
    ));
    assert!(matches!(
        parse_redirect_metadata(303, &fields(&[("location", b"https://example.com/\xff")])),
        Err(HttpError::InvalidRedirectMetadata)
    ));

    let query_only =
        parse_redirect_metadata(300, &fields(&[("location", b"https://example.com?next=1")]))
            .expect("valid absolute redirect")
            .expect("metadata");
    assert!(!query_only.is_relative());
}

#[test]
fn mime_parser_rejects_every_ambiguous_syntax_class_and_classifies_signatures() {
    for value in [
        b"text".as_slice(),
        b"/plain",
        b"text/",
        b"text/plain/extra",
        b"text/plain; charset",
        b"text/plain; =utf-8",
        b"text/plain; char set=utf-8",
        b"text/plain; charset=",
        b"text/plain; charset=utf-8; CHARSET=ascii",
        b"text/plain; charset=\"unterminated",
        b"text/plain; charset=\"a\\\x01b\"",
        b"text/plain; charset=\xff",
    ] {
        assert!(matches!(
            MimeType::parse(value),
            Err(HttpError::InvalidMimeType)
        ));
    }

    assert!(matches!(
        supplied_mime_type(&fields(&[
            ("content-type", b"text/plain"),
            ("content-type", b"text/html"),
        ])),
        Err(HttpError::InvalidMimeType)
    ));
    assert!(matches!(
        no_sniff_status(&fields(&[("x-content-type-options", b"nosniff, other")])),
        Err(HttpError::InvalidNoSniffDirective)
    ));

    let javascript = MimeType::parse(b"text/javascript").expect("JS MIME");
    let signatures: &[(&[u8], Option<&MimeType>, &str, ContentRiskClass)] = &[
        (
            b"%PDF-1.7",
            None,
            "application/pdf",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (
            b"\x89PNG\r\n\x1a\nrest",
            None,
            "image/png",
            ContentRiskClass::Passive,
        ),
        (
            b"\xff\xd8\xffrest",
            None,
            "image/jpeg",
            ContentRiskClass::Passive,
        ),
        (b"GIF87arest", None, "image/gif", ContentRiskClass::Passive),
        (b"GIF89arest", None, "image/gif", ContentRiskClass::Passive),
        (
            b"RIFF\x04\x00\x00\x00WEBP",
            None,
            "image/webp",
            ContentRiskClass::Passive,
        ),
        (
            b"PK\x03\x04rest",
            None,
            "application/zip",
            ContentRiskClass::ArchiveOrContainer,
        ),
        (
            b"  <SvG xmlns='x'>",
            None,
            "image/svg+xml",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (
            b"\t<?XML version='1.0'?>",
            None,
            "text/xml",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (
            b"<!doctype HTML><html>",
            None,
            "text/html",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (
            b"const answer = 42;",
            Some(&javascript),
            "text/javascript",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (
            b"ordinary text",
            None,
            "text/plain",
            ContentRiskClass::Passive,
        ),
        (
            b"\x00\xff\x10",
            None,
            "application/octet-stream",
            ContentRiskClass::UnknownBinary,
        ),
    ];
    for (content, supplied, expected_essence, expected_risk) in signatures {
        let observed = classify_observed_mime(content, *supplied);
        assert_eq!(observed.mime_type().essence(), *expected_essence);
        assert_eq!(observed.risk_class(), *expected_risk);
    }
}

#[test]
fn request_target_rejects_invalid_second_percent_nibble() {
    let origin = originweave_core::Origin::parse("https://example.com").expect("origin");
    assert!(matches!(
        crate::HttpRequestTarget::parse(origin, "/%0G"),
        Err(HttpError::InvalidPercentEncoding { byte_index: 1 })
    ));
}

#[test]
fn response_head_rejects_status_line_line_ending_and_budget_edge_cases() {
    let policy = HttpClientPolicy::strict_defaults();
    assert!(matches!(
        parse_response_head(b"HTTP/1.1 200", &policy),
        Ok(crate::response_head::HeadParseResult::Incomplete)
    ));
    for response in [
        b"HTTX/1.1 200 OK\r\n\r\n".as_slice(),
        b"HTTP/1.0 200 OK\r\n\r\n",
        b"HTTP/1.1X200 OK\r\n\r\n",
        b"HTTP/1.1 2A0 OK\r\n\r\n",
        b"HTTP/1.1 200XOK\r\n\r\n",
        b"HTTP/1.1 099 Bad\r\n\r\n",
        b"HTTP/1.1 600 Bad\r\n\r\n",
        b"HTTP/1.1 200 bad\x00reason\r\n\r\n",
        b"HTTP/1.1 200 OK\n\n",
        b"HTTP/1.1 200 OK\rX\r\n",
        b"HTTP/1.1 200 OK\r\n folded: x\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nmissing-colon\r\n\r\n",
    ] {
        assert!(parse_response_head(response, &policy).is_err());
    }

    let mut oversized_status = b"HTTP/1.1 200 ".to_vec();
    oversized_status.extend(std::iter::repeat_n(
        b'a',
        policy.max_status_line_bytes() + 1,
    ));
    assert!(matches!(
        parse_response_head(&oversized_status, &policy),
        Err(HttpError::StatusLineTooLarge { .. })
    ));

    let oversized_section = vec![b'a'; policy.max_header_section_bytes() + 1];
    assert!(matches!(
        parse_response_head(&oversized_section, &policy),
        Err(HttpError::HeaderSectionTooLarge { .. })
    ));

    assert!(matches!(
        parse_final_response_head(b"HTTP/1.1 101 Switching Protocols\r\n\r\n", &policy),
        Err(HttpError::SwitchingProtocolsUnsupported)
    ));

    let mut too_many_interim = Vec::new();
    for _ in 0..=policy.max_interim_response_count() {
        too_many_interim.extend_from_slice(b"HTTP/1.1 100 Continue\r\n\r\n");
    }
    assert!(matches!(
        parse_final_response_head(&too_many_interim, &policy),
        Err(HttpError::ExcessiveInterimResponses { .. })
    ));

    assert!(matches!(
        parse_final_response_head(b"HTTP/1.1 200 OK\r", &policy),
        Ok(FinalHeadParseResult::Incomplete)
    ));
}

#[test]
fn chunked_parser_rejects_line_chunk_and_trailer_boundary_failures() {
    let policy = HttpClientPolicy::strict_defaults();
    assert!(matches!(
        parse_chunked_body(b"", &policy),
        Ok(ChunkParseResult::Incomplete)
    ));
    for body in [
        b"1\nA\r\n0\r\n\r\n".as_slice(),
        b"1\rXA\r\n0\r\n\r\n",
        b"Z\r\n",
        b"1\r\naX0\r\n\r\n",
        b"0\r\n bad: x\r\n\r\n",
        b"0\r\nmissing-colon\r\n\r\n",
        b"0\r\ncontent-length: 0\r\n\r\n",
    ] {
        assert!(parse_chunked_body(body, &policy).is_err());
    }

    let mut long_line = vec![b'a'; 1_026];
    long_line.extend_from_slice(b"\r\n");
    assert!(matches!(
        parse_chunked_body(&long_line, &policy),
        Err(HttpError::ChunkLineTooLarge { .. })
    ));

    let huge_hex = format!("{:x}\r\n", usize::MAX);
    assert!(matches!(
        parse_chunked_body(huge_hex.as_bytes(), &policy),
        Err(HttpError::EncodedContentTooLarge { .. })
    ));
}

#[test]
fn digest_parser_rejects_invalid_structured_fields_and_context_conflicts() {
    let empty = FieldBlock::default();
    assert!(matches!(
        validate_content_digest(
            &empty,
            &empty,
            b"payload",
            IntegrityRequirement::RequireSupportedDigest,
        ),
        Err(HttpError::SupportedDigestRequired)
    ));

    for value in [
        b"sha-256".as_slice(),
        b"Sha-256=:AQ==:",
        b"sha/256=:AQ==:",
        b"sha-256=AQ==",
        b"sha-256=:***:",
        b"=:AQ==:",
    ] {
        assert!(matches!(
            validate_content_digest(
                &fields(&[("content-digest", value)]),
                &empty,
                b"payload",
                IntegrityRequirement::Optional,
            ),
            Err(HttpError::InvalidDigestField)
        ));
    }

    assert!(matches!(
        validate_content_digest(
            &fields(&[("content-digest", b"sha-256=::")]),
            &empty,
            b"payload",
            IntegrityRequirement::Optional,
        ),
        Err(HttpError::DigestMismatch {
            algorithm: "sha-256"
        })
    ));

    let unsupported = validate_content_digest(
        &fields(&[("content-digest", b"md5=:AQ==:")]),
        &empty,
        b"payload",
        IntegrityRequirement::Optional,
    )
    .expect("syntactically valid unsupported digest");
    assert!(matches!(
        unsupported,
        crate::IntegrityStatus::UnsupportedAlgorithm
    ));

    assert!(matches!(
        validate_content_digest(
            &fields(&[("content-digest", b"md5=:AQ==:")]),
            &empty,
            b"payload",
            IntegrityRequirement::RequireSupportedDigest,
        ),
        Err(HttpError::SupportedDigestRequired)
    ));

    let representation = validate_representation_digest(
        &fields(&[("repr-digest", b"md5=:AQ==:")]),
        &empty,
        b"payload",
        206,
        true,
        IntegrityRequirement::Optional,
    )
    .expect("unsupported representation context");
    assert!(matches!(
        representation,
        crate::IntegrityStatus::UnsupportedContext
    ));

    assert!(matches!(
        validate_representation_digest(
            &fields(&[("repr-digest", b"sha-256")]),
            &empty,
            b"payload",
            200,
            false,
            IntegrityRequirement::Optional,
        ),
        Err(HttpError::InvalidDigestField)
    ));

    let merged_unsupported = validate_content_digest(
        &fields(&[("content-digest", b"md5=:AQ==:")]),
        &fields(&[("content-digest", b"other=:Ag==:")]),
        b"payload",
        IntegrityRequirement::Optional,
    )
    .expect("RFC 9530 permits merging digest trailer members into the header dictionary");
    assert_eq!(
        merged_unsupported,
        crate::IntegrityStatus::UnsupportedAlgorithm
    );
}
