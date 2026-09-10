# ADR 0114: Browser Session disposable-context authority

- Status: Proposed
- Date: 2026-09-10

## Context

OriginWeave's WebDriver BiDi presentation adapter requires opaque ownership witnesses before it can plan viewport/device-pixel-ratio, timezone, or screen-area mutation. That closes an adapter-level gap: a caller that merely knows a browsing-context identifier cannot overwrite another owner's presentation state and later clear it to an implementation default.

The remaining gap is upstream of the adapter. A production Browser Session must establish why a context is exclusively OriginWeave-owned before any presentation-mutation authority can be issued. The first Browser Session implementation bound authority to `(BrowserSessionId, BrowsingContextId, local epoch)`, but those values can be reused by separate aggregate incarnations. Two aggregates that receive the same external session/context identifiers and both start at epoch 1 can therefore alias unless the disposable lifecycle carries a separate non-aliasing isolation identity through mutation validation and destruction I/O.

The 9 September 2026 WebDriver BiDi Working Draft provides a standards-aligned isolation identity. A user context has a user-context id defined as a unique string set when the user context is created. `browser.createUserContext` creates a new user context, `browsingContext.create` can create a browsing context inside it, and `browser.removeUserContext` removes that user context after closing its navigables. These protocol operations are adapter capabilities; they do not themselves define OriginWeave policy authority, and a command acknowledgement alone is not cleanup proof.

## Decision drivers

- Remote-issued browser-session and browsing-context identifiers are addressability, not mutation authority.
- Reuse of external session/context identifiers across aggregate incarnations must not create authority aliasing.
- Shared or attached human contexts must never acquire disposable-owner semantics by implication.
- Presentation reset must not destroy a predecessor override owned by another task/session.
- Destruction I/O must be scoped by the exact disposable isolation boundary, not reconstructed from aliasable session/context identifiers.
- Navigation, renderer replacement, crash, cleanup failure, and transport loss must invalidate stale authority.
- The Browser Session domain must remain independent of WebDriver BiDi, CDP, MCP, and LLM policy decisions.
- An adapter acknowledgement is not a successful cleanup post-condition.

## Assumptions and authority boundaries

`originweave-browser-session` owns Browser Session lifecycle state, owned-context membership, monotonic context epochs, validated disposable-isolation identity, and opaque presentation-mutation authority. It consumes validated `BrowserSessionId` and `BrowsingContextId` values from `originweave-core`.

A narrow `DisposableContextPort` is the anti-corruption boundary to a future browser adapter. The port must return a `DisposableContextHandle` containing the browsing-context address and a live-lifetime non-aliasing `DisposableIsolationId`. For WebDriver BiDi, the adapter proof obligation is a one-to-one mapping from that isolation id to the specification-defined unique user-context id returned by fresh user-context creation. The same handle must scope destruction; reconstructing cleanup authority from `(BrowserSessionId, BrowsingContextId)` is forbidden.

`DisposableIsolationId` is addressability and lifecycle identity, not policy or presentation authority. Callers can validate an identifier value, but they cannot mint `PresentationMutationAuthority`; only the Browser Session aggregate can bind a port-created isolation boundary to a context epoch and issue the opaque authority token.

The implementation deliberately does not convert `PresentationMutationAuthority` into the WebDriver BiDi crate's private presentation/screen-area witnesses. That bridge belongs to a later integration slice after both sides' contracts are reviewed. It also does not claim real-Chromium cleanup evidence.

## Options considered

### A. Treat any known browsing context as owned

Rejected. It recreates the original authority-confusion defect and allows one task to erase another task's predecessor state.

### B. Add only an aggregate-local incarnation or epoch

Rejected as insufficient. An incarnation field can prevent one aggregate from accepting another aggregate's token, but if adapter destruction is still addressed only by reused session/context identifiers, a valid token from aggregate B can still cause the adapter to destroy aggregate A's boundary. The non-aliasing identity therefore has to reach the port boundary itself.

### C. Snapshot every predecessor presentation override and restore it exactly

Deferred. Exact predecessor capture can support reusable/attached contexts later, but today OriginWeave does not have a complete standard protocol snapshot for every governed presentation surface. Partial restoration would be a false safety claim.

### D. Own a disposable isolation lifecycle and issue opaque authority only after creation

Selected. The Browser Session records a port-proved non-aliasing isolation identity together with its browsing context and epoch. A WebDriver BiDi adapter should map that identity one-to-one to a fresh user context and remove that exact user context during cleanup. This keeps raw driver identifiers as addresses while carrying lifecycle ownership to the destruction boundary.

## Decision

Introduce `originweave-browser-session` as an independent Rust bounded context with these invariants:

