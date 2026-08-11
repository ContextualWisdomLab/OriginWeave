#![allow(clippy::expect_used)]

use originweave_core::{
    ExtensionId, NativeMessagingAccessDecision, NativeMessagingAccessRequest,
    NativeMessagingHostGrant, NativeMessagingHostName, evaluate_native_messaging_access,
};

fn extension_id(value: &str) -> ExtensionId {
    ExtensionId::parse(value).expect("valid extension id")
}

fn host_name(value: &str) -> NativeMessagingHostName {
    NativeMessagingHostName::parse(value).expect("valid native messaging host name")
}

#[test]
fn native_messaging_host_name_matches_chromium_manifest_syntax() {
    let canonical = "com.contextualwisdom.originweave_host";
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

    let exact = NativeMessagingAccessRequest::new(allowed_extension.clone(), allowed_host.clone());
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
    let grant = NativeMessagingHostGrant::new(extension.clone(), host.clone());
    let request = NativeMessagingAccessRequest::new(extension, host);

    assert_eq!(
        evaluate_native_messaging_access(&request, Some(&grant)),
        NativeMessagingAccessDecision::Allow
    );
}
