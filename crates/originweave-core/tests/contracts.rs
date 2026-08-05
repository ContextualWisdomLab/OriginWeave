#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

use originweave_core::{
    ActionKind, ActionRequest, ApprovalEvidence, ApprovalScope, Capability, ExecutionPurpose,
    InstructionSource, Origin, OriginError, PolicyContext, RiskClass, RobotsDecision,
    SecretDelivery, SessionMode,
};

#[test]
fn origin_accepts_secure_and_loopback_origins() {
    let secure = Origin::parse("HTTPS://Example.COM:443").expect("secure origin");
    let secure_custom = Origin::parse("https://Example.COM:8443").expect("custom port");
    let localhost = Origin::parse("HTTP://LOCALHOST:80").expect("loopback origin");
    let localhost_custom = Origin::parse("http://localhost:8080").expect("custom loopback");
    let ipv4 = Origin::parse("http://127.0.0.1").expect("IPv4 loopback origin");
    let ipv6 = Origin::parse("http://[::1]:9222").expect("IPv6 loopback origin");
    let secure_ipv6 = Origin::parse("https://[2001:db8::1]").expect("secure IPv6 origin");

    assert_eq!(secure.as_str(), "https://example.com");
    assert_eq!(secure_custom.as_str(), "https://example.com:8443");
    assert_eq!(localhost.as_str(), "http://localhost");
    assert_eq!(localhost_custom.as_str(), "http://localhost:8080");
    assert_eq!(ipv4.as_str(), "http://127.0.0.1");
    assert_eq!(ipv6.as_str(), "http://[::1]:9222");
    assert_eq!(secure_ipv6.as_str(), "https://[2001:db8::1]");
    assert_eq!(secure.to_string(), secure.as_str());
    assert_eq!(
        secure,
        Origin::parse("https://example.com").expect("canonical equivalent")
    );
}

#[test]
fn origin_rejects_ambiguous_or_insecure_remote_inputs() {
    let cases = [
        ("example.com", OriginError::MissingScheme),
        ("ftp://example.com", OriginError::UnsupportedScheme),
        ("http://example.com", OriginError::InsecureRemoteOrigin),
        ("https://", OriginError::MissingAuthority),
        ("https://user@example.com", OriginError::UserInfoNotAllowed),
        ("https://example.com/path", OriginError::PathNotAllowed),
        ("https://example.com?x=1", OriginError::PathNotAllowed),
        ("https://example.com#fragment", OriginError::PathNotAllowed),
        (" https://example.com", OriginError::InvalidAuthority),
        ("https://exa mple.com", OriginError::InvalidAuthority),
        ("https://[::1", OriginError::InvalidAuthority),
        ("https://2001:db8::1", OriginError::InvalidAuthority),
        ("https://example.com:0", OriginError::InvalidPort),
        ("https://example.com:65536", OriginError::InvalidPort),
        ("https://example.com:not-a-port", OriginError::InvalidPort),
        ("https://example.com:", OriginError::InvalidPort),
    ];

    for (input, expected) in cases {
        assert_eq!(Origin::parse(input), Err(expected), "input={input}");
    }
}

#[test]
fn origin_rejects_malformed_dns_and_ipv6_authorities() {
    let long_label = "a".repeat(64);
    let cases = [
        "https://:443".to_owned(),
        "https://example..com".to_owned(),
        "https://-example.com".to_owned(),
        "https://example-.com".to_owned(),
        "https://exa_mple.com".to_owned(),
        "https://example.com.".to_owned(),
        format!("https://{long_label}.example"),
        "https://[not-ipv6]".to_owned(),
        "https://[::1]junk".to_owned(),
        "https://münich.example".to_owned(),
    ];

    for input in cases {
        assert_eq!(
            Origin::parse(&input),
            Err(OriginError::InvalidAuthority),
            "input={input}"
        );
    }
}

#[test]
fn action_contracts_cover_every_risk_and_capability() {
    let cases = [
        (
            ActionKind::Observe,
            RiskClass::R0,
            Capability::Observe,
            false,
            false,
        ),
        (
            ActionKind::Extract,
            RiskClass::R0,
            Capability::Extract,
            false,
            false,
        ),
        (
            ActionKind::Navigate,
            RiskClass::R1,
            Capability::Navigate,
            false,
            false,
        ),
        (
            ActionKind::Download,
            RiskClass::R1,
            Capability::Download,
            false,
            false,
        ),
        (
            ActionKind::Draft,
            RiskClass::R2,
            Capability::Draft,
            true,
            false,
        ),
        (
            ActionKind::Submit,
            RiskClass::R3,
            Capability::Submit,
            true,
            false,
        ),
        (
            ActionKind::Upload,
            RiskClass::R3,
            Capability::Upload,
            true,
            false,
        ),
        (
            ActionKind::FillSecret,
            RiskClass::R3,
            Capability::FillSecret,
            true,
            true,
        ),
        (
            ActionKind::Purchase,
            RiskClass::R4,
            Capability::Purchase,
            true,
            false,
        ),
        (
            ActionKind::Delete,
            RiskClass::R4,
            Capability::Delete,
            true,
            false,
        ),
        (
            ActionKind::ManagePermission,
            RiskClass::R4,
            Capability::ManagePermission,
            true,
            false,
        ),
        (
            ActionKind::LegalConsent,
            RiskClass::R5,
            Capability::LegalConsent,
            true,
            false,
        ),
    ];

    for (action, risk, capability, mutates_state, uses_secret) in cases {
        assert_eq!(action.risk_class(), risk);
        assert_eq!(action.required_capability(), capability);
        assert_eq!(action.mutates_state(), mutates_state);
        assert_eq!(action.uses_secret(), uses_secret);
    }

    assert!(!RiskClass::R0.requires_approval());
    assert!(!RiskClass::R1.requires_approval());
    assert!(!RiskClass::R2.requires_approval());
    assert!(RiskClass::R3.requires_approval());
    assert!(RiskClass::R4.requires_approval());
    assert!(RiskClass::R5.requires_approval());
}

