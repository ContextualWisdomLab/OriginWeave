#![allow(clippy::expect_used)]

//! Fail-closed regressions for the RFC 9309 robots evaluator found in independent review.

use originweave_core::RobotsDecision;
use originweave_policy::{RobotsEvaluation, RobotsFetchOutcome, RobotsProductToken, RobotsRules};

fn token(value: &str) -> RobotsProductToken {
    RobotsProductToken::parse(value).expect("valid product token")
}

fn evaluate(body: &[u8], agent: &str, path: &str) -> RobotsEvaluation {
    RobotsRules::parse(body)
        .decide(&token(agent), path)
        .expect("valid path")
}

fn decide(body: &[u8], agent: &str, path: &str) -> RobotsDecision {
    evaluate(body, agent, path).decision()
}

#[test]
fn non_utf8_user_agent_lines_still_start_their_own_group() {
    // A Latin-1 byte in a trailing comment must not move `Allow: /` into the `*` group.
    let comment = b"User-agent: *\nDisallow: /\n\nUser-agent: bot # caf\xe9\nAllow: /\n";
    assert_eq!(
        decide(comment, "otherbot", "/private"),
        RobotsDecision::Disallowed
    );
    assert_eq!(decide(comment, "bot", "/private"), RobotsDecision::Allowed);

    // A Latin-1 byte in the user-agent value still starts a group; it is not a wildcard.
    let value = b"User-agent: *\nDisallow: /\n\nUser-agent: b\xf6t\nAllow: /\n";
    assert_eq!(
        decide(value, "otherbot", "/private"),
        RobotsDecision::Disallowed
    );

    // A non-UTF-8 comment does not discard the rule that precedes it.
    let rule = b"User-agent: *\nDisallow: /x # caf\xe9\n";
    assert_eq!(decide(rule, "bot", "/x/y"), RobotsDecision::Disallowed);

    // A rule line with a non-UTF-8 value is ignored but still ends the user-agent run.
    let split = b"User-agent: a\nDisallow: /bad\xff\nUser-agent: b\nDisallow: /\n";
    assert_eq!(decide(split, "a", "/x"), RobotsDecision::Allowed);
    assert_eq!(decide(split, "b", "/x"), RobotsDecision::Disallowed);
    assert_eq!(RobotsRules::parse(split).rule_count(), 1);
}

#[test]
fn truncated_bodies_never_authorize_crawling() {
    for body in [vec![b'#'; originweave_policy::MAX_ROBOTS_BODY_BYTES + 1], {
        let head = "User-agent: *\nDisallow: /\n";
        let tail = "\nUser-agent: bot\n";
        let mut body = String::from(head);
        body.push_str(
            &"#".repeat(originweave_policy::MAX_ROBOTS_BODY_BYTES - head.len() - tail.len()),
        );
        body.push_str(tail);
        body.push_str("Disallow: /\n");
        body.into_bytes()
    }] {
        let result = evaluate(&body, "bot", "/private");
        assert_eq!(result.decision(), RobotsDecision::Unknown);
        assert!(result.body_truncated());
        assert_eq!(decide(&body, "bot", "/robots.txt"), RobotsDecision::Allowed);
    }
}
#[test]
fn ascii_octets_that_uris_cannot_carry_literally_match_their_percent_encoding() {
    for (rule, path) in [
        ("/a%20b", "/a b"),
        ("/x%7Cy", "/x|y"),
        ("/x|y", "/x%7Cy"),
        ("/x|y", "/x%7cy"),
        ("/q%22", "/q\""),
        ("/q\"", "/q%22"),
        ("/a%5Cb", "/a\\b"),
        ("/%7Bk%7D", "/{k}"),
        ("/{k}", "/%7bk%7d"),
        ("/%3Ctag%3E", "/<tag>"),
        ("/up%5E", "/up^"),
        ("/tick%60", "/tick`"),
    ] {
        let body = format!("User-agent: *\nDisallow: {rule}\n");
        assert_eq!(
            decide(body.as_bytes(), "bot", path),
            RobotsDecision::Disallowed,
            "{rule:?} vs {path:?}"
        );
    }
    // Ordinary octets stay distinct from unrelated encodings.
    assert_eq!(
        decide(b"User-agent: *\nDisallow: /x%7Cy\n", "bot", "/x/y"),
        RobotsDecision::Allowed
    );
}

#[test]
fn non_ascii_whitespace_in_a_pattern_is_a_literal_octet_sequence() {
    let body = "User-agent: *\nDisallow: /a\u{a0}b\nDisallow: /c d\n";
    assert_eq!(
        decide(body.as_bytes(), "bot", "/a\u{a0}b/x"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(body.as_bytes(), "bot", "/a%C2%A0b"),
        RobotsDecision::Disallowed
    );
    assert_eq!(
        decide(body.as_bytes(), "bot", "/a"),
        RobotsDecision::Allowed
    );
    // An ASCII space inside a pattern is not a valid rule (RFC 9309 section 2.2), so it is ignored.
    assert_eq!(
        decide(body.as_bytes(), "bot", "/c%20d"),
        RobotsDecision::Allowed
    );
    assert_eq!(RobotsRules::parse(body.as_bytes()).rule_count(), 1);
}

#[test]
fn parsed_rules_and_fetch_outcomes_debug_without_values() {
    let rules = RobotsRules::parse(b"User-agent: secretbot\nDisallow: /secret-pattern\n");
    let debug = format!("{rules:?}");
    assert!(!debug.contains("secretbot"), "{debug}");
    assert!(!debug.contains("secret-pattern"), "{debug}");
    assert_eq!(
        debug,
        "RobotsRules { group_count: 1, rule_count: 1, body_truncated: false, rule_limit_exceeded: false }"
    );

    let success = RobotsFetchOutcome::Success {
        body: b"Disallow: /secret-pattern",
    };
    assert_eq!(format!("{success:?}"), "Success { body_bytes: 25 }");
    assert_eq!(
        format!("{:?}", RobotsFetchOutcome::Unavailable),
        "Unavailable"
    );
    assert_eq!(
        format!("{:?}", RobotsFetchOutcome::Unreachable),
        "Unreachable"
    );
    assert_eq!(
        format!("{:?}", RobotsFetchOutcome::RedirectLimitExceeded),
        "RedirectLimitExceeded"
    );
}
