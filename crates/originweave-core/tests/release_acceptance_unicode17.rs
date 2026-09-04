use originweave_core::release_acceptance::{DeclaredLimitation, ReleaseDecisionError};

const UNICODE_17_DEFAULT_IGNORABLE_CODE_POINT_COUNT: usize = 4_174;

#[test]
fn generic_constructor_input_shapes_cover_fail_closed_empty_boundaries() {
    assert_eq!(
        DeclaredLimitation::new(String::new(), "Linux ARM64 is unsupported."),
        Err(ReleaseDecisionError::EmptyLimitationClaim),
    );
    assert!(
        DeclaredLimitation::new(String::from("linux_arm64"), "Linux ARM64 is unsupported.").is_ok()
    );
    assert_eq!(
        DeclaredLimitation::new("linux_arm64", String::new()),
        Err(ReleaseDecisionError::EmptyLimitationConsequence),
    );
    assert!(
        DeclaredLimitation::new("linux_arm64", String::from("Linux ARM64 is unsupported.")).is_ok()
    );
}

#[test]
fn limitation_rejects_unicode_17_default_ignorable_code_points() -> Result<(), &'static str> {
    // Unicode 17.0.0 DerivedCoreProperties.txt (2025-07-30),
    // Default_Ignorable_Code_Point. The reviewed ranges contain exactly 4,174 code points.
    let ranges = [
        (0x00ad_u32, 0x00ad_u32),
        (0x034f, 0x034f),
        (0x061c, 0x061c),
        (0x115f, 0x1160),
        (0x17b4, 0x17b5),
        (0x180b, 0x180f),
        (0x200b, 0x200f),
        (0x202a, 0x202e),
        (0x2060, 0x206f),
        (0x3164, 0x3164),
        (0xfe00, 0xfe0f),
        (0xfeff, 0xfeff),
        (0xffa0, 0xffa0),
        (0xfff0, 0xfff8),
        (0x1bca0, 0x1bca3),
        (0x1d173, 0x1d17a),
        (0xe0000, 0xe0fff),
    ];
    let mut tested_code_points = 0_usize;

    for (start, end) in ranges {
        for code_point in start..=end {
            let character = char::from_u32(code_point)
                .ok_or("reviewed Unicode 17 default-ignorable range must contain scalar values")?;
            tested_code_points += 1;

            assert_eq!(
                DeclaredLimitation::new(
                    format!("linux_arm64{character}forged_release_claim"),
                    "Linux ARM64 is unsupported.",
                ),
                Err(ReleaseDecisionError::InvalidLimitationClaim),
                "U+{code_point:04X} must be rejected in the unsupported claim",
            );
            assert_eq!(
                DeclaredLimitation::new(
                    "linux_arm64",
                    format!("Linux ARM64 is unsupported.{character}forged_release_consequence"),
                ),
                Err(ReleaseDecisionError::InvalidLimitationConsequence),
                "U+{code_point:04X} must be rejected in the buyer consequence",
            );
        }
    }

    assert_eq!(
        tested_code_points, UNICODE_17_DEFAULT_IGNORABLE_CODE_POINT_COUNT,
        "reviewed Unicode 17 Default_Ignorable_Code_Point ranges must match the authoritative cardinality",
    );
    Ok(())
}

#[test]
fn limitation_rejects_line_and_paragraph_separators_beyond_default_ignorable_set() {
    for (name, separator) in [("U+2028", '\u{2028}'), ("U+2029", '\u{2029}')] {
        assert_eq!(
            DeclaredLimitation::new(
                format!("linux_arm64{separator}forged_release_claim"),
                "Linux ARM64 is unsupported.",
            ),
            Err(ReleaseDecisionError::InvalidLimitationClaim),
            "{name} must be rejected in the unsupported claim to prevent line-forging ambiguity",
        );
        assert_eq!(
            DeclaredLimitation::new(
                "linux_arm64",
                format!("Linux ARM64 is unsupported.{separator}forged_release_consequence"),
            ),
            Err(ReleaseDecisionError::InvalidLimitationConsequence),
            "{name} must be rejected in the buyer consequence to prevent line-forging ambiguity",
        );
    }
}

#[test]
fn limitation_does_not_blanket_reject_unicode_17_whitespace() -> Result<(), ReleaseDecisionError> {
    let medium_mathematical_space = '\u{205f}';
    let ideographic_space = '\u{3000}';

    let limitation = DeclaredLimitation::new(
        format!("east{ideographic_space}asia"),
        format!("Support is limited{medium_mathematical_space}to the declared profile."),
    )?;

    assert_eq!(
        limitation.unsupported_claim(),
        format!("east{ideographic_space}asia")
    );
    assert_eq!(
        limitation.buyer_consequence(),
        format!("Support is limited{medium_mathematical_space}to the declared profile.")
    );
    Ok(())
}
