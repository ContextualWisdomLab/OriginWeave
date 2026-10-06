#![allow(clippy::expect_used)]

/// Return the consecutive rustdoc immediately preceding the named public method.
///
/// This literal source guard does not interpret arbitrary prose or prove authentication.
fn method_docs(source: &str, signature: &str) -> String {
    assert!(
        source.contains(signature),
        "public recovery method is required"
    );
    let prefix = source.split(signature).next().expect("method signature");
    let mut lines: Vec<_> = prefix
        .lines()
        .rev()
        .take_while(|line| line.trim_start().starts_with("///"))
        .collect();
    assert!(!lines.is_empty(), "method rustdoc is required");
    lines.reverse();
    lines.join("\n")
}

/// Reject the exact issuance/reconstruction contradiction and require local trust disclosures.
///
/// The assertions bind named method rustdoc and the active API-contract paragraph; they are
/// deliberately literal documentation checks, not an authentication or whole-history oracle.
#[test]
fn recovery_docs_distinguish_normal_issuance_from_caller_trusted_reconstruction() {
    let source = include_str!("../src/lib.rs");
    let issuance = method_docs(source, "    pub fn apply_with_receipt(");
    assert!(
        !issuance.contains("Receipts can only be minted")
            && !issuance.contains("callers cannot rebind an accepted transition"),
        "normal issuance rustdoc contradicts public caller-trusted restore",
    );
    assert!(issuance.contains("normal issuance path"));
    assert!(issuance.contains("[`BapCommandReceipt::restore`]"));
    assert!(issuance.contains("cannot verify original issuance bindings"));

    let receipt_impl = source
        .split("impl BapCommandReceipt {")
        .nth(1)
        .expect("receipt impl");
    let restore = method_docs(receipt_impl, "    pub fn restore(");
    assert!(restore.contains("caller-trusted metadata"));
    assert!(restore.contains("cannot verify original issuance bindings"));
    let validation = method_docs(source, "    pub fn validate_replay(");
    assert!(validation.contains("differing last-transition evidence"));
    assert!(validation.contains("instance identity or earlier-history authentication"));

    let contract = include_str!("../../../docs/API_CONTRACT.md");
    let active = contract
        .split("The rules above define the product-wide target contract.")
        .nth(1)
        .expect("active BAP contract")
        .split("## 8. Deadline and cancellation")
        .next()
        .expect("contract boundary");
    assert!(active.contains("caller-trusted metadata"));
    assert!(active.contains("cannot verify original issuance bindings"));
    assert!(active.contains("instance identity or earlier-history authentication"));
}
