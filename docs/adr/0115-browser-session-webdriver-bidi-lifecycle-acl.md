# ADR 0115: Browser Session to WebDriver BiDi lifecycle ACL

- Status: Proposed
- Date: 2026-09-10

## Context

OriginWeave must translate Browser Session-owned presentation authority into WebDriver BiDi addressability without allowing raw protocol identifiers to become authority. ADR 0114 establishes disposable-context ownership, session incarnation, context epochs, recovery evidence, and transport-liveness semantics in the Browser Session bounded context. The WebDriver BiDi adapter still needs a separate anti-corruption boundary that binds those domain identities to the browser-issued browsing-context string used by presentation commands.

An exact domain key alone is insufficient. A backend can return two apparently different lifecycle handles whose isolation and OriginWeave browsing-context identities differ while their opaque WebDriver BiDi browsing-context string aliases the same live remote target. If both results are accepted, Browser Session can mint two valid authorities that later project to one browser context. That is cross-owner authority confusion even though neither domain key collides.

Local Rust lifetime also is not browser-liveness proof. The browser can destroy a navigable or terminate the BiDi session while OriginWeave still holds immutable `BrowserSession` and adapter borrows. A plan that remains type-valid after `browsingContext.contextDestroyed` or transport loss therefore cannot be executable merely because no local aggregate mutation occurred.

The 9 September 2026 WebDriver BiDi Working Draft defines `browser.createUserContext`, `browsingContext.create`, `browser.removeUserContext`, and the `browsingContext.contextDestroyed` event. User-context and browsing-context identifiers provide protocol addressability. They do not replace OriginWeave's policy and lifecycle authority, and a successful command response does not by itself prove the browser-side post-condition required for buyer evidence.

## Decision drivers

- Browser Session remains the authority owner; WebDriver BiDi remains an adapter.
- Raw browser-session, user-context, domain-context, or remote BiDi identifiers must not mint presentation authority or directly authorize lifecycle mutation.
- A retained authority must be revalidated against the current Browser Session immediately before browser mutation, not only when a plan is created.
- The same execution boundary must verify current adapter-observed remote-context and BiDi-session liveness immediately before I/O.
- One live opaque remote browsing-context string must not be bound to multiple independently owned lifecycle handles.
- Sequential external identifier reuse must not revive stale authority across Browser Session incarnations.
- Remote creation must not become an authorizing binding until Browser Session accepts the returned domain handle.
- Standard apply/reset must have one canonical command vocabulary while durable command intent remains distinct from ephemeral mutation authority.
- Planning must remain distinct from browser command acknowledgement and observed post-condition evidence.
- Ambiguous remote state must fail closed without speculative cleanup of potentially foreign browser state.

## Assumptions and authority boundaries

ADR 0114's `BrowserSession`, `BrowserSessionIncarnation`, `DisposableContextHandle`, context epoch, and `PresentationMutationAuthority` are the canonical domain inputs. `WebDriverBidiLifecycleAdapter` owns a private anti-corruption mapping from that lifecycle identity to `WebDriverBidiBrowsingContext`. The mapping is addressability state, not authority state.

The reviewed lifecycle backend may obtain browser-issued user-context and browsing-context identifiers by WebDriver BiDi. It may allocate the OriginWeave domain `BrowsingContextId` required by the port contract. It cannot manufacture Browser Session authority, weaken epoch or incarnation checks, or accept caller-supplied remote identifiers as substitutes for the stored mapping. Browser lifecycle events remain protocol evidence in the BiDi ACL; the Browser Session domain receives only the domain transition required to invalidate or recover ownership/transport state.

Screen-area mutation remains outside the standard reusable plan because its complete presentation-surface ownership contract is not yet established. This ADR does not broaden that capability.

## Options considered

### Reconstruct the remote context from the OriginWeave browsing-context id

Rejected. Domain identity and protocol addressability have different semantics, and numeric/string coercion would create an implicit cross-boundary alias.

### Let callers supply the remote BiDi context during authorization

Rejected. A caller that possesses raw protocol addressability would be able to redirect an otherwise valid Browser Session authority to another target.

### Key the adapter only by Browser Session lifecycle identity

Rejected as incomplete. Exact-key uniqueness does not prevent two distinct keys from aliasing the same live remote browsing-context string, and separate adapter instances can otherwise maintain contradictory mappings for the same domain lifecycle tuple.

### Treat WebDriver BiDi identifiers as durable capabilities

Rejected. The protocol identifiers are addressability. OriginWeave authority is generated and revalidated by Browser Session lifecycle state, incarnation, isolation, and epoch.

### Treat an immutable Rust borrow as proof that the remote target still exists

Rejected. Remote `contextDestroyed`, browser crash, WebSocket/session termination, or implementation-owned user-context removal can occur without a mutable borrow of either local object. Local alias safety cannot establish remote liveness.

