#![allow(clippy::expect_used)]

use originweave_core::release_acceptance::{
    BenchmarkSuite, BenchmarkSuiteOutcome, ReleaseDecision, ReleaseDecisionError, decide_release,
};

fn passing_results() -> Vec<(BenchmarkSuite, BenchmarkSuiteOutcome)> {
    BenchmarkSuite::ALL
        .into_iter()
        .map(|suite| (suite, BenchmarkSuiteOutcome::Passed))
        .collect()
}

#[test]
fn complete_passing_evidence_is_accepted_without_declared_limitations() {
    let report = decide_release(passing_results(), false).expect("complete unique suite evidence");

    assert_eq!(report.decision(), ReleaseDecision::Accepted);
    assert!(report.failed_suites().is_empty());
    assert!(report.inconclusive_suites().is_empty());
    assert!(report.missing_suites().is_empty());
}

#[test]
fn complete_passing_evidence_preserves_declared_limitation_decision() {
    let report = decide_release(passing_results(), true).expect("complete unique suite evidence");

    assert_eq!(
        report.decision(),
        ReleaseDecision::AcceptedWithDeclaredLimitations
    );
}

#[test]
fn every_mandatory_suite_is_required_for_acceptance() {
    for omitted_suite in BenchmarkSuite::ALL {
        let evidence = passing_results()
            .into_iter()
            .filter(|(suite, _)| *suite != omitted_suite)
            .collect::<Vec<_>>();

        let report =
            decide_release(evidence, false).expect("remaining suite identities are unique");

        assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
        assert_eq!(report.missing_suites(), &[omitted_suite]);
        assert!(report.failed_suites().is_empty());
    }
}

#[test]
fn explicit_inconclusive_suite_evidence_cannot_be_promoted_to_acceptance() {
    for inconclusive_suite in BenchmarkSuite::ALL {
        let evidence = passing_results()
            .into_iter()
            .map(|(suite, outcome)| {
                if suite == inconclusive_suite {
                    (suite, BenchmarkSuiteOutcome::Inconclusive)
                } else {
                    (suite, outcome)
                }
            })
            .collect::<Vec<_>>();

        let report = decide_release(evidence, true).expect("suite identities are unique");

        assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
        assert_eq!(report.inconclusive_suites(), &[inconclusive_suite]);
    }
}

#[test]
fn any_known_threshold_failure_rejects_release_and_identifies_the_suite() {
    for failed_suite in BenchmarkSuite::ALL {
        let evidence = passing_results()
            .into_iter()
            .map(|(suite, outcome)| {
                if suite == failed_suite {
                    (suite, BenchmarkSuiteOutcome::Failed)
                } else {
                    (suite, outcome)
                }
            })
            .collect::<Vec<_>>();

        let report = decide_release(evidence, true).expect("suite identities are unique");

        assert_eq!(report.decision(), ReleaseDecision::Rejected);
        assert_eq!(report.failed_suites(), &[failed_suite]);
    }
}

#[test]
fn known_failure_remains_rejected_when_other_evidence_is_incomplete() {
    let report = decide_release(
        [
            (
                BenchmarkSuite::ControlledDeterministic,
                BenchmarkSuiteOutcome::Failed,
            ),
            (
                BenchmarkSuite::WebCompatibility,
                BenchmarkSuiteOutcome::Inconclusive,
            ),
        ],
        false,
    )
    .expect("suite identities are unique");

    assert_eq!(report.decision(), ReleaseDecision::Rejected);
    assert_eq!(
        report.failed_suites(),
        &[BenchmarkSuite::ControlledDeterministic]
    );
    assert_eq!(
        report.inconclusive_suites(),
        &[BenchmarkSuite::WebCompatibility]
    );
    assert_eq!(
        report.missing_suites(),
        &[
            BenchmarkSuite::SecurityAdversarial,
            BenchmarkSuite::ReliabilityRecovery,
            BenchmarkSuite::EnterpriseOperability,
        ]
    );
}

#[test]
fn duplicate_suite_evidence_fails_closed_instead_of_overwriting_results() {
    for duplicate_suite in BenchmarkSuite::ALL {
        let error = decide_release(
            [
                (duplicate_suite, BenchmarkSuiteOutcome::Passed),
                (duplicate_suite, BenchmarkSuiteOutcome::Failed),
            ],
            false,
        )
        .expect_err("duplicate suite evidence must fail closed");

        assert_eq!(error, ReleaseDecisionError::DuplicateSuite(duplicate_suite));
        assert_eq!(
            error.to_string(),
            format!(
                "benchmark release evidence contains duplicate suite: {}",
                duplicate_suite.as_str()
            )
        );
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}

#[test]
fn decision_is_independent_of_evidence_input_order() {
    let mut reversed = passing_results();
    reversed.reverse();

    assert_eq!(
        decide_release(reversed, false).expect("suite identities are unique"),
        decide_release(passing_results(), false).expect("suite identities are unique")
    );
}
