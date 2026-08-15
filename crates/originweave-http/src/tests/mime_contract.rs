#![allow(clippy::expect_used)]

use crate::field::{FieldBlock, FieldLine};
use crate::mime::{
    ContentRiskClass, MIME_CLASSIFIER_VERSION, MimeMismatch, MimeType, NoSniffStatus,
    classify_mismatch, classify_observed_mime, no_sniff_status, risk_class, supplied_mime_type,
};
use crate::{HttpError, MAX_MIME_SNIFF_BYTES};

fn fields(entries: &[(&str, &[u8])]) -> FieldBlock {
    FieldBlock::new(
        entries
            .iter()
            .map(|(name, value)| FieldLine::new(name.as_bytes(), value, 256, 8_192).expect("field"))
            .collect(),
    )
}

#[test]
fn supplied_mime_parsing_normalizes_essence_and_parameters() {
    let parsed =
        MimeType::parse(b" Text/HTML ; Charset=\"utf-8\"; boundary=abc ").expect("valid MIME");
    assert_eq!(parsed.type_name(), "text");
    assert_eq!(parsed.subtype_name(), "html");
    assert_eq!(parsed.essence(), "text/html");
    assert_eq!(
        parsed.parameters(),
        &[
            ("charset".to_owned(), "utf-8".to_owned()),
            ("boundary".to_owned(), "abc".to_owned()),
        ]
    );
    assert!(parsed.same_essence(&MimeType::parse(b"text/html").expect("essence")));
}

#[test]
fn essence_constructor_rejects_parameter_injection() {
    assert_eq!(
        MimeType::from_essence("text", "plain")
            .expect("reviewed essence")
            .essence(),
        "text/plain"
    );
    for (type_name, subtype_name) in [
        ("text; charset=utf-8", "plain"),
        ("text", "plain; charset=utf-8"),
    ] {
        assert!(matches!(
            MimeType::from_essence(type_name, subtype_name),
            Err(HttpError::InvalidMimeType)
        ));
    }
}

#[test]
fn quoted_parameter_escapes_are_bounded_and_deterministic() {
    let parsed =
        MimeType::parse(b"application/example; note=\"a\\\"b\\\\c\"").expect("quoted parameter");
    assert_eq!(parsed.parameters()[0].1, "a\"b\\c");

    assert!(matches!(
        MimeType::parse(b"application/example; note=\"a\\\""),
        Err(HttpError::InvalidMimeType)
    ));
}

#[test]
fn invalid_mime_syntax_and_duplicate_parameters_fail_closed() {
    for invalid in [
        b"".as_slice(),
        b"text",
        b"/plain",
        b"text/",
        b"text/plain/extra",
        b"te xt/plain",
        b"text/plain; charset",
        b"text/plain; charset=",
        b"text/plain; charset=\"unterminated",
        b"text/plain; charset=utf-8; CHARSET=ascii",
        b"text/plain; note=\"a\0b\"",
    ] {
        assert!(matches!(
            MimeType::parse(invalid),
            Err(HttpError::InvalidMimeType)
        ));
    }
}

#[test]
fn content_type_and_nosniff_fields_are_single_and_strict() {
    let field_block = fields(&[
        ("content-type", b"text/plain; charset=utf-8"),
        ("x-content-type-options", b" NoSniff \t"),
    ]);
    assert_eq!(
        supplied_mime_type(&field_block)
            .expect("supplied MIME")
            .expect("present")
            .essence(),
        "text/plain"
    );
    assert_eq!(
        supplied_mime_type(&FieldBlock::default()).expect("absent supplied MIME"),
        None
    );
    assert_eq!(
        no_sniff_status(&field_block).expect("nosniff"),
        NoSniffStatus::Enabled
    );
    let repeated_nosniff = fields(&[
        ("x-content-type-options", b"nosniff"),
        ("x-content-type-options", b" NoSniff \t"),
    ]);
    assert_eq!(
        no_sniff_status(&repeated_nosniff).expect("repeated nosniff"),
        NoSniffStatus::Enabled
    );
    assert_eq!(
        no_sniff_status(&FieldBlock::default()).expect("absence"),
        NoSniffStatus::Absent
    );
    assert!(matches!(
        supplied_mime_type(&fields(&[
            ("content-type", b"text/plain"),
            ("content-type", b"text/html"),
        ])),
        Err(HttpError::InvalidMimeType)
    ));
    assert!(matches!(
        no_sniff_status(&fields(&[("x-content-type-options", b"other")])),
        Err(HttpError::InvalidMimeType)
    ));
}

