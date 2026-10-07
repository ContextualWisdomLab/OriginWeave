#![allow(clippy::expect_used)]

//! RFC 9309 robots-exclusion parsing and matching contract tests.

use std::collections::BTreeSet;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, ExecutionPurpose,
    InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery, SessionMode,
};
use originweave_policy::{
    Decision, DenialReason, MAX_ROBOTS_BODY_BYTES, MAX_ROBOTS_PATH_BYTES,
    MAX_ROBOTS_PRODUCT_TOKEN_BYTES, MAX_ROBOTS_RULES, RobotsBasis, RobotsFetchOutcome,
    RobotsPathError, RobotsProductToken, RobotsProductTokenError, RobotsRuleKind, RobotsRules,
    UnavailableRobotsPolicy, evaluate, evaluate_robots,
};

fn token(value: &str) -> RobotsProductToken {
    RobotsProductToken::parse(value).expect("valid product token")
}

fn decide(body: &str, agent: &str, path: &str) -> RobotsDecision {
    RobotsRules::parse(body.as_bytes())
        .decide(&token(agent), path)
        .expect("valid path")
        .decision()
}

/// RFC 9309 section 5.1 example, reduced to the groups exercised here.
const RFC_EXAMPLE: &str = "User-Agent: *\n\
Disallow: *.gif$\n\
Disallow: /example/\n\
Allow: /publications/\n\
\n\
User-Agent: foobot\n\
Disallow:/\n\
Allow:/example/page.html\n\
Allow:/example/allowed.gif\n\
\n\
User-Agent: barbot\n\
User-Agent: bazbot\n\
Disallow: /example/page.html\n\
\n\
User-Agent: quxbot\n";

