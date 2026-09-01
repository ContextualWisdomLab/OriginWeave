use std::error::Error;

use originweave_core::{
    MAX_BROWSER_ACCESSIBILITY_QUERY_NAME_BYTES, MAX_BROWSER_ACCESSIBILITY_QUERY_NODE_COUNT,
    MAX_BROWSER_ACCESSIBILITY_QUERY_ROLE_BYTES, UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS,
    WEBDRIVER_BIDI_LOCATE_NODES_METHOD, WEBDRIVER_BIDI_QUERY_INCLUDE_SHADOW_TREE,
    WEBDRIVER_BIDI_QUERY_MAX_DOM_DEPTH, WEBDRIVER_BIDI_QUERY_MAX_OBJECT_DEPTH,
    WebDriverBiDiAccessibilityQuery, WebDriverBiDiAccessibilityQueryError,
};

#[test]
fn accessibility_query_exposes_exact_bidi_method_and_locator_contract() -> Result<(), Box<dyn Error>>
{
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), Some("Task text"), 32)?;

    assert_eq!(query.method(), WEBDRIVER_BIDI_LOCATE_NODES_METHOD);
    assert_eq!(query.method(), "browsingContext.locateNodes");
    assert_eq!(query.locator_type(), "accessibility");
    assert_eq!(query.role(), Some("textbox"));
    assert_eq!(query.name(), Some("Task text"));
    assert_eq!(query.max_node_count(), 32);
    Ok(())
}

#[test]
fn accessibility_query_fixes_minimal_serialization_options() -> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), None, 8)?;

    assert_eq!(WEBDRIVER_BIDI_QUERY_MAX_DOM_DEPTH, 0);
    assert_eq!(WEBDRIVER_BIDI_QUERY_MAX_OBJECT_DEPTH, 0);
    assert_eq!(WEBDRIVER_BIDI_QUERY_INCLUDE_SHADOW_TREE, "none");
    assert_eq!(query.serialization_max_dom_depth(), 0);
    assert_eq!(query.serialization_max_object_depth(), 0);
    assert_eq!(query.serialization_include_shadow_tree(), "none");
    Ok(())
}

#[test]
fn role_only_and_name_only_queries_are_valid() -> Result<(), Box<dyn Error>> {
    let role_only = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 1)?;
    assert_eq!(role_only.role(), Some("button"));
    assert_eq!(role_only.name(), None);

    let name_only = WebDriverBiDiAccessibilityQuery::new(None, Some("Submit task"), 1)?;
    assert_eq!(name_only.role(), None);
    assert_eq!(name_only.name(), Some("Submit task"));
    Ok(())
}

#[test]
fn missing_or_empty_accessibility_locator_fields_fail_closed() {
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(None, None, 1),
        Err(WebDriverBiDiAccessibilityQueryError::MissingLocatorValue)
    );
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(Some(""), None, 1),
        Err(WebDriverBiDiAccessibilityQueryError::EmptyRole)
    );
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(None, Some(""), 1),
        Err(WebDriverBiDiAccessibilityQueryError::EmptyName)
    );
}

#[test]
fn accessibility_role_rejects_whitespace_and_control_injection() {
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(Some("text box"), None, 1),
        Err(WebDriverBiDiAccessibilityQueryError::InvalidRole)
    );
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(Some("button\n"), None, 1),
        Err(WebDriverBiDiAccessibilityQueryError::InvalidRole)
    );
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(Some("button\u{0000}"), None, 1),
        Err(WebDriverBiDiAccessibilityQueryError::InvalidRole)
    );
}

#[test]
fn accessibility_locator_text_rejects_unicode_format_and_bidi_overrides() {
    for character in UNICODE_PROTOCOL_FORMAT_INJECTION_CHARS {
        let role = format!("button{character}");
        let name = format!("Submit{character}task");
        assert_eq!(
            WebDriverBiDiAccessibilityQuery::new(Some(&role), None, 1),
            Err(WebDriverBiDiAccessibilityQueryError::InvalidRole)
        );
        assert_eq!(
            WebDriverBiDiAccessibilityQuery::new(None, Some(&name), 1),
            Err(WebDriverBiDiAccessibilityQueryError::InvalidName)
        );
    }
}

