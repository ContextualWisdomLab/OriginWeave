use originweave_core::release_acceptance::{DeclaredLimitation, ReleaseDecisionError};

#[test]
fn limitation_rejects_unicode_17_default_ignorable_code_points() -> Result<(), &'static str> {
    // Unicode 17.0.0 DerivedCoreProperties.txt, Default_Ignorable_Code_Point.
    // Endpoints plus a midpoint make every reviewed inclusive range executable evidence.
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

    for (start, end) in ranges {
        let midpoint = start + (end - start) / 2;
        for code_point in [start, midpoint, end] {
            let character = char::from_u32(code_point)
                .ok_or("reviewed Unicode 17 default-ignorable range must contain scalar values")?;

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

    Ok(())
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
