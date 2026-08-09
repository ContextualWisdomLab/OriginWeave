#![allow(clippy::expect_used)]

// Red-first contracts: hard pressure must reduce the active consumer, not only future admission.
use originweave_resource::{ResourceBudget, ResourceGovernor, ResourceSnapshot};

fn governor() -> ResourceGovernor {
    ResourceGovernor::new(ResourceBudget::new(4_096, 8_192, 2_048, 4_096, 8, 16).expect("budget"))
}

#[test]
fn hard_vram_pressure_stops_the_active_consumer_and_rejects_admission() {
    let plan = governor().decide(ResourceSnapshot::new(2_000, 4_096, 8, true, 10, 1));

    assert!(plan.pause_current_agent());
    assert!(plan.reject_new_agent_work());
    assert!(plan.offload_inference_to_cpu());
    assert!(!plan.spill_observation_cache());
    assert_eq!(plan.next_batch_size(), None);
    assert!(!plan.is_noop());
}

#[test]
fn hard_ram_pressure_spills_pauses_and_rejects_independently() {
    let plan = governor().decide(ResourceSnapshot::new(8_192, 1_000, 8, false, 10, 1));

    assert!(plan.pause_current_agent());
    assert!(plan.reject_new_agent_work());
    assert!(plan.spill_observation_cache());
    assert!(!plan.offload_inference_to_cpu());
    assert_eq!(plan.next_batch_size(), None);
}

#[test]
fn simultaneous_pressure_retains_every_applicable_mitigation() {
    let plan = governor().decide(ResourceSnapshot::new(8_500, 4_500, 8, true, 30, 1));

    assert!(plan.pause_current_agent());
    assert!(plan.reject_new_agent_work());
    assert!(plan.spill_observation_cache());
    assert!(plan.offload_inference_to_cpu());
    assert_eq!(plan.next_batch_size(), None);
}

#[test]
fn independent_soft_pressures_can_spill_and_reduce_the_same_step() {
    let plan = governor().decide(ResourceSnapshot::new(5_000, 2_500, 8, false, 10, 1));

    assert!(!plan.pause_current_agent());
    assert!(!plan.reject_new_agent_work());
    assert!(plan.spill_observation_cache());
    assert!(!plan.offload_inference_to_cpu());
    assert_eq!(plan.next_batch_size(), Some(4));
}
