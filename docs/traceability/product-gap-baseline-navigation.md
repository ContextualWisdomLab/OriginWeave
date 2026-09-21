# Product-gap baseline navigation traceability

- Status: Implemented on active documentation parent; protected-main acceptance pending.
- Scope: `docs/product-technical-gap-baseline.md` buyer navigation, dated observation semantics, historical evidence retention, and historical-contract input isolation only.
- Preserved predecessor baseline: Git blob `c4a87e6c70f2c48b6e4ef820f18e8ddb0dca5696`, 186,500 bytes, at `docs/evidence/product-technical-gap-baseline-through-2026-09-09.md`.
- Unicode execution currentization RED: `08c34e52346d488fd8bf4c4e97da912ad7de129b`.
- Unicode execution buyer-surface repair: `5e33798e037186af110fae36f3e0785c80d19286`.
- Ready-lifecycle receipt RED: `b389ee795f5d66161a815c1d51afd80001c00d0e`.
- Ready-lifecycle buyer-surface repair: `20184b1173704fd79b4a674f3ce1dc0e2e2e1d9d`.
- Earlier currentization lineage: `32b01ebda4ce061e63c926434d4320847c93fbcf` → `84340f2ac35ddbc0571038418acc93e0f9882595` → `523a714bbeaec3d882e04355de8b83b6bff67176` → `1c9a14ae9010149a1c22bc85a76c3d16bd10b3c7` → `97395c35b514acdc4c5e48f9f551d384347cbc05`.
- Historical-input repair: `cbe046bacbe34a470fb73ea38cf81049e9d5812d`, refined at `20ac2502e0fb80e7c7c1b92851fb7ddef8ba6df0`.
- #309 verified successor: `6c4187f0849fb0ff89087c0a47eaef4666345ec0`, native CI `34365887571` successful; normal adoption into #238: `37fcd5702c7785f7f353bf4527ff56e368068cfc`.

## Problem

The canonical buyer surface is a bounded dated receipt rather than a live dashboard. That design exists because the former 186,500-byte baseline mixed current decisions, historical runs and implementation detail densely enough that stale exact-head prose could be mistaken for current acceptance.

The 2026-09-20 receipt correctly moved Unicode #324 from a pre-release publication gate to stable Unicode 18.0.0 provenance. It then became stale when #324 exact `accdd2d194f21ae1444ccca5297ce6590bc5384e` was finally admitted to hosted execution: CI `35531334971` failed only at canonical rustfmt after repository contracts, its Production coverage job succeeded, and Manifest V3 `35531334972` succeeded. Two ordinary-forward formatting-only repairs advanced #324 to `b89b40351152abe6a9c15fffd67f785933f6e164`, with current CI `35556275575` and Manifest V3 `35556275524` queued.

That change produced the 2026-09-21 03:59 receipt. Completing the bounded documentation source repair then allowed #238 itself to move from Draft to Ready for review. Because #238 is part of the observed PR inventory, that lifecycle transition immediately changed the live split from **5 Ready / 130 Draft** to **6 Ready / 129 Draft** while total PR and issue counts remained unchanged. Keeping the 03:59 receipt as the active cut after intentionally changing the observed object would repeat the self-staleness defect this contract exists to prevent.

The currentization therefore advances the full bounded receipt again. This is not a source-neutral wake: it records the causal Ready transition required for normal review/execution admission and makes that transition test-visible.

## Constraints

- Do not rewrite the immutable historical dossier or its predecessor-bound companion inputs.
- Do not turn an active PR, predecessor GREEN, mergeability, a protocol acknowledgement, or a clean scan into protected-main or release truth.
- Preserve exact heads, run IDs, measurements and dates for the generation that produced them.
- A static Markdown receipt must state its observation time and that later GitHub state supersedes it.
- If the documentation PR itself affects queue membership, include that lifecycle state in the cut.
- Do not mix newer owner/runtime state into an older dated receipt.
- Keep production browser/runtime source, `.github/**`, rulesets, secrets and sibling-owner source out of this documentation lane.
- Stable Unicode provenance and executable product acceptance are separate authorities.
- Current-head results never inherit predecessor results merely because later commits are formatting-only or documentation-only.
- Browser-issued WebDriver BiDi/Chromium identifiers remain outside the benchmark-owned Unicode evidence grammar.

## Alternatives considered

1. Keep the 2026-09-20 receipt and patch only #324. Rejected because it creates a false mixed-time receipt.
2. Keep the 03:59 receipt after moving #238 Ready. Rejected because the PR itself changed the queue split that the receipt describes.
3. Present queue counts as continuously current. Rejected because a static document cannot maintain that guarantee.
4. Add a no-op commit solely to wake CI. Rejected. The only post-Ready source change is the real self-staleness repair caused by the Ready transition itself.
5. Transfer predecessor #324 coverage/MV3 success to `b89b403...`. Rejected because exact-head acceptance is generation-specific.
6. Rewrite the historical dossier to make the live page easier to read. Rejected because that destroys evidence identity and old relative-link semantics.
7. Advance the whole bounded receipt and retain earlier cuts as historical evidence. Selected.

## Decision

The current buyer receipt is **2026-09-21 04:07 UTC**. Fresh complete GitHub searches returned **135 open PRs / 6 Ready/non-draft / 129 Draft / 19 open non-PR issues** after #238 moved to Ready. #309 remains merged into the #238 documentation lineage. #238 Ready status is review/execution admission only; it is not merge acceptance. Later live GitHub state explicitly supersedes this cut.

The previous cuts remain evidence, not active acceptance:

- **Historical 2026-09-21 03:59 receipt:** 135 open PRs / 5 Ready/non-draft / 130 Draft / 19 open non-PR issues; #238 was still Draft at that cut.
- **Historical 2026-09-20 receipt:** 135 open PRs / 5 Ready/non-draft / 130 Draft / 19 open non-PR issues.
- **Historical 2026-09-15 receipt:** 135 open PRs / 13 Ready/non-draft / 122 Draft / 19 open non-PR issues.
- **Historical 2026-09-09 receipt:** 130 open PRs / 12 Ready / 118 Draft / 14 open non-PR issues.

The Unicode row follows Ready/mergeable #324 exact `b89b40351152abe6a9c15fffd67f785933f6e164` and profile `unicode-18.0.0-default-ignorable-exclusion`. Stable Unicode 18.0.0 publication provenance remains unchanged:

- versioned DICP: **1,159,889 bytes**, SHA-256 `09c928886a178fcafd93c29e4bd59073a058e5a100b716d425cb563ab50f68c9`;
- `Default_Ignorable_Code_Point`: **27 source entries / 4,174 scalars**;
- ascending `%06X\n` normalization: **29,218 bytes**, SHA-256 `673264e62183e35f6055a2ad4940403e706669e0750fcc5d56a99f158fb3bb93`, equal to the implementation expansion.

Executable state is recorded separately. Predecessor `accdd2d...` produced CI `35531334971` **failure** because `cargo fmt --all --check` rejected two test files after repository contracts; the same generation's Production coverage was **success**, and Manifest V3 `35531334972` was **success**. Formatting-only successors `1348f798e7b19df21b0930be704b47f1e4fedb06` and `b89b40351152abe6a9c15fffd67f785933f6e164` repair exactly those diagnostics. Current CI `35556275575` and Manifest V3 `35556275524` are queued, so no current-head GREEN is claimed.

#325 remains the release/provenance acceptance gate. A later authoritative Unicode property mismatch requires a fresh security/compatibility RED and a new profile identity rather than silent mutation.

The rest of the buyer matrix keeps its owner boundaries: #299 preserves the real Chrome/ChromeDriver `150.0.7871.129` 0/3 session-creation RED; `.github#1857` remains the central reusable sandboxed browser-evidence workflow owner; #287 remains the docs-only classifier foundation; #308 remains the governed Rust `1.98.1` successor. None is copied into this branch.

## Historical-evidence boundary

The archived dossier remains byte-identical. Historical compatibility tests consume predecessor-bound copies rather than mutable current controls. Existing preserved blobs include documentation fitness `69f60325297e27d1f216a2b3cd83dab6acfee2a1`, active-PR maturity `71353dca28208ca2756a12f25223e0d13c109f65`, `AGENTS.md` `82e594adfd66e9d5b5f5686fdc7beecb39d19213`, evidence collector `f530df1aaf2eb3265037c4a416ddb20d28cb3a46`, and predecessor changelog `84b262257aa06fe8a426b8df95d5bfcb1270507b`.

`docs/evidence/product-technical-gap-baseline-through-2026-09-09-navigation.md` records how the relocated archive's original `docs/`-relative links map without rewriting the archive itself.

## Risk and mitigation

- **Receipt staleness:** observation time and supersession sentence make the boundary explicit; lifecycle changes to the documentation PR itself require a new bounded cut.
- **CI wake disguised as content:** the Ready transition occurred first and changed a measured fact; the follow-up source delta records that fact and is independently testable rather than being a no-op wake.
- **Predecessor evidence transfer:** current candidate and predecessor execution are named separately, with current queued runs required to finish on the unchanged head.
- **Stable artifact mistaken for product acceptance:** Unicode hashes and property equality are recorded independently from repository/browser/review/ruleset gates.
- **Archive divergence:** historical files retain predecessor Git blobs instead of reserialization.
- **Owner duplication:** browser workflow, security and orchestration mechanics remain in their canonical owners and enter OriginWeave only through normal protected/versioned adoption.

## Acceptance

- `docs/product-technical-gap-baseline.md` remains at most 220 lines and 24,000 UTF-8 bytes, with `## Buyer gap matrix` within the first 90 lines.
- The current receipt is `2026-09-21 04:07 UTC`, records **135 open PRs / 6 Ready/non-draft / 129 Draft / 19 open non-PR issues**, names #238 Ready status as review/execution admission only, and states that later live GitHub state supersedes it.
- Historical 2026-09-21 03:59, 2026-09-20, 2026-09-15 and 2026-09-09 receipts remain explicitly historical.
- The current Unicode row names #324 exact `b89b40351152abe6a9c15fffd67f785933f6e164`, stable raw/semantic receipts, predecessor CI formatting-only RED, predecessor coverage/MV3 successes, and current queued runs `35556275575` / `35556275524` without transferring acceptance.
- The archive remains exact predecessor blob `c4a87e6c70f2c48b6e4ef820f18e8ddb0dca5696` and remains larger than 180,000 bytes with its historical anchors.
- Historical contract inputs stay predecessor-bound; current navigation regressions remain discoverable under their original `test_*.py` entry points.
- No production source, workflow, ruleset, secret, release metadata or sibling-owner source is changed by this currentization.
- Before protected integration, re-read the exact #238 head/base, required workflows, reviews, unresolved threads, protected main, rulesets and release inventory. Skipped/queued/predecessor-only results are not passing evidence.
