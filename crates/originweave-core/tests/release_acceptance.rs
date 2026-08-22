use originweave_core::release_acceptance::{
    BenchmarkSuite, BenchmarkSuiteOutcome, DeclaredLimitation, ReleaseDecision, ReleaseDecisionError,
    decide_release,
};

fn passing_results() -> Vec<(BenchmarkSuite, BenchmarkSuiteOutcome)> {
    BenchmarkSuite::ALL
        .into_iter()
        .map(|suite| (suite, BenchmarkSuiteOutcome::Passed))
        .collect()
}

fn declared_limitation() -> DeclaredLimitation {
    let Ok(limitation) = DeclaredLimitation::new(
        "linux_arm64",
        "Linux ARM64 is not included in the declared release support profile.",
    ) else {
        panic!("fixture limitation must be valid");
    };
    limitation
}

#[test]
fn complete_passing_evidence_is_accepted_without_declared_limitations() {
    let Ok(report) = decide_release(passing_results(), &[]) else {
        panic!("complete unique suite evidence must produce a report");
    };

    assert_eq!(report.decision(), ReleaseDecision::Accepted);
    assert!(report.failed_suites().is_empty());
    assert!(report.inconclusive_suites().is_empty());
    assert!(report.missing_suites().is_empty());
    assert!(report.declared_limitations().is_empty());
}

#[test]
fn complete_passing_evidence_preserves_declared_limitation_details() {
    let limitation = declared_limitation();
    let Ok(report) = decide_release(passing_results(), std::slice::from_ref(&limitation)) else {
        panic!("complete unique suite evidence must produce a report");
    };

    assert_eq!(
        report.decision(),
        ReleaseDecision::AcceptedWithDeclaredLimitations
    );
    assert_eq!(report.declared_limitations(), &[limitation]);
}

#[test]
fn limitation_requires_an_unsupported_claim() {
    assert_eq!(
        DeclaredLimitation::new(
            "   ",
            "A buyer-visible consequence must not stand without the narrowed claim.",
        ),
        Err(ReleaseDecisionError::EmptyLimitationClaim)
    );
}

#[test]
fn limitation_requires_a_buyer_visible_consequence() {
    assert_eq!(
        DeclaredLimitation::new("linux_arm64", "\t\n"),
        Err(ReleaseDecisionError::EmptyLimitationConsequence)
    );
}

#[test]
fn limitation_exposes_the_exact_narrowed_claim_and_consequence() {
    let limitation = declared_limitation();

    assert_eq!(limitation.unsupported_claim(), "linux_arm64");
    assert_eq!(
        limitation.buyer_consequence(),
        "Linux ARM64 is not included in the declared release support profile."
    );
}

#[test]
fn every_mandatory_suite_is_required_for_acceptance() {
    for omitted_suite in BenchmarkSuite::ALL {
        let evidence = passing_results()
            .into_iter()
            .filter(|(suite, _)| *suite != omitted_suite)
            .collect::<Vec<_>>();

        let Ok(report) = decide_release(evidence, &[]) else {
            panic!("remaining suite identities must be unique");
        };

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
        let limitation = declared_limitation();

        let Ok(report) = decide_release(evidence, std::slice::from_ref(&limitation)) else {
            panic!("suite identities must be unique");
        };

        assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
        assert_eq!(report.inconclusive_suites(), &[inconclusive_suite]);
        assert_eq!(report.declared_limitations(), &[limitation]);
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
        let limitation = declared_limitation();

        let Ok(report) = decide_release(evidence, std::slice::from_ref(&limitation)) else {
            panic!("suite identities must be unique");
        };

        assert_eq!(report.decision(), ReleaseDecision::Rejected);
        assert_eq!(report.failed_suites(), &[failed_suite]);
        assert_eq!(report.declared_limitations(), &[limitation]);
    }
}

#[test]
fn known_failure_remains_rejected_when_other_evidence_is_incomplete() {
    let Ok(report) = decide_release(
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
        &[],
    ) else {
        panic!("suite identities must be unique");
    };

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
        let Err(error) = decide_release(
            [
                (duplicate_suite, BenchmarkSuiteOutcome::Passed),
                (duplicate_suite, BenchmarkSuiteOutcome::Failed),
            ],
            &[],
        ) else {
            panic!("duplicate suite evidence must fail closed");
        };

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
fn duplicate_suite_evidence_in_vector_input_also_fails_closed() {
    let duplicate_suite = BenchmarkSuite::ControlledDeterministic;
    let mut evidence = passing_results();
    evidence.push((duplicate_suite, BenchmarkSuiteOutcome::Failed));

    assert_eq!(
        decide_release(evidence, &[]),
        Err(ReleaseDecisionError::DuplicateSuite(duplicate_suite))
    );
}

#[test]
fn decision_is_independent_of_evidence_input_order() {
    let mut reversed = passing_results();
    reversed.reverse();

    assert_eq!(
        decide_release(reversed, &[]),
        decide_release(passing_results(), &[])
    );
}
