# Active pull-request maturity evidence — 2026-08-11 delta

- **Protected-main anchor:** `67af7c87589edc2039545af335c95064d9b8391c`
- **Canonical documentation verdict:** **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**
- **Relationship to the existing series:** this file advances the dated evidence in [`2026-08-10-active-pr-maturity.md`](2026-08-10-active-pr-maturity.md) for active lanes opened after that appendix was refreshed through PR #66. It is volatile implementation evidence, not timeless architecture truth.

Protected `main` remains the only shipped-code authority. Active pull requests, exact heads, CI runs, reviews, and coverage reports are evidence about non-shipped work until dependency-ordered integration and fresh protected-main acceptance are re-established.

## Newly active implementation evidence

| PR | Scope | Maturity | Exact evidence / authority boundary |
|---|---|---|---|
| #67 | Browser-task interruption and recovery evidence | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `9d9ebffee234ed4ab662dab7850bd08450ec365b` is stacked on unchanged #64 head `2c45411ed9aa0eecca2d06c85659db9f4bb85e4d`. CI run `31448465680` is successful, including exact owned production function/line/region/branch coverage. The value contract distinguishes an interruption proven before external effect from an effect that may have committed and requires browser-context closure, task-resource reclamation, and evidence finalization before `SafeToRetry`. It does **not** detect Chromium crashes, prove caller-supplied cleanup facts, reconcile external mutations, restart Chromium, dispatch a retry, persist checkpoints, or complete issue #28's real-browser vertical slice. |
| #68 | Identity-bound settlement of failed sensitive-handle reservations | **PARTIAL** | The lane is stacked on exact #55 head `8d3ccf0a3b99fd9789210dd9798b422431fab7d8`. Exact predecessor head `add3599bee784c58dfaa4275d17c477eaed781a9` passed repository contracts, formatting, workspace tests, strict Clippy and rustdoc but failed exact coverage at `branches=495/496`, `lines=3666/3667`, `regions=4575/4576`. The uncovered production `next_reservation_sequence == None` branch was synthetic/private-test-only, so the production design was replaced rather than weakening the gate. Exact head `17bc00790e75424afd97c8a73800d9b16c766300` replaced the finite sequence with an allocation-bound, non-copyable in-process reservation identity and passed CI `31451682170`, including exact owned function/line/region/branch coverage and CodeRabbit exact-head status. Current exact head `aa46d982b2bf786fe297744ac99f88b6c4c5f4cf` additionally proves a token from one state instance cannot commit or compensate another identical-scope state. Fresh CI run `31451963178` is still in progress, so predecessor-head success is not promoted to the current head. The lane still provides no authenticated workload identity, protected-value resolution, durable/cross-process transaction, KMS, persistence, or proof that compensation is truthful. |

## Documentation-fitness reconciliation

The addition of #67 and #68 does **not** require another ADR, a new deployed component, or a physical ERD entity at this stage.

- **ADR:** #67 refines the existing evidence/recovery architecture without changing a trust-domain or persistence-owner decision. #68 refines the in-process sensitive-handle lifecycle governed by Accepted ADR 0007; it remains short of the trusted broker required by issue #10. Existing ADR breadth remains sufficient.
- **PRD/TRD:** current requirements already separate verified post-condition/recovery evidence from browser dispatch and separate purpose-bound sensitive policy from the future trusted broker. Both lanes are active/non-shipped evidence and must not be described as `Implemented` on protected main.
- **Architecture/UML:** neither lane introduces a new deployed service or browser-protocol boundary. Detailed real-Chromium crash/retry sequencing remains legitimately deferred until issue #28 has an executable adapter path whose recovery facts can be authoritative rather than caller-supplied.
- **ERD/data model:** #67 is an immutable evidence value and #68 is explicitly in-process policy state. Neither creates an OriginWeave-owned durable persistence schema. The conceptual ERD remains the truthful artifact; manufacturing tables would overstate the implementation.
- **Security/privacy:** #67 remains credential-free and quarantines ambiguous-effect/incomplete-cleanup states. #68 narrows settlement to exact in-process reservation identity while preserving the rule that only a trusted broker may decide that compensation is valid before disclosure.
- **Test/release/traceability:** #67 has exact-head green CI/coverage evidence. #68 is currently `PARTIAL` until its unchanged exact current head proves the fresh gates. No predecessor-head success is transferable.

## Interpretation rules

1. `IMPLEMENTED_ON_ACTIVE_PR` and `PARTIAL` never mean shipped.
2. Exact-head CI/coverage evidence becomes stale immediately when that head moves.
3. A stacked PR cannot be independently integrated before its exact prerequisite lineage.
4. An active implementation refinement does not manufacture a new ADR merely to mirror every PR; create or supersede an ADR only when the governing architecture decision changes.
5. In-memory identities, immutable evidence values, controlled fixtures, and bounded samplers do not justify physical ERD entities without a real durable ownership boundary.
6. After any of these lanes integrates, re-evaluate PRD/TRD/Architecture/UML/ERD/traceability from the new protected-main head before changing maturity claims.
