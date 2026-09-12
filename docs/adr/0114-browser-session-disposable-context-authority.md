# ADR 0114: Browser Session disposable-context authority

- Status: Proposed
- Date: 2026-09-10

## Context

OriginWeave's WebDriver BiDi presentation adapter requires opaque ownership witnesses before viewport/device-pixel-ratio, timezone, or screen-area mutation can be planned. A caller that merely knows a browser-session or browsing-context identifier therefore cannot overwrite another owner's presentation state and later clear it to an implementation default.

The Browser Session boundary must establish why a context is exclusively OriginWeave-owned before presentation authority exists. External browser-session, user-context/isolation, and browsing-context identifiers are protocol addressability. They may be reused after a prior lifecycle ends, so `(BrowserSessionId, DisposableIsolationId, BrowsingContextId, local epoch)` is not by itself a durable capability generation.

Lifecycle failures also need lossless evidence. A BiDi adapter can successfully create a user context before later browsing-context creation or verification becomes uncertain. Duplicate adapter output can expose an offending handle that must not be silently discarded or automatically destroyed. Destruction can fail without proving that the exact isolation boundary is gone. These outcomes require recovery quarantine while retaining every exact browser-issued identity that is already known.

Transport liveness is independent from ownership certainty. A session already in `RecoveryRequired` can subsequently lose its transport; that new fact must be recorded without erasing the recovery evidence. Conversely, merely entering recovery does not prove the transport is dead.

The 18 August 2026 WebDriver BiDi Working Draft defines user-context identifiers and the `browser.createUserContext`, `browsingContext.create`, and `browser.removeUserContext` lifecycle. Those commands remain adapter capabilities rather than OriginWeave policy authority, and command ACK alone is not destruction proof.

## Decision drivers

- Raw WebDriver/BiDi identifiers are addressability, not mutation or cleanup authority.
- Sequential aggregate recreation must not make a retained stale authority valid again.
- The lifecycle adapter must receive the same non-reused session incarnation used by authority validation; an aggregate-only nonce is insufficient.
- Known remote identities from partial creation, duplicate output, or unproven destruction must be retained as recovery evidence without becoming command authority.
- Ownership recovery and transport liveness must remain orthogonal.
- Duplicate or uncertain outcomes fail closed and must not permit false normal completion.
- Destruction I/O must use the exact stored handle and session incarnation rather than reconstructing authority from raw identifiers.
- Browser Session remains the domain authority; WebDriver BiDi, CDP, MCP, and LLMs remain adapters or consumers.

## Decision

Introduce `originweave-browser-session` as an independent Rust bounded context and retain ADR status `Proposed` until protected-main and real-browser acceptance exist.

1. `BrowserSession` is the aggregate root. `BrowserSession::start` allocates a process-local, monotonically non-reused `BrowserSessionIncarnation` before browser I/O. Allocation fails closed before `u64` wrap.
2. Presentation authority is intentionally non-serializable. A process restart destroys every outstanding in-memory authority. Within one process, `BrowserSessionIncarnation` prevents sequential ABA when a later aggregate reuses the same external session, isolation, context, and local epoch values.
3. The same `BrowserSessionIncarnation` is passed through `DisposableContextPort` create and destroy calls. Adapters must scope their remote ownership mapping to that incarnation. Ignoring it violates the port contract.
4. A context enters the owned set only after `DisposableContextPort::create_disposable_context` returns a `DisposableContextHandle`. Raw `BrowsingContextId` input never creates ownership.
5. `PresentationMutationAuthority` is opaque and binds browser session, Browser Session incarnation, disposable isolation, browsing context, and context epoch. All fields must match current aggregate ownership before adapter I/O.
6. `DisposableContextCreateError::CreateFailedClean` is valid only when no remote boundary exists. `DisposableContextCreateError::CreateFailedUncertain(Option<DisposableIsolationId>)` enters `RecoveryRequired`; when the browser-issued isolation/user-context identity is known, it is preserved exactly.
7. Duplicate browsing-context or isolation output enters `RecoveryRequired` and stores the complete offending `DisposableContextHandle` as recovery evidence. OriginWeave does not auto-destroy it because the adapter may have returned foreign state.
8. `BrowserSessionRecoveryEvidence` records only reconciliation evidence: `PartialCreationIsolation`, `DuplicateAdapterHandle`, and `UnprovenDestruction`. It grants no browser command authority.
9. Destruction validates exact authority before I/O, passes the current incarnation and stored handle to the port, and succeeds only after the adapter proves the exact boundary is gone. `DisposableContextDestroyError` moves the record and aggregate into recovery and retains the exact failed handle.
10. Transport liveness is stored separately from ownership state. The first `record_transport_loss()` records the fact even after `RecoveryRequired`; later duplicate reports are idempotent. If transport is lost while the aggregate is `Active`, the lifecycle state becomes `TransportLost` and active contexts become uncertain. If ownership was already uncertain, `RecoveryRequired` remains the lifecycle state and the transport-loss fact is retained alongside it.
11. `RecoveryRequired`, `TransportLost`, and `Ended` reject active-only creation, authority issuance/advance, destruction, and normal end. Reconciliation is a later, separately authorized design.
12. Context epochs remain monotonic authority identities within one aggregate. They invalidate older authority after navigation or another lifecycle boundary but are not a substitute for session incarnation.

## Alternatives considered

### Treat any known context as owned

Rejected. It restores the authority-confusion defect and allows one task to clear another task's state.

### Depend only on browser-issued isolation identity

Rejected. The WebDriver BiDi user-context identifier is suitable lifecycle addressability, but this ADR does not assume a historical non-reuse guarantee after removal. A later aggregate therefore needs a separate OriginWeave lifecycle generation.

