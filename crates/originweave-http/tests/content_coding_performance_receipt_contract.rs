const PERFORMANCE_SOURCE: &str =
    include_str!("../examples/content_coding_stack_performance.rs");

#[test]
fn performance_receipt_requires_exact_source_and_environment_identity() -> Result<(), String> {
    let required_markers = [
        "ORIGINWEAVE_PERFORMANCE_SOURCE_REVISION",
        "ORIGINWEAVE_PERFORMANCE_ENVIRONMENT_ID",
        "GITHUB_SHA",
        "source_revision={}",
        "environment_id={}",
        "runtime_os={}",
        "runtime_arch={}",
        "runtime_parallelism={}",
    ];

    for marker in required_markers {
        if !PERFORMANCE_SOURCE.contains(marker) {
            return Err(format!(
                "stacked content-coding performance receipt is missing provenance marker {marker}"
            ));
        }
    }

    Ok(())
}
