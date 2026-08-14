#![allow(clippy::expect_used)]

use crate::HttpError;
use crate::disposition::{parse_content_disposition, parse_redirect_metadata};
use crate::field::{FieldBlock, FieldLine};
use crate::mime::classify_observed_mime;

fn fields(entries: &[(&str, &[u8])]) -> FieldBlock {
    FieldBlock::new(
        entries
            .iter()
            .map(|(name, value)| FieldLine::new(name.as_bytes(), value, 256, 8_192).expect("field"))
            .collect(),
    )
}

#[test]
fn network_path_redirects_do_not_lose_their_authority() {
    let error = parse_redirect_metadata(302, &fields(&[("location", b"//evil.example/path")]))
        .expect_err(
            "network-path reference must not be represented as same-origin relative metadata",
        );
    assert!(matches!(error, HttpError::InvalidRedirectMetadata));
}

#[test]
fn deprecated_use_proxy_status_does_not_create_redirect_metadata() {
    let metadata = parse_redirect_metadata(
        305,
        &fields(&[("location", b"https://proxy.example/path")]),
    )
    .expect("deprecated status must not be treated as malformed");

    assert_eq!(metadata, None);
}

#[test]
fn windows_reserved_filename_characters_are_rejected_after_quoted_decoding() {
    let observed = classify_observed_mime(b"plain text", None);
    for filename in [
        "bad<name.txt",
        "bad>name.txt",
        "bad\"name.txt",
        "bad|name.txt",
        "bad?name.txt",
        "bad*name.txt",
    ] {
        let escaped = filename.replace('"', "\\\"");
        let value = format!("attachment; filename=\"{escaped}\"");
        let error = parse_content_disposition(
            &fields(&[("content-disposition", value.as_bytes())]),
            &observed,
        )
        .expect_err("Windows-reserved filename character must fail closed");
        assert!(matches!(error, HttpError::InvalidContentDisposition));
    }
}

#[test]
fn windows_superscript_device_names_are_rejected_after_extended_filename_decoding() {
    let observed = classify_observed_mime(b"plain text", None);
    for value in [
        b"attachment; filename*=UTF-8''COM%C2%B9.txt".as_slice(),
        b"attachment; filename*=UTF-8''LPT%C2%B2.txt".as_slice(),
    ] {
        let error =
            parse_content_disposition(&fields(&[("content-disposition", value)]), &observed)
                .expect_err("Windows superscript device name must fail closed");
        assert!(matches!(error, HttpError::InvalidContentDisposition));
    }
}
