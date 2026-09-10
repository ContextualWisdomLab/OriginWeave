# ADR 0114: Browser Session disposable-context authority

- Status: Proposed
- Date: 2026-09-10

## Context

OriginWeave's WebDriver BiDi presentation adapter requires opaque ownership witnesses before it can plan viewport/device-pixel-ratio, timezone, or screen-area mutation. A caller that merely knows a browsing-context identifier therefore cannot overwrite another owner's presentation state and later clear it to an implementation default.

The Browser Session boundary must also establish why a context is exclusively OriginWeave-owned before any presentation-mutation authority is issued. External browser-session and browsing-context identifiers can be reused across aggregate incarnations, so ownership cannot be reconstructed from `(BrowserSessionId, BrowsingContextId, local epoch)`. The active implementation carries a separate non-aliasing disposable isolation identity through authority validation and destruction I/O.

A second lifecycle gap appears whenever the adapter does not have a proved-clean post-condition. During creation, an adapter can fail after browser state may already have been created, or can return a duplicate context/isolation identity. During destruction, an adapter can fail after the cleanup command has been sent without proving that the exact owned boundary is gone. In either case OriginWeave cannot safely keep the aggregate `Active`: further authority issuance would continue operating beside unresolved browser state. Creation and destruction therefore require explicit clean-versus-uncertain handling and a recovery-required state.

The 9 September 2026 WebDriver BiDi Working Draft provides a standards-aligned isolation identity. A user context has a user-context id defined as a unique string set when the user context is created. `browser.createUserContext` creates a new user context, `browsingContext.create` can create a browsing context inside it, and `browser.removeUserContext` removes that user context after closing its navigables. These protocol operations are adapter capabilities; they do not themselves define OriginWeave policy authority, and a command acknowledgement alone is not cleanup proof.

## Decision drivers

- Remote-issued browser-session and browsing-context identifiers are addressability, not mutation authority.
- Reuse of external session/context identifiers across aggregate incarnations must not create authority aliasing.
- Shared or attached human contexts must never acquire disposable-owner semantics by implication.
- Presentation reset must not destroy a predecessor override owned by another task/session.
- Destruction I/O must be scoped by the exact disposable isolation boundary, not reconstructed from aliasable session/context identifiers.
- Creation failure must distinguish proved-clean failure from an uncertain post-condition.
- Creation-only and destruction-only adapter failures must be different types so phase-invalid outcomes are not representable.
- Duplicate or partial-create outcomes must not permit false normal completion.
- An unproven destroy must quarantine the aggregate before any later context creation or authority issuance.
- Navigation, renderer replacement, crash, cleanup failure, and transport loss must invalidate stale authority.
- The Browser Session domain must remain independent of WebDriver BiDi, CDP, MCP, and LLM policy decisions.
- An adapter acknowledgement is not a successful cleanup post-condition.

## Assumptions and authority boundaries

`originweave-browser-session` owns Browser Session lifecycle state, owned-context membership, monotonic context epochs, validated disposable-isolation identity, recovery-required state, and opaque presentation-mutation authority. It consumes validated `BrowserSessionId` and `BrowsingContextId` values from `originweave-core`.

A narrow `DisposableContextPort` is the anti-corruption boundary to a future browser adapter. The port must return a `DisposableContextHandle` containing the browsing-context address and a live-lifetime non-aliasing `DisposableIsolationId`. For WebDriver BiDi, the adapter proof obligation is a one-to-one mapping from that isolation id to the specification-defined unique user-context id returned by fresh user-context creation. The same handle must scope destruction; reconstructing cleanup authority from `(BrowserSessionId, BrowsingContextId)` is forbidden.

Creation and destruction expose separate failure types. `DisposableContextCreateError::CreateFailedClean` is valid only when the adapter can prove that no disposable browser state was created. `DisposableContextCreateError::CreateFailedUncertain` is required after any partial-create or unknown post-condition. An uncertain outcome moves the aggregate to `RecoveryRequired`, invalidates active owned-context authority, blocks further creation/authority issuance, and prevents normal `end()` until a separate reconciliation design proves what happened remotely. A destruction-only failure is not representable from the creation method.

Destruction returns `DisposableContextDestroyError`. Its failure means destruction could not be proved: the exact record becomes uncertain and the whole aggregate moves to `RecoveryRequired`; every remaining active context becomes uncertain and all active-only transitions fail before further adapter I/O. Creation-only failures are not representable from the destruction method. The current slice intentionally has no implicit retry or reopen transition because doing so would restore authority while remote ownership remains unresolved.

`DisposableIsolationId` is addressability and lifecycle identity, not policy or presentation authority. Callers can validate an identifier value, but they cannot mint `PresentationMutationAuthority`; only the Browser Session aggregate can bind a port-created isolation boundary to a context epoch and issue the opaque authority token.

