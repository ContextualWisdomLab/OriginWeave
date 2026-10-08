#![allow(clippy::expect_used)]

//! Offline robots example acceptance tests.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use originweave_core::{
    ActionIntentDigest, ActionKind, ActionRequest, ApprovalEvidence, Capability, ExecutionPurpose,
    InstructionSource, Origin, PolicyContext, RobotsDecision, SecretDelivery, SessionMode,
};
use originweave_policy::{Decision, DenialReason, evaluate};

const ROBOTS_BODY: &str = "User-agent: *\n\
Disallow: /private\n\
Allow: /private/public\n\
Disallow: /a%2Fb\n\
Disallow: /*.pdf$\n";

struct TempCase {
    root: PathBuf,
}

impl TempCase {
    fn new(name: &str) -> Self {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "originweave-robots-offline-{}-{}-{}",
            name,
            std::process::id(),
            unique_suffix()
        ));
        fs::create_dir(&root).expect("create temporary directory");
        Self { root }
    }

    fn write_body(&self, name: &str, body: &[u8]) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, body).expect("write robots fixture");
        path
    }
}

impl Drop for TempCase {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos()
}

/// Use file-backed capture to keep the child deadline independent of pipe capacity.
fn bounded_output(command: &mut Command, timeout: Duration) -> Output {
    let capture = TempCase::new("capture");
    let output_path = capture.root.join("stdout");
    let error_path = capture.root.join("stderr");
    command
        .stdout(Stdio::from(
            fs::File::create(&output_path).expect("stdout file"),
        ))
        .stderr(Stdio::from(
            fs::File::create(&error_path).expect("stderr file"),
        ));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn().expect("spawn owned child");
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("inspect child") {
            break status;
        }
        if started.elapsed() >= timeout {
            #[cfg(unix)]
            let stopped = Command::new("/bin/kill")
                .args(["-KILL", "--", &format!("-{}", child.id())])
                .status()
                .expect("signal owned process group");
            #[cfg(not(unix))]
            child.kill().expect("stop timed-out child");
            let status = child.wait().expect("wait for timed-out child");
            #[cfg(unix)]
            assert!(stopped.success(), "owned group signal failed");
            assert!(
                started.elapsed() < timeout,
                "child deadline exceeded: {status}"
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    Output {
        status,
        stdout: fs::read(output_path).expect("read stdout"),
        stderr: fs::read(error_path).expect("read stderr"),
    }
}

/// Build once per test process instead of racing repeated Cargo rebuilds per case.
fn example_binary() -> &'static PathBuf {
    static BINARY: OnceLock<PathBuf> = OnceLock::new();
    BINARY.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let target = root.join("target/robots-offline-cli-tests");
        let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
        command
            .current_dir(&root)
            .args([
                "build",
                "--quiet",
                "--locked",
                "--package",
                "originweave-policy",
                "--example",
                "robots_offline",
                "--target-dir",
            ])
            .arg(&target);
        let output = bounded_output(&mut command, Duration::from_secs(180));
        assert_status(&output, 0);
        target
            .join("debug/examples")
            .join(format!("robots_offline{}", std::env::consts::EXE_SUFFIX))
    })
}

fn run_example<I, S>(arguments: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    bounded_output(
        Command::new(example_binary()).args(arguments),
        Duration::from_secs(10),
    )
}

fn run_for(body_path: &Path, token: &str, target: &str) -> Output {
    run_example([
        OsStr::new("--body-file"),
        body_path.as_os_str(),
        OsStr::new("--crawler-token"),
        OsStr::new(token),
        OsStr::new("--request-target"),
        OsStr::new(target),
    ])
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

fn assert_status(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stderr: {}",
        stderr(output)
    );
}