#[test]
fn accessibility_name_rejects_control_injection_and_whitespace_only_values() {
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(None, Some("Submit\ntask"), 1),
        Err(WebDriverBiDiAccessibilityQueryError::InvalidName)
    );
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(None, Some("Submit\u{0000}task"), 1),
        Err(WebDriverBiDiAccessibilityQueryError::InvalidName)
    );
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(None, Some("   "), 1),
        Err(WebDriverBiDiAccessibilityQueryError::InvalidName)
    );
}

#[test]
fn accessibility_name_keeps_ordinary_spaces_and_multibyte_text() -> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("textbox"), Some("작업 텍스트"), 1)?;
    assert_eq!(query.role(), Some("textbox"));
    assert_eq!(query.name(), Some("작업 텍스트"));
    Ok(())
}

#[test]
fn accessibility_locator_text_is_bounded_by_utf8_bytes() -> Result<(), Box<dyn Error>> {
    let maximum_role = "r".repeat(MAX_BROWSER_ACCESSIBILITY_QUERY_ROLE_BYTES);
    let maximum_name = "n".repeat(MAX_BROWSER_ACCESSIBILITY_QUERY_NAME_BYTES);
    let query = WebDriverBiDiAccessibilityQuery::new(
        Some(&maximum_role),
        Some(&maximum_name),
        MAX_BROWSER_ACCESSIBILITY_QUERY_NODE_COUNT,
    )?;
    assert_eq!(query.role(), Some(maximum_role.as_str()));
    assert_eq!(query.name(), Some(maximum_name.as_str()));

    let overlong_role = "r".repeat(MAX_BROWSER_ACCESSIBILITY_QUERY_ROLE_BYTES + 1);
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(Some(&overlong_role), None, 1),
        Err(WebDriverBiDiAccessibilityQueryError::RoleTooLong)
    );

    let overlong_name = "n".repeat(MAX_BROWSER_ACCESSIBILITY_QUERY_NAME_BYTES + 1);
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(None, Some(&overlong_name), 1),
        Err(WebDriverBiDiAccessibilityQueryError::NameTooLong)
    );
    Ok(())
}

#[test]
fn accessibility_query_node_count_is_finite_and_nonzero() {
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 0),
        Err(WebDriverBiDiAccessibilityQueryError::InvalidNodeCount)
    );
    assert_eq!(
        WebDriverBiDiAccessibilityQuery::new(
            Some("button"),
            None,
            MAX_BROWSER_ACCESSIBILITY_QUERY_NODE_COUNT + 1,
        ),
        Err(WebDriverBiDiAccessibilityQueryError::InvalidNodeCount)
    );
}

#[test]
fn accessibility_query_revalidates_returned_node_count() -> Result<(), Box<dyn Error>> {
    let query = WebDriverBiDiAccessibilityQuery::new(Some("button"), None, 2)?;

    assert_eq!(query.validate_result_count(0), Ok(()));
    assert_eq!(query.validate_result_count(2), Ok(()));
    assert_eq!(
        query.validate_result_count(3),
        Err(WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded)
    );
    Ok(())
}

#[test]
fn accessibility_query_error_contract_is_source_free() {
    let errors = [
        WebDriverBiDiAccessibilityQueryError::MissingLocatorValue,
        WebDriverBiDiAccessibilityQueryError::EmptyRole,
        WebDriverBiDiAccessibilityQueryError::RoleTooLong,
        WebDriverBiDiAccessibilityQueryError::EmptyName,
        WebDriverBiDiAccessibilityQueryError::InvalidRole,
        WebDriverBiDiAccessibilityQueryError::InvalidName,
        WebDriverBiDiAccessibilityQueryError::NameTooLong,
        WebDriverBiDiAccessibilityQueryError::InvalidNodeCount,
        WebDriverBiDiAccessibilityQueryError::ResultNodeCountExceeded,
    ];

    for error in errors {
        assert!(error.source().is_none());
        assert!(!error.to_string().is_empty());
    }
}
