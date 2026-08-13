//! Standard error contract for exact sensitive-deletion receipt-set verification.
//!
//! Public verification failures must integrate with ordinary Rust error handling without leaking
//! request, tenant, retention-policy, storage-scope, or target-reference values into diagnostics.

use originweave_evidence::SensitiveDeletionReceiptSetError;

#[test]
fn receipt_set_failures_are_standard_non_leaking_errors() {
    let cases = [
        (
            SensitiveDeletionReceiptSetError::TooManyRequirements,
            "too many sensitive deletion requirements",
        ),
        (
            SensitiveDeletionReceiptSetError::TooManyReceipts,
            "too many sensitive deletion receipts",
        ),
        (
            SensitiveDeletionReceiptSetError::RequestMismatch,
            "sensitive deletion request mismatch",
        ),
        (
            SensitiveDeletionReceiptSetError::TenantMismatch,
            "sensitive deletion tenant mismatch",
        ),
        (
            SensitiveDeletionReceiptSetError::RetentionPolicyMismatch,
            "sensitive deletion retention policy mismatch",
        ),
        (
            SensitiveDeletionReceiptSetError::EmptyRequirementSet,
            "empty sensitive deletion requirement set",
        ),
        (
            SensitiveDeletionReceiptSetError::DuplicateRequirement,
            "duplicate sensitive deletion requirement",
        ),
        (
            SensitiveDeletionReceiptSetError::MissingReceipt,
            "missing sensitive deletion receipt",
        ),
        (
            SensitiveDeletionReceiptSetError::UnexpectedReceipt,
            "unexpected sensitive deletion receipt",
        ),
        (
            SensitiveDeletionReceiptSetError::DuplicateReceipt,
            "duplicate sensitive deletion receipt",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        assert!(std::error::Error::source(&error).is_none());
    }
}