#[test]
fn offline_example_reports_value_free_policy_results() {
    let temp = TempCase::new("value-free");
    let body = temp.write_body("robots.txt", ROBOTS_BODY.as_bytes());

    for (target, expected, exit) in [
        (
            "/private/public/x",
            [
                "robots_decision=Allowed",
                "basis=matched_rule",
                "rule_kind=Allow",
                "line_number=3",
                "body_truncated=false",
                "fixture_policy_decision=Allow",
                "network_authority=none",
            ]
            .as_slice(),
            0,
        ),
        (
            "/private/x",
            [
                "robots_decision=Disallowed",
                "basis=matched_rule",
                "rule_kind=Disallow",
                "line_number=2",
                "body_truncated=false",
                "fixture_policy_decision=Deny",
                "fixture_policy_reason=RobotsDisallowed",
                "network_authority=none",
            ]
            .as_slice(),
            1,
        ),
        (
            "/elsewhere",
            [
                "robots_decision=Allowed",
                "basis=no_matching_rule",
                "body_truncated=false",
                "fixture_policy_decision=Allow",
                "network_authority=none",
            ]
            .as_slice(),
            0,
        ),
        (
            "/a%2Fb",
            [
                "robots_decision=Disallowed",
                "rule_kind=Disallow",
                "line_number=4",
                "fixture_policy_reason=RobotsDisallowed",
            ]
            .as_slice(),
            1,
        ),
        (
            "/a/b",
            [
                "robots_decision=Allowed",
                "basis=no_matching_rule",
                "fixture_policy_decision=Allow",
            ]
            .as_slice(),
            0,
        ),
        (
            "/doc.pdf",
            [
                "robots_decision=Disallowed",
                "rule_kind=Disallow",
                "line_number=5",
                "fixture_policy_reason=RobotsDisallowed",
            ]
            .as_slice(),
            1,
        ),
        (
            "/doc.pdf?x=1",
            [
                "robots_decision=Allowed",
                "basis=no_matching_rule",
                "fixture_policy_decision=Allow",
            ]
            .as_slice(),
            0,
        ),
        (
            "/robots.txt",
            [
                "robots_decision=Allowed",
                "basis=implicit_robots_txt",
                "fixture_policy_decision=Allow",
            ]
            .as_slice(),
            0,
        ),
    ] {
        let output = run_for(&body, "OriginWeaveBot", target);
        assert_status(&output, exit);
        assert_eq!(stderr(&output), "");
        let printed = stdout(&output);
        for line in expected {
            assert!(printed.contains(line), "missing {line:?} in {printed}");
        }
        for secret in ["OriginWeaveBot", target, body.to_string_lossy().as_ref()] {
            assert!(!printed.contains(secret), "leaked sentinel {secret:?}");
        }
    }
}

