#![allow(clippy::expect_used)]

use originweave_resource::{ResourceBudget, ResourceGovernor, ResourceSnapshot};

fn governor() -> ResourceGovernor {
    ResourceGovernor::new(ResourceBudget::new(4_096, 8_192, 2_048, 4_096, 8, 16).expect("budget"))
}

#[test]
fn cpu_worker_budget_is_an_exact_admission_boundary() {
    let below_limit = governor().decide(ResourceSnapshot::new(2_000, 1_000, 8, false, 10, 7));
    assert!(!below_limit.reject_new_agent_work());
    assert!(!below_limit.pause_current_agent());
    assert!(below_limit.is_noop());

    let at_limit = governor().decide(ResourceSnapshot::new(2_000, 1_000, 8, false, 10, 8));
    assert!(at_limit.reject_new_agent_work());
    assert!(!at_limit.pause_current_agent());
    assert!(!at_limit.spill_observation_cache());
    assert_eq!(at_limit.next_batch_size(), None);
    assert!(!at_limit.offload_inference_to_cpu());
    assert!(!at_limit.is_noop());
}

#[test]
fn cpu_admission_combines_with_independent_hard_memory_pressure() {
    let plan = governor().decide(ResourceSnapshot::new(8_192, 1_000, 8, false, 10, 8));

    assert!(plan.reject_new_agent_work());
    assert!(plan.pause_current_agent());
    assert!(plan.spill_observation_cache());
    assert!(!plan.offload_inference_to_cpu());
    assert_eq!(plan.next_batch_size(), None);
}
