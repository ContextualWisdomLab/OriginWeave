#![allow(clippy::expect_used)]

use crate::chunked::parse_chunked_body;
use crate::disposition::parse_content_disposition;
use crate::field::{FieldBlock, FieldLine};
use crate::integrity::validate_content_digest;
use crate::mime::{MimeType, observe_mime_type};
use crate::response_head::{FinalHeadParseResult, parse_final_response_head, parse_response_head};
use crate::{HttpClientPolicy, HttpError, IntegrityRequirement, IntegrityStatus};

fn fields(entries: &[(&str, &[u8])]) -> FieldBlock {
    FieldBlock::new(
        entries
            .iter()
            .map(|(name, value)| FieldLine::new(name.as_bytes(), value, 256, 8_192).expect("field"))
            .collect(),
    )
}

#[test]
fn content_disposition_absence_duplicates_and_dot_names_are_explicit() {
    let observed = observe_mime_type(b"plain text", None).expect("observed MIME");
    assert_eq!(
        parse_content_disposition(&FieldBlock::default(), &observed).expect("absent disposition"),
        None
    );
    assert!(matches!(
        parse_content_disposition(
            &fields(&[
                ("content-disposition", b"inline"),
                ("content-disposition", b"attachment"),
            ]),
            &observed,
        ),
        Err(HttpError::InvalidContentDisposition)
    ));
    assert!(matches!(
        parse_content_disposition(&fields(&[("content-disposition", b"form-data")]), &observed,),
        Err(HttpError::InvalidContentDisposition)
    ));
    for filename in [".", ".."] {
        let value = format!("attachment; filename=\"{filename}\"");
        assert!(matches!(
            parse_content_disposition(
                &fields(&[("content-disposition", value.as_bytes())]),
                &observed,
            ),
            Err(HttpError::InvalidContentDisposition)
        ));
    }
}

#[test]
fn extended_filename_requires_both_rfc5987_separators() {
    let observed = observe_mime_type(b"plain text", None).expect("observed MIME");
    for value in [
        b"attachment; filename*=UTF-8".as_slice(),
        b"attachment; filename*=UTF-8'en",
    ] {
        assert!(matches!(
            parse_content_disposition(&fields(&[("content-disposition", value)]), &observed),
            Err(HttpError::InvalidContentDisposition)
        ));
    }
}

#[test]
fn every_filename_extension_mapping_is_exercised_before_download_handoff() {
    let observed = observe_mime_type(b"plain text", None).expect("observed MIME");
    for filename in [
        "page.html",
        "page.htm",
        "feed.xml",
        "icon.svg",
        "report.pdf",
        "archive.zip",
        "image.png",
        "photo.jpg",
        "photo.jpeg",
        "image.gif",
        "image.webp",
        "notes.txt",
        "script.js",
        "module.mjs",
        "data.custom",
        "README",
    ] {
        let value = format!("attachment; filename={filename}");
        parse_content_disposition(
            &fields(&[("content-disposition", value.as_bytes())]),
            &observed,
        )
        .expect("safe mapped filename")
        .expect("disposition");
    }
}

#[test]
fn public_mime_constructor_normalizes_and_rejects_invalid_essences() {
    let mime = MimeType::from_essence("Text", "Plain").expect("reviewed MIME essence");
    assert_eq!(mime.essence(), "text/plain");
    assert!(matches!(
        MimeType::from_essence("text", "not/one-token"),
        Err(HttpError::InvalidMimeType)
    ));
}

#[test]
fn mime_classifier_exercises_every_html_signature_and_plain_text_byte_class() {
    for signature in [
        "<!doctype html",
        "<html",
        "<head",
        "<script",
        "<iframe",
        "<h1",
        "<div",
        "<font",
        "<table",
        "<a",
        "<style",
        "<title",
        "<b",
        "<body",
        "<br",
        "<p",
        "<!--",
    ] {
        let observed = observe_mime_type(signature.as_bytes(), None).expect("HTML signature");
        assert_eq!(observed.mime_type().essence(), "text/html");
    }

    for text in ["\ttext", "\ntext", "\u{000c}text", "\rtext", " text", "é"] {
        let observed = observe_mime_type(text.as_bytes(), None).expect("plain text byte class");
        assert_eq!(observed.mime_type().essence(), "text/plain");
    }
}

