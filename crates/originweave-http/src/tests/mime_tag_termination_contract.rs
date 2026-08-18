use crate::ContentRiskClass;
use crate::mime::classify_observed_mime;

const HTML_TAG_SIGNATURES: &[&[u8]] = &[
    b"<!doctype html",
    b"<html",
    b"<head",
    b"<script",
    b"<iframe",
    b"<h1",
    b"<div",
    b"<font",
    b"<table",
    b"<a",
    b"<style",
    b"<title",
    b"<b",
    b"<body",
    b"<br",
    b"<p",
    b"<!--",
];

#[test]
fn every_html_tag_signature_requires_tag_termination() {
    for signature in HTML_TAG_SIGNATURES {
        for invalid_next in [
            None,
            Some(b'x'),
            Some(b'\t'),
            Some(b'\n'),
            Some(b'\x0c'),
            Some(b'\r'),
        ] {
            let mut content = signature.to_vec();
            if let Some(next) = invalid_next {
                content.push(next);
                content.extend_from_slice(b"not an HTML tag");
            }
            let observed = classify_observed_mime(&content, None);
            assert_eq!(observed.mime_type().essence(), "text/plain");
            assert_eq!(observed.risk_class(), ContentRiskClass::Passive);
        }

        for terminator in *b" >" {
            let mut content = signature.to_vec();
            content.push(terminator);
            content.extend_from_slice(b"payload");
            let observed = classify_observed_mime(&content, None);
            assert_eq!(observed.mime_type().essence(), "text/html");
            assert_eq!(observed.risk_class(), ContentRiskClass::ActiveOrScriptable);
        }
    }

    let observed = classify_observed_mime(b"\xef\xbb\xbf \t<htmlx>not an html tag</htmlx>", None);
    assert_eq!(observed.mime_type().essence(), "text/plain");
    assert_eq!(observed.risk_class(), ContentRiskClass::Passive);
}
