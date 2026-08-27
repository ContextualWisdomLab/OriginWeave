use originweave_resource::BudgetError;
use std::error::Error as _;

#[test]
fn budget_errors_expose_stable_standard_error_contract() {
    let cases = [
        (
            BudgetError::ZeroLimit,
            "resource budget limits must be nonzero",
        ),
        (
            BudgetError::SoftExceedsHard,
            "resource budget soft limits must not exceed hard limits",
        ),
    ];

    for (error, expected_message) in cases {
        assert_eq!(error.to_string(), expected_message);
        assert!(error.source().is_none());
    }
}
