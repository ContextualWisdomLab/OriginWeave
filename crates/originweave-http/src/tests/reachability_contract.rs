#![allow(clippy::expect_used)]

use std::time::Duration;

use originweave_core::Origin;

use crate::chunked::{ChunkParseResult, MAX_CHUNK_LINE_BYTES, parse_chunked_body};
use crate::disposition::{parse_content_disposition, parse_redirect_metadata};
use crate::field::{FieldBlock, FieldLine};
use crate::integrity::validate_content_digest;
use crate::mime::{MimeType, observe_mime_type};
use crate::response_head::{HeadParseResult, parse_response_head};
use crate::{
    AlpnHttp11Policy, HttpClientPolicy, HttpError, HttpRequestTarget, IntegrityRequirement,
};

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

fn observed_text() -> crate::ObservedMimeClassification {
    observe_mime_type(b"plain text", None).expect("observed text MIME")
}

fn response_policy(max_status_line_bytes: usize, max_header_section_bytes: usize) -> HttpClientPolicy {
    HttpClientPolicy::new(
        Duration::from_secs(1),
        1_024,
        max_status_line_bytes,
        8,
        64,
        256,
        max_header_section_bytes,
        2,
        16,
        8,
        128,
        1_024,
        1_024,
        4,
        AlpnHttp11Policy::RequireHttp11,
        IntegrityRequirement::Optional,
    )
    .expect("response test policy")
}

#[test]
fn request_target_rejects_invalid_second_hex_digit_and_delete() {
    let origin = Origin::parse("https://example.com").expect("origin");
    assert!(matches!(
        HttpRequestTarget::parse(origin.clone(), "/%0G"),
        Err(HttpError::InvalidPercentEncoding { byte_index: 1 })
    ));
    assert!(matches!(
        HttpRequestTarget::parse(origin, "/\x7f"),
        Err(HttpError::InvalidRequestTarget)
    ));
}

#[test]
fn disposition_rejects_quoted_and_extended_filename_edge_classes() {
    let observed = observed_text();
    for value in [
        b"attachment; filename=\"a\"x".as_slice(),
        b"attachment; filename=\"a\\\x7fb\"",
        b"attachment; filename=\"a\x7fb\"",
        b"attachment; filename=\"safe.\"",
        b"attachment; filename=\"a:b\"",
        b"attachment; filename*=UTF-8''%0G",
    ] {
        assert!(matches!(
            parse_content_disposition(&fields(&[("content-disposition", value)]), &observed),
            Err(HttpError::InvalidContentDisposition)
        ));
    }

    let unicode_control = "attachment; filename=\"a\u{0080}b\"";
    assert!(matches!(
        parse_content_disposition(
            &fields(&[("content-disposition", unicode_control.as_bytes())]),
            &observed,
        ),
        Err(HttpError::InvalidContentDisposition)
    ));

    let quoted_semicolon = parse_content_disposition(
        &fields(&[(
            "content-disposition",
            b"attachment; filename=\"a;b.txt\"",
        )]),
        &observed,
    )
    .expect("quoted semicolon is valid")
    .expect("disposition");
    assert_eq!(quoted_semicolon.filename(), Some("a;b.txt"));
}

#[test]
fn redirect_metadata_rejects_delete_and_fragment_bytes() {
    for value in [b"/next\x7f".as_slice(), b"/next#fragment"] {
        assert!(matches!(
            parse_redirect_metadata(302, &fields(&[("location", value)])),
            Err(HttpError::InvalidRedirectMetadata)
        ));
    }
}

#[test]
fn mime_parameters_and_remaining_zip_signatures_are_bounded() {
    let quoted_semicolon =
        MimeType::parse(b"text/plain; title=\"a;b\"").expect("quoted semicolon parameter");
    assert_eq!(quoted_semicolon.parameters()[0].1, "a;b");

    for invalid in [
        b"text/plain; title=\"a\"x".as_slice(),
        b"text/plain; title=\"a\x01b\"",
    ] {
        assert!(matches!(
            MimeType::parse(invalid),
            Err(HttpError::InvalidMimeType)
        ));
    }

    assert_eq!(
        observe_mime_type(b"PK\x05\x06rest", None)
            .expect("empty ZIP signature")
            .mime_type()
            .essence(),
        "application/zip"
    );
    assert_eq!(
        observe_mime_type(b"PK\x07\x08rest", None)
            .expect("spanned ZIP signature")
            .mime_type()
            .essence(),
        "application/zip"
    );
}

#[test]
fn digest_dictionary_rejects_short_value_and_digit_started_key() {
    for value in [b"sha-256=:".as_slice(), b"1sha=:AQ==:"] {
        assert!(matches!(
            validate_content_digest(
                &fields(&[("content-digest", value)]),
                &FieldBlock::default(),
                b"payload",
                IntegrityRequirement::Optional,
            ),
            Err(HttpError::InvalidDigestField)
        ));
    }
}

#[test]
fn response_header_scanner_enforces_crossing_budget_and_non_status_tail() {
    assert!(matches!(
        parse_response_head(
            b"HTTP/1.1 200 OK\r\n\r\n",
            &response_policy(64, 16),
        ),
        Err(HttpError::HeaderSectionTooLarge {
            byte_count: 17,
            maximum_bytes: 16,
        })
    ));

    assert_eq!(
        parse_response_head(
            b"HTTP/1.1 200 OK\r\nX",
            &response_policy(15, 64),
        )
        .expect("incomplete field tail"),
        HeadParseResult::Incomplete
    );
}

#[test]
fn chunk_line_limit_is_enforced_at_exact_unterminated_boundary() {
    let input = vec![b'1'; MAX_CHUNK_LINE_BYTES + 1];
    assert!(matches!(
        parse_chunked_body(&input, &HttpClientPolicy::strict_defaults()),
        Err(HttpError::ChunkLineTooLarge {
            byte_count,
            maximum_bytes: MAX_CHUNK_LINE_BYTES,
        }) if byte_count == MAX_CHUNK_LINE_BYTES + 1
    ));
    assert_eq!(
        parse_chunked_body(b"", &HttpClientPolicy::strict_defaults())
            .expect("empty chunk prefix"),
        ChunkParseResult::Incomplete
    );
}
