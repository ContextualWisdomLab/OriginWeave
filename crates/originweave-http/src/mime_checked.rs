//! Fail-closed MIME classification facade for active-markup signature boundaries.

#[path = "mime.rs"]
mod raw;

use crate::field::FieldBlock;
use crate::{HttpError, MAX_MIME_SNIFF_BYTES};

pub use raw::{
    ContentRiskClass, MIME_CLASSIFIER_VERSION, MimeMismatch, MimeType, NoSniffStatus,
    ObservedMimeClassification,
};

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
];

pub(crate) fn supplied_mime_type(fields: &FieldBlock) -> Result<Option<MimeType>, HttpError> {
    raw::supplied_mime_type(fields)
}

pub(crate) fn no_sniff_status(fields: &FieldBlock) -> Result<NoSniffStatus, HttpError> {
    raw::no_sniff_status(fields)
}

pub(crate) fn classify_observed_mime(
    content: &[u8],
    supplied: Option<&MimeType>,
) -> ObservedMimeClassification {
    let prefix = &content[..content.len().min(MAX_MIME_SNIFF_BYTES)];
    let trimmed = trim_text_prefix(prefix);
    if has_unterminated_html_signature(trimmed) {
        let mut neutralized = prefix.to_vec();
        let signature_offset = prefix.len() - trimmed.len();
        neutralized[signature_offset] = b'x';
        raw::classify_observed_mime(&neutralized, supplied)
    } else {
        raw::classify_observed_mime(content, supplied)
    }
}

pub(crate) fn classify_mismatch(
    supplied: Option<&MimeType>,
    observed: &ObservedMimeClassification,
) -> MimeMismatch {
    raw::classify_mismatch(supplied, observed)
}

pub(crate) fn risk_class(mime_type: &MimeType) -> ContentRiskClass {
    raw::risk_class(mime_type)
}

fn has_unterminated_html_signature(input: &[u8]) -> bool {
    HTML_TAG_SIGNATURES.iter().any(|signature| {
        starts_ascii_case_insensitive(input, signature)
            && !input
                .get(signature.len())
                .is_some_and(|next| next.is_ascii_whitespace() || *next == b'>')
    })
}

fn starts_ascii_case_insensitive(input: &[u8], prefix: &[u8]) -> bool {
    input
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}

fn trim_text_prefix(input: &[u8]) -> &[u8] {
    let input = input.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(input);
    let start = input
        .iter()
        .position(|byte| !matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' '))
        .unwrap_or(input.len());
    &input[start..]
}
