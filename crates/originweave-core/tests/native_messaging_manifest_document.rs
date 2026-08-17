#![allow(clippy::expect_used)]

use std::error::Error;

use originweave_core::{
    MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES, NativeMessagingManifestDocument,
    NativeMessagingManifestDocumentError,
};

#[test]
fn native_messaging_manifest_document_is_bounded_before_text_storage() {
    let exact_limit = vec![b' '; MAX_NATIVE_MESSAGING_MANIFEST_DOCUMENT_BYTES];
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
    ] {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
