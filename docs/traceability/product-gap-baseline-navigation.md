# Product-gap baseline navigation traceability

- Status: Implemented on active documentation parent; protected-main acceptance pending.
- Scope: `docs/product-technical-gap-baseline.md` buyer navigation, dated observation semantics, historical evidence retention, and historical-contract input isolation only.
- Predecessor: #238 branch exact `16098a0eecc57bfc545d22ce632d95d1ffccfe67`.
- Preserved predecessor baseline blob: Git blob `c4a87e6c70f2c48b6e4ef820f18e8ddb0dca5696`, 186,500 bytes.
- Triggering buyer-navigation finding: #238 comment `5598004000`.
- Self-staleness finding: #309 review `5154249954` on exact `187310346c5c42b22e53f734c2c9ce7122df0d7f`.
- Self-staleness RED: `32b01ebda4ce061e63c926434d4320847c93fbcf`.
- Minimal dated-receipt repair: `84340f2ac35ddbc0571038418acc93e0f9882595`.
- Verified #309 successor: `6c4187f0849fb0ff89087c0a47eaef4666345ec0`, native CI `34365887571` successful.
- Normal #309 adoption into #238: merge commit `37fcd5702c7785f7f353bf4527ff56e368068cfc`; protected `main` unchanged.
- Post-adoption review findings: #238 CodeRabbit comment `5674573338` — mutable historical inputs and relocated relative-link semantics.
- Post-adoption hostile RED: `cbe046bacbe34a470fb73ea38cf81049e9d5812d`, refined at `20ac2502e0fb80e7c7c1b92851fb7ddef8ba6df0`.
- Current-receipt consistency finding: focused review of #238 exact `d62e7a3b8915a13cb40f02c882a698ec12eaec0f` found this traceability file still treating the superseded September 9 receipt as current acceptance.
- Current-receipt RED: `523a714bbeaec3d882e04355de8b83b6bff67176` requires the traceability contract to follow the September 15 buyer receipt while retaining September 9 as historical evidence.

## Problem

The baseline had accumulated current observations, repeated same-day checkpoints, old exact heads, measurements, browser screenshots and long implementation runbooks in one 186,500-byte file. The latest observation was accurate, but the commercial gap matrix was no longer a practical decision surface: a buyer or maintainer had to traverse historical evidence before reaching the current gap/owner/acceptance view.

That is a documentation correctness issue, not cosmetic cleanup. When current delivery truth and historical evidence use the same unbounded surface, stale exact-head prose becomes easier to mistake for current acceptance and the next causal action is harder to identify.

The first compact successor exposed a second correctness defect. It labelled a static queue snapshot `Current exact observation` and recorded 129 open PRs before #309 itself existed. Creating #309 changed the queue, and restoring #309 to Draft changed the Ready/Draft split again. A static Markdown file cannot truthfully promise continuously current volatile GitHub counts; self-modifying queue membership makes that claim especially unstable.

After the exact #309 successor was normally adopted into #238, fresh review exposed two additional defects in the historical-evidence boundary. First, compatibility wrappers redirected only `BASELINE` and sometimes `CHANGELOG`; several legacy contracts still consumed mutable current `DOCUMENTATION_FITNESS.md`, active-PR maturity evidence, `AGENTS.md`, and `collect_live_merge_evidence.sh`, while one completion contract performed direct reads through its current repository `ROOT`. Second, the byte-identical baseline archive moved from `docs/` to `docs/evidence/`, so relative links inside the immutable blob no longer resolve from their original base path.

A later #238 refresh exposed a fourth defect: the live buyer baseline had moved to the `2026-09-15 08:57 UTC` receipt, but this traceability file still described the `2026-09-09 12:57 UTC` receipt and its 130 / 12 / 118 split as active acceptance. That created two conflicting current contracts for the same buyer decision surface even though the September 9 receipt should remain historical evidence only.

## Constraints

- Do not discard or rewrite historical evidence to make the current file shorter.
- Do not turn active-PR evidence into protected-main or release truth.
- Preserve exact heads, measurements and dated claims for their recorded generation.
- Do not present a volatile GitHub snapshot as continuously current after its observation cut.
- If the documentation PR itself affects the observed queue, include that PR and its lifecycle state in the cut.
- Do not touch production browser/runtime code, `.github/**`, rulesets, secrets, review state or another bounded context's authority.
- Preserve existing historical regression coverage rather than deleting assertions that become inconvenient after extraction.
- Historical compatibility tests must consume predecessor-bound evidence inputs, not mutable current controls.
- Preserve the archived baseline Git blob byte-for-byte; repair relocated link navigation in a companion receipt rather than rewriting the archive.
- Keep the canonical buyer path `docs/product-technical-gap-baseline.md` stable so PRD/TRD/ARCHITECTURE/docs indexes do not need a new authority name.

## Alternatives considered

1. **Keep appending to one file.** Rejected because it preserves evidence but does not repair decision-surface reachability.
2. **Delete old checkpoints.** Rejected because it destroys traceability and makes previous exact-head claims unauditable.
3. **Collapse old prose with HTML details while retaining it in the same source file.** Rejected as the final design because rendered navigation improves but source-level history and current authority remain coupled.
4. **Keep volatile queue counts under a `Current exact` heading.** Rejected because the claim becomes stale as soon as the queue changes and can be invalidated by the documentation PR itself.
5. **Rewrite relative links inside the relocated archive.** Rejected because it would destroy byte identity of the historical receipt.
6. **Let historical tests keep reading current control files.** Rejected because later edits could make a historical generation pass or fail for reasons absent from that generation.
7. **Keep the September 9 receipt as the active traceability contract after the buyer baseline advances.** Rejected because it creates contradictory current acceptance criteria; the older receipt belongs in historical evidence.
8. **Move the historical dossier to immutable evidence, bind compatibility tests to predecessor inputs, and use an explicitly dated observation receipt at the canonical path.** Selected. It separates current decision authority from historical receipts, preserves the buyer entry point, and makes both freshness and provenance boundaries explicit.

## Decision

The predecessor baseline blob is reused byte-for-byte at `docs/evidence/product-technical-gap-baseline-through-2026-09-09.md`. The canonical baseline becomes a bounded dated observation receipt, buyer gap matrix, acceptance order and compact evidence/history index.

Review `5154249954` was repaired test-first for the original September 9 generation. Commit `32b01ebda4ce061e63c926434d4320847c93fbcf` required an explicit `2026-09-09 12:57 UTC` observation cut, rejected the continuously-current heading, included #309's then-Draft state in the 130-PR / 12-Ready / 118-Draft count, and required later GitHub state to supersede the cut. Commit `84340f2ac35ddbc0571038418acc93e0f9882595` made that bounded repair without changing the archived dossier, production source, workflows, rules, or browser evidence. That generation is now the **Historical 2026-09-09 receipt**: **130 open PRs / 12 Ready / 118 Draft / 14 open non-PR issues**. It remains preserved evidence, not the current acceptance condition.

The current buyer receipt is `2026-09-15 08:57 UTC`: **135 open PRs / 13 Ready/non-draft / 122 Draft / 19 open non-PR issues**, and **#309 is merged into the #238 documentation lineage**. The receipt explicitly states that it is dated rather than continuously live, and later live GitHub state supersedes it. The traceability acceptance below follows this current bounded receipt while retaining September 9 only as historical evidence.

The completed #309 successor `6c4187f0849fb0ff89087c0a47eaef4666345ec0` passed native CI `34365887571`: Rust contracts `102514364056` passed repository contracts, formatting, workspace tests, strict Clippy and API docs; Production coverage `102514364737` passed exact function/line/region/branch enforcement. It was then normally adopted into #238 as merge commit `37fcd5702c7785f7f353bf4527ff56e368068cfc`; comparison from the child head to the merge commit has no file delta, so the verified child tree was carried without rewriting it. This adoption is parent-branch lineage, not protected-main shipment.

Fresh review comment `5674573338` produced the next test-first repair. `cbe046bacbe34a470fb73ea38cf81049e9d5812d` first requires predecessor-bound historical inputs and an original-location navigation receipt; `20ac2502e0fb80e7c7c1b92851fb7ddef8ba6df0` refines direct dynamic reads to an archived `ROOT` rather than rewriting large legacy contract modules. The repair then reuses exact predecessor blobs for documentation fitness (`69f60325297e27d1f216a2b3cd83dab6acfee2a1`), active-PR maturity (`71353dca28208ca2756a12f25223e0d13c109f65`), `AGENTS.md` (`82e594adfd66e9d5b5f5686fdc7beecb39d19213`), and the executable evidence collector (`f530df1aaf2eb3265037c4a416ddb20d28cb3a46`). The archive-root also reuses the predecessor changelog blob `84b262257aa06fe8a426b8df95d5bfcb1270507b` and collector blob, so direct `ROOT` reads are generation-bound without weakening legacy assertions.

`docs/evidence/product-technical-gap-baseline-through-2026-09-09-navigation.md` records the archive's original `docs/` relative-link base and deterministic mappings for the collector, `doctoring.md`, `PRD.md`, and `TRD.md`. The live baseline links that receipt. The immutable baseline blob itself remains unchanged.

Legacy tests whose purpose is to validate historical checkpoint integrity execute against the archived dossier and predecessor-bound companion inputs. They remain exposed through the original discovery wrappers. The current buyer surface receives its own navigation/freshness regressions. This avoids weakening historical assertions while preventing them from forcing historical runbooks or mutable current controls back into the live decision surface.

## Risk and mitigation

- **Risk: archive divergence.** Mitigation: archived inputs reuse predecessor Git blobs rather than reserializing them.
- **Risk: relocated links become misleading or broken.** Mitigation: a companion navigation receipt preserves the original `docs/` base semantics while the archive blob remains byte-identical.
- **Risk: dated observation mistaken for a live dashboard.** Mitigation: the heading carries the observation time, the text says it is not continuously live, and later GitHub state explicitly supersedes the cut.
- **Risk: current traceability silently pins a superseded receipt.** Mitigation: the navigation regression requires the `2026-09-15 08:57 UTC` receipt and its 135 / 13 / 122 / 19 inventory plus merged #309 lineage, while the Historical 2026-09-09 receipt remains explicitly historical.
- **Risk: old tests silently stop running or bind to current files.** Mitigation: original `test_*.py` entry points remain discoverable; the loader redirects baseline, changelog, fitness, maturity, AGENTS, evidence script, and direct-root reads to predecessor-bound evidence copies.
- **Risk: compactness hides material gaps.** Mitigation: the matrix keeps buyer risk, exact evidence, canonical owner/acceptance and state together, while linked PRs/issues and the evidence dossier retain detail.

## Acceptance

- `docs/product-technical-gap-baseline.md` is at most 220 lines and 24,000 UTF-8 bytes.
- `## Buyer gap matrix` appears within the first 90 lines.
- The live baseline contains no `### Previous verified cut:` daily-history section and no `## Current exact observation` heading.
- The current dated observation receipt records `2026-09-15 08:57 UTC`, **135 open PRs / 13 Ready/non-draft / 122 Draft / 19 open non-PR issues**, states that **#309 is merged into the #238 documentation lineage**, and says later live GitHub state supersedes the cut.
- The **Historical 2026-09-09 receipt** remains preserved as **130 open PRs / 12 Ready / 118 Draft / 14 open non-PR issues** and is not treated as the current acceptance condition.
- The archived dossier remains larger than 180,000 bytes and contains the known 2026-09-08 and 2026-08-29 historical anchors.
- The archive remains the exact predecessor baseline blob `c4a87e6c70f2c48b6e4ef820f18e8ddb0dca5696`.
- Historical contract inputs that were mutable at `37fcd570...` are bound to predecessor blobs, including direct-root changelog/collector reads.
- The navigation receipt is discoverable from the live buyer surface and preserves original `docs/`-relative link semantics without rewriting the archive.
- Existing historical regression classes remain discoverable through their original test entry points and run against archived predecessor inputs rather than being deleted or relaxed.
- Protected-main, browser/runtime source, workflows, review rules and external owner contracts remain unchanged by this documentation repair.

## Follow-up

Fresh exact-head hosted checks and independent review are required after the post-adoption repair. Draft/skipped jobs and predecessor GREEN do not prove the repaired head. If hosted runner admission stalls before any step executes, preserve the sole current-head jobs and hand the exact run/job identities to canonical `.github#712` rather than rerunning or mutating `runs-on`.

After this repair is accepted on #238, later refreshes should replace the bounded dated receipt/matrix with a new observed cut and add detailed evidence receipts only when that detail no longer belongs on the buyer surface.