#[test]
fn offline_example_preserves_bounds_and_rejects_invalid_inputs() {
    let temp = TempCase::new("bounds");
    let exact = temp.write_body("exact.txt", "#".repeat(512_000).as_bytes());
    let truncated = temp.write_body("truncated.txt", "#".repeat(512_001).as_bytes());

    let exact_output = run_for(&exact, "a", "/public");
    assert_status(&exact_output, 0);
    assert!(stdout(&exact_output).contains("body_truncated=false"));
    assert!(stdout(&exact_output).contains("basis=no_applicable_group"));

    let truncated_output = run_for(&truncated, "a", "/public");
    assert_status(&truncated_output, 1);
    assert!(stdout(&truncated_output).contains("robots_decision=Unknown"));
    assert!(stdout(&truncated_output).contains("basis=truncated_body"));
    assert!(stdout(&truncated_output).contains("body_truncated=true"));
    assert!(stdout(&truncated_output).contains("fixture_policy_reason=RobotsUnknown"));

    let implicit_output = run_for(&truncated, "a", "/robots.txt");
    assert_status(&implicit_output, 0);
    assert!(stdout(&implicit_output).contains("basis=implicit_robots_txt"));
    assert!(stdout(&implicit_output).contains("body_truncated=true"));

    let max_token = "a".repeat(128);
    assert_status(&run_for(&exact, &max_token, "/public"), 0);
    for bad_token in ["*", "Bot/1.0", "", &"a".repeat(129)] {
        let output = run_for(&exact, bad_token, "/public");
        assert_status(&output, 2);
        assert_eq!(stdout(&output), "");
        assert_eq!(stderr(&output), "input_error\n");
    }

    for bad_target in [
        "https://example.com/",
        "relative",
        "/has\nnewline",
        "/x#fragment",
    ] {
        let output = run_for(&exact, "a", bad_target);
        assert_status(&output, 2);
        assert_eq!(stdout(&output), "");
        assert_eq!(stderr(&output), "input_error\n");
    }

    let long_target = format!("/{}", "a".repeat(8_191));
    assert_status(&run_for(&exact, "a", &long_target), 0);
    let too_long_target = format!("/{}", "a".repeat(8_192));
    assert_status(&run_for(&exact, "a", &too_long_target), 2);
    let percent_target = format!("/{}", "%61".repeat(8_191));
    assert_status(&run_for(&exact, "a", &percent_target), 0);

    let ten_thousand_rules = temp.write_body(
        "limit.txt",
        format!("User-agent: *\n{}", "Disallow: /blocked\n".repeat(10_000)).as_bytes(),
    );
    let within_limit = run_for(&ten_thousand_rules, "a", "/blocked");
    assert_status(&within_limit, 1);
    assert!(stdout(&within_limit).contains("basis=matched_rule"));
    assert!(stdout(&within_limit).contains("fixture_policy_reason=RobotsDisallowed"));

    let too_many_rules = temp.write_body(
        "too-many.txt",
        format!("User-agent: *\n{}", "Disallow: /blocked\n".repeat(10_001)).as_bytes(),
    );
    let exceeded = run_for(&too_many_rules, "a", "/blocked");
    assert_status(&exceeded, 1);
    assert!(stdout(&exceeded).contains("robots_decision=Unknown"));
    assert!(stdout(&exceeded).contains("basis=rule_limit_exceeded"));
    assert!(stdout(&exceeded).contains("fixture_policy_reason=RobotsUnknown"));

    let missing = temp.root.join("missing.txt");
    assert_status(&run_for(&missing, "a", "/public"), 2);
    assert_status(&run_for(&temp.root, "a", "/public"), 2);

    let missing_option = run_example([
        OsStr::new("--body-file"),
        exact.as_os_str(),
        OsStr::new("--crawler-token"),
        OsStr::new("a"),
    ]);
    assert_status(&missing_option, 2);
    assert_eq!(stdout(&missing_option), "");
    assert_eq!(stderr(&missing_option), "input_error\n");

    let unknown_option = run_example([
        OsStr::new("--body-file"),
        exact.as_os_str(),
        OsStr::new("--crawler-token"),
        OsStr::new("a"),
        OsStr::new("--request-target"),
        OsStr::new("/public"),
        OsStr::new("--fetch-status"),
        OsStr::new("unavailable"),
    ]);
    assert_status(&unknown_option, 2);
    assert_eq!(stdout(&unknown_option), "");
    assert_eq!(stderr(&unknown_option), "input_error\n");

    let duplicate = run_example([
        OsStr::new("--body-file"),
        exact.as_os_str(),
        OsStr::new("--body-file"),
        exact.as_os_str(),
        OsStr::new("--request-target"),
        OsStr::new("/public"),
    ]);
    assert_status(&duplicate, 2);
    assert_eq!(stdout(&duplicate), "");
    assert_eq!(stderr(&duplicate), "input_error\n");
}

#[cfg(unix)]
#[test]
fn child_timeout_stops_descendant_work_before_returning() {
    let temp = TempCase::new("timeout-descendant");
    let ready = temp.root.join("ready");
    let late = temp.root.join("late");
    let outcome = std::panic::catch_unwind(|| {
        bounded_output(
            Command::new("/bin/sh")
                .args([
                    "-c",
                    "(sleep 1; printf late > \"$2\") & printf ready > \"$1\"; wait",
                    "owned-fixture",
                ])
                .arg(&ready)
                .arg(&late),
            Duration::from_millis(300),
        )
    });
    assert!(outcome.is_err(), "timeout must refuse completion");
    assert!(ready.is_file(), "fixture reached descendant launch");
    // The finite child expires even on RED, so the fixture cannot leak indefinitely.
    std::thread::sleep(Duration::from_millis(1200));
    assert!(
        !late.exists(),
        "descendant continued writing after timeout returned"
    );
}

