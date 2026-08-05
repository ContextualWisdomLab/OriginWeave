#![allow(clippy::expect_used)]

use originweave_resource::{
    BudgetError, ResourceBudget, ResourceDirective, ResourceGovernor, ResourceSnapshot,
};

#[test]
fn budget_rejects_invalid_limits() {
    assert_eq!(
        ResourceBudget::new(0, 8, 2, 4, 4, 16),
        Err(BudgetError::ZeroLimit)
    );
    assert_eq!(
        ResourceBudget::new(9, 8, 2, 4, 4, 16),
        Err(BudgetError::SoftExceedsHard)
    );
    assert_eq!(
        ResourceBudget::new(4, 8, 5, 4, 4, 16),
        Err(BudgetError::SoftExceedsHard)
    );
    assert_eq!(
        ResourceBudget::new(4, 8, 2, 4, 0, 16),
        Err(BudgetError::ZeroLimit)
    );
    assert_eq!(
        ResourceBudget::new(4, 8, 2, 4, 4, 0),
        Err(BudgetError::ZeroLimit)
    );
}

#[test]
fn governor_preserves_interactivity_before_agent_throughput() {
    let governor = ResourceGovernor::new(
        ResourceBudget::new(4_096, 8_192, 2_048, 4_096, 8, 16).expect("budget"),
    );

    assert_eq!(
        governor.decide(ResourceSnapshot::new(2_000, 1_000, 8, false, 10)),
        ResourceDirective::Continue
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(5_000, 1_000, 8, false, 10)),
        ResourceDirective::SpillObservationCache
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(2_000, 2_500, 8, false, 10)),
        ResourceDirective::ReduceAgentBatch { next_batch_size: 4 }
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(2_000, 2_500, 1, true, 10)),
        ResourceDirective::OffloadInferenceToCpu
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(2_000, 2_500, 1, false, 10)),
        ResourceDirective::PauseAgent
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(2_000, 1_000, 8, true, 20)),
        ResourceDirective::OffloadInferenceToCpu
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(2_000, 1_000, 8, false, 20)),
        ResourceDirective::PauseAgent
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(8_500, 1_000, 8, false, 10)),
        ResourceDirective::PauseAgent
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(2_000, 4_500, 8, false, 10)),
        ResourceDirective::RejectNewAgentWork
    );
    assert_eq!(
        governor.decide(ResourceSnapshot::new(8_500, 4_500, 8, true, 30)),
        ResourceDirective::RejectNewAgentWork
    );
}

#[test]
fn budget_accessors_expose_the_validated_contract() {
    let budget = ResourceBudget::new(4, 8, 2, 4, 6, 17).expect("budget");
    assert_eq!(budget.soft_ram_mebibytes(), 4);
    assert_eq!(budget.hard_ram_mebibytes(), 8);
    assert_eq!(budget.soft_vram_mebibytes(), 2);
    assert_eq!(budget.hard_vram_mebibytes(), 4);
    assert_eq!(budget.cpu_threads(), 6);
    assert_eq!(budget.frame_budget_milliseconds(), 17);
}
