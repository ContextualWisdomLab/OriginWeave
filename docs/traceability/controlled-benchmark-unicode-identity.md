# Controlled benchmark Unicode identity traceability

- **Documentation status:** Active-PR evidence dossier
- **Protected-main capability status:** Not shipped
- **Active implementation lane:** PR #324, stacked on #322 / #237
- **Implementation issue:** #323
- **Unicode 18.0.0 provenance gate:** #325
- **Canonical bounded context:** controlled-benchmark evidence identity
- **Excluded boundary:** browser-issued protocol identity, including WebDriver BiDi `browser.UserContext`

## Requirement → evidence map

| Requirement | Standards / decision evidence | Implementation | Test evidence | Current status |
|---|---|---|---|---|
| Reject Unicode default-ignorable scalars from benchmark-owned run-context identity | UAX #31 Rev. 45 §7.3; formally released Unicode 18.0.0 versioned `DerivedCoreProperties.txt` | `CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE`; `is_unicode_18_default_ignorable`; `validate_run_context_field` | `unicode_18_default_ignorable_reproducibility_context_fails_closed` | Active PR #324 only; immutable artifact receipt pending #325 |
| Bind the property set to a named Unicode version | UAX31-C1 requires version identification; DICP has no general stability guarantee | profile value `unicode-18.0.0-default-ignorable-exclusion`; exact local range table | profile-string assertion plus property-range boundary cases | Grammar pinned; formal release and property equivalence confirmed; byte-level receipt pending #325 |
| Reconcile the pre-release observation after formal publication | Unicode Consortium release registry and 2026-09-16 release announcement; Version 18.0.0 formal citation; direct versioned UCD artifact | no silent table rewrite; #325 preserves immutable-receipt requirement | direct 27-entry / 4,174-scalar source comparison against the 17 compressed implementation ranges | Formal finality + property-set equivalence satisfied; byte length/SHA-256/normalized digest open |
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
- 2026-09-15 pre-release provenance correction: `b8fc24ff0bbf4d2d8ed1c07a9846f1aefd026bce`.
- 2026-09-20 formal-release doctoring reconciliation: `bc01beb027718585fc78696c90f7d5944057f906`.

The semantic RED was established against exact predecessor source; no hosted Rust RED is claimed. The previous Ready probe received no runner before #324 returned to Draft, then became cancelled; Draft-policy generations are skipped. None is GREEN evidence. Promotion requires executable exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, 100% production function/line/region/branch coverage, required security/review workflows, the normal repository merge gate, and #325 immutable-receipt closure.

## Property fidelity

The table originated from `DerivedCoreProperties-18.0.0.txt` dated 2026-08-07. Before Unicode 18.0.0 formal publication, that file was observed through the draft path and was correctly documented as pre-release evidence. The publication state has since changed.

Fresh 2026-09-20 KST primary-source revalidation establishes that Unicode 18.0.0 is formally released: the Unicode Consortium release registry lists **Unicode 18.0.0 — 2026-09-16**, the official announcement says Version 18.0 is now available, the Version 18.0.0 page provides the formal citation and says the Version 18.0 UCD data files are available, and the direct versioned `https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt` artifact is available with header `DerivedCoreProperties-18.0.0.txt` and date `2026-08-07, 16:19:42 GMT`. The same Version 18.0.0 summary page still contains a contradictory preliminary-draft status banner; #325 records that banner as a site inconsistency rather than release authority.

The direct versioned artifact enumerates **27 source entries / 4,174 `Default_Ignorable_Code_Point` scalars**. Those entries compress exactly to #324's 17 implementation ranges/singletons. In particular, source subdivisions at `180B..180D` + `180E` + `180F`, `2060..2064` + `2065` + `2066..206F`, and the seven adjacent `E0000..E0FFF` entries contain no gaps, so the implementation's compressed ranges add no code points. The formally released property set is therefore equivalent to the pinned admission grammar.

No tailored exception is present. In particular, ZWNJ (`U+200C`), ZWJ (`U+200D`), variation selectors, tag characters, fillers, and reserved code points carrying DICP remain rejected for benchmark-owned evidence identity. A future exception requires its own security/compatibility decision and new versioned profile.

#325 no longer waits for the stale summary-page banner to disappear. The remaining provenance gap is an immutable byte-level receipt for the versioned artifact: resolved URL/redirect result, header/date, byte length, full-file SHA-256, and a normalized DICP-property digest. If that receipt confirms the same property set, traceability can close provenance without changing admission semantics. If it unexpectedly differs, the table must not be silently rewritten under the existing evidence identity: a fresh hostile/compatibility RED and a new profile identity are required.

## References

Unicode Consortium. (2026, September 16). *Unicode recent releases*. https://www.unicode.org/releases/

Unicode Consortium. (2026, September 16). *Announcing the Unicode Standard, Version 18.0*. https://blog.unicode.org/2026/09/announcing-unicode-standard-version-180.html

Unicode Consortium. (2026). *The Unicode Standard, Version 18.0.0*. https://www.unicode.org/versions/Unicode18.0.0/

Unicode Consortium. (2026, September 1). *Unicode identifiers and syntax* (Unicode Standard Annex #31, Version 18.0.0, Revision 45). https://www.unicode.org/reports/tr31/tr31-45.html

Unicode Consortium. (2026, August 27). *Unicode security mechanisms* (Unicode Technical Standard #39, Version 18.0.0, Revision 34). https://www.unicode.org/reports/tr39/tr39-34.html

Unicode Consortium. (2026, August 7). *DerivedCoreProperties-18.0.0.txt* [Unicode 18.0.0 Unicode Character Database artifact]. https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt
