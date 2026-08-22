use originweave_core::release_acceptance::{
    BenchmarkSuite, BenchmarkSuiteOutcome, DeclaredLimitation, ReleaseDecision,
    ReleaseDecisionError, decide_release,
};

fn passing_results() -> Vec<(BenchmarkSuite, BenchmarkSuiteOutcome)> {
    BenchmarkSuite::ALL
        .into_iter()
        .map(|suite| (suite, BenchmarkSuiteOutcome::Passed))
        .collect()
}

fn declared_limitation() -> Result<DeclaredLimitation, ReleaseDecisionError> {
    DeclaredLimitation::new(
        "linux_arm64",
        "Linux ARM64 is not included in the declared release support profile.",
    )
}

#[test]
fn complete_passing_evidence_is_accepted_without_declared_limitations()
-> Result<(), ReleaseDecisionError> {
    let report = decide_release(passing_results(), &[])?;

    assert_eq!(report.decision(), ReleaseDecision::Accepted);
    assert!(report.failed_suites().is_empty());
    assert!(report.inconclusive_suites().is_empty());
    assert!(report.missing_suites().is_empty());
    assert!(report.declared_limitations().is_empty());
    Ok(())
}

#[test]
fn complete_passing_evidence_preserves_declared_limitation_details()
-> Result<(), ReleaseDecisionError> {
    let limitation = declared_limitation()?;
    let report = decide_release(passing_results(), std::slice::from_ref(&limitation))?;

    assert_eq!(
        report.decision(),
        ReleaseDecision::AcceptedWithDeclaredLimitations
    );
    assert_eq!(report.declared_limitations(), &[limitation]);
    Ok(())
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
fn limitation_rejects_control_characters_in_release_metadata() {
    assert_eq!(
        DeclaredLimitation::new(
            "linux_arm64\nforged_release_claim",
            "Linux ARM64 is unsupported."
        ),
        Err(ReleaseDecisionError::InvalidLimitationClaim)
    );
    assert_eq!(
        DeclaredLimitation::new(
            "linux_arm64",
            "Linux ARM64 is unsupported.\rforged_release_consequence"
        ),
        Err(ReleaseDecisionError::InvalidLimitationConsequence)
    );
}

#[test]
fn limitation_rejects_ambiguous_unicode_formatting_characters() {
    for character in ['\u{202e}', '\u{200b}', '\u{00ad}', '\u{2066}', '\u{feff}'] {
        assert_eq!(
            DeclaredLimitation::new(
                format!("linux_arm64{character}forged_release_claim"),
                "Linux ARM64 is unsupported."
            ),
            Err(ReleaseDecisionError::InvalidLimitationClaim)
        );
        assert_eq!(
            DeclaredLimitation::new(
                "linux_arm64",
                format!("Linux ARM64 is unsupported.{character}forged_release_consequence")
            ),
            Err(ReleaseDecisionError::InvalidLimitationConsequence)
        );
    }
}

#[test]
fn limitation_errors_have_deterministic_standard_error_contracts() {
    let cases = [
        (
            ReleaseDecisionError::EmptyLimitationClaim,
            "declared release limitation must name an unsupported claim",
        ),
        (
            ReleaseDecisionError::InvalidLimitationClaim,
            "declared release limitation claim contains a control character",
        ),
        (
            ReleaseDecisionError::EmptyLimitationConsequence,
            "declared release limitation must state a buyer-visible consequence",
        ),
        (
            ReleaseDecisionError::InvalidLimitationConsequence,
            "declared release limitation consequence contains a control character",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }
}

#[test]
fn limitation_exposes_the_exact_narrowed_claim_and_consequence() -> Result<(), ReleaseDecisionError>
{
    let limitation = declared_limitation()?;

    assert_eq!(limitation.unsupported_claim(), "linux_arm64");
    assert_eq!(
        limitation.buyer_consequence(),
        "Linux ARM64 is not included in the declared release support profile."
    );
    Ok(())
}

#[test]
fn every_mandatory_suite_is_required_for_acceptance() -> Result<(), ReleaseDecisionError> {
    for omitted_suite in BenchmarkSuite::ALL {
        let evidence = passing_results()
            .into_iter()
            .filter(|(suite, _)| *suite != omitted_suite)
            .collect::<Vec<_>>();

        let report = decide_release(evidence, &[])?;

        assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
        assert_eq!(report.missing_suites(), &[omitted_suite]);
        assert!(report.failed_suites().is_empty());
    }
    Ok(())
}

#[test]
fn explicit_inconclusive_suite_evidence_cannot_be_promoted_to_acceptance()
-> Result<(), ReleaseDecisionError> {
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
        let limitation = declared_limitation()?;

        let report = decide_release(evidence, std::slice::from_ref(&limitation))?;

        assert_eq!(report.decision(), ReleaseDecision::Inconclusive);
        assert_eq!(report.inconclusive_suites(), &[inconclusive_suite]);
        assert_eq!(report.declared_limitations(), &[limitation]);
    }
    Ok(())
}

#[test]
fn any_known_threshold_failure_rejects_release_and_identifies_the_suite()
-> Result<(), ReleaseDecisionError> {
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
        let limitation = declared_limitation()?;

        let report = decide_release(evidence, std::slice::from_ref(&limitation))?;

        assert_eq!(report.decision(), ReleaseDecision::Rejected);
        assert_eq!(report.failed_suites(), &[failed_suite]);
        assert_eq!(report.declared_limitations(), &[limitation]);
    }
    Ok(())
}

#[test]
fn known_failure_remains_rejected_when_other_evidence_is_incomplete()
-> Result<(), ReleaseDecisionError> {
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
        &[],
    )?;

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
    Ok(())
}

#[test]
fn duplicate_suite_evidence_fails_closed_instead_of_overwriting_results() {
    for duplicate_suite in BenchmarkSuite::ALL {
        let expected_error = ReleaseDecisionError::DuplicateSuite(duplicate_suite);
        assert_eq!(
            decide_release(
                [
                    (duplicate_suite, BenchmarkSuiteOutcome::Passed),
                    (duplicate_suite, BenchmarkSuiteOutcome::Failed),
                ],
                &[],
            ),
            Err(expected_error)
        );

        assert_eq!(
            expected_error.to_string(),
            format!(
                "benchmark release evidence contains duplicate suite: {}",
                duplicate_suite.as_str()
            )
        );
        let standard_error: &dyn std::error::Error = &expected_error;
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