1. `BrowserSession` is the aggregate root. It begins `Active` and may end normally only after every owned disposable context has proven destruction.
2. A context enters the aggregate's owned set only after `DisposableContextPort::create_disposable_context` succeeds with a `DisposableContextHandle`. Supplying a raw `BrowsingContextId` never creates ownership.
3. The handle contains both the browsing-context address and a `DisposableIsolationId` that the adapter contract requires to be non-aliasing for the live lifetime of the isolation boundary. A WebDriver BiDi adapter maps it one-to-one to the unique user-context id.
4. Successful owned-context creation mints a non-caller-constructible `PresentationMutationAuthority` bound to the exact browser session transport identity, disposable isolation identity, browsing context, and context epoch.
5. Two aggregates may reuse the same external `BrowserSessionId`, `BrowsingContextId`, and local epoch without sharing authority when their disposable isolation identities differ. Foreign isolation authority is rejected before adapter I/O.
6. Advancing the context epoch invalidates previously issued authority. Adapter integration must use this transition at navigation/renderer lifecycle boundaries that invalidate the prior authority scope.
7. Destruction requires exact current authority and passes the stored `DisposableContextHandle` back to the port. A stale, foreign-session, foreign-isolation, unknown, already-destroyed, or uncertain context fails closed before destruction I/O.
8. If destruction cannot be proved, the context becomes `Uncertain` and its authority is invalidated. Normal session end is prohibited.
9. Transport loss moves the Browser Session to `TransportLost`, marks still-active owned contexts uncertain, and prevents further authority issuance.
10. Epoch sequence numbers are monotonic authority identities, not business counters; gaps are allowed after failed creation or rejected duplicate adapter output.

## Consequences

Browser Session ownership becomes a domain fact carried through the adapter lifecycle instead of a convention reconstructed from transport identifiers. This gives the future BiDi/Chromium bridge a legitimate place to mint presentation witnesses without making raw driver identifiers authoritative.

The Browser Session domain relies on an explicit adapter proof obligation for global live-lifetime non-aliasing of `DisposableIsolationId`. For WebDriver BiDi that proof is the standard's unique user-context identifier plus adapter conformance tests that preserve the mapping and remove the exact user context. A generic random adapter token without a verified one-to-one browser lifecycle mapping is not sufficient.

The slice remains incomplete for buyer acceptance. No real Chromium user-context adapter, presentation-witness bridge, observed cleanup receipt, crash-recovery reconciliation, or #299 full browser replay is claimed here.

## Failure and degraded behavior

Creation failure produces no authority. Duplicate browsing-context or disposable-isolation identities returned inside one aggregate are rejected and are not automatically destroyed, because a port that violates the fresh-boundary contract may have returned another owner's state. Cross-aggregate aliasing is prevented by requiring authority and destruction to carry the distinct isolation identity. Destruction failure and transport loss quarantine the affected lifecycle rather than assuming cleanup. Once a Browser Session is `Ended` or `TransportLost`, creation, authority lookup, destruction, and normal end transitions that require an active session fail closed.

## Security / privacy / governance impact

Disposable context ownership reduces cross-task presentation-state interference and is compatible with isolated Agent Task profiles. It is not a substitute for Chromium sandboxing, egress policy, origin capability policy, Keyverse secret handling, or evidence retention controls. Those remain with their canonical owners.

No page-controlled value, secret, provider/model choice, LLM result, raw browser-session id, or raw browsing-context id can mint Browser Session presentation authority.

## Tests and acceptance evidence

The owning crate tests hostile raw-context lookup, creation failure, duplicate adapter output, epoch exhaustion, stale authority, cross-session authority, cleanup failure, transport loss, unknown context, epoch advancement, successful destroy-before-end behavior, and a two-aggregate alias case. In the hostile alias case, both aggregates deliberately reuse the same external session and browsing-context identifiers at the same local epoch but receive distinct disposable isolation identities; aggregate B must reject aggregate A's authority before adapter I/O, while B's own authority destroys only B's isolation handle.

Repository contracts require the bounded context to be a workspace member, keep ADR 0114 indexed, and preserve the non-aliasing port contract. Exact-head CI, Clippy, rustdoc, function/line/region/branch coverage and independent review remain required before integration. Real-browser acceptance is deferred to a later adapter slice and must prove unique user-context creation, page-observed mutation, exact-boundary cleanup/destruction and post-cleanup isolation in pinned Chromium; command ACK alone is not success.

## Migration and rollback

This is additive. Until a reviewed adapter bridge consumes the new authority, existing presentation code remains fail closed behind its private ownership witnesses. Rollback removes the new crate, workspace/lockfile entries, tests and Proposed ADR without changing protected Chromium or central workflow policy.

## Open follow-ups

- Implement the WebDriver BiDi disposable-user-context adapter using the runtime-qualified protocol contract and prove the one-to-one `DisposableIsolationId` mapping.
- Define the narrow conversion/ACL from `PresentationMutationAuthority` to BiDi presentation/screen-area ownership witnesses without exposing public constructors.
- Specify observed user-context destruction/reconciliation after browser crash or transport loss.
- Replay #299 with three complete real-Chromium trials after the canonical sandbox/runtime owner path is usable.
- Evaluate exact predecessor capture/restore only if attached/reusable contexts become a buyer requirement.

## Supersession / reversal conditions

Supersede this ADR if WebDriver/Chromium gains a complete, queryable and exactly restorable predecessor-state contract for all governed presentation surfaces, or if OriginWeave adopts another isolation primitive with equivalent non-aliasing ownership and destruction evidence. Do not replace disposable ownership with raw context identity.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
