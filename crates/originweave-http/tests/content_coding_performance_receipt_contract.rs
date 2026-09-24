const PERFORMANCE_SOURCE: &str =
    include_str!("../examples/content_coding_stack_performance.rs");

#[test]
fn performance_receipt_requires_exact_source_environment_and_acceptance_identity() -> Result<(), String> {
    let required_markers = [
        "ORIGINWEAVE_PERFORMANCE_SOURCE_REVISION",
        "ORIGINWEAVE_PERFORMANCE_ENVIRONMENT_ID",
        "GITHUB_SHA",
        "source_revision={}",
        "source_revision_source={}",
        "environment_id={}",
        "runtime_os={}",
        "runtime_arch={}",
        "runtime_parallelism={}",
        "budget_status={}",
        "acceptance_status={}",
        "UNACCEPTED_SOURCE_FALLBACK",
    ];

    for marker in required_markers {
        if !PERFORMANCE_SOURCE.contains(marker) {
            return Err(format!(
                "stacked content-coding performance receipt is missing provenance/acceptance marker {marker}"
            ));
        }
    }

    Ok(())
}
