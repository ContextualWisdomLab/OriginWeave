# ADR 0114: Browser Session disposable-context authority

- Status: Proposed
- Date: 2026-09-10

## Context

OriginWeave's WebDriver BiDi presentation adapter now requires opaque ownership witnesses before it can plan viewport/device-pixel-ratio, timezone, or screen-area mutation. That closes a dangerous adapter-level gap: a caller that merely knows a browsing-context identifier cannot overwrite another owner's presentation state and later clear it to an implementation default.

The remaining gap is upstream of the adapter. A production Browser Session must establish why a context is exclusively OriginWeave-owned before any presentation-mutation authority can be issued. Without that lifecycle, a hidden constructor or driver shortcut would simply reintroduce ambient authority under a different type name.

The 9 September 2026 WebDriver BiDi Working Draft provides a suitable standards-aligned isolation mechanism. `browser.createUserContext` creates a new user context. `browsingContext.create` can create a browsing context inside a selected user context. `browser.removeUserContext` closes that user context and every navigable in it without running `beforeunload` handlers. These protocol operations are adapter capabilities; they do not themselves define OriginWeave's domain ownership or prove cleanup merely because a command was acknowledged.

## Decision drivers

- A remote-issued browsing-context identifier is addressability, not mutation authority.
- Shared or attached human contexts must never acquire disposable-owner semantics by implication.
- Presentation reset must not destroy a predecessor override owned by another task/session.
- Navigation, renderer replacement, crash, cleanup failure, and transport loss must invalidate stale authority.
- The Browser Session domain must remain independent of WebDriver BiDi, CDP, MCP, and LLM policy decisions.
- An adapter acknowledgement is not a successful cleanup post-condition.

## Assumptions and authority boundaries

`originweave-browser-session` owns Browser Session lifecycle state, owned-context membership, monotonic context epochs, and opaque presentation-mutation authority. It consumes validated `BrowserSessionId` and `BrowsingContextId` values from `originweave-core`.

A narrow `DisposableContextPort` is the anti-corruption boundary to a future browser adapter. The port may be implemented with WebDriver BiDi user contexts, a separately reviewed Chromium path, or another released adapter, but the adapter does not become the policy or ownership authority.

The first implementation deliberately does not convert `PresentationMutationAuthority` into the WebDriver BiDi crate's private presentation/screen-area witnesses. That bridge belongs to a later integration slice after both sides' contracts are reviewed. It also does not claim real-Chromium cleanup evidence.

## Options considered

### A. Treat any known browsing context as owned

Rejected. It recreates the original authority-confusion defect and allows one task to erase another task's predecessor state.

### B. Snapshot every predecessor presentation override and restore it exactly

Deferred. Exact predecessor capture can support reusable/attached contexts later, but today OriginWeave does not have a complete standard protocol snapshot for every governed presentation surface. Partial restoration would be a false safety claim.

### C. Own a disposable isolated context lifecycle and issue opaque authority only after creation

Selected for the first production slice. Isolation gives the aggregate a tractable ownership invariant and a clear terminal action: destruction of the task-owned context boundary. A future WebDriver BiDi adapter should normally map this to a fresh user context plus a browsing context created inside it, then remove the user context during cleanup.

## Decision

Introduce `originweave-browser-session` as an independent Rust bounded context with these invariants:

1. `BrowserSession` is the aggregate root. It begins `Active` and may end normally only after every owned disposable context has proven destruction.
2. A context enters the aggregate's owned set only after `DisposableContextPort::create_disposable_context` succeeds. Supplying a raw `BrowsingContextId` never creates ownership.
3. Successful owned-context creation mints a non-caller-constructible `PresentationMutationAuthority` bound to the exact browser session, browsing context, and context epoch.
4. Advancing the context epoch invalidates previously issued authority. The adapter integration must use this transition at navigation/renderer lifecycle boundaries that invalidate the prior authority scope.
5. Destruction requires exact current authority. A stale, foreign-session, unknown, already-destroyed, or uncertain context fails closed.
6. If destruction cannot be proved, the context becomes `Uncertain` and its authority is invalidated. Normal session end is prohibited.
7. Transport loss moves the Browser Session to `TransportLost`, marks still-active owned contexts uncertain, and prevents further authority issuance.
8. Epoch sequence numbers are monotonic authority identities, not business counters; gaps are allowed after failed creation or rejected duplicate adapter output.

## Consequences

Browser Session ownership becomes a domain fact rather than an adapter convention. This gives the future BiDi/Chromium bridge a legitimate place to mint presentation witnesses without making raw driver identifiers authoritative.

The first slice remains intentionally incomplete for buyer acceptance. No real Chromium user-context adapter, presentation-witness bridge, observed cleanup receipt, crash-recovery reconciliation, or #299 full browser replay is claimed here.

## Failure and degraded behavior

Creation failure produces no authority. A duplicate context returned by a supposedly fresh-context adapter is rejected and is not automatically destroyed, because destroying that identifier could target another owner's context. Destruction failure and transport loss quarantine the affected lifecycle rather than assuming cleanup. Once a Browser Session is `Ended` or `TransportLost`, creation, authority lookup, destruction, and normal end transitions that require an active session fail closed.

## Security / privacy / governance impact

Disposable context ownership reduces cross-task presentation-state interference and is compatible with isolated Agent Task profiles. It is not a substitute for Chromium sandboxing, egress policy, origin capability policy, Keyverse secret handling, or evidence retention controls. Those remain with their canonical owners.

No page-controlled value, secret, provider/model choice, or LLM result can mint Browser Session authority.

## Tests and acceptance evidence

The owning crate tests hostile raw-context lookup, creation failure, duplicate adapter output, epoch exhaustion, stale authority, cross-session authority, cleanup failure, transport loss, unknown context, epoch advancement, and successful destroy-before-end behavior. Repository contracts require the bounded context to be a workspace member.

Exact-head CI, Clippy, rustdoc, function/line/region/branch coverage and independent review remain required before integration. Real-browser acceptance is deferred to a later adapter slice and must prove creation, page-observed mutation, cleanup/destruction and post-cleanup isolation in pinned Chromium; command ACK alone is not success.

## Migration and rollback

This is additive. Until a reviewed adapter bridge consumes the new authority, existing presentation code remains fail closed behind its private ownership witnesses. Rollback removes the new crate, workspace/lockfile entries, tests and Proposed ADR without changing protected Chromium or central workflow policy.

## Open follow-ups

- Implement the WebDriver BiDi disposable-user-context adapter using the runtime-qualified protocol contract.
- Define the narrow conversion/ACL from `PresentationMutationAuthority` to BiDi presentation/screen-area ownership witnesses without exposing public constructors.
- Specify observed destruction/reconciliation after browser crash or transport loss.
- Replay #299 with three complete real-Chromium trials after the canonical sandbox/runtime owner path is usable.
- Evaluate exact predecessor capture/restore only if attached/reusable contexts become a buyer requirement.

## Supersession / reversal conditions

Supersede this ADR if WebDriver/Chromium gains a complete, queryable and exactly restorable predecessor-state contract for all governed presentation surfaces, or if OriginWeave adopts another isolation primitive with equivalent ownership and destruction evidence. Do not replace disposable ownership with raw context identity.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
