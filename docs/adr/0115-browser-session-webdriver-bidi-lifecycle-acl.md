# ADR 0115: Browser Session to WebDriver BiDi lifecycle ACL

- Status: Proposed
- Date: 2026-09-10

## Context

OriginWeave must translate Browser Session-owned presentation authority into WebDriver BiDi addressability without allowing raw protocol identifiers to become authority. ADR 0114 establishes disposable-context ownership, session incarnation, context epochs, recovery evidence, and transport-liveness semantics in the Browser Session bounded context. The WebDriver BiDi adapter still needs a separate anti-corruption boundary that binds those domain identities to the browser-issued browsing-context string used by presentation commands.

An exact domain key alone is insufficient. A backend can return two apparently different lifecycle handles whose isolation and OriginWeave browsing-context identities differ while their opaque WebDriver BiDi browsing-context string aliases the same live remote target. If both results are accepted, Browser Session can mint two valid authorities that later project to one browser context. That is cross-owner authority confusion even though neither domain key collides.

The 9 September 2026 WebDriver BiDi Working Draft defines `browser.createUserContext`, `browsingContext.create`, and `browser.removeUserContext`. User-context and browsing-context identifiers provide protocol addressability. They do not replace OriginWeave's policy and lifecycle authority, and a successful command response does not by itself prove the browser-side post-condition required for buyer evidence.

## Decision drivers

- Browser Session remains the authority owner; WebDriver BiDi remains an adapter.
- Raw browser-session, user-context, domain-context, or remote BiDi identifiers must not mint presentation authority.
- A retained authority must be revalidated against the current Browser Session immediately before remote-target projection.
- One live opaque remote browsing-context string must not be bound to multiple independently owned lifecycle handles.
- Sequential external identifier reuse must not revive stale authority across Browser Session incarnations.
- Planning must remain distinct from browser command acknowledgement and observed post-condition evidence.
- Ambiguous remote state must fail closed without speculative cleanup of potentially foreign browser state.

## Assumptions and authority boundaries

ADR 0114's `BrowserSession`, `BrowserSessionIncarnation`, `DisposableContextHandle`, context epoch, and `PresentationMutationAuthority` are the canonical domain inputs. `WebDriverBidiLifecycleAdapter` owns a private anti-corruption mapping from that lifecycle identity to `WebDriverBidiBrowsingContext`. The mapping is addressability state, not authority state.

The reviewed lifecycle backend may obtain browser-issued user-context and browsing-context identifiers by WebDriver BiDi. It may allocate the OriginWeave domain `BrowsingContextId` required by the port contract. It cannot manufacture Browser Session authority, weaken epoch or incarnation checks, or accept caller-supplied remote identifiers as substitutes for the stored mapping.

Screen-area mutation remains outside the standard reusable plan because its complete presentation-surface ownership contract is not yet established. This ADR does not broaden that capability.

## Options considered

### Reconstruct the remote context from the OriginWeave browsing-context id

Rejected. Domain identity and protocol addressability have different semantics, and numeric/string coercion would create an implicit cross-boundary alias.

### Let callers supply the remote BiDi context during authorization

Rejected. A caller that possesses raw protocol addressability would be able to redirect an otherwise valid Browser Session authority to another target.

### Key the adapter only by Browser Session lifecycle identity

Rejected as incomplete. Exact-key uniqueness does not prevent two distinct keys from aliasing the same live remote browsing-context string.

### Treat WebDriver BiDi identifiers as durable capabilities

Rejected. The protocol identifiers are addressability. OriginWeave authority is generated and revalidated by Browser Session lifecycle state, incarnation, isolation, and epoch.

### Automatically destroy a duplicate or ambiguous remote result

Rejected. When ownership is ambiguous, cleanup itself can become a cross-owner destructive action. The adapter must quarantine/fail closed and preserve recovery evidence instead of guessing ownership.

## Decision

1. `WebDriverBidiLifecycleAdapter` owns the private mapping from the exact Browser Session lifecycle key to the opaque `WebDriverBidiBrowsingContext` returned by its reviewed backend.
2. The mapping key binds browser session, `BrowserSessionIncarnation`, disposable isolation identity, and OriginWeave `BrowsingContextId`. No public constructor exposes an equivalent command capability.
3. Before accepting a newly created lifecycle result, the adapter rejects any result whose opaque remote browsing-context string is already bound to another live lifecycle key. The existing binding is not replaced. The Browser Session receives an uncertain creation result and enters its fail-closed recovery path before a second authority can be minted.
4. Exact lifecycle-key reuse is also rejected before replacement. Domain-key collision and remote-target collision are independent checks.
5. `authorize_standard_presentation` asks the live `BrowserSession` for the current authority for the requested domain context and requires exact equality with the retained token before consulting the private remote mapping. Stale epoch, destruction, transport loss, recovery state, session end, foreign isolation, foreign session, or foreign incarnation therefore fails before a remote target is returned.
6. The authorized presentation plan borrows both the Browser Session and lifecycle adapter. While it is alive, safe Rust cannot mutably advance, destroy, lose, or end that session or replace the adapter mapping. The plan and its actions have private construction paths.
7. The standard plan may express only the already-admitted viewport/device-pixel-ratio and timezone apply/reset operations. It does not grant screen-area mutation or any unrelated BiDi command authority.
8. Plan creation proves policy/lifecycle admission only. Transport command acknowledgement, page-observed state, cleanup, and destruction post-conditions require separate runtime evidence.
9. Ambiguous post-create results must not be speculatively destroyed. The current Browser Session recovery contract can preserve a known isolation identity, but complete BiDi-specific recovery must additionally retain the domain browsing-context and opaque remote browsing-context identities when they are known. That adapter-specific recovery evidence is a follow-up and grants no command authority.