### Add an aggregate-only random or monotonic nonce

Rejected if it does not reach the lifecycle adapter. It would stop one aggregate from accepting another aggregate's token while still allowing a valid current token to address a remote boundary through aliasable adapter keys. The selected `BrowserSessionIncarnation` participates in both authority validation and port calls.

### Persist authority generations globally

Deferred and unnecessary for the current in-process authority model. Presentation authority is not durable across process restart; recovery across restart belongs to evidence/reconciliation design, not silent authority resurrection.

### Treat every uncertain lifecycle failure as transport loss

Rejected. Ownership uncertainty and transport liveness answer different operational questions. Collapsing them loses information needed for safe reconciliation.

### Automatically clean duplicate or partial state

Rejected. When ownership is ambiguous, cleanup itself can become a cross-owner destructive action. Exact recovery evidence is retained while normal authority stays blocked.

### Snapshot and restore every predecessor presentation override

Deferred. OriginWeave does not yet have a complete queryable predecessor-state contract for every governed presentation surface. Disposable ownership remains the stronger first implementation.

## Consequences

The Browser Session aggregate now carries an explicit lifecycle generation through the anti-corruption boundary instead of treating protocol identifiers as durable capabilities. A retained token from aggregate A cannot validate against aggregate B solely because the browser or adapter later reused the same external identifiers and local epoch.

Recovery is also diagnosable rather than merely terminal. Known partial user-context identities, duplicate returned handles, and exact handles whose destruction could not be proven remain available as `BrowserSessionRecoveryEvidence`. This evidence is purpose-bound to later reconciliation; it is not a cleanup credential.

Transport failure can now be observed after ownership has already become uncertain without replacing or erasing that uncertainty. This supports later recovery planning that distinguishes “ownership uncertain but transport still live” from “ownership uncertain and transport lost.”

The selected process-local incarnation has a deliberate scope. It prevents ABA only for outstanding in-memory authority within the running process. Durable restart reconciliation must use separately persisted evidence and browser observation; this ADR does not serialize or resurrect authority across restart.

## Security and governance impact

No page-controlled value, raw browser-session id, raw browsing-context id, user-context string, provider/model decision, or LLM output can mint presentation authority. The adapter receives domain-issued incarnation information only as a lifecycle-scoping input and cannot manufacture Browser Session policy authority.

Unknown or duplicate remote state is quarantined rather than destroyed speculatively. This reduces the risk that recovery logic removes another owner's user context. It does not replace Chromium sandboxing, egress policy, Keyverse secret handling, Wardnet controls, or central workflow security.

## Tests and exact evidence

The test suite covers raw-context rejection, bounded isolation identity parsing, typed clean/uncertain creation, retained partial identity, duplicate-handle evidence, epoch exhaustion, stale epoch rejection, foreign-session/isolation rejection, destruction failure, transport loss, normal end, and incarnation-allocation exhaustion.

A dedicated hostile test, `stale_authority_cannot_cross_sequential_session_incarnations`, creates aggregate A, destroys and ends it, creates aggregate B with the same external session/user-context/browsing-context values and local epoch, and requires A's retained authority to fail before B adapter I/O while B's current authority succeeds. The port records incarnation values so the test also proves that the lifecycle mapping receives the new generation.

`destroy_failure_requires_recovery_before_any_new_authority` requires an unproven destruction to retain the exact failed handle, enter `RecoveryRequired`, then record a later real transport loss without erasing ownership evidence; repeated loss reports are idempotent.

The RED for the sequential ABA defect was captured on exact `ec145963ad8fe19c9416f2b3856b94660082dbf7` in CI `34469580144`: repository contracts and formatting passed, and Rust `Run tests` failed at the new hostile test before Clippy/rustdoc. The production fix and subsequent documentation/test updates must earn a new exact-head GREEN; predecessor evidence does not transfer.

Repository contracts, canonical formatting, locked Rust tests, strict Clippy, rustdoc/API docs, exact function/line/region/branch coverage, independent review, and applicable central checks remain required before ordinary adoption into #313.

## Buyer acceptance still open

This slice does not yet prove real WebDriver BiDi `browser.createUserContext`/`browsingContext.create`/`browser.removeUserContext` integration, browser-observed destruction, recovery reconciliation, Browser Session→BiDi private-witness conversion, pinned Chromium presentation post-conditions, crash/restart cleanup, #299 3/3 Agent Task replay, or protected-main release/SBOM/provenance/reproducibility/rollback.

## Migration and rollback

The change remains additive on the active stacked branch. Consumers must adopt the new `BrowserSession::start` result and incarnation-aware `DisposableContextPort` contract. Until a reviewed adapter bridge exists, presentation mutation remains fail closed behind private ownership witnesses. Rollback removes this active-PR bounded-context slice without weakening protected Chromium or central security policy.

## Open follow-ups

- Implement the WebDriver BiDi disposable-user-context adapter with incarnation-scoped mapping and observed destruction post-condition.
- Define the Browser Session→BiDi ACL without exposing public ownership constructors.
- Design separately authorized reconciliation for `BrowserSessionRecoveryEvidence`, including browser/process restart.
- Replay #299 historical pinned Chromium evidence after the canonical sandbox/runtime repair, then run a separate current-Stable qualification.
- Revisit predecessor capture/restore only if reusable attached contexts become a buyer requirement.

## Supersession / reversal conditions

Supersede this ADR if the browser platform provides a complete, queryable, generation-safe ownership primitive with exact destruction evidence, or if OriginWeave adopts another isolation primitive with equivalent guarantees. Do not regress to raw context identity as authority.

## References

Browser Testing and Tools Working Group. (2026, August 18). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/
