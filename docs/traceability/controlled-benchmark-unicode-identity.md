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
| Reject Unicode default-ignorable scalars from benchmark-owned run-context identity | UAX #31 Rev. 45 §7.3; stable Unicode 18.0.0 DICP property set | `CONTROLLED_BENCHMARK_UNICODE_IDENTITY_PROFILE`; `is_unicode_18_default_ignorable`; `validate_run_context_field` | focused `unicode_18_default_ignorable_reproducibility_context_fails_closed`; exhaustive expected/observed coverage in `every_unicode_18_default_ignorable_expected_scalar_fails_closed` and `every_unicode_18_default_ignorable_observed_scalar_fails_closed` over all 27 stable source ranges / 4,174 scalars | Active PR #324 only; exact-head repository acceptance still pending |
| Bind the property set to a named Unicode version | UAX31-C1 requires implementations claiming UAX #31 conformance to identify the specification version; DICP has no general stability guarantee | profile value `unicode-18.0.0-default-ignorable-exclusion`; exact local range table | profile-string assertion plus stable-source full-property conformance | Grammar pinned; formal release, immutable raw-file receipt, normalized property equality, and exhaustive source-range test contract confirmed |
| Reconcile the pre-release observation after formal publication | Unicode release registry; Version 18.0.0 citation; stable `/Public/18.0.0/` artifact route | no silent table rewrite; #325 records immutable artifact identity | 27-entry / 4,174-scalar comparison; raw-file receipt; normalized source/implementation digest equality | Artifact-dependent provenance checks satisfied; exact-head repository/review gates remain |
| Preserve ordinary visible multilingual / RTL labels | UTS #39 allows profiles and explicit tailoring decisions; #323 chooses only DICP exclusion for this machine/audit identity | no normalization, script allow-list, case fold, or confusable transform | `visible_unicode_reproducibility_context_remains_valid` | Active PR #324 only |
| Validate expected and observed identity before equality affects suite evidence | OriginWeave controlled-benchmark fail-closed contract | `evaluate_controlled_benchmark_suite_for_run` validates both sides before byte equality | hostile-expected / valid-observed and valid-expected / hostile-observed representative and exhaustive DICP cases plus existing mismatch, blank, whitespace, control, bidi, and line-separator cases | Active PR stack |
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
- 2026-09-20 normalized-property doctoring receipt: `f8c8468ae43cab5a2aa281cc68c4f0ec736ee57d`.
- 2026-09-20 live-route correction: doctoring `eaf033a461092c299fbd228bf0f486986adf9238`; traceability `2a9ec91f9892bd2f78a2d72286cf33d6bbfedbf9`.
- 2026-09-21 production-rustdoc provenance correction: `e0b9bd793ceaf784b45ec9c4a720c5243892abdf`.
- 2026-09-21 stable-versioned-artifact doctoring receipt: `d8586c30b933c71f263b747c7aba6de965d46308`.
- 2026-09-21 hostile-scalar test-lifetime repair: `916481bb79d6acd2f42f67bfd7e98ba33976e686`.
- 2026-09-21 exhaustive stable-source DICP conformance test: `506b315d3beb2059389ab8b66f7f3399b4f9c1d4`.
- 2026-09-21 exhaustive observed-context validation regression: `f0edc784aef61f5636b8edbfe1da895f2d55368e`.
- 2026-09-21 focused observed-context validation regression: `362cfc3dd299abf4dbfbdbdac458eddf5a7bf3d4`.
- 2026-09-22 exhaustive expected-context validation isolation: `01984398f9534903ecff38c908673830a8010de1`.

The semantic RED was established against exact predecessor source; no hosted Rust RED is claimed. Fresh review after the Ready transition found a separate test-harness compile defect: the focused DICP loop attempted to assign a loop-local `String` borrow into a `ControlledBenchmarkRunContext<'static>` returned by `run_context()`. `916481bb...` repairs that by constructing a fresh aggregate whose inherited `'static` fields shorten to the loop-local aggregate lifetime. `506b315d...` then adds an independent integration-test fixture that expands the stable UCD's 27 source entries and requires every one of the 4,174 scalars to fail closed, with the exact total asserted. These test successors do not change production semantics.

Current-head review then exposed a coverage asymmetry: the focused and exhaustive hostile DICP cases supplied the same hostile context as both expected and observed input. Because `evaluate_controlled_benchmark_suite_for_run` validates expected before observed, those cases could not detect an accidental removal of observed-context validation. `f0edc784...` adds an exhaustive valid-expected / hostile-observed regression for every one of the 4,174 DICP scalars. `362cfc3d...` applies the same separation to the representative focused test. Production code remains unchanged; these are regression-strengthening repairs for the existing two-sided fail-closed contract.

