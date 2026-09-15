# Controlled benchmark Unicode identity traceability

- **Documentation status:** Active-PR evidence dossier
- **Protected-main capability status:** Not shipped
- **Active implementation lane:** PR #324, stacked on #322 / #237
- **Issue:** #323
- **Canonical bounded context:** controlled-benchmark evidence identity
- **Excluded boundary:** browser-issued protocol identity, including WebDriver BiDi `browser.UserContext`

## Requirement → evidence map

| Requirement | Standards / decision evidence | Implementation | Test evidence | Current status |
|---|---|---|---|---|
| Reject Unicode default-ignorable scalars from benchmark-owned run-context identity | UAX #31 Rev. 45 §7.3; Unicode 18.0.0 `DerivedCoreProperties.txt` | `CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE`; `is_unicode_18_default_ignorable`; `validate_run_context_field` | `unicode_18_default_ignorable_reproducibility_context_fails_closed` | Active PR #324 only |
| Bind the property set to a named Unicode version | UAX31-C1 requires version identification; DICP has no general stability guarantee | profile value `unicode-18.0.0-default-ignorable-exclusion`; exact local range table | profile-string assertion plus property-range boundary cases | Active PR #324 only |
| Preserve ordinary visible multilingual / RTL labels | UTS #39 allows profiles and explicit tailoring decisions; #323 chooses only DICP exclusion for this machine/audit identity | no normalization, script allow-list, case fold, or confusable transform | `visible_unicode_reproducibility_context_remains_valid` | Active PR #324 only |
| Validate expected and observed identity before equality affects suite evidence | OriginWeave controlled-benchmark fail-closed contract | `evaluate_controlled_benchmark_suite_for_run` validates both sides before byte equality | existing mismatch, blank, whitespace, control, bidi, line-separator, DICP cases | Active PR stack |
| Keep browser protocol identifiers lossless | Browser Session owns protocol-value preservation rather than benchmark label grammar | no Browser Session, WebDriver BiDi, CDP, extension, or native-host source changed in #324 | changed-file boundary and PR review | Preserved |
| Do not claim complete Unicode spoofing resistance | UTS #39 covers broader confusable, mixed-script, and restriction mechanisms | DICP exclusion only | doctoring non-claim | Preserved |

## Exact lineage

- Parent #322 exact: `a4c8ceaf67a075ef483334802aacfc54cf502068`.
- Semantic RED: `592a1a3bc49a922df86285eab332308770b77474`.
- Minimal production repair: `19a37a667baeffe824c4fb025942eecb40bc67c8`.
- Range/profile coverage successor: `02edb5b2cb5ff9e1237afbc83d9a0f3e7aa48d6a`.
- Doctoring successor: `641c1eee5d2a2184f4e04d5630a8839db35d407f`.
- CHANGELOG successor: `267ad7d7b55040391de3d9c608224dbd7716372c`.

The semantic RED was established against exact predecessor source; no hosted Rust RED is claimed. Draft-policy CI/MV3 jobs are skipped and are not GREEN evidence. Promotion requires executable exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, 100% production function/line/region/branch coverage, required security/review workflows, and the normal repository merge gate.

## Property fidelity

Unicode 18.0.0 `DerivedCoreProperties.txt` enumerates 4,174 `Default_Ignorable_Code_Point` scalars. The implementation compresses only contiguous published ranges; it does not infer a property from General Category or a moving dependency. The DICP data explicitly has no general stability guarantee, so an upgrade must create a new profile identity and compare the property data before evidence admission changes.

No tailored exception is present. In particular, ZWNJ (`U+200C`), ZWJ (`U+200D`), variation selectors, tag characters, fillers, and reserved code points carrying DICP in Unicode 18.0.0 remain rejected for benchmark-owned evidence identity. A future exception requires its own security/compatibility decision and new versioned profile.

## References

Unicode Consortium. (2026, September 1). *Unicode identifiers and syntax* (Unicode Standard Annex #31, Version 18.0.0, Revision 45). https://www.unicode.org/reports/tr31/tr31-45.html

Unicode Consortium. (2026, August 27). *Unicode security mechanisms* (Unicode Technical Standard #39, Version 18.0.0, Revision 34). https://www.unicode.org/reports/tr39/tr39-34.html

Unicode Consortium. (2026, August 7). *DerivedCoreProperties-18.0.0.txt* [Unicode Character Database data file]. https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt
