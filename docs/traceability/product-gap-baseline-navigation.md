# Product-gap baseline navigation traceability

- Status: Implemented on active documentation successor; protected-main acceptance pending.
- Scope: `docs/product-technical-gap-baseline.md` buyer navigation, dated observation semantics, and historical evidence retention only.
- Predecessor: #238 branch exact `16098a0eecc57bfc545d22ce632d95d1ffccfe67`.
- Preserved predecessor baseline blob: Git blob `c4a87e6c70f2c48b6e4ef820f18e8ddb0dca5696`, 186,500 bytes.
- Triggering buyer-navigation finding: #238 comment `5598004000`.
- Self-staleness finding: #309 review `5154249954` on exact `187310346c5c42b22e53f734c2c9ce7122df0d7f`.
- Self-staleness RED: `32b01ebda4ce061e63c926434d4320847c93fbcf`.
- Minimal dated-receipt repair: `84340f2ac35ddbc0571038418acc93e0f9882595`.

## Problem

The baseline had accumulated current observations, repeated same-day checkpoints, old exact heads, measurements, browser screenshots and long implementation runbooks in one 186,500-byte file. The latest observation was accurate, but the commercial gap matrix was no longer a practical decision surface: a buyer or maintainer had to traverse historical evidence before reaching the current gap/owner/acceptance view.

That is a documentation correctness issue, not cosmetic cleanup. When current delivery truth and historical evidence use the same unbounded surface, stale exact-head prose becomes easier to mistake for current acceptance and the next causal action is harder to identify.

The first compact successor exposed a second correctness defect. It labelled a static queue snapshot `Current exact observation` and recorded 129 open PRs before #309 itself existed. Creating #309 changed the queue, and restoring #309 to Draft changed the Ready/Draft split again. A static Markdown file cannot truthfully promise continuously current volatile GitHub counts; self-modifying queue membership makes that claim especially unstable.

## Constraints

- Do not discard or rewrite historical evidence to make the current file shorter.
- Do not turn active-PR evidence into protected-main or release truth.
- Preserve exact heads, measurements and dated claims for their recorded generation.
- Do not present a volatile GitHub snapshot as continuously current after its observation cut.
- If the documentation PR itself affects the observed queue, include that PR and its lifecycle state in the cut.
- Do not touch production browser/runtime code, `.github/**`, rulesets, secrets, review state or another bounded context's authority.
- Preserve existing historical regression coverage rather than deleting assertions that become inconvenient after extraction.
- Keep the canonical buyer path `docs/product-technical-gap-baseline.md` stable so PRD/TRD/ARCHITECTURE/docs indexes do not need a new authority name.

## Alternatives considered

1. **Keep appending to one file.** Rejected because it preserves evidence but does not repair decision-surface reachability.
2. **Delete old checkpoints.** Rejected because it destroys traceability and makes previous exact-head claims unauditable.
3. **Collapse old prose with HTML details while retaining it in the same source file.** Rejected as the final design because rendered navigation improves but source-level history and current authority remain coupled.
4. **Keep volatile queue counts under a `Current exact` heading.** Rejected because the claim becomes stale as soon as the queue changes and can be invalidated by the documentation PR itself.
5. **Move the historical dossier to immutable evidence and use an explicitly dated, self-inclusive observation receipt at the canonical path.** Selected. It separates current decision authority from historical receipts, preserves the buyer entry point, and makes the freshness boundary explicit.

## Decision

The predecessor baseline blob is reused byte-for-byte at `docs/evidence/product-technical-gap-baseline-through-2026-09-09.md`. The canonical baseline becomes a bounded dated observation receipt, buyer gap matrix, acceptance order and compact evidence/history index.

Review `5154249954` is repaired test-first. Commit `32b01ebda4ce061e63c926434d4320847c93fbcf` requires the live surface to use an explicit `2026-09-09 12:57 UTC` observation cut, reject the continuously-current heading, include #309's Draft state in the 130-PR / 12-Ready / 118-Draft count, and state that later live GitHub state supersedes the receipt. On predecessor `187310346c5c42b22e53f734c2c9ce7122df0d7f`, those assertions are source-semantic RED. Commit `84340f2ac35ddbc0571038418acc93e0f9882595` makes the smallest documentation repair without changing the archived dossier, production source, workflows, rules, or browser evidence.

Legacy tests whose purpose is to validate historical checkpoint integrity execute against the archived dossier. They are retained as byte-identical non-discovered modules and exposed through small compatibility wrappers that redirect only their `BASELINE` input. The current buyer surface receives its own navigation/freshness regression. This avoids weakening historical assertions while preventing them from forcing historical runbooks back into the live decision surface.

## Risk and mitigation

- **Risk: archive divergence.** Mitigation: the archive reuses the predecessor Git blob rather than reserializing it.
- **Risk: dated observation mistaken for a live dashboard.** Mitigation: the heading carries the observation time, the text says it is not continuously live, and later GitHub state explicitly supersedes the cut.
- **Risk: self-reference drift.** Mitigation: the observation cut includes #309 as Draft and the contract pins the complete 130 / 12 / 118 split observed after that lifecycle repair.
- **Risk: old tests silently stop running.** Mitigation: original `test_*.py` entry points remain discoverable and import/export the preserved legacy test classes after binding their historical baseline input to the archive.
- **Risk: compactness hides material gaps.** Mitigation: the matrix keeps buyer risk, exact evidence, canonical owner/acceptance and state together, while linked PRs/issues and the evidence dossier retain detail.

## Acceptance

- `docs/product-technical-gap-baseline.md` is at most 220 lines and 24,000 UTF-8 bytes.
- `## Buyer gap matrix` appears within the first 90 lines.
- The live baseline contains no `### Previous verified cut:` daily-history section and no `## Current exact observation` heading.
- The dated observation receipt records `2026-09-09 12:57 UTC`, includes #309 as Draft, records 130 open PRs / 12 Ready / 118 Draft, and says later live GitHub state supersedes the cut.
- The archived dossier remains larger than 180,000 bytes and contains the known 2026-09-08 and 2026-08-29 historical anchors.
- Existing historical regression classes remain discoverable through their original test entry points and run against the archived baseline rather than being deleted or relaxed.
- Protected-main, browser/runtime source, workflows, review rules and external owner contracts remain unchanged by this documentation repair.

## Follow-up

Fresh exact-head hosted checks and rendered GitHub inspection are still required before this documentation successor can be considered accepted. After normal integration into #238, later refreshes should replace the bounded dated receipt/matrix with a new observed cut and add detailed evidence receipts only when that detail no longer belongs on the buyer surface.