The implementation deliberately does not convert `PresentationMutationAuthority` into the WebDriver BiDi crate's private presentation/screen-area witnesses. That bridge belongs to a later integration slice after both sides' contracts are reviewed. It also does not claim real-Chromium cleanup evidence.

## Options considered

### A. Treat any known browsing context as owned

Rejected. It recreates the original authority-confusion defect and allows one task to erase another task's predecessor state.

### B. Add only an aggregate-local incarnation or epoch

Rejected as insufficient. An incarnation field can prevent one aggregate from accepting another aggregate's token, but if adapter destruction is still addressed only by reused session/context identifiers, a valid token from aggregate B can still cause the adapter to destroy aggregate A's boundary. The non-aliasing identity therefore has to reach the port boundary itself.

### C. Treat every creation failure as clean

Rejected. A transport or adapter failure after `browser.createUserContext` may leave a remote boundary whose ownership was never recorded. Normal completion after such a failure would produce false cleanup evidence.

### D. Treat every uncertain lifecycle failure as transport loss

Rejected as semantically imprecise. Browser transport may still be healthy while ownership of one create or destroy attempt is unknown. A distinct `RecoveryRequired` state preserves the causal distinction while remaining fail closed.

### E. Keep the aggregate active after an unproven destroy

Rejected. Marking only one record uncertain blocks normal `end()` but still allows new disposable contexts and unrelated authority to be created in an aggregate whose remote cleanup state is unresolved. That compounds uncertainty and weakens the ownership boundary.

### F. Snapshot every predecessor presentation override and restore it exactly

Deferred. Exact predecessor capture can support reusable/attached contexts later, but today OriginWeave does not have a complete standard protocol snapshot for every governed presentation surface. Partial restoration would be a false safety claim.

### G. Own a disposable isolation lifecycle and issue opaque authority only after proved creation

Selected. The Browser Session records a port-proved non-aliasing isolation identity together with its browsing context and epoch. A WebDriver BiDi adapter should map that identity one-to-one to a fresh user context and remove that exact user context during cleanup. Proved-clean create failure may leave the aggregate active; uncertain create failure, duplicate adapter output, or unproven destruction requires recovery. Creation and destruction errors remain method-specific so the ACL cannot express a failure from the wrong lifecycle phase.

## Decision

Introduce `originweave-browser-session` as an independent Rust bounded context with these invariants:

1. `BrowserSession` is the aggregate root. It begins `Active` and may end normally only after every owned disposable context has proven destruction.
2. A context enters the aggregate's owned set only after `DisposableContextPort::create_disposable_context` succeeds with a `DisposableContextHandle`. Supplying a raw `BrowsingContextId` never creates ownership.
3. The handle contains both the browsing-context address and a `DisposableIsolationId` that the adapter contract requires to be non-aliasing for the live lifetime of the isolation boundary. A WebDriver BiDi adapter maps it one-to-one to the unique user-context id.
4. Successful owned-context creation mints a non-caller-constructible `PresentationMutationAuthority` bound to the exact browser session transport identity, disposable isolation identity, browsing context, and context epoch.
5. Two aggregates may reuse the same external `BrowserSessionId`, `BrowsingContextId`, and local epoch without sharing authority when their disposable isolation identities differ. Foreign isolation authority is rejected before adapter I/O.
6. `DisposableContextCreateError::CreateFailedClean` means no remote boundary exists and leaves the aggregate active. `DisposableContextCreateError::CreateFailedUncertain`, duplicate browsing-context output, or duplicate isolation output moves the aggregate to `RecoveryRequired` and invalidates active authority. Destruction-only failures cannot appear on this method boundary.
7. Advancing the context epoch invalidates previously issued authority. Adapter integration must use this transition at navigation/renderer lifecycle boundaries that invalidate the prior authority scope.
8. Destruction requires exact current authority and passes the stored `DisposableContextHandle` back to the port. The aggregate retains that already-validated mutable record across the port call; it does not perform a second impossible lookup after I/O. The method returns only `DisposableContextDestroyError`, so creation-only outcomes cannot cross into cleanup semantics.
9. If destruction cannot be proved, the failed record becomes `Uncertain`, the Browser Session moves to `RecoveryRequired`, every remaining active record becomes uncertain, and further creation, authority lookup/advance, destruction, and normal end are rejected until an explicit reconciliation design exists.
10. Transport loss moves the Browser Session to `TransportLost`, marks still-active owned contexts uncertain, and prevents further authority issuance.
11. `RecoveryRequired`, `TransportLost`, and `Ended` reject all transitions that require an active session. Reconciliation is a later explicit design; none of these states silently reopens ownership.
12. Epoch sequence numbers are monotonic authority identities, not business counters; gaps are allowed after failed creation or rejected duplicate adapter output.

## Consequences

Browser Session ownership becomes a domain fact carried through the adapter lifecycle instead of a convention reconstructed from transport identifiers. Proved-clean and uncertain outcomes are no longer conflated, so normal completion or continued mutation cannot hide a potentially leaked browser boundary. Once cleanup becomes uncertain, the aggregate stops issuing new authority rather than accumulating more browser state beside an unresolved boundary.