## Consequences

A valid Browser Session authority can no longer be redirected by supplying or reconstructing a remote BiDi context. A second lifecycle result that aliases an already-live remote target is quarantined before Browser Session can mint another authority, even when its user-context and domain-context identities are distinct.

The adapter now has a stronger uniqueness invariant than its map key alone expresses: live remote browsing-context identity is unique across accepted bindings. This check is intentionally local to the BiDi anti-corruption boundary because Browser Session must not depend on protocol-specific strings.

The lifetime-bound plan reduces time-of-check/time-of-use drift inside safe Rust, but it does not prove external browser state. A transport or browser crash after planning remains an execution/recovery concern.

The current uncertain-create interface loses part of a complete BiDi-created tuple when the adapter rejects an alias after the backend has returned it. Until adapter-specific quarantine evidence retains the known isolation, domain context, and remote context together, recovery diagnostics are incomplete. This is an explicit open gap rather than a reason to relax the fail-closed behavior.

## Failure and degraded behavior

Missing lifecycle mappings, stale or foreign authority, duplicate lifecycle keys, and duplicate live remote targets fail closed before presentation planning. Creation ambiguity moves Browser Session into `RecoveryRequired`; no new normal authority is issued. A failed remote destruction remains unproven and blocks further authorization according to ADR 0114.

If the adapter has an ambiguous browser-created tuple that cannot yet be represented completely in recovery evidence, it must retain fail-closed product behavior and must not auto-destroy the remote state. Reconciliation requires a separately authorized design.

## Security / privacy / governance impact

This ACL prevents protocol-addressability confusion from crossing the Browser Session authority boundary. Page content, LLM output, MCP input, extension input, or raw BiDi identifiers cannot construct the plan or choose its remote target.

The change does not replace Chromium sandboxing, EgressWeave network policy, Keyverse identity/secrets, Wardnet controls, contextual-orchestrator model governance, or central repository security gates. It introduces no cross-service SQL, mutable external dependency, provider/model routing, or new secret surface.

## Tests and acceptance evidence

The ACL tests cover exact lifecycle mapping, raw/unrelated mapping rejection, stale epoch, destruction, transport loss, session end, clean creation failure, exact lifecycle-key reuse, failed destruction, and lifetime-bound planning.

A hostile test, `duplicate_remote_context_is_rejected_before_second_authority_is_minted`, creates one accepted lifecycle result and then returns a second result with distinct isolation and OriginWeave browsing-context identities but the same opaque remote BiDi browsing-context string. On exact commit `215338b77ddb3d828ebc95b796f5e8080000afde`, CI run `34487233621` failed because the second creation incorrectly returned a valid `PresentationMutationAuthority`, establishing the RED. The minimal causal fix rejects the alias before insertion and authority minting while leaving the existing binding intact.

Repository contracts, canonical formatting, locked Rust tests, strict Clippy, rustdoc/API docs, and exact production function/line/region/branch coverage remain mandatory on the final exact head. Predecessor GREEN does not transfer after source or documentation changes. Independent review and applicable central checks remain required before ordinary adoption.

Real-browser acceptance remains separate: a production backend must prove `browser.createUserContext`/`browsingContext.create` creation, exact `browser.removeUserContext` destruction or equivalent observed absence, presentation application, page-observed post-conditions, cleanup, crash/restart behavior, and the applicable pinned/current Chromium qualification.

## Migration and rollback

This is an additive stacked-branch ACL over the Browser Session lifecycle foundation. Consumers should obtain remote context only through the lifecycle adapter and should not persist or reconstruct the authorized plan. No protected-main migration or durable database schema is introduced.

Rollback removes this active feature slice while leaving ADR 0114's Browser Session authority fail closed. It must not restore caller-supplied remote contexts, raw-id coercion, or duplicate-target acceptance.

## Open follow-ups

- Add adapter-specific quarantine/recovery evidence that losslessly preserves every known `WebDriverBidiCreatedContext` identity after uncertain post-create outcomes without turning that evidence into cleanup authority.
- Reject or quarantine all other partial identity aliases that can leave an adapter mapping for a handle Browser Session did not adopt, including isolation-only or domain-context-only reuse.
- Implement the real WebDriver BiDi disposable-user-context backend and prove browser-observed destruction rather than command ACK.
- Integrate authorized presentation plans with the pinned Chromium Agent Task lane and page-observed evidence.
- Reconcile the parent Browser Session allocator deprecation (`AtomicU64::fetch_update` renamed to `try_update`) in its canonical owner without mixing that maintenance change into this ACL authority slice.

## Supersession / reversal conditions

Supersede this ADR if the browser protocol or a later OriginWeave adapter provides a stronger generation-safe lifecycle primitive that directly proves unique remote ownership and observed destruction while preserving Browser Session as policy authority. Do not regress to raw protocol identity as authority.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
