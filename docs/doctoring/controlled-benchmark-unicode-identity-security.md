# Controlled benchmark Unicode identity security

Status: active-PR doctoring for #322, stacked on #237. This record does not describe protected-main shipment.

## Problem

`ControlledBenchmarkRunContext` carries OriginWeave-owned reproducibility identities such as source revision, Chromium revision, OS image, hardware profile, protocol-adapter set, model/provider route, reasoning configuration, fixture/corpus version, and seed-set identity. These strings are compared byte-for-byte and are expected to appear in logs, reports, retained benchmark evidence, and later signed evidence summaries.

The #237 parent rejects blank values, surrounding whitespace, and C0/C1 control characters. The first #322 slice then closed Unicode bidirectional formatting controls. A second review of the same evidence-rendering boundary found a separate gap: Rust `char::is_control` covers General Category `Cc`, while `U+2028` LINE SEPARATOR and `U+2029` PARAGRAPH SEPARATOR are `Zl` and `Zp`. An internal LS or PS survives surrounding-whitespace checks yet can render one byte-exact identity as multiple lines or paragraphs. That can make log/report review disagree with the stored scalar sequence even without a bidi override.

Unicode UAX #44 defines `Zl` as `U+2028 LINE SEPARATOR` only, `Zp` as `U+2029 PARAGRAPH SEPARATOR` only, and `Cc` separately as C0/C1 controls. The Unicode Standard's newline guidance states that LS and PS are unambiguous Unicode line and paragraph separators and, unlike the other newline forms it discusses, are not encoded as control codes. The application therefore cannot rely on `char::is_control` to close this rendering boundary.

This differs from browser-issued protocol identity. OriginWeave must preserve WebDriver BiDi addresses such as `browser.UserContext` losslessly because Browser Session does not own that external identifier grammar. The controlled benchmark does own its reproducibility-label grammar, so a narrower fail-closed admission rule is appropriate here.

## Decision

Reject rendering controls that can make controlled-benchmark identity presentation ambiguous:

- C0/C1 control characters already covered by `char::is_control`;
- `U+2028` LINE SEPARATOR and `U+2029` PARAGRAPH SEPARATOR;
- `U+061C` ARABIC LETTER MARK;
- `U+200E` LEFT-TO-RIGHT MARK and `U+200F` RIGHT-TO-LEFT MARK;
- `U+202A..U+202E` embedding, override, and pop-directional-formatting controls; and
- `U+2066..U+2069` isolate controls.

Do not reject ordinary visible right-to-left scripts. The regression therefore keeps Arabic and Hebrew text admissible when no formatting control or line/paragraph separator is present. No Unicode normalization, confusable folding, ASCII-only restriction, or browser-protocol normalization is introduced by this slice.

The implementation remains dependency-free. One private helper matches `U+2028`/`U+2029`; another matches the exact `Bidi_Control` scalar set. Both run before run-context equality can influence suite evidence. The existing typed `ControlCharacterRunContext` failure remains the fail-closed public diagnostic so this repair does not widen the public error surface unnecessarily.

## Test-first evidence

Parent exact: `ea92c326e2dc4e3daa869aff1266c10b05453e7d`.

Bidi test-first exact: `a02b1d6e3a2034944ba337973bc9d0385976907c`. The test injects all 12 `Bidi_Control` scalar values and requires `ControlCharacterRunContext`; it also adds a visible Arabic/Hebrew acceptance case. The parent source only used `char::is_control`, so the bidi-control assertions were semantic RED by source inspection. Hosted execution was unavailable at creation time; this is not an executed RED claim.

Bidi production repair exact: `11b2c16422481eb674a90ae61e941a5b247c4024`. `validate_run_context_field` rejects C0/C1 controls or the exact bidi-control set before equality comparison.

Line-separator test-first exact: `4c17e0b36aa627e036dcbe3d82332a7bc0cc594c`. It adds hostile `U+2028` and `U+2029` identities and requires the same typed fail-closed error. The predecessor source accepted both because they are not `Cc` and are outside `Bidi_Control`; this is again a semantic RED established from the exact predecessor source, not a claim that a hosted runner executed the failing test.

Line-separator production repair exact: `c89f76c5761a46745a5d1d2176828bfd6c3c2f32`. The validator adds the narrow `is_unicode_line_separator` predicate, and rustdoc plus the public diagnostic are widened only enough to describe the actual admitted rendering-control boundary. Exact `4f8bce55d2f03a73260c2629c0eca2012f79a539` aligns the diagnostic regression with that public message.

None of these changes alter benchmark thresholds, registry membership/versioning, browser authority, model/provider routing, evidence signing/persistence, workflows, or release authority.

## Alternatives considered

Rejecting all non-ASCII text was rejected because it would conflate script diversity with rendering-control risk and would break legitimate internationalized operator-controlled labels. Applying Unicode normalization was rejected because canonicalization would change exact evidence identity and does not by itself solve directional reordering or explicit line/paragraph separation. Treating byte equality as sufficient was rejected because UAX #9 separates logical order from rendered order for bidirectional text, while the Unicode Standard explicitly assigns LS and PS line/paragraph boundary semantics.

A broader Unicode security profile may be warranted later for externally supplied product identifiers. That is not silently introduced here: any expansion to confusable, default-ignorable, script-restriction, or normalization policy needs its own threat model, compatibility analysis, tests, and versioned contract.

## Risks and follow-up

The current repair prevents the reviewed directional and line/paragraph rendering controls from entering benchmark-owned run-context identities but does not authenticate those identities. The durable evidence owner must still bind them to execution artifacts. It also does not claim complete Unicode spoofing resistance; UTS #39 covers a wider space of identifier-security mechanisms.

The parent #237 and child #322 remain active-PR evidence. Exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, 100% production function/line/region/branch coverage, and required security/review workflows must execute before promotion. Cancelled, skipped, queued-only, predecessor, or status-only jobs are not passing evidence.

## References

Unicode Consortium. (2026, August 27). *Unicode security mechanisms* (Unicode Technical Standard #39, Version 18.0.0, Revision 34). https://www.unicode.org/reports/tr39/tr39-34.html

Unicode Consortium. (2025, August 13). *Unicode bidirectional algorithm* (Unicode Standard Annex #9, Version 17.0.0, Revision 51). https://www.unicode.org/reports/tr9/tr9-51.html

Unicode Consortium. (2025). *Unicode Character Database* (Unicode Standard Annex #44, Version 17.0.0). https://www.unicode.org/reports/tr44/

Unicode Consortium. (2025). *The Unicode Standard, Version 17.0.0: Chapter 5, Implementation guidelines—Newline guidelines*. https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-5/