#[test]
fn signature_table_classifies_active_archival_image_text_and_binary_content() {
    let cases = [
        (
            b"%PDF-1.7\n".as_slice(),
            "application/pdf",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (
            b"\x89PNG\r\n\x1a\nrest",
            "image/png",
            ContentRiskClass::Passive,
        ),
        (b"\xff\xd8\xffrest", "image/jpeg", ContentRiskClass::Passive),
        (b"GIF89arest", "image/gif", ContentRiskClass::Passive),
        (
            b"RIFF\x04\x00\x00\x00WEBPrest",
            "image/webp",
            ContentRiskClass::Passive,
        ),
        (
            b"PK\x03\x04rest",
            "application/zip",
            ContentRiskClass::ArchiveOrContainer,
        ),
        (
            b"\xef\xbb\xbf  <SVG></SVG>",
            "image/svg+xml",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (
            b"\n<?xml version=\"1.0\"?>",
            "application/xml",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (
            b"  <!DOCTYPE HTML><html>",
            "text/html",
            ContentRiskClass::ActiveOrScriptable,
        ),
        (b"plain UTF-8 text", "text/plain", ContentRiskClass::Passive),
        (
            b"binary\0content",
            "application/octet-stream",
            ContentRiskClass::UnknownBinary,
        ),
    ];
    for (content, essence, risk) in cases {
        let observed = classify_observed_mime(content, None);
        assert_eq!(observed.mime_type().essence(), essence);
        assert_eq!(observed.risk_class(), risk);
        assert_eq!(observed.classifier_version(), MIME_CLASSIFIER_VERSION);
    }
}

#[test]
fn short_html_signatures_require_tag_termination() {
    for content in [
        b"<article>not an anchor signature</article>".as_slice(),
        b"<audio>not an anchor signature</audio>",
        b"<basefont>not a bold signature</basefont>",
        b"<picture>not a paragraph signature</picture>",
    ] {
        let observed = classify_observed_mime(content, None);
        assert_eq!(observed.mime_type().essence(), "text/plain");
        assert_eq!(observed.risk_class(), ContentRiskClass::Passive);
    }

    for content in [
        b"<a href=\"/\">link</a>".as_slice(),
        b"<b>bold</b>",
        b"<p paragraph>text</p>",
    ] {
        let observed = classify_observed_mime(content, None);
        assert_eq!(observed.mime_type().essence(), "text/html");
        assert_eq!(observed.risk_class(), ContentRiskClass::ActiveOrScriptable);
    }
}

#[test]
fn active_mime_aliases_share_the_same_downstream_risk_class() {
    for essence in [
        "text/html",
        "application/xml",
        "text/xml",
        "image/svg+xml",
        "text/javascript",
        "application/javascript",
        "application/pdf",
    ] {
        let mime = MimeType::parse(essence.as_bytes()).expect("active MIME essence");
        assert_eq!(risk_class(&mime), ContentRiskClass::ActiveOrScriptable);
    }
    let archive = MimeType::parse(b"application/zip").expect("archive MIME");
    assert_eq!(risk_class(&archive), ContentRiskClass::ArchiveOrContainer);
    let binary = MimeType::parse(b"application/octet-stream").expect("binary MIME");
    assert_eq!(risk_class(&binary), ContentRiskClass::UnknownBinary);
    let passive = MimeType::parse(b"text/plain").expect("passive MIME");
    assert_eq!(risk_class(&passive), ContentRiskClass::Passive);
}

#[test]
fn javascript_observation_requires_supplied_javascript_metadata() {
    for essence in ["application/javascript", "text/javascript"] {
        let supplied = MimeType::parse(essence.as_bytes()).expect("JavaScript MIME");
        let observed = classify_observed_mime(b"const answer = 42;", Some(&supplied));
        assert_eq!(observed.mime_type().essence(), "text/javascript");
        assert_eq!(observed.risk_class(), ContentRiskClass::ActiveOrScriptable);
    }
    assert_eq!(
        classify_observed_mime(b"const answer = 42;", None)
            .mime_type()
            .essence(),
        "text/plain"
    );
}

#[test]
fn observation_reads_only_the_bounded_prefix() {
    let mut content = vec![b'a'; MAX_MIME_SNIFF_BYTES];
    content.extend_from_slice(b"\0after-boundary");
    assert_eq!(
        classify_observed_mime(&content, None).mime_type().essence(),
        "text/plain"
    );
}

#[test]
fn observation_accepts_utf8_prefix_truncated_mid_scalar() {
    let mut content = vec![b'a'; MAX_MIME_SNIFF_BYTES - 1];
    content.extend_from_slice("é".as_bytes());
    assert_eq!(
        classify_observed_mime(&content, None).mime_type().essence(),
        "text/plain"
    );
}

#[test]
fn supplied_and_observed_mismatch_states_are_explicit() {
    let html = classify_observed_mime(b"<html>", None);
    let binary = classify_observed_mime(b"\0", None);
    let supplied_html = MimeType::parse(b"text/html").expect("HTML MIME");
    let supplied_text = MimeType::parse(b"text/plain").expect("text MIME");
    assert_eq!(
        classify_mismatch(Some(&supplied_html), &html),
        MimeMismatch::Match
    );
    assert_eq!(
        classify_mismatch(Some(&supplied_text), &html),
        MimeMismatch::Mismatch
    );
    assert_eq!(
        classify_mismatch(Some(&supplied_text), &binary),
        MimeMismatch::SuppliedOnly
    );
    assert_eq!(classify_mismatch(None, &html), MimeMismatch::ObservedOnly);
    assert_eq!(
        classify_mismatch(None, &binary),
        MimeMismatch::BinaryFallback
    );
}