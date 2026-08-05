#![allow(clippy::expect_used)]

use originweave_resource::{
    BudgetError, ResourceBudget, ResourceGovernor, ResourceMitigationPlan, ResourceSnapshot,
};

fn assert_plan(
    plan: ResourceMitigationPlan,
    spill: bool,
    next_batch_size: Option<u32>,
    offload: bool,
    pause: bool,
    reject: bool,
) {
    assert_eq!(plan.spill_observation_cache(), spill);
    assert_eq!(plan.next_batch_size(), next_batch_size);
    assert_eq!(plan.offload_inference_to_cpu(), offload);
    assert_eq!(plan.pause_current_agent(), pause);
    assert_eq!(plan.reject_new_agent_work(), reject);
    assert_eq!(plan.is_noop(), !(spill || next_batch_size.is_some() || offload || pause || reject));
}

#[test]
fn budget_rejects_invalid_limits() {
    for budget in [
        ResourceBudget::new(0, 8, 2, 4, 4, 16),
        ResourceBudget::new(4, 0, 2, 4, 4, 16),
        ResourceBudget::new(4, 8, 0, 4, 4, 16),
        ResourceBudget::new(4, 8, 2, 0, 4, 16),
        ResourceBudget::new(4, 8, 2, 4, 0, 16),
        ResourceBudget::new(4, 8, 2, 4, 4, 0),
    ] {
        assert_eq!(budget, Err(BudgetError::ZeroLimit));
    }
    assert_eq!(
        ResourceBudget::new(9, 8, 2, 4, 4, 16),
        Err(BudgetError::SoftExceedsHard)
    );
    assert_eq!(
        ResourceBudget::new(4, 8, 5, 4, 4, 16),
        Err(BudgetError::SoftExceedsHard)
    );
}

#[test]
fn governor_preserves_interactivity_before_agent_throughput() {
    let governor = ResourceGovernor::new(
        ResourceBudget::new(4_096, 8_192, 2_048, 4_096, 8, 16).expect("budget"),
    );

    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 1_000, 8, false, 10)),
        false,
        None,
        false,
        false,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(5_000, 1_000, 8, false, 10)),
        true,
        None,
        false,
        false,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 2_500, 8, false, 10)),
        false,
        Some(4),
        false,
        false,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 2_500, 1, true, 10)),
        false,
        None,
        true,
        false,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 2_500, 1, false, 10)),
        false,
        None,
        false,
        true,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 1_000, 8, true, 20)),
        false,
        None,
        true,
        false,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 1_000, 8, false, 20)),
        false,
        None,
        false,
        true,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(8_500, 1_000, 8, false, 10)),
        true,
        None,
        false,
        true,
        true,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 4_500, 8, false, 10)),
        false,
        None,
        false,
        true,
        true,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(8_500, 4_500, 8, true, 30)),
        true,
        None,
        true,
        true,
        true,
    );
}

#[test]
fn governor_treats_budget_boundaries_as_pressure() {
    let governor = ResourceGovernor::new(
        ResourceBudget::new(4_096, 8_192, 2_048, 4_096, 8, 16).expect("budget"),
    );

    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 4_096, 8, true, 10)),
        false,
        None,
        true,
        true,
        true,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(8_192, 1_000, 8, false, 10)),
        true,
        None,
        false,
        true,
        true,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 1_000, 8, true, 16)),
        false,
        None,
        true,
        false,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(4_096, 1_000, 8, false, 10)),
        true,
        None,
        false,
        false,
        false,
    );
    assert_plan(
        governor.decide(ResourceSnapshot::new(2_000, 2_048, 8, false, 10)),
        false,
        Some(4),
        false,
        false,
        false,
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
