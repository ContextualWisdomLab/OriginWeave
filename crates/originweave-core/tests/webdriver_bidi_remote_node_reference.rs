use std::error::Error;

use originweave_core::{
    MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES, WEBDRIVER_BIDI_NODE_REMOTE_VALUE_TYPE,
    WebDriverBiDiRemoteNodeReference, WebDriverBiDiRemoteNodeReferenceError,
};

#[test]
fn remote_node_reference_requires_exact_node_type_and_shared_id() -> Result<(), Box<dyn Error>> {
    let reference = WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42"))?;

    assert_eq!(
        reference.remote_type(),
        WEBDRIVER_BIDI_NODE_REMOTE_VALUE_TYPE
    );
    assert_eq!(reference.remote_type(), "node");
    assert_eq!(reference.shared_id(), "shared-node-42");
    Ok(())
}

#[test]
fn remote_node_reference_rejects_non_node_remote_values() {
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("object", Some("shared-node-42")),
        Err(WebDriverBiDiRemoteNodeReferenceError::UnexpectedRemoteType)
    );
}

#[test]
fn remote_node_reference_requires_a_usable_shared_id() {
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", None),
        Err(WebDriverBiDiRemoteNodeReferenceError::MissingSharedId)
    );
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", Some("")),
        Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId)
    );
}

#[test]
fn remote_node_reference_rejects_unicode_format_and_bidi_overrides() {
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42\u{200B}")),
        Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId)
    );
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42\u{202E}")),
        Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId)
    );
}

#[test]
fn remote_node_reference_rejects_whitespace_and_control_injection() {
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", Some(" ")),
        Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId)
    );
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42\n")),
        Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId)
    );
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", Some("shared-node-42\u{0000}")),
        Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId)
    );
}

#[test]
fn remote_node_reference_reuses_the_registry_identifier_budget() -> Result<(), Box<dyn Error>> {
    let maximum = "n".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES);
    let reference = WebDriverBiDiRemoteNodeReference::new("node", Some(&maximum))?;
    assert_eq!(reference.shared_id(), maximum);

    let overlong = "n".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES + 1);
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", Some(&overlong)),
        Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId)
    );
    Ok(())
}

#[test]
fn remote_node_reference_bounds_multibyte_shared_ids_by_utf8_bytes() -> Result<(), Box<dyn Error>> {
    let exact = "한".repeat(MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES / "한".len());
    let reference = WebDriverBiDiRemoteNodeReference::new("node", Some(&exact))?;
    assert_eq!(reference.shared_id(), exact);

    let overlong = format!("{exact}한");
    assert!(overlong.len() > MAX_EXTERNAL_BROWSER_IDENTIFIER_BYTES);
    assert_eq!(
        WebDriverBiDiRemoteNodeReference::new("node", Some(&overlong)),
        Err(WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId)
    );
    Ok(())
}

#[test]
fn remote_node_reference_error_contract_is_source_free() {
    let errors = [
        WebDriverBiDiRemoteNodeReferenceError::UnexpectedRemoteType,
        WebDriverBiDiRemoteNodeReferenceError::MissingSharedId,
        WebDriverBiDiRemoteNodeReferenceError::InvalidSharedId,
    ];

    for error in errors {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
    }
}