### Return cloneable canonical commands as already-authorized executable capability

Rejected. A caller could retain the command after the live Browser Session check, allow the lifecycle to advance or end, and later submit stale intent. Canonical command payload and execution authority therefore have different lifetimes.

### Automatically destroy a duplicate or ambiguous remote result

Rejected. When ownership is ambiguous, cleanup itself can become a cross-owner destructive action. The adapter must quarantine/fail closed and preserve recovery evidence instead of guessing ownership.

## Decision

1. `WebDriverBidiLifecycleAdapter` owns the private mapping from an accepted Browser Session lifecycle key to the opaque `WebDriverBidiBrowsingContext` returned by its reviewed backend.
2. The mapping key binds browser session, `BrowserSessionIncarnation`, disposable isolation identity, and OriginWeave `BrowsingContextId`. No public constructor or raw identifier tuple exposes an equivalent lifecycle or command capability.
3. Lifecycle create/destroy is entered only through non-caller-constructible Browser Session-issued request/capability values bound to the aggregate-approved lifecycle-port ownership. A separate adapter instance cannot replay those capabilities to seed or destroy an unrelated remote mapping.
4. Creation is a transaction across the domain/adapter boundary. The adapter retains the complete protocol tuple as pending evidence; Browser Session validates the returned domain handle; only an aggregate-issued acceptance promotes the pending tuple into normal authorizing bindings. Rejection keeps the known tuple solely as non-authorizing quarantine/recovery evidence.
5. Before accepting a newly created lifecycle result, the adapter rejects any result whose opaque remote browsing-context string is already bound to another live lifecycle key. Exact lifecycle-key reuse, isolation-only alias, domain-context-only alias, and remote-target alias are distinct failure classes; none replaces an existing accepted binding.
6. Presentation authorization asks the live `BrowserSession` for the current authority for the requested domain context and requires exact equality with the retained token before consulting the private remote mapping. Stale epoch, destruction, transport loss, recovery state, session end, foreign isolation, foreign session, or foreign incarnation fails before a remote target is submitted.
7. Standard viewport/device-pixel-ratio and timezone apply/reset use the existing canonical `WebDriverBidiPresentationCommand` semantics. The ACL must not maintain a second operation/payload vocabulary. Durable command intent is not durable mutation authority.
8. Any authorized execution wrapper is ephemeral and non-caller-constructible. The sole public browser-mutation execution boundary revalidates Browser Session authority, exact lifecycle-adapter ownership, and current adapter-observed remote-context/session membership immediately before backend I/O. A plan's Rust borrow lifetime alone is insufficient.
9. The adapter consumes remote lifecycle events. `browsingContext.contextDestroyed`, BiDi session loss, or equivalent observed disappearance invalidates the corresponding executable mapping and is reconciled into Browser Session recovery/transport-liveness state before another mutation can proceed. OriginWeave does not silently recreate or rebind the lost target under the same domain identity.
10. Screen-area mutation remains separately ownership-gated and is not admitted by this standard presentation path.
11. Plan or intent creation proves neither command success nor browser state. Command acknowledgement, page-observed state, cleanup, and destruction post-conditions require separate runtime evidence.
12. Ambiguous post-create results must not be speculatively destroyed. Complete known BiDi recovery identity—disposable isolation, domain browsing context, and opaque remote browsing context—must be retained in adapter-specific quarantine evidence and grants no command authority.

## Consequences

A valid Browser Session authority cannot be redirected by supplying or reconstructing a remote BiDi context. A second lifecycle result that aliases an already-live remote target is rejected before a second authority becomes executable, including when a different adapter instance is involved.

The adapter has a stronger invariant than its map key alone expresses: accepted remote browsing-context identity is unique within an aggregate-approved lifecycle-port ownership, and a backend result is not an authorizing binding until the aggregate accepts it. Protocol-specific quarantine remains outside Browser Session so the domain does not depend on remote strings.

Lifetime-bound local values reduce accidental local time-of-check/time-of-use drift, but mutation safety ultimately terminates at the transport submission boundary. A remote destruction or session loss after planning makes previously prepared intent non-executable until fresh domain and protocol liveness are proven.

The canonical command vocabulary remains reusable as typed intent without turning its cloneability into a stale capability. Authority is attached only for the duration of a fresh, checked execution.

## Failure and degraded behavior

Missing lifecycle mappings, stale or foreign authority, duplicate lifecycle keys, duplicate isolation/domain identities, duplicate live remote targets, wrong adapter ownership, destroyed remote contexts, and lost BiDi transport fail closed before browser mutation. Creation ambiguity moves Browser Session into `RecoveryRequired`; no new normal authority is issued.

