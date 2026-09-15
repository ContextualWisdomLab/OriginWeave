# Controlled benchmark Unicode identity traceability

- **Documentation status:** Active-PR evidence dossier
- **Protected-main capability status:** Not shipped
- **Active implementation lane:** PR #324, stacked on #322 / #237
- **Implementation issue:** #323
- **Final-UCD provenance gate:** #325
- **Canonical bounded context:** controlled-benchmark evidence identity
- **Excluded boundary:** browser-issued protocol identity, including WebDriver BiDi `browser.UserContext`

## Requirement → evidence map

| Requirement | Standards / decision evidence | Implementation | Test evidence | Current status |
|---|---|---|---|---|
| Reject Unicode default-ignorable scalars from benchmark-owned run-context identity | UAX #31 Rev. 45 §7.3; 2026-08-07 Unicode 18.0.0 pre-release `DerivedCoreProperties.txt` snapshot | `CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE`; `is_unicode_18_default_ignorable`; `validate_run_context_field` | `unicode_18_default_ignorable_reproducibility_context_fails_closed` | Active PR #324 only; final UCD provenance pending #325 |
| Bind the property set to a named Unicode version | UAX31-C1 requires version identification; DICP has no general stability guarantee | profile value `unicode-18.0.0-default-ignorable-exclusion`; exact local range table | profile-string assertion plus property-range boundary cases | Grammar pinned; final Unicode 18.0.0 UCD receipt pending #325 |
| Reconcile pre-release property data before promotion | Unicode 18 beta status treats UCD data as prepublication until formal release; versioned UCD URL still redirected to `/Public/draft/` on 2026-09-15 KST | no silent table rewrite; #325 requires final dataset comparison before promotion | final source-entry/scalar/range comparison and immutable receipt required by #325 | Open release/provenance gate |
| Preserve ordinary visible multilingual / RTL labels | UTS #39 allows profiles and explicit tailoring decisions; #323 chooses only DICP exclusion for this machine/audit identity | no normalization, script allow-list, case fold, or confusable transform | `visible_unicode_reproducibility_context_remains_valid` | Active PR #324 only |
| Validate expected and observed identity before equality affects suite evidence | OriginWeave controlled-benchmark fail-closed contract | `evaluate_controlled_benchmark_suite_for_run` validates both sides before byte equality | existing mismatch, blank, whitespace, control, bidi, line-separator, DICP cases | Active PR stack |
| Keep browser protocol identifiers lossless | Browser Session owns protocol-value preservation rather than benchmark label grammar | no Browser Session, WebDriver BiDi, CDP, extension, or native-host source changed in #324 | changed-file boundary and PR review | Preserved |
| Do not claim complete Unicode spoofing resistance | UTS #39 covers broader confusable, mixed-script, and restriction mechanisms | DICP exclusion only | doctoring non-claim | Preserved |

## Exact lineage

- Parent #322 exact: `a4c8ceaf67a075ef483334802aacfc54cf502068`.
- Semantic RED: `592a1a3bc49a922df86285eab332308770b77474`.
- Minimal production repair: `19a37a667baeffe824c4fb025942eecb40bc67c8`.
- Range/profile coverage successor: `02edb5b2cb5ff9e1237afbc83d9a0f3e7aa48d6a`.
- Original doctoring successor: `641c1eee5d2a2184f4e04d5630a8839db35d407f`.
- CHANGELOG successor: `267ad7d7b55040391de3d9c608224dbd7716372c`.
- Initial traceability successor: `b84e28f140a2c0a35b7bf82fdf0bf25b21c6801f`.
- 2026-09-15 provenance doctoring correction: `b8fc24ff0bbf4d2d8ed1c07a9846f1aefd026bce`.

The semantic RED was established against exact predecessor source; no hosted Rust RED is claimed. The previous current-head Ready probe received no runner before #324 returned to Draft, then became cancelled; Draft-policy generations are skipped. None is GREEN evidence. Promotion requires executable exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, 100% production function/line/region/branch coverage, required security/review workflows, the normal repository merge gate, and #325 final-UCD reconciliation.

## Property fidelity

The 2026-08-07 `DerivedCoreProperties-18.0.0.txt` pre-release snapshot enumerates 4,174 `Default_Ignorable_Code_Point` scalars. The implementation compresses only contiguous ranges observed in that snapshot; it does not infer the property from General Category or a moving dependency. The DICP data explicitly has no general stability guarantee.

As observed on 2026-09-15 KST, `https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt` redirected to `https://www.unicode.org/Public/draft/ucd/DerivedCoreProperties.txt`. The stable UAX #31 Rev. 45 and UTS #39 Rev. 34 publications establish the target profile and security rationale, but that redirect prevents the snapshot itself from being treated as immutable final Unicode 18.0.0 UCD provenance. #325 therefore requires a post-publication exact comparison of source entries, scalar count, ranges/code points, file date, and an immutable content receipt.

No tailored exception is present. In particular, ZWNJ (`U+200C`), ZWJ (`U+200D`), variation selectors, tag characters, fillers, and reserved code points carrying DICP in the observed snapshot remain rejected for benchmark-owned evidence identity. A future exception requires its own security/compatibility decision and new versioned profile.

If the final Unicode 18.0.0 DICP property is equivalent to the pinned snapshot, traceability may record equivalence and final provenance without changing admission semantics. If it differs, the table must not be silently rewritten under the existing evidence identity: a fresh hostile/compatibility RED and a new profile identity are required.

## References

Unicode Consortium. (2026, September 1). *Unicode identifiers and syntax* (Unicode Standard Annex #31, Version 18.0.0, Revision 45). https://www.unicode.org/reports/tr31/tr31-45.html

Unicode Consortium. (2026, August 27). *Unicode security mechanisms* (Unicode Technical Standard #39, Version 18.0.0, Revision 34). https://www.unicode.org/reports/tr39/tr39-34.html

Unicode Consortium. (2026, August 7). *DerivedCoreProperties-18.0.0.txt* [pre-release Unicode Character Database snapshot observed via `/Public/draft/ucd/` on 2026-09-15]. https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt

Unicode Consortium. (2026). *BETA Unicode 18.0.0*. https://www.unicode.org/versions/beta-18.0.0.html
