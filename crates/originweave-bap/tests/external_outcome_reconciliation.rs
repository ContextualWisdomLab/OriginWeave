use std::error::Error;

use originweave_bap::{
    BapExternalOutcome, BapExternalOutcomeError, BapRecoveryDirective, BapTaskEvent,
    BapTaskLifecycle, BapTaskState,
};

fn running_lifecycle() -> Result<BapTaskLifecycle, Box<dyn Error>> {
    let mut lifecycle = BapTaskLifecycle::new();
    lifecycle.apply(BapTaskEvent::Admit)?;
    lifecycle.apply(BapTaskEvent::Start)?;
    Ok(lifecycle)
}

#[test]
fn confirmed_no_side_effect_allows_retry_without_mutating_lifecycle() -> Result<(), Box<dyn Error>> {
    let lifecycle = running_lifecycle()?;
    let sequence = lifecycle.transition_sequence();

    let directive = lifecycle.classify_external_outcome(BapExternalOutcome::ConfirmedNoSideEffect)?;

    assert_eq!(directive, BapRecoveryDirective::RetryCommand);
    assert_eq!(lifecycle.state(), BapTaskState::Running);
    assert_eq!(lifecycle.transition_sequence(), sequence);
    Ok(())
}

#[test]
fn confirmed_side_effect_requires_post_condition_verification_without_mutation(
) -> Result<(), Box<dyn Error>> {
    let lifecycle = running_lifecycle()?;
    let sequence = lifecycle.transition_sequence();

    let directive = lifecycle.classify_external_outcome(BapExternalOutcome::ConfirmedSideEffect)?;

    assert_eq!(directive, BapRecoveryDirective::VerifyPostCondition);
    assert_eq!(lifecycle.state(), BapTaskState::Running);
    assert_eq!(lifecycle.transition_sequence(), sequence);
    Ok(())
}

#[test]
fn unknown_outcome_requires_explicit_reconciliation_transition() -> Result<(), Box<dyn Error>> {
    let mut lifecycle = running_lifecycle()?;

    let directive = lifecycle.classify_external_outcome(BapExternalOutcome::Unknown)?;

    assert_eq!(
        directive,
        BapRecoveryDirective::RequireReconciliation {
            outcome: BapExternalOutcome::Unknown,
        }
    );
    lifecycle.apply(BapTaskEvent::RequireReconciliation)?;
    assert_eq!(lifecycle.state(), BapTaskState::ReconciliationRequired);
    Ok(())
}

#[test]
fn explicit_reconciliation_outcome_preserves_its_distinct_cause() -> Result<(), Box<dyn Error>> {
    let lifecycle = running_lifecycle()?;

    let directive = lifecycle.classify_external_outcome(BapExternalOutcome::ReconciliationRequired)?;

    assert_eq!(
        directive,
        BapRecoveryDirective::RequireReconciliation {
            outcome: BapExternalOutcome::ReconciliationRequired,
        }
    );
    Ok(())
}

#[test]
fn outcome_classification_fails_closed_outside_running_state() {
    let lifecycle = BapTaskLifecycle::new();

    let error = lifecycle
        .classify_external_outcome(BapExternalOutcome::Unknown)
        .err();

    assert_eq!(
        error,
        Some(BapExternalOutcomeError::InvalidLifecycleState {
            state: BapTaskState::Created,
        })
    );
}

#[test]
fn outcome_error_has_stable_public_error_contract() {
    let error = BapExternalOutcomeError::InvalidLifecycleState {
        state: BapTaskState::WaitingForApproval,
    };

    assert_eq!(
        error.to_string(),
        "BAP external outcome cannot be classified from state WaitingForApproval"
    );
    assert!(error.source().is_none());
}
