# Controlled benchmark Unicode identity security

Status: active-PR doctoring for #322, stacked on #237. This record does not describe protected-main shipment.

## Problem

`ControlledBenchmarkRunContext` carries OriginWeave-owned reproducibility identities such as source revision, Chromium revision, OS image, hardware profile, protocol-adapter set, model/provider route, reasoning configuration, fixture/corpus version, and seed-set identity. These strings are compared byte-for-byte and are expected to appear in logs, reports, retained benchmark evidence, and later signed evidence summaries.

The #237 parent already rejects blank values, surrounding whitespace, and C0/C1 control characters. Rust `char::is_control` does not cover Unicode bidirectional formatting characters. A value containing a directional override, embedding, isolate, or mark can therefore remain byte-exact while its rendered presentation differs from logical storage order. That is an evidence-review ambiguity at an application-owned identifier boundary.

This differs from browser-issued protocol identity. OriginWeave must preserve WebDriver BiDi addresses such as `browser.UserContext` losslessly because Browser Session does not own that external identifier grammar. The controlled benchmark does own its reproducibility-label grammar, so a narrower fail-closed admission rule is appropriate here.

## Decision

Reject the Unicode `Bidi_Control` set in controlled-benchmark run-context identities in addition to C0/C1 controls:

- `U+061C` ARABIC LETTER MARK;
- `U+200E` LEFT-TO-RIGHT MARK and `U+200F` RIGHT-TO-LEFT MARK;
- `U+202A..U+202E` embedding, override, and pop-directional-formatting controls; and
- `U+2066..U+2069` isolate controls.

Do not reject ordinary visible right-to-left scripts. The regression therefore keeps Arabic and Hebrew text admissible when no bidi formatting control is present. No Unicode normalization, confusable folding, ASCII-only restriction, or browser-protocol normalization is introduced by this slice.

The implementation remains dependency-free: a private Rust helper matches exactly the `Bidi_Control` scalar set before run-context equality can influence suite evidence. The existing typed `ControlCharacterRunContext` failure remains the fail-closed diagnostic so this repair does not widen the public error surface unnecessarily.

## Test-first evidence

Parent exact: `ea92c326e2dc4e3daa869aff1266c10b05453e7d`.

Test-first exact: `a02b1d6e3a2034944ba337973bc9d0385976907c`. The test injects all 12 `Bidi_Control` scalar values and requires `ControlCharacterRunContext`; it also adds a visible Arabic/Hebrew acceptance case. The parent source only used `char::is_control`, so the new bidi-control assertions are semantic RED by source inspection. Hosted execution was unavailable at creation time; this is not an executed RED claim.

Production repair exact: `11b2c16422481eb674a90ae61e941a5b247c4024`. `validate_run_context_field` rejects C0/C1 controls or the exact bidi-control set before equality comparison. The production change does not alter benchmark thresholds, registry membership/versioning, browser authority, model/provider routing, evidence signing/persistence, workflows, or release authority.

## Alternatives considered

Rejecting all non-ASCII text was rejected because it would conflate script diversity with display-control risk and would break legitimate internationalized operator-controlled labels. Applying Unicode normalization was rejected because canonicalization would change exact evidence identity and does not by itself solve bidi formatting. Treating byte equality as sufficient was rejected because UAX #9 explicitly separates logical order from rendered order for bidirectional text; human review of evidence can therefore differ from stored scalar order.

A broader Unicode security profile may be warranted later for externally supplied product identifiers. That is not silently introduced here: any expansion to confusable, default-ignorable, or script-restriction policy needs its own threat model, compatibility analysis, tests, and versioned contract.

## Risks and follow-up

The current repair prevents directional formatting controls from entering benchmark-owned run-context identities but does not authenticate those identities. The durable evidence owner must still bind them to execution artifacts. It also does not claim complete Unicode spoofing resistance; UTS #39 covers a wider space of confusable and identifier-security mechanisms.

The parent #237 and child #322 remain active-PR evidence. Exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, 100% production function/line/region/branch coverage, and required security/review workflows must execute before promotion. Runner-queued or skipped jobs are not passing evidence.

## References

Unicode Consortium. (2026, August 27). *Unicode security mechanisms* (Unicode Technical Standard #39, Version 18.0.0, Revision 34). https://www.unicode.org/reports/tr39/tr39-34.html

Unicode Consortium. (2025, August 13). *Unicode bidirectional algorithm* (Unicode Standard Annex #9, Version 17.0.0, Revision 51). https://www.unicode.org/reports/tr9/tr9-51.html