Method-specific port errors also remove a class of defensive branches that had no valid domain meaning. An adapter cannot report destruction failure from creation or creation failure from destruction, so the aggregate no longer has to interpret an impossible phase transition as a degraded case.

The Browser Session domain relies on an explicit adapter proof obligation for global live-lifetime non-aliasing of `DisposableIsolationId`. For WebDriver BiDi that proof is the standard's unique user-context identifier plus adapter conformance tests that preserve the mapping and remove the exact user context. A generic random adapter token without a verified one-to-one browser lifecycle mapping is not sufficient.

The slice remains incomplete for buyer acceptance. No real Chromium user-context adapter, presentation-witness bridge, observed cleanup receipt, crash/recovery reconciliation, or #299 full browser replay is claimed here.

## Failure and degraded behavior

`DisposableContextCreateError::CreateFailedClean` produces no authority and permits continued active operation because the adapter has proved that no disposable boundary exists. `DisposableContextCreateError::CreateFailedUncertain`, duplicate adapter output, and any `DisposableContextDestroyError` move the Browser Session to `RecoveryRequired`; existing active records become uncertain and all active-only transitions are blocked. Duplicate output is never automatically destroyed because a contract-violating adapter may have returned another owner's state. A failed destroy preserves the exact failed handle as uncertain evidence; it is not retried implicitly and no later adapter I/O is admitted from that aggregate. Transport loss uses the separate `TransportLost` state. Once a Browser Session is `Ended`, `TransportLost`, or `RecoveryRequired`, creation, authority lookup, destruction, epoch advancement, and normal end transitions that require an active session fail closed.

## Security / privacy / governance impact

Disposable context ownership reduces cross-task presentation-state interference and is compatible with isolated Agent Task profiles. Typed lifecycle outcomes prevent a failed browser command from being misreported as a clean lifecycle or followed by fresh authority while cleanup is unresolved. Method-specific failure types also prevent invalid lifecycle-phase semantics from crossing the Browser Session anti-corruption boundary. This is not a substitute for Chromium sandboxing, egress policy, origin capability policy, Keyverse secret handling, or evidence retention controls. Those remain with their canonical owners.

No page-controlled value, secret, provider/model choice, LLM result, raw browser-session id, or raw browsing-context id can mint Browser Session presentation authority.

## Tests and acceptance evidence

The owning crate tests hostile raw-context lookup, bounded isolation identity parsing and handle accessors, proved-clean versus uncertain creation failure, duplicate adapter output, epoch exhaustion, stale authority, cross-session authority, foreign isolation authority, cleanup failure, transport loss, unknown context, epoch advancement, successful destroy-before-end behavior, and a two-aggregate alias case. Duplicate and uncertain-create branches assert `RecoveryRequired`; the dedicated `destroy_failure_requires_recovery_before_any_new_authority` hostile test requires an unproven destroy to quarantine the whole aggregate and rejects later creation/authority/epoch/end before adapter I/O.

Repository contracts require the bounded context to be a workspace member, keep ADR 0114 indexed, preserve the non-aliasing port contract, retain `RecoveryRequired`, and require distinct `DisposableContextCreateError` and `DisposableContextDestroyError` types. Exact-head CI, Clippy, rustdoc, function/line/region/branch coverage and independent review remain required before integration. Real-browser acceptance is deferred to a later adapter slice and must prove unique user-context creation, page-observed mutation, exact-boundary cleanup/destruction and post-cleanup isolation in pinned Chromium; command ACK alone is not success.

## Migration and rollback

This is additive. Until a reviewed adapter bridge consumes the new authority, existing presentation code remains fail closed behind its private ownership witnesses. Rollback removes the new crate, workspace/lockfile entries, tests and Proposed ADR without changing protected Chromium or central workflow policy.

## Open follow-ups

- Implement the WebDriver BiDi disposable-user-context adapter using the runtime-qualified protocol contract and prove the one-to-one `DisposableIsolationId` mapping.
- Define the narrow conversion/ACL from `PresentationMutationAuthority` to BiDi presentation/screen-area ownership witnesses without exposing public constructors.
- Specify observed user-context destruction/reconciliation after `RecoveryRequired`, browser crash, or transport loss.
- Replay #299 with three complete real-Chromium trials after the canonical sandbox/runtime owner path is usable.
- Evaluate exact predecessor capture/restore only if attached/reusable contexts become a buyer requirement.

## Supersession / reversal conditions

Supersede this ADR if WebDriver/Chromium gains a complete, queryable and exactly restorable predecessor-state contract for all governed presentation surfaces, or if OriginWeave adopts another isolation primitive with equivalent non-aliasing ownership and destruction evidence. Do not replace disposable ownership with raw context identity.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
