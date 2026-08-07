#![allow(clippy::expect_used)]

use crate::chunked::parse_chunked_body;
use crate::disposition::parse_content_disposition;
use crate::field::{FieldBlock, FieldLine};
use crate::integrity::validate_content_digest;
use crate::mime::observe_mime_type;
use crate::{HttpClientPolicy, IntegrityRequirement, IntegrityStatus};

fn fields(entries: &[(&str, &[u8])]) -> FieldBlock {
    FieldBlock::new(
        entries
            .iter()
            .map(|(name, value)| FieldLine::new(name.as_bytes(), value, 256, 8_192).expect("field"))
            .collect(),
    )
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
fn structured_field_extension_keys_cover_the_complete_allowed_punctuation() {
    let value = b"a_b=:AQ==:, a-b=:AQ==:, a.b=:AQ==:, a*b=:AQ==:, a/b=:AQ==:";
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