A failed remote destruction remains unproven and blocks further authorization according to ADR 0114. If the adapter has an ambiguous browser-created tuple, it preserves the exact known tuple as non-authorizing recovery evidence and must not auto-destroy or guess another target. A remote `contextDestroyed` received before local cleanup is evidence of lifecycle change, not an implicit successful cleanup result unless the separately defined destruction post-condition is satisfied.

## Security / privacy / governance impact

This ACL prevents protocol-addressability confusion and remote-liveness drift from crossing the Browser Session authority boundary. Page content, LLM output, MCP input, extension input, raw BiDi identifiers, stale command intent, or a second adapter instance cannot construct executable lifecycle/presentation authority.

The change does not replace Chromium sandboxing, EgressWeave network policy, Keyverse identity/secrets, Wardnet controls, contextual-orchestrator model governance, or central repository security gates. It introduces no cross-service SQL, mutable external dependency, provider/model routing, or new secret surface.

## Tests and acceptance evidence

The final ACL test set must cover exact lifecycle mapping, raw/unrelated mapping rejection, stale epoch, destruction, transport loss, session end, clean creation failure, exact lifecycle-key reuse, failed destruction, cross-adapter aliasing, wrong-adapter target redirect, pending acceptance/rejection, quarantine isolation, retained-intent staleness, and remote lifecycle loss.

The hostile remote-liveness RED is: create and adopt `(S,I,U,C)->R1`, obtain a presentation intent through the current ACL, then observe `browsingContext.contextDestroyed` or BiDi session loss before submission. The intent must fail before remote mutation unless the execution boundary has freshly re-established both current Browser Session authority and current adapter-observed remote membership. No silent recreation/rebinding is accepted.

A prior hostile test, `duplicate_remote_context_is_rejected_before_second_authority_is_minted`, created one accepted lifecycle result and then returned a second result with distinct isolation and OriginWeave browsing-context identities but the same opaque remote BiDi browsing-context string. On exact commit `215338b77ddb3d828ebc95b796f5e8080000afde`, CI run `34487233621` failed because the second creation incorrectly returned a valid `PresentationMutationAuthority`, establishing that narrower RED.

Current exact source is still under repair. Repository contracts, canonical formatting, locked Rust tests, strict Clippy, rustdoc/API docs, and exact production function/line/region/branch coverage remain mandatory on the final exact head. Predecessor GREEN does not transfer after source or documentation changes. Independent review and applicable central checks remain required before ordinary adoption.

Real-browser acceptance remains separate: a production backend must prove `browser.createUserContext`/`browsingContext.create` creation, lifecycle-event observation, exact `browser.removeUserContext` destruction or equivalent observed absence, presentation application, page-observed post-conditions, cleanup, crash/restart behavior, and the applicable pinned/current Chromium qualification.

## Migration and rollback

This is an additive stacked-branch ACL over the Browser Session lifecycle foundation. Consumers should obtain remote context only through the lifecycle adapter and should not persist an authorized execution wrapper. Durable typed intent may be persisted only if later execution necessarily performs fresh authority/liveness validation.

No protected-main migration or durable database schema is introduced. Rollback removes this active feature slice while leaving ADR 0114's Browser Session authority fail closed. It must not restore caller-supplied remote contexts, raw-id coercion, duplicate-target acceptance, or executable stale intents.

## Open follow-ups

The following are adoption blockers for this Proposed ADR, not optional post-merge debt:

- Replace the public raw lifecycle-port side door with Browser Session-issued non-forgeable create/accept/reject/destroy capability bound to lifecycle-port ownership.
- Implement pending → accepted/quarantined creation so complete known `WebDriverBidiCreatedContext` identity is never promoted before aggregate acceptance or discarded after rejection.
- Collapse the parallel ACL action vocabulary into the canonical `WebDriverBidiPresentationCommand` semantics while keeping execution authority ephemeral.
- Add adapter remote-lifecycle observation for `browsingContext.contextDestroyed` and BiDi-session loss, with reconciliation into Browser Session recovery/transport-liveness before further mutation.
- Restore exact function/line/region/branch 100% coverage and both canonical ADR indexes on one fresh exact head, then obtain independent review.
- Implement the real WebDriver BiDi disposable-user-context backend and prove browser-observed destruction rather than command ACK.
- Integrate the verified execution path with the pinned Chromium Agent Task lane and page-observed evidence.
- Reconcile the parent Browser Session allocator deprecation (`AtomicU64::fetch_update` renamed to `try_update`) in its canonical owner without mixing that maintenance change into this ACL authority slice.

## Supersession / reversal conditions

Supersede this ADR if the browser protocol or a later OriginWeave adapter provides a stronger generation-safe lifecycle primitive that directly proves unique remote ownership and observed destruction while preserving Browser Session as policy authority. Do not regress to raw protocol identity, local borrow lifetime, or command acknowledgement as authority/liveness proof.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