#[test]
fn rfc_9309_example_selects_specific_groups_and_falls_back_to_the_wildcard_group() {
    assert_eq!(
        decide(RFC_EXAMPLE, "foobot", "/example/page.html"),
        RobotsDecision::Allowed
    );
    assert_eq!(
        decide(RFC_EXAMPLE, "foobot", "/elsewhere"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(RFC_EXAMPLE, "bazbot", "/example/page.html"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(RFC_EXAMPLE, "barbot", "/example/other.html"),
        RobotsDecision::Allowed
    );
    // A matching group without rules allows everything; it does not inherit `*`.
    assert_eq!(
        decide(RFC_EXAMPLE, "quxbot", "/example/"),
        RobotsDecision::Allowed
    );
    // Unknown crawlers obey the `*` group.
    assert_eq!(
        decide(RFC_EXAMPLE, "OtherBot", "/example/x"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(RFC_EXAMPLE, "OtherBot", "/img/a.gif"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(RFC_EXAMPLE, "OtherBot", "/publications/x"),
        RobotsDecision::Allowed
    );
}

#[test]
fn product_tokens_match_case_insensitively_and_reject_non_identifier_input() {
    let body = "user-agent: EXAMPLEBOT\ndisallow: /private\n";
    assert_eq!(
        decide(body, "examplebot", "/private/a"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(body, "ExampleBot", "/public"),
        RobotsDecision::Allowed
    );

    assert_eq!(
        RobotsProductToken::parse(""),
        Err(RobotsProductTokenError::Empty)
    );
    for invalid in ["Example Bot", "ExampleBot/1.0", "Bót", "bot\0", "*"] {
        assert_eq!(
            RobotsProductToken::parse(invalid),
            Err(RobotsProductTokenError::InvalidCharacter),
            "{invalid:?}"
        );
    }
    let longest = "a".repeat(MAX_ROBOTS_PRODUCT_TOKEN_BYTES);
    assert_eq!(
        RobotsProductToken::parse(&longest)
            .expect("bounded token")
            .as_str(),
        longest
    );
    assert_eq!(
        RobotsProductToken::parse(&"a".repeat(MAX_ROBOTS_PRODUCT_TOKEN_BYTES + 1)),
        Err(RobotsProductTokenError::TooLong)
    );
    assert_eq!(token("Under_score-Bot").as_str(), "Under_score-Bot");
}

#[test]
fn separate_groups_for_the_same_agent_are_combined() {
    let body = "User-agent: ExampleBot\nDisallow: /a\n\nUser-agent: other\nDisallow: /\n\nuser-agent: examplebot\nAllow: /a/b\n";
    assert_eq!(
        decide(body, "ExampleBot", "/a/b/c"),
        RobotsDecision::Allowed
    );
    assert_eq!(
        decide(body, "ExampleBot", "/a/x"),
        RobotsDecision::Disallowed
    );
    assert_eq!(decide(body, "other", "/a/b"), RobotsDecision::Disallowed);
}

#[test]
fn the_longest_match_wins_and_equal_length_ties_allow() {
    let body = "User-agent: *\nAllow: /p\nDisallow: /page\nDisallow: /x\nAllow: /x\n";
    assert_eq!(
        decide(body, "bot", "/page.html"),
        RobotsDecision::Disallowed
    );
    assert_eq!(decide(body, "bot", "/p.html"), RobotsDecision::Allowed);
    assert_eq!(decide(body, "bot", "/x/y"), RobotsDecision::Allowed);

    let evaluation = RobotsRules::parse(body.as_bytes())
        .decide(&token("bot"), "/page.html")
        .expect("valid path");
    let rule = match evaluation.basis() {
        RobotsBasis::MatchedRule(rule) => Some(rule),
        _ => None,
    }
    .expect("matched rule basis");
    assert_eq!(rule.kind(), RobotsRuleKind::Disallow);
    assert_eq!(rule.line_number(), 3);
}

#[test]
fn wildcards_and_end_anchors_follow_rfc_9309_special_characters() {
    let body = "User-agent: *\nDisallow: /*.pdf$\nDisallow: /tmp*/cache\nDisallow: /price$x\n";
    assert_eq!(
        decide(body, "bot", "/doc/a.pdf"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(body, "bot", "/doc/a.pdf?x=1"),
        RobotsDecision::Allowed
    );
    assert_eq!(decide(body, "bot", "/a.pdfx"), RobotsDecision::Allowed);
    assert_eq!(
        decide(body, "bot", "/tmp-1/x/cache/y"),
        RobotsDecision::Disallowed
    );
    assert_eq!(decide(body, "bot", "/tmp/cach"), RobotsDecision::Allowed);
    // `$` before the end of a pattern is an ordinary octet.
    assert_eq!(
        decide(body, "bot", "/price$x/1"),
        RobotsDecision::Disallowed
    );
    assert_eq!(decide(body, "bot", "/price"), RobotsDecision::Allowed);

    let everything = "User-agent: *\nDisallow: /*\n";
    assert_eq!(decide(everything, "bot", "/"), RobotsDecision::Disallowed);
    assert_eq!(
        decide(everything, "bot", "/robots.txt"),
        RobotsDecision::Allowed
    );
    let anchored_root = "User-agent: *\nDisallow: /$\n";
    assert_eq!(
        decide(anchored_root, "bot", "/"),
        RobotsDecision::Disallowed
    );
    assert_eq!(decide(anchored_root, "bot", "/a"), RobotsDecision::Allowed);
    let leading_star = "User-agent: *\nDisallow: *private\n";
    assert_eq!(
        decide(leading_star, "bot", "/a/private"),
        RobotsDecision::Disallowed
    );
}

#[test]
fn percent_encoding_is_normalized_identically_for_rules_and_requests() {
    let body =
        "User-agent: *\nDisallow: /%7ejoe\nDisallow: /Ä\nDisallow: /a%2fb\nDisallow: /100%\n";
    assert_eq!(decide(body, "bot", "/~joe/x"), RobotsDecision::Disallowed);
    assert_eq!(decide(body, "bot", "/%7Ejoe"), RobotsDecision::Disallowed);
    assert_eq!(decide(body, "bot", "/%C3%84/x"), RobotsDecision::Disallowed);
    assert_eq!(decide(body, "bot", "/%c3%84"), RobotsDecision::Disallowed);
    assert_eq!(decide(body, "bot", "/Ä"), RobotsDecision::Disallowed);
    // Reserved `/` stays encoded, so `%2F` and `/` are distinct.
    assert_eq!(decide(body, "bot", "/a%2Fb"), RobotsDecision::Disallowed);
    assert_eq!(decide(body, "bot", "/a/b"), RobotsDecision::Allowed);
    // A stray `%` is treated as a literal percent sign on both sides.
    assert_eq!(decide(body, "bot", "/100%25"), RobotsDecision::Disallowed);
    assert_eq!(decide(body, "bot", "/100%"), RobotsDecision::Disallowed);
    // Matching is case-sensitive for path octets.
    assert_eq!(decide(body, "bot", "/~JOE"), RobotsDecision::Allowed);

    // RFC 9309 section 2.2.3: encoded `*` and `$` in a pattern match those octets literally.
    let literal = "User-agent: *\nDisallow: /star%2Aend\nDisallow: /cost%24\n";
    assert_eq!(
        decide(literal, "bot", "/star*end"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(literal, "bot", "/star%2aend"),
        RobotsDecision::Disallowed
    );
    assert_eq!(decide(literal, "bot", "/starXend"), RobotsDecision::Allowed);
    assert_eq!(decide(literal, "bot", "/cost$"), RobotsDecision::Disallowed);
    assert_eq!(
        decide(literal, "bot", "/cost%24/x"),
        RobotsDecision::Disallowed
    );
    assert_eq!(decide(literal, "bot", "/cost"), RobotsDecision::Allowed);
}

#[test]
fn hostile_formatting_is_parsed_deterministically_and_only_valid_lines_apply() {
    let mut body = Vec::new();
    body.extend_from_slice(b"\xEF\xBB\xBF# leading comment\r\n");
    body.extend_from_slice(b"Disallow: /orphan\r\n");
    body.extend_from_slice(b"  USER-AGENT :\tbot # trailing comment\r");
    body.extend_from_slice(b"Sitemap: https://example.com/sitemap.xml\n");
    body.extend_from_slice(b"Crawl-delay: 10\n");
    body.extend_from_slice(b"Disallow: /bad\xFF\n");
    body.extend_from_slice(b"Disallow /nocolon\n");
    body.extend_from_slice(b"Disallow: relative\n");
    body.extend_from_slice(b"Disallow: /ctl\x01x\n");
    body.extend_from_slice(b"DISALLOW: /secret#fragment\r\n");
    body.extend_from_slice(b"allow:/secret/open\n");
    let rules = RobotsRules::parse(&body);
    let agent = token("bot");
    let check = |path: &str| rules.decide(&agent, path).expect("path").decision();
    assert_eq!(check("/secret/x"), RobotsDecision::Disallowed);
    assert_eq!(check("/secret/open"), RobotsDecision::Allowed);
    assert_eq!(check("/orphan"), RobotsDecision::Allowed);
    assert_eq!(check("/bad"), RobotsDecision::Allowed);
    assert_eq!(check("/bad%FF"), RobotsDecision::Disallowed);
    assert_eq!(check("/nocolon"), RobotsDecision::Allowed);
    assert_eq!(check("/relative"), RobotsDecision::Allowed);
    assert_eq!(check("/ctl"), RobotsDecision::Allowed);
    assert_eq!(rules.rule_count(), 3);
    assert!(!rules.body_truncated());
}

#[test]
fn user_agent_values_use_their_leading_product_token_and_invalid_values_match_nothing() {
    // `bot/2.0` names the `bot` product token, so both groups merge for `bot`.
    // `2bot` and `*bot` have no valid leading token and start groups that match no crawler.
    let body = "User-agent: bot\nDisallow: /a\nUser-agent: bot/2.0\nDisallow: /b\n\
User-agent: 2bot\nDisallow: /c\nUser-agent: *bot\nDisallow: /e\nUser-agent: *\nDisallow: /d\n";
    assert_eq!(decide(body, "bot", "/a"), RobotsDecision::Disallowed);
    assert_eq!(decide(body, "bot", "/b"), RobotsDecision::Disallowed);
    assert_eq!(decide(body, "bot", "/c"), RobotsDecision::Allowed);
    assert_eq!(decide(body, "bot", "/d"), RobotsDecision::Allowed);
    assert_eq!(decide(body, "other", "/b"), RobotsDecision::Allowed);
    assert_eq!(decide(body, "other", "/c"), RobotsDecision::Allowed);
    assert_eq!(decide(body, "other", "/e"), RobotsDecision::Allowed);
    assert_eq!(decide(body, "other", "/d"), RobotsDecision::Disallowed);
    // A product token is not matched as a prefix of a longer group token.
    assert_eq!(
        decide("User-agent: botnet\nDisallow: /\n", "bot", "/x"),
        RobotsDecision::Allowed
    );
}

#[test]
fn empty_rules_bodies_and_robots_txt_itself_are_allowed() {
    for body in ["", "   \n# only comments\n\n", "User-agent: *\nDisallow:\n"] {
        let evaluation = RobotsRules::parse(body.as_bytes())
            .decide(&token("bot"), "/anything")
            .expect("path");
        assert_eq!(evaluation.decision(), RobotsDecision::Allowed, "{body:?}");
    }
    let none = RobotsRules::parse(b"User-agent: other\nDisallow: /\n")
        .decide(&token("bot"), "/x")
        .expect("path");
    assert_eq!(none.basis(), RobotsBasis::NoApplicableGroup);
    let unmatched = RobotsRules::parse(b"User-agent: bot\nDisallow: /a\n")
        .decide(&token("bot"), "/b")
        .expect("path");
    assert_eq!(unmatched.basis(), RobotsBasis::NoMatchingRule);

    let robots = RobotsRules::parse(b"User-agent: *\nDisallow: /robots.txt\n")
        .decide(&token("bot"), "/robots.txt")
        .expect("path");
    assert_eq!(robots.decision(), RobotsDecision::Allowed);
    assert_eq!(robots.basis(), RobotsBasis::ImplicitRobotsTxt);
    assert_eq!(
        decide("User-agent: *\nDisallow: /robots\n", "bot", "/robots.txt?x"),
        RobotsDecision::Disallowed
    );
}

#[test]
fn fetch_outcomes_follow_rfc_9309_access_results() {
    let agent = token("bot");
    let evaluate_outcome =
        |outcome, policy| evaluate_robots(outcome, policy, &agent, "/x").expect("valid path");

    let unreachable = evaluate_outcome(
        RobotsFetchOutcome::Unreachable,
        UnavailableRobotsPolicy::AllowPerRfc9309,
    );
    assert_eq!(unreachable.decision(), RobotsDecision::Disallowed);
    assert_eq!(unreachable.basis(), RobotsBasis::Unreachable);

    for (outcome, basis) in [
        (RobotsFetchOutcome::Unavailable, RobotsBasis::Unavailable),
        (
            RobotsFetchOutcome::RedirectLimitExceeded,
            RobotsBasis::RedirectLimitExceeded,
        ),
    ] {
        let allowed = evaluate_outcome(outcome, UnavailableRobotsPolicy::AllowPerRfc9309);
        assert_eq!(allowed.decision(), RobotsDecision::Allowed);
        assert_eq!(allowed.basis(), basis);
        let unknown = evaluate_outcome(outcome, UnavailableRobotsPolicy::TreatAsUnknown);
        assert_eq!(unknown.decision(), RobotsDecision::Unknown);
        assert_eq!(unknown.basis(), basis);
    }

    let success = evaluate_outcome(
        RobotsFetchOutcome::Success {
            body: b"User-agent: *\nDisallow: /x\n",
        },
        UnavailableRobotsPolicy::TreatAsUnknown,
    );
    assert_eq!(success.decision(), RobotsDecision::Disallowed);

    // Even an unreachable robots file never makes a hostile path acceptable.
    assert_eq!(
        evaluate_robots(
            RobotsFetchOutcome::Unreachable,
            UnavailableRobotsPolicy::AllowPerRfc9309,
            &agent,
            "x",
        ),
        Err(RobotsPathError::MissingLeadingSlash)
    );
}

#[test]
fn bodies_beyond_the_parse_limit_fail_closed_without_admitting_a_partial_line() {
    assert_eq!(MAX_ROBOTS_BODY_BYTES, 512_000);
    let prefix = "User-agent: *\n";
    let rule = "Disallow: /late\n";

    let build = |rule_end: usize| {
        let mut body = String::from(prefix);
        let padding = rule_end - prefix.len() - rule.len();
        body.push_str(&"#".repeat(padding - 1));
        body.push('\n');
        body.push_str(rule);
        assert_eq!(body.len(), rule_end);
        body.push_str("Disallow: /after\n");
        body
    };

    // The complete rule is retained, but omitted bytes make the overall decision unknown.
    let at_limit = RobotsRules::parse(build(MAX_ROBOTS_BODY_BYTES).as_bytes());
    assert!(at_limit.body_truncated());
    let agent = token("bot");
    let check =
        |rules: &RobotsRules, path: &str| rules.decide(&agent, path).expect("path").decision();
    assert_eq!(at_limit.rule_count(), 1);
    assert_eq!(check(&at_limit, "/late"), RobotsDecision::Unknown);
    assert_eq!(check(&at_limit, "/after"), RobotsDecision::Unknown);

    // One byte later the rule line is cut before its newline and is dropped.
    let over = RobotsRules::parse(build(MAX_ROBOTS_BODY_BYTES + 1).as_bytes());
    assert!(over.body_truncated());
    assert_eq!(over.rule_count(), 0);
    assert_eq!(check(&over, "/late"), RobotsDecision::Unknown);

    // A truncated `Allow: /public` must not become `Allow: /`.
    let mut widening = String::from("User-agent: *\nDisallow: /\n");
    widening.push_str(&"#".repeat(MAX_ROBOTS_BODY_BYTES - widening.len() - 9));
    widening.push('\n');
    widening.push_str("Allow: /public\n");
    let widening = RobotsRules::parse(widening.as_bytes());
    assert_eq!(widening.rule_count(), 1);
    assert_eq!(check(&widening, "/private"), RobotsDecision::Unknown);

    // Exactly-sized body without a trailing newline is complete, not truncated.
    let mut exact = String::from("User-agent: *\n");
    exact.push_str(&"#".repeat(MAX_ROBOTS_BODY_BYTES - exact.len() - "\nDisallow: /e".len()));
    exact.push_str("\nDisallow: /e");
    assert_eq!(exact.len(), MAX_ROBOTS_BODY_BYTES);
    let exact = RobotsRules::parse(exact.as_bytes());
    assert!(!exact.body_truncated());
    assert_eq!(check(&exact, "/e"), RobotsDecision::Disallowed);

    let evaluation = evaluate_robots(
        RobotsFetchOutcome::Success {
            body: build(MAX_ROBOTS_BODY_BYTES + 1).as_bytes(),
        },
        UnavailableRobotsPolicy::AllowPerRfc9309,
        &agent,
        "/x",
    )
    .expect("path");
    assert!(evaluation.body_truncated());
}

#[test]
fn excessive_rule_counts_fail_closed_as_unknown() {
    let mut body = String::from("User-agent: *\n");
    for index in 0..MAX_ROBOTS_RULES {
        body.push_str(&format!("Allow: /r{index}\n"));
    }
    let bounded = RobotsRules::parse(body.as_bytes());
    assert_eq!(bounded.rule_count(), MAX_ROBOTS_RULES);
    assert!(!bounded.rule_limit_exceeded());
    assert_eq!(
        bounded
            .decide(&token("bot"), "/r7")
            .expect("path")
            .decision(),
        RobotsDecision::Allowed
    );

    body.push_str("Allow: /overflow\n");
    let exceeded = RobotsRules::parse(body.as_bytes());
    assert!(exceeded.rule_limit_exceeded());
    let evaluation = exceeded.decide(&token("bot"), "/r7").expect("path");
    assert_eq!(evaluation.decision(), RobotsDecision::Unknown);
    assert_eq!(evaluation.basis(), RobotsBasis::RuleLimitExceeded);
    // robots.txt stays implicitly allowed even when the rest is unknown.
    assert_eq!(
        exceeded
            .decide(&token("bot"), "/robots.txt")
            .expect("path")
            .decision(),
        RobotsDecision::Allowed
    );
}

#[test]
fn pathological_wildcards_complete_in_bounded_time() {
    let mut body = String::from("User-agent: *\n");
    let pattern = format!("/{}b", "*a".repeat(1_200));
    for _ in 0..200 {
        body.push_str("Disallow: ");
        body.push_str(&pattern);
        body.push('\n');
    }
    let rules = RobotsRules::parse(body.as_bytes());
    assert!(!rules.body_truncated());
    assert_eq!(rules.rule_count(), 200);
    let path = format!("/{}", "a".repeat(MAX_ROBOTS_PATH_BYTES - 1));
    let started = std::time::Instant::now();
    let evaluation = rules.decide(&token("bot"), &path).expect("path");
    assert_eq!(evaluation.decision(), RobotsDecision::Allowed);
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
}

#[test]
fn hostile_request_paths_are_rejected_rather_than_matched() {
    let rules = RobotsRules::parse(b"User-agent: *\nDisallow: /admin\n");
    let agent = token("bot");
    for (path, error) in [
        ("", RobotsPathError::MissingLeadingSlash),
        ("admin", RobotsPathError::MissingLeadingSlash),
        ("http://x/admin", RobotsPathError::MissingLeadingSlash),
        ("/admin\n", RobotsPathError::ControlCharacter),
        ("/ad\0min", RobotsPathError::ControlCharacter),
        ("/admin\u{7f}", RobotsPathError::ControlCharacter),
        ("/public#/admin", RobotsPathError::Fragment),
    ] {
        assert_eq!(rules.decide(&agent, path), Err(error), "{path:?}");
    }
    let longest = format!("/{}", "a".repeat(MAX_ROBOTS_PATH_BYTES - 1));
    assert!(rules.decide(&agent, &longest).is_ok());
    let too_long = format!("/{}", "a".repeat(MAX_ROBOTS_PATH_BYTES));
    assert_eq!(
        rules.decide(&agent, &too_long),
        Err(RobotsPathError::TooLong)
    );
    // Expansion by percent-encoding counts toward the bound.
    let expanding = format!("/{}", "Ä".repeat(MAX_ROBOTS_PATH_BYTES / 4));
    assert_eq!(
        rules.decide(&agent, &expanding),
        Err(RobotsPathError::TooLong)
    );
}

#[test]
fn errors_have_stable_display_text() {
    assert_eq!(
        RobotsPathError::MissingLeadingSlash.to_string(),
        "robots request path must start with '/'"
    );
    assert_eq!(
        RobotsPathError::ControlCharacter.to_string(),
        "robots request path contains a control character"
    );
    assert_eq!(
        RobotsPathError::Fragment.to_string(),
        "robots request path must not contain a fragment"
    );
    assert_eq!(
        RobotsPathError::TooLong.to_string(),
        "robots request path exceeds the normalized length limit"
    );
    assert_eq!(
        RobotsProductTokenError::Empty.to_string(),
        "robots product token is empty"
    );
    assert_eq!(
        RobotsProductTokenError::TooLong.to_string(),
        "robots product token exceeds the length limit"
    );
    assert_eq!(
        RobotsProductTokenError::InvalidCharacter.to_string(),
        "robots product token may contain only letters, '_' and '-'"
    );
    let error: &dyn std::error::Error = &RobotsPathError::TooLong;
    assert!(error.source().is_none());
    let error: &dyn std::error::Error = &RobotsProductTokenError::Empty;
    assert!(error.source().is_none());
}

#[test]
fn robots_evidence_feeds_policy_without_granting_crawler_mutation() {
    let site = Origin::parse("https://example.com").expect("origin");
    let digest = ActionIntentDigest::parse(
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    )
    .expect("digest");
    let agent = token("OriginWeaveBot");
    let body = b"User-agent: *\nDisallow: /private\n";

    let decision_for = |path: &str| {
        evaluate_robots(
            RobotsFetchOutcome::Success { body },
            UnavailableRobotsPolicy::TreatAsUnknown,
            &agent,
            path,
        )
        .expect("path")
        .decision()
    };
    let context = |robots: RobotsDecision, capability| {
        PolicyContext::new(
            SessionMode::Crawler,
            ExecutionPurpose::PublicCrawl,
            BTreeSet::from([capability]),
            BTreeSet::from([site.clone()]),
            BTreeSet::from([site.clone()]),
            robots,
            ApprovalEvidence::None,
        )
    };
    let request = |action: ActionKind| {
        ActionRequest::new(
            action,
            site.clone(),
            site.clone(),
            InstructionSource::EnterprisePolicy,
            SecretDelivery::None,
            digest.clone(),
        )
    };

    let observe = ActionKind::Observe;
    assert_eq!(
        evaluate(
            &request(observe),
            &context(decision_for("/public"), observe.required_capability())
        ),
        Decision::Allow
    );
    assert_eq!(
        evaluate(
            &request(observe),
            &context(decision_for("/private/x"), observe.required_capability())
        ),
        Decision::Deny(DenialReason::RobotsDisallowed)
    );
    let unknown = evaluate_robots(
        RobotsFetchOutcome::Unavailable,
        UnavailableRobotsPolicy::TreatAsUnknown,
        &agent,
        "/public",
    )
    .expect("path")
    .decision();
    assert_eq!(
        evaluate(
            &request(observe),
            &context(unknown, observe.required_capability())
        ),
        Decision::Deny(DenialReason::RobotsUnknown)
    );

    let submit = ActionKind::Submit;
    assert_eq!(
        evaluate(
            &request(submit),
            &context(decision_for("/public"), submit.required_capability())
        ),
        Decision::Deny(DenialReason::CrawlerMutation)
    );
}

#[test]
fn evaluation_evidence_is_value_free() {
    let rules = RobotsRules::parse(b"User-agent: *\nDisallow: /secret-path-value\n");
    let evaluation = rules
        .decide(&token("bot"), "/secret-path-value/x")
        .expect("path");
    let debug = format!("{evaluation:?}");
    assert!(!debug.contains("secret-path-value"), "{debug}");
    assert!(debug.contains("Disallow"));
}