A fresh exact-head re-read found the inverse test-oracle ambiguity still present in the exhaustive source fixture: its expected-side case continued to pass the same hostile context as both expected and observed. If expected-context validation were accidentally removed while observed validation remained intact, that test would still pass and would no longer prove the expected side of the contract. `01984398...` renames the fixture to state its authority and supplies hostile expected / valid observed input for all 4,174 scalars. Together with `f0edc784...`, the exhaustive regressions now isolate both validation directions. Production code, Unicode property data, policy, workflow, and provenance receipts remain unchanged.

Draft-policy generations and queued runner-less jobs are not GREEN evidence. Promotion requires executable exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, 100% production function/line/region/branch coverage, required security/review workflows, and the normal repository merge gate.

## Property fidelity

The table originated from `DerivedCoreProperties-18.0.0.txt` dated 2026-08-07. Before and immediately after Unicode 18.0.0 formal publication, the nominal versioned route was observed resolving through `/Public/draft/`; those observations were correctly documented as insufficient for immutable raw-file provenance.

Fresh 2026-09-21 KST revalidation changes the artifact state. The Unicode Consortium release registry still establishes the formal **Unicode 18.0.0 — 2026-09-16** release independently. The canonical data URL `https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt` now responds directly with HTTP 200 and identifies the body as `DerivedCoreProperties-18.0.0.txt`; `https://www.unicode.org/Public/18.0.0/` now directly exposes the versioned `ucd/` directory. The earlier draft redirect is therefore historical evidence, not the current route authority.

The stable versioned file receipt is **1,159,889 raw file bytes**, SHA-256 **`09c928886a178fcafd93c29e4bd59073a058e5a100b716d425cb563ab50f68c9`**. The returned artifact reports embedded date `2026-08-07, 16:19:42 GMT`; observed HTTP metadata includes `Last-Modified: Tue, 01 Sep 2026 20:29:02 GMT` and `ETag: "11b2d1-65a71c4edff80-gzip"`. The cryptographic receipt is over the decoded file bytes rather than the gzip transfer representation. A repeated direct retrieval returned the same byte length and SHA-256.

The stable DICP enumerates **27 source entries / 4,174 `Default_Ignorable_Code_Point` scalars**. Those entries compress exactly to #324's 17 implementation ranges/singletons. In particular, source subdivisions at `180B..180D` + `180E` + `180F`, `2060..2064` + `2065` + `2066..206F`, and the seven adjacent `E0000..E0FFF` entries contain no gaps, so the implementation's compressed ranges add no code points.

A deterministic normalized property receipt independently verifies semantic equality. Expand the DICP source entries to the ascending scalar set; encode each scalar as six uppercase hexadecimal digits followed by LF (`%06X\n`); concatenate all records. The 4,174-scalar stream is **29,218 bytes** with SHA-256 `673264e62183e35f6055a2ad4940403e706669e0750fcc5d56a99f158fb3bb93`. Expanding #324's 17 compressed implementation ranges under the same normalization yields the same byte stream and digest. The raw-file SHA identifies the stable Unicode artifact; the normalized digest proves the exact property semantics OriginWeave pins. Neither substitutes for the other.

The exhaustive integration test intentionally uses the **27 source-entry boundaries**, rather than copying the production function's 17 compressed ranges. This keeps source-data structure and implementation structure independent enough to catch a future compressed-range omission while directly exercising the public benchmark admission path for every stable DICP scalar.

No tailored exception is present. In particular, ZWNJ (`U+200C`), ZWJ (`U+200D`), variation selectors, tag characters, fillers, and reserved code points carrying DICP remain rejected for benchmark-owned evidence identity. A future exception requires its own security/compatibility decision and new versioned profile.

Issue `#325` no longer has an external Unicode-publication blocker: the versioned route is stable, the raw byte length/full-file SHA-256 are captured, and normalized comparison against those stable bytes matches the existing implementation. The issue remains open only until the current #324 exact head satisfies repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, 100% owned-production function/line/region/branch coverage, required security/browser workflows, review, and normal repository policy. A later authoritative artifact mismatch must trigger a fresh hostile/compatibility RED and a new profile identity rather than a silent table rewrite.

## References

Unicode Consortium. (2026, September 16). *Unicode recent releases*. https://www.unicode.org/releases/

Unicode Consortium. (2026, September 16). *Announcing the Unicode Standard, Version 18.0*. https://blog.unicode.org/2026/09/announcing-unicode-standard-version-180.html

Unicode Consortium. (2026). *The Unicode Standard, Version 18.0.0*. https://www.unicode.org/versions/Unicode18.0.0/

Unicode Consortium. (2026, September 1). *Unicode identifiers and syntax* (Unicode Standard Annex #31, Version 18.0.0, Revision 45). https://www.unicode.org/reports/tr31/tr31-45.html

Unicode Consortium. (2026, August 27). *Unicode security mechanisms* (Unicode Technical Standard #39, Version 18.0.0, Revision 34). https://www.unicode.org/reports/tr39/tr39-34.html

Unicode Consortium. (2026, August 7; stable versioned route revalidated 2026-09-21). *DerivedCoreProperties-18.0.0.txt* [Unicode 18.0.0 Unicode Character Database artifact]. https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt
