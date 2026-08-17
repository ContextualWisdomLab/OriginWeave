#![allow(clippy::expect_used)]

use originweave_core::{
    BrowserSessionId, BrowsingContextId, ExtensionAccessDecision, ExtensionAccessRequest,
    ExtensionAgentCapability, ExtensionId, NativeMessagingAccessDecision,
    NativeMessagingAccessRequest, NativeMessagingHostGrant, NativeMessagingHostName, Origin,
    evaluate_extension_access, evaluate_native_messaging_access,
};

const UNEXPIRED_NOW_EPOCH_SECONDS: u64 = 1_700_000_000;

fn extension_id(value: &str) -> ExtensionId {
    ExtensionId::parse(value).expect("valid extension id")
}

fn host_name(value: &str) -> NativeMessagingHostName {
    NativeMessagingHostName::parse(value).expect("valid native messaging host name")
}

fn session(value: u64) -> BrowserSessionId {
    BrowserSessionId::new(value).expect("nonzero browser session")
}

fn context(value: u64) -> BrowsingContextId {
    BrowsingContextId::new(value).expect("nonzero browsing context")
}

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("valid test origin")
}

#[test]
fn native_messaging_host_name_matches_chromium_manifest_syntax() {
    let canonical = "com.contextualwisdom.originweave_host1";
    assert_eq!(host_name(canonical).as_str(), canonical);

    for invalid in [
        "",
        ".com.contextualwisdom.originweave",
        "com.contextualwisdom.originweave.",
        "com..contextualwisdom.originweave",
        "Com.contextualwisdom.originweave",
        "com.contextual-wisdom.originweave",
        "com/contextualwisdom/originweave",
        "com.contextualwisdom.originweave\n",
        "com.contextualwisdom.originweaveπ",
    ] {
        assert!(
            NativeMessagingHostName::parse(invalid).is_err(),
            "unexpected host name: {invalid:?}"
        );
    }
}

#[test]
fn native_messaging_requires_an_explicit_exact_extension_and_host_grant() {
    let allowed_extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let other_extension = extension_id("bcdefghijklmnopabcdefghijklmnopa");
    let allowed_host = host_name("com.contextualwisdom.originweave");
    let other_host = host_name("com.contextualwisdom.other_host");
    let grant = NativeMessagingHostGrant::new(allowed_extension.clone(), allowed_host.clone());

    assert_eq!(grant.extension_id(), &allowed_extension);
    assert_eq!(grant.host_name(), &allowed_host);

    let exact = NativeMessagingAccessRequest::new(allowed_extension.clone(), allowed_host.clone());
    assert_eq!(exact.extension_id(), &allowed_extension);
    assert_eq!(exact.host_name(), &allowed_host);
    assert_eq!(
        evaluate_native_messaging_access(&exact, Some(&grant)),
        NativeMessagingAccessDecision::Allow
    );
    assert_eq!(
        evaluate_native_messaging_access(&exact, None),
        NativeMessagingAccessDecision::DenyMissingGrant
    );

    let wrong_extension = NativeMessagingAccessRequest::new(other_extension, allowed_host);
    assert_eq!(
        evaluate_native_messaging_access(&wrong_extension, Some(&grant)),
        NativeMessagingAccessDecision::DenyExtensionMismatch
    );

    let wrong_host = NativeMessagingAccessRequest::new(allowed_extension, other_host);
    assert_eq!(
        evaluate_native_messaging_access(&wrong_host, Some(&grant)),
        NativeMessagingAccessDecision::DenyHostMismatch
    );
}

#[test]
fn native_messaging_grant_does_not_mint_agent_capability() {
    let extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let host = host_name("com.contextualwisdom.originweave");
    let native_grant = NativeMessagingHostGrant::new(extension.clone(), host.clone());
    let native_request = NativeMessagingAccessRequest::new(extension.clone(), host);

    assert_eq!(
        evaluate_native_messaging_access(&native_request, Some(&native_grant)),
        NativeMessagingAccessDecision::Allow
    );

    let agent_request = ExtensionAccessRequest::new(
        extension,
        session(23),
        context(29),
        origin("https://native-messaging.example"),
        UNEXPIRED_NOW_EPOCH_SECONDS,
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&agent_request, None),
        ExtensionAccessDecision::DenyMissingGrant
    );
}