#[test]
fn mime_quoted_values_reject_escaped_controls_and_invalid_utf8() {
    for invalid in [
        b"text/plain; note=\"a\\\nb\"".as_slice(),
        b"text/plain; note=\"\xff\"".as_slice(),
        b"text/plain; note=\"a\\\xff\"".as_slice(),
    ] {
        assert!(matches!(
            MimeType::parse(invalid),
            Err(HttpError::InvalidMimeType)
        ));
    }
}

#[test]
fn structured_field_extension_keys_cover_the_complete_allowed_punctuation() {
    let value = b"*root=:AQ==:, a_b=:AQ==:, a-b=:AQ==:, a.b=:AQ==:, a*b=:AQ==:, a/b=:AQ==:";
    assert_eq!(
        validate_content_digest(
            &fields(&[("content-digest", value)]),
            &FieldBlock::default(),
            b"payload",
            IntegrityRequirement::Optional,
        )
        .expect("syntactically valid extension keys"),
        IntegrityStatus::UnsupportedAlgorithm
    );
    assert!(matches!(
        validate_content_digest(
            &fields(&[("content-digest", b"a!=:AQ==:")]),
            &FieldBlock::default(),
            b"payload",
            IntegrityRequirement::Optional,
        ),
        Err(HttpError::InvalidDigestField)
    ));
}

#[test]
fn trailer_only_digest_dictionary_is_supported() {
    // Reuse the independently verified SHA-256 vector from the integrity contract so this test
    // isolates trailer-only field selection rather than depending on a hand-copied digest.
    let value = b"sha-256=:Y39Vfsc6JaKuw7be30VwWgoMK//ZDxApEd+FUBoaVH8=:";
    assert_eq!(
        validate_content_digest(
            &FieldBlock::default(),
            &fields(&[("content-digest", value)]),
            b"deterministic content",
            IntegrityRequirement::Optional,
        )
        .expect("trailer-only digest"),
        IntegrityStatus::Verified(vec![crate::IntegrityAlgorithm::Sha256])
    );
}

#[test]
fn malformed_digest_trailers_fail_through_the_public_validation_path() {
    assert!(matches!(
        validate_content_digest(
            &FieldBlock::default(),
            &fields(&[("content-digest", b"sha-256=not-a-byte-sequence")]),
            b"payload",
            IntegrityRequirement::Optional,
        ),
        Err(HttpError::InvalidDigestField)
    ));
}

#[test]
fn final_response_head_reports_incomplete_and_malformed_prefixes() {
    assert_eq!(
        parse_final_response_head(b"", &HttpClientPolicy::strict_defaults())
            .expect("empty prefix is incomplete"),
        FinalHeadParseResult::Incomplete
    );
    assert!(matches!(
        parse_final_response_head(b"\r\n\r\n", &HttpClientPolicy::strict_defaults()),
        Err(HttpError::InvalidResponseStatusLine)
    ));
}

#[test]
fn response_head_rejects_a_missing_status_line_before_fields() {
    assert!(matches!(
        parse_response_head(b"\r\n\r\n", &HttpClientPolicy::strict_defaults()),
        Err(HttpError::InvalidResponseStatusLine)
    ));
}

#[test]
fn overlong_trailer_line_is_rejected_through_the_trailer_parser() {
    let policy = HttpClientPolicy::strict_defaults();
    let mut body = b"0\r\nX-Long: ".to_vec();
    body.extend(std::iter::repeat_n(
        b'a',
        policy.max_trailer_section_bytes(),
    ));
    assert!(parse_chunked_body(&body, &policy).is_err());
}
