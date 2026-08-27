use originweave_policy::{ApprovalPrincipalRef, ApprovalPrincipalRefError};

#[test]
fn approval_principal_rejects_invisible_format_controls() {
    for subject in [
        "user\u{00ad}123",
        "user\u{200b}123",
        "user\u{200c}123",
        "user\u{200d}123",
        "user\u{2060}123",
        "user\u{2065}123",
        "user\u{206a}123",
        "user\u{206f}123",
        "user\u{feff}123",
    ] {
        assert_eq!(
            ApprovalPrincipalRef::new("https://id.example", subject),
            Err(ApprovalPrincipalRefError::InvalidSubject),
            "invisible formatting must not create an operator-confusable principal identity"
        );
    }

    assert_eq!(
        ApprovalPrincipalRef::new("https://id\u{200b}.example", "user-123"),
        Err(ApprovalPrincipalRefError::InvalidIssuer)
    );
}
