#![allow(clippy::expect_used)]

use originweave_core::{
    BrowserSessionId, BrowsingContextId, ExtensionAccessDecision, ExtensionAccessRequest,
    ExtensionAgentCapability, ExtensionAgentGrant, ExtensionId, ManagedExtensionAdmission,
    ManagedExtensionPolicy, ManagedExtensionPolicyError, admit_agent_task_extension,
    evaluate_extension_access,
};

fn extension_id(value: &str) -> ExtensionId {
    ExtensionId::parse(value).expect("valid extension id")
}

fn session(value: u64) -> BrowserSessionId {
    BrowserSessionId::new(value).expect("nonzero browser session")
}

fn context(value: u64) -> BrowsingContextId {
    BrowsingContextId::new(value).expect("nonzero browsing context")
}

#[test]
fn extension_id_accepts_only_canonical_chromium_extension_ids() {
    let canonical = "abcdefghijklmnopabcdefghijklmnop";
    assert_eq!(extension_id(canonical).as_str(), canonical);

    for invalid in [
        "",
        "abcdefghijklmnopabcdefghijklmno",
        "abcdefghijklmnopabcdefghijklmnopq",
        "ABCDEFGHIJKLMNOPABCDEFGHIJKLMNOP",
        "abcdefghijklmnopabcdefghijklmno0",
        "abcdefghijklmnopabcdefghijklmno-",
        "abcdefghijklmnopabcdefghijklmno\n",
        "abcdefghijklmnopabcdefghijklmnoπ",
    ] {
        assert!(
            ExtensionId::parse(invalid).is_err(),
            "unexpected id: {invalid:?}"
        );
    }
}

#[test]
fn extension_agent_access_requires_an_explicit_exact_grant() {
    let allowed_extension = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let other_extension = extension_id("bcdefghijklmnopabcdefghijklmnopa");
    let grant = ExtensionAgentGrant::new(
        allowed_extension.clone(),
        session(7),
        context(11),
        [ExtensionAgentCapability::ObserveCurrentContext],
    );

    let exact = ExtensionAccessRequest::new(
        allowed_extension.clone(),
        session(7),
        context(11),
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&exact, Some(&grant)),
        ExtensionAccessDecision::Allow
    );

    let no_grant = evaluate_extension_access(&exact, None);
    assert_eq!(no_grant, ExtensionAccessDecision::DenyMissingGrant);

    let wrong_extension = ExtensionAccessRequest::new(
        other_extension,
        session(7),
        context(11),
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_extension, Some(&grant)),
        ExtensionAccessDecision::DenyExtensionMismatch
    );

    let wrong_session = ExtensionAccessRequest::new(
        allowed_extension.clone(),
        session(8),
        context(11),
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_session, Some(&grant)),
        ExtensionAccessDecision::DenyBrowserSessionMismatch
    );

    let wrong_context = ExtensionAccessRequest::new(
        allowed_extension,
        session(7),
        context(12),
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&wrong_context, Some(&grant)),
        ExtensionAccessDecision::DenyBrowsingContextMismatch
    );
}

#[test]
fn chrome_permissions_never_imply_originweave_agent_capabilities() {
    let id = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let grant = ExtensionAgentGrant::new(
        id.clone(),
        session(3),
        context(5),
        [ExtensionAgentCapability::ObserveCurrentContext],
    );

    let propose_action = ExtensionAccessRequest::new(
        id,
        session(3),
        context(5),
        ExtensionAgentCapability::ProposeTypedAction,
    );
    assert_eq!(
        evaluate_extension_access(&propose_action, Some(&grant)),
        ExtensionAccessDecision::DenyCapabilityNotGranted
    );
}

