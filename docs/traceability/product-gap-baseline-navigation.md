# Product-gap baseline navigation traceability

- Status: Implemented on active documentation successor; protected-main acceptance pending.
- Scope: `docs/product-technical-gap-baseline.md` buyer navigation and historical evidence retention only.
- Predecessor: #238 branch exact `16098a0eecc57bfc545d22ce632d95d1ffccfe67`.
- Preserved predecessor baseline blob: Git blob `c4a87e6c70f2c48b6e4ef820f18e8ddb0dca5696`, 186,500 bytes.
- Triggering review finding: #238 comment `5598004000`.

## Problem

The baseline had accumulated current observations, repeated same-day checkpoints, old exact heads, measurements, browser screenshots and long implementation runbooks in one 186,500-byte file. The latest observation was accurate, but the commercial gap matrix was no longer a practical decision surface: a buyer or maintainer had to traverse historical evidence before reaching the current gap/owner/acceptance view.

That is a documentation correctness issue, not cosmetic cleanup. When current delivery truth and historical evidence use the same unbounded surface, stale exact-head prose becomes easier to mistake for current acceptance and the next causal action is harder to identify.

## Constraints

- Do not discard or rewrite historical evidence to make the current file shorter.
- Do not turn active-PR evidence into protected-main or release truth.
- Preserve exact heads, measurements and dated claims for their recorded generation.
- Do not touch production browser/runtime code, `.github/**`, rulesets, secrets, review state or another bounded context's authority.
- Preserve existing historical regression coverage rather than deleting assertions that become inconvenient after extraction.
- Keep the canonical buyer path `docs/product-technical-gap-baseline.md` stable so PRD/TRD/ARCHITECTURE/docs indexes do not need a new authority name.

## Alternatives considered

1. **Keep appending to one file.** Rejected because it preserves evidence but does not repair decision-surface reachability.
2. **Delete old checkpoints.** Rejected because it destroys traceability and makes previous exact-head claims unauditable.
3. **Collapse old prose with HTML details while retaining it in the same source file.** Rejected as the final design because rendered navigation improves but source-level history and current authority remain coupled.
4. **Move the historical dossier to immutable evidence and keep a bounded current surface at the canonical path.** Selected. It separates current decision authority from historical receipts without changing the buyer entry point.

## Decision

The predecessor baseline blob is reused byte-for-byte at `docs/evidence/product-technical-gap-baseline-through-2026-09-09.md`. The canonical baseline becomes a bounded current exact-observation summary, buyer gap matrix, acceptance order and compact evidence/history index.

Legacy tests whose purpose is to validate historical checkpoint integrity execute against the archived dossier. They are retained as byte-identical non-discovered modules and exposed through small compatibility wrappers that redirect only their `BASELINE` input. The current buyer surface receives its own navigation regression. This avoids weakening historical assertions while preventing them from forcing historical runbooks back into the live decision surface.

## Risk and mitigation

- **Risk: archive divergence.** Mitigation: the archive reuses the predecessor Git blob rather than reserializing it.
- **Risk: current claims become stale.** Mitigation: keep the current observation bounded and replace it on a fresh evidence cut; historical details remain linked, not copied forward.
- **Risk: old tests silently stop running.** Mitigation: original `test_*.py` entry points remain discoverable and import/export the preserved legacy test classes after binding their historical baseline input to the archive.
- **Risk: compactness hides material gaps.** Mitigation: the matrix keeps buyer risk, exact evidence, canonical owner/acceptance and state together, while linked PRs/issues and the evidence dossier retain detail.

## Acceptance

- `docs/product-technical-gap-baseline.md` is at most 220 lines and 24,000 UTF-8 bytes.
- `## Buyer gap matrix` appears within the first 90 lines.
- The live baseline contains no `### Previous verified cut:` daily-history section.
- The archived dossier remains larger than 180,000 bytes and contains the known 2026-09-08 and 2026-08-29 historical anchors.
- Existing historical regression classes remain discoverable through their original test entry points and run against the archived baseline rather than being deleted or relaxed.
- Protected-main, browser/runtime source, workflows, review rules and external owner contracts remain unchanged by this documentation repair.

## Follow-up

Fresh exact-head hosted checks and rendered GitHub inspection are still required before this documentation successor can be considered accepted. After normal integration into #238, later refreshes should update the bounded current observation/matrix and add new evidence receipts only when detail no longer belongs on the buyer surface.
