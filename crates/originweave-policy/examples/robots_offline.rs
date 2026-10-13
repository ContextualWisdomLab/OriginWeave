//! Evaluate a caller-provided local robots body without fetching or executing actions.
//!
//! The policy result uses a fixed, synthetic read-only example.com fixture. It is
//! not an origin attestation, a request-intent binding, or network authority.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, Capability, ExecutionPurpose,
    InstructionSource, Origin, PolicyContext, SecretDelivery, SessionMode,
};
use originweave_policy::{
    Decision, MAX_ROBOTS_BODY_BYTES, RobotsBasis, RobotsEvaluation, RobotsFetchOutcome,
    RobotsProductToken, UnavailableRobotsPolicy, evaluate, evaluate_robots,
};

/// Explicit local inputs; no environment-derived crawler or request defaults.
struct Inputs {
    body_file: PathBuf,
    crawler_token: RobotsProductToken,
    request_target: String,
}

/// Admit three unique option/value pairs without printing caller-controlled values.
fn parse_inputs(arguments: Vec<OsString>) -> Result<Inputs, &'static str> {
    if arguments.len() != 6 {
        return Err("input_error");
    }
    let mut body_file = None;
    let mut crawler_token = None;
    let mut request_target = None;
    for pair in arguments.chunks_exact(2) {
        let option = pair.first().and_then(|value| value.to_str());
        let value = pair.get(1).ok_or("input_error")?;
        match option {
            Some("--body-file") if body_file.is_none() && !value.is_empty() => {
                body_file = Some(PathBuf::from(value));
            }
            Some("--crawler-token") if crawler_token.is_none() => {
                crawler_token = Some(
                    RobotsProductToken::parse(value.to_str().ok_or("input_error")?)
                        .map_err(|_| "input_error")?,
                );
            }
            Some("--request-target") if request_target.is_none() => {
                request_target = Some(value.to_str().ok_or("input_error")?.to_owned());
            }
            _ => return Err("input_error"),
        }
    }
    Ok(Inputs {
        body_file: body_file.ok_or("input_error")?,
        crawler_token: crawler_token.ok_or("input_error")?,
        request_target: request_target.ok_or("input_error")?,
    })
}

/// Read a cooperative local regular file with one retained overflow byte.
///
/// The pathname and descriptor checks reject static symlinks/special files, but
/// do not constitute a filesystem sandbox or prevent adversarial path replacement.
fn read_body(path: &PathBuf) -> Result<Vec<u8>, &'static str> {
    let before = fs::symlink_metadata(path).map_err(|_| "input_error")?;
    if !before.file_type().is_file() {
        return Err("input_error");
    }
    let file = File::open(path).map_err(|_| "input_error")?;
    if !file.metadata().map_err(|_| "input_error")?.is_file() {
        return Err("input_error");
    }
    let mut body = Vec::new();
    file.take((MAX_ROBOTS_BODY_BYTES + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|_| "input_error")?;
    Ok(body)
}

/// Compose the real evaluator with an explicitly synthetic read-only policy context.
fn fixture_policy(evaluation: RobotsEvaluation) -> Result<Decision, &'static str> {
    let site = Origin::parse("https://example.com").map_err(|_| "fixture_error")?;
    let digest = ActionIntentDigest::parse(
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    )
    .map_err(|_| "fixture_error")?;
    let request = ActionRequest::new(
        ActionKind::Observe,
        site.clone(),
        site.clone(),
        InstructionSource::User,
        SecretDelivery::None,
        digest,
    );
    let context = PolicyContext::new(
        SessionMode::Crawler,
        ExecutionPurpose::PublicCrawl,
        BTreeSet::from([Capability::Observe]),
        BTreeSet::from([site]),
        BTreeSet::new(),
        evaluation.decision(),
        ApprovalEvidence::None,
    );
    Ok(evaluate(&request, &context))
}

/// Format value-free decision evidence only, never request/path/body/argument values.
fn report(evaluation: RobotsEvaluation, policy: &Decision) -> String {
    let basis = match evaluation.basis() {
        RobotsBasis::MatchedRule(rule) => format!(
            "basis=matched_rule\nrule_kind={:?}\nline_number={}\n",
            rule.kind(),
            rule.line_number()
        ),
        RobotsBasis::NoApplicableGroup => "basis=no_applicable_group\n".into(),
        RobotsBasis::NoMatchingRule => "basis=no_matching_rule\n".into(),
        RobotsBasis::ImplicitRobotsTxt => "basis=implicit_robots_txt\n".into(),
        RobotsBasis::RuleLimitExceeded => "basis=rule_limit_exceeded\n".into(),
        RobotsBasis::TruncatedBody => "basis=truncated_body\n".into(),
        RobotsBasis::Unavailable => "basis=unavailable\n".into(),
        RobotsBasis::Unreachable => "basis=unreachable\n".into(),
        RobotsBasis::RedirectLimitExceeded => "basis=redirect_limit_exceeded\n".into(),
    };
    let policy_text = match policy {
        Decision::Allow => "fixture_policy_decision=Allow\n".into(),
        Decision::Deny(reason) => {
            format!("fixture_policy_decision=Deny\nfixture_policy_reason={reason:?}\n")
        }
        Decision::RequireApproval(_) => "fixture_policy_decision=RequireApproval\n".into(),
    };
    format!(
        "robots_decision={:?}\n{basis}body_truncated={}\n{policy_text}network_authority=none\n",
        evaluation.decision(),
        evaluation.body_truncated()
    )
}

/// Evaluate only local bytes, preserving the library's binary and truncation semantics.
fn run(arguments: Vec<OsString>, output: &mut impl Write) -> Result<u8, &'static str> {
    let inputs = parse_inputs(arguments)?;
    let body = read_body(&inputs.body_file)?;
    let evaluation = evaluate_robots(
        RobotsFetchOutcome::Success { body: &body },
        UnavailableRobotsPolicy::TreatAsUnknown,
        &inputs.crawler_token,
        &inputs.request_target,
    )
    .map_err(|_| "input_error")?;
    let policy = fixture_policy(evaluation)?;
    output
        .write_all(report(evaluation, &policy).as_bytes())
        .map_err(|_| "output_error")?;
    Ok(u8::from(policy != Decision::Allow))
}

/// Return policy Allow=0, policy refusal=1, and local input/output failure=2.
fn main() -> ExitCode {
    match run(
        std::env::args_os().skip(1).collect(),
        &mut io::stdout().lock(),
    ) {
        Ok(code) => ExitCode::from(code),
        Err(code) => {
            let _ = writeln!(io::stderr().lock(), "{code}");
            ExitCode::from(2)
        }
    }
}
