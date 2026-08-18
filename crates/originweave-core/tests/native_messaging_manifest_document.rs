#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES, NativeMessagingManifestDocument,
    NativeMessagingManifestDocumentError,
};

#[test]
fn native_messaging_manifest_document_is_bounded_before_text_storage() {
    let mut exact_limit = vec![b' '; MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES];
    exact_limit[0] = b'{';
    exact_limit[MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES - 1] = b'}';
    let document = NativeMessagingManifestDocument::parse(&exact_limit)
        .expect("the exact OriginWeave manifest-document safety bound remains accepted");
    assert_eq!(document.as_str().len(), exact_limit.len());

    let one_over = vec![b' '; MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES + 1];
    assert_eq!(
        NativeMessagingManifestDocument::parse(&one_over),
        Err(NativeMessagingManifestDocumentError::DocumentTooLarge)
    );
}

#[test]
fn native_messaging_manifest_document_rejects_empty_and_invalid_utf8() {
    assert_eq!(
        NativeMessagingManifestDocument::parse(&[]),
        Err(NativeMessagingManifestDocumentError::EmptyDocument)
    );
    assert_eq!(
        NativeMessagingManifestDocument::parse(&[0xff]),
        Err(NativeMessagingManifestDocumentError::InvalidUtf8)
    );
}

#[test]
fn native_messaging_manifest_document_requires_outer_object_boundary() {
    assert_eq!(
        NativeMessagingManifestDocument::parse(b"[]"),
        Err(NativeMessagingManifestDocumentError::InvalidObjectBoundary)
    );
    assert_eq!(
        NativeMessagingManifestDocument::parse(b"{"),
        Err(NativeMessagingManifestDocumentError::InvalidObjectBoundary)
    );

    let document = NativeMessagingManifestDocument::parse(b" \r\n{\n}\t ")
        .expect("JSON whitespace around an object-shaped document remains accepted");
    assert_eq!(document.as_str(), " \r\n{\n}\t ");
}

#[test]
fn native_messaging_manifest_document_errors_are_standard_and_source_free() {
    for (error, expected) in [
        (
            NativeMessagingManifestDocumentError::EmptyDocument,
            "native messaging host manifest document is empty",
        ),
        (
            NativeMessagingManifestDocumentError::DocumentTooLarge,
            "native messaging host manifest document exceeds the OriginWeave safety budget",
        ),
        (
            NativeMessagingManifestDocumentError::InvalidUtf8,
            "native messaging host manifest document is not valid UTF-8",
        ),
        (
            NativeMessagingManifestDocumentError::InvalidObjectBoundary,
            "native messaging host manifest document must have one outer JSON object boundary",
        ),
    ] {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