#[test]
fn approval_evidence_is_bound_to_the_exact_action_and_origin() {
    let source = Origin::parse("https://shop.example").expect("source");
    let target = Origin::parse("https://pay.example").expect("target");
    let scope = ApprovalScope::new(ActionKind::Purchase, target.clone());
    let same = ApprovalScope::new(ActionKind::Purchase, target);
    let wrong_action = ApprovalScope::new(ActionKind::Delete, source.clone());

    assert_eq!(scope.action(), ActionKind::Purchase);
    assert_eq!(scope.target_origin().as_str(), "https://pay.example");
    assert!(ApprovalEvidence::UserConfirmed(scope.clone()).authorizes(&same));
    assert!(ApprovalEvidence::EnterprisePolicy(scope).authorizes(&same));
    assert!(!ApprovalEvidence::None.authorizes(&same));
    assert!(!ApprovalEvidence::UserConfirmed(wrong_action).authorizes(&same));
}

#[test]
fn request_and_context_accessors_preserve_explicit_authority() {
    let source = Origin::parse("https://app.example").expect("source");
    let target = Origin::parse("https://api.example").expect("target");
    let request = ActionRequest::new(
        ActionKind::Submit,
        source.clone(),
        target.clone(),
        InstructionSource::EnterprisePolicy,
        SecretDelivery::None,
    );
    assert_eq!(request.action(), ActionKind::Submit);
    assert_eq!(request.source_origin(), &source);
    assert_eq!(request.target_origin(), &target);
    assert_eq!(
        request.instruction_source(),
        InstructionSource::EnterprisePolicy
    );
    assert_eq!(request.secret_delivery(), SecretDelivery::None);

    let mut context = PolicyContext::new(
        SessionMode::AgentTask,
        ExecutionPurpose::EnterpriseAuthorizedTask,
        BTreeSet::from([Capability::Submit]),
        BTreeSet::from([source.clone(), target.clone()]),
        BTreeSet::from([target.clone()]),
        RobotsDecision::NotApplicable,
        ApprovalEvidence::None,
    );
    assert_eq!(context.mode(), SessionMode::AgentTask);
    assert_eq!(
        context.purpose(),
        ExecutionPurpose::EnterpriseAuthorizedTask
    );
    assert!(context.capabilities().contains(&Capability::Submit));
    assert!(context.read_origins().contains(&source));
    assert!(context.write_origins().contains(&target));
    assert_eq!(context.robots_decision(), RobotsDecision::NotApplicable);
    assert_eq!(context.approval(), &ApprovalEvidence::None);

    context.set_robots_decision(RobotsDecision::Allowed);
    context.set_approval(ApprovalEvidence::UserConfirmed(ApprovalScope::new(
        ActionKind::Submit,
        target,
    )));
    assert_eq!(context.robots_decision(), RobotsDecision::Allowed);
    assert!(matches!(
        context.approval(),
        ApprovalEvidence::UserConfirmed(_)
    ));
}

#[test]
fn governance_enums_are_distinct_and_copyable() {
    let modes = [
        SessionMode::Human,
        SessionMode::Assist,
        SessionMode::AgentTask,
        SessionMode::Crawler,
    ];
    let purposes = [
        ExecutionPurpose::PublicCrawl,
        ExecutionPurpose::UserDelegatedTask,
        ExecutionPurpose::EnterpriseAuthorizedTask,
        ExecutionPurpose::TestingEnvironment,
    ];
    let sources = [
        InstructionSource::User,
        InstructionSource::EnterprisePolicy,
        InstructionSource::WebContent,
    ];
    let robots = [
        RobotsDecision::Allowed,
        RobotsDecision::Disallowed,
        RobotsDecision::Unknown,
        RobotsDecision::NotApplicable,
    ];
    let secrets = [
        SecretDelivery::None,
        SecretDelivery::BrokerHandle,
        SecretDelivery::RawValue,
    ];

    assert_eq!(modes.len(), 4);
    assert_eq!(purposes.len(), 4);
    assert_eq!(sources.len(), 3);
    assert_eq!(robots.len(), 4);
    assert_eq!(secrets.len(), 3);
}