#[cfg(unix)]
#[test]
fn offline_example_preserves_binary_input_and_rejects_special_files() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;

    let temp = TempCase::new("unix-inputs");
    let body = temp.write_body("BODY_SENTINEL.txt", b"User-agent: *\nDisallow: /caf\xe9\n");
    let denied = run_for(&body, "TOKEN_SENTINEL", "/caf%E9");
    assert_status(&denied, 1);
    assert!(stdout(&denied).contains("fixture_policy_reason=RobotsDisallowed"));
    assert_eq!(stderr(&denied), "");
    for sentinel in ["BODY_SENTINEL", "TOKEN_SENTINEL", "/caf%E9"] {
        assert!(!stdout(&denied).contains(sentinel));
    }

    for arguments in [
        vec![
            OsString::from_vec(b"--body-\xff".to_vec()),
            body.clone().into_os_string(),
            "--crawler-token".into(),
            "a".into(),
            "--request-target".into(),
            "/public".into(),
        ],
        vec![
            "--body-file".into(),
            body.clone().into_os_string(),
            "--crawler-token".into(),
            OsString::from_vec(vec![0xff]),
            "--request-target".into(),
            "/public".into(),
        ],
        vec![
            "--body-file".into(),
            body.clone().into_os_string(),
            "--crawler-token".into(),
            "a".into(),
            "--request-target".into(),
            OsString::from_vec(b"/\xff".to_vec()),
        ],
    ] {
        let output = run_example(arguments);
        assert_status(&output, 2);
        assert_eq!(stdout(&output), "");
        assert_eq!(stderr(&output), "input_error\n");
    }

    let link = temp.root.join("symlink");
    symlink(&body, &link).expect("create symlink");
    let fifo = temp.root.join("fifo");
    let created = bounded_output(Command::new("mkfifo").arg(&fifo), Duration::from_secs(10));
    assert_status(&created, 0);
    for special in [&link, &fifo, Path::new("/dev/null")] {
        let output = run_for(special, "a", "/public");
        assert_status(&output, 2);
        assert_eq!(stdout(&output), "");
        assert_eq!(stderr(&output), "input_error\n");
    }
}

#[test]
fn robots_fixture_policy_composition_keeps_other_policy_gates_authoritative() {
    let site = Origin::parse("https://example.com").expect("origin");
    let digest = ActionIntentDigest::parse(
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    )
    .expect("digest");
    let request = |action: ActionKind, instruction_source| {
        ActionRequest::new(
            action,
            site.clone(),
            site.clone(),
            instruction_source,
            SecretDelivery::None,
            digest.clone(),
        )
    };
    let context = |capabilities: BTreeSet<Capability>, read_origins: BTreeSet<Origin>| {
        PolicyContext::new(
            SessionMode::Crawler,
            ExecutionPurpose::PublicCrawl,
            capabilities,
            read_origins,
            BTreeSet::new(),
            RobotsDecision::Allowed,
            ApprovalEvidence::None,
        )
    };
    let observe = ActionKind::Observe;
    assert_eq!(
        evaluate(
            &request(observe, InstructionSource::User),
            &context(BTreeSet::new(), BTreeSet::from([site.clone()]))
        ),
        Decision::Deny(DenialReason::MissingCapability(Capability::Observe))
    );
    assert_eq!(
        evaluate(
            &request(observe, InstructionSource::User),
            &context(BTreeSet::from([Capability::Observe]), BTreeSet::new())
        ),
        Decision::Deny(DenialReason::OriginNotReadable)
    );
    let submit = ActionKind::Submit;
    assert_eq!(
        evaluate(
            &request(submit, InstructionSource::User),
            &context(
                BTreeSet::from([Capability::Submit]),
                BTreeSet::from([site.clone()]),
            )
        ),
        Decision::Deny(DenialReason::CrawlerMutation)
    );
    assert_eq!(
        evaluate(
            &request(observe, InstructionSource::WebContent),
            &context(
                BTreeSet::from([Capability::Observe]),
                BTreeSet::from([Origin::parse("https://example.com").expect("origin")])
            )
        ),
        Decision::Deny(DenialReason::UntrustedInstructionSource)
    );
}