#[test]
fn explicit_grant_can_authorize_multiple_bounded_agent_capabilities() {
    let id = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let grant = ExtensionAgentGrant::new(
        id.clone(),
        session(13),
        context(17),
        [
            ExtensionAgentCapability::ObserveCurrentContext,
            ExtensionAgentCapability::ProposeTypedAction,
        ],
    );

    for capability in [
        ExtensionAgentCapability::ObserveCurrentContext,
        ExtensionAgentCapability::ProposeTypedAction,
    ] {
        let request = ExtensionAccessRequest::new(id.clone(), session(13), context(17), capability);
        assert_eq!(
            evaluate_extension_access(&request, Some(&grant)),
            ExtensionAccessDecision::Allow
        );
    }
}

#[test]
fn agent_task_defaults_to_no_managed_extension_admission() {
    let policy = ManagedExtensionPolicy::from_exact_lists(&[], &[]).expect("empty policy");
    let admitted = admit_agent_task_extension("abcdefghijklmnopabcdefghijklmnop", &policy);
    assert_eq!(admitted, ManagedExtensionAdmission::DeniedByDefault);
}

#[test]
fn chrome_force_installed_token_is_not_an_extension_identity() {
    let policy = ManagedExtensionPolicy::from_exact_lists(&[], &[]).expect("empty policy");
    assert_eq!(
        admit_agent_task_extension("force_installed", &policy),
        ManagedExtensionAdmission::DeniedInvalidIdentity
    );
    assert_eq!(
        admit_agent_task_extension("*", &policy),
        ManagedExtensionAdmission::DeniedInvalidIdentity
    );
}

#[test]
fn managed_allow_list_admits_compatibility_surface_only() {
    let allowed = "abcdefghijklmnopabcdefghijklmnop";
    let other = "bcdefghijklmnopabcdefghijklmnopa";
    let policy = ManagedExtensionPolicy::from_exact_lists(&[allowed], &[]).expect("allow list");

    assert_eq!(
        admit_agent_task_extension(allowed, &policy),
        ManagedExtensionAdmission::CompatibilitySurfaceOnly
    );
    assert_eq!(
        admit_agent_task_extension(other, &policy),
        ManagedExtensionAdmission::DeniedByDefault
    );
}

#[test]
fn managed_block_list_wins_over_an_allow_list_hit() {
    let blocked = "abcdefghijklmnopabcdefghijklmnop";
    let policy = ManagedExtensionPolicy::from_exact_lists(&[], &[blocked]).expect("block list");
    assert_eq!(
        admit_agent_task_extension(blocked, &policy),
        ManagedExtensionAdmission::DeniedBlocked
    );
}

#[test]
fn contradictory_allow_and_block_lists_fail_closed() {
    let same = "abcdefghijklmnopabcdefghijklmnop";
    assert_eq!(
        ManagedExtensionPolicy::from_exact_lists(&[same], &[same]),
        Err(ManagedExtensionPolicyError::AllowAndBlockOverlap)
    );
}

#[test]
fn invalid_managed_policy_identities_fail_closed() {
    assert_eq!(
        ManagedExtensionPolicy::from_exact_lists(&["FORCE_INSTALLED"], &[]),
        Err(ManagedExtensionPolicyError::InvalidExtensionId)
    );
    assert_eq!(
        ManagedExtensionPolicy::from_exact_lists(&[], &["*"]),
        Err(ManagedExtensionPolicyError::InvalidExtensionId)
    );
}

#[test]
fn managed_admission_never_mints_an_extension_agent_grant() {
    let allowed = extension_id("abcdefghijklmnopabcdefghijklmnop");
    let policy =
        ManagedExtensionPolicy::from_exact_lists(&[allowed.as_str()], &[]).expect("allow list");
    assert_eq!(
        admit_agent_task_extension(allowed.as_str(), &policy),
        ManagedExtensionAdmission::CompatibilitySurfaceOnly
    );

    let request = ExtensionAccessRequest::new(
        allowed,
        session(3),
        context(5),
        ExtensionAgentCapability::ObserveCurrentContext,
    );
    assert_eq!(
        evaluate_extension_access(&request, None),
        ExtensionAccessDecision::DenyMissingGrant
    );
}
