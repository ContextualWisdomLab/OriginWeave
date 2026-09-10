# ADR 0114: Browser Session disposable-context authority

- Status: Proposed
- Date: 2026-09-10

## Context

OriginWeave's WebDriver BiDi presentation adapter requires opaque ownership witnesses before viewport/device-pixel-ratio, timezone, or screen-area mutation can be planned. A caller that merely knows a browser-session or browsing-context identifier therefore cannot overwrite another owner's presentation state and later clear it to an implementation default.

Browser-session, user-context/isolation, browsing-context, and adapter-selected identifiers are protocol or implementation addressability. They are not Browser Session authority. A previous repair introduced opaque `DisposableContextCreateRequest` and `DisposableContextDestroyRequest`, but also asked each adapter to self-report a public numeric port id. That left two defects: a second adapter could select the same id, and Browser Session had to invoke arbitrary adapter code to read that id before lifecycle authority existed. Rust `&self` does not make such a callback pure.

Lifecycle failures also need lossless evidence. A BiDi adapter can successfully create a user context before later browsing-context creation or verification becomes uncertain. Duplicate adapter output can expose an offending handle that must not be silently discarded or automatically destroyed. Destruction can fail without proving that the exact isolation boundary is gone. These outcomes require recovery quarantine while retaining every exact browser-issued identity that is already known.

Transport liveness is independent from ownership certainty. A session already in `RecoveryRequired` can subsequently lose its transport; that new fact must be recorded without erasing recovery evidence. Conversely, merely entering recovery does not prove the transport is dead.

The 9 September 2026 WebDriver BiDi Working Draft defines user-context identifiers and the `browser.createUserContext`, `browsingContext.create`, and `browser.removeUserContext` lifecycle. Those commands remain adapter capabilities rather than OriginWeave policy authority, and command ACK alone is not destruction proof.

## Decision drivers

- Raw WebDriver/BiDi identifiers and adapter-chosen ids are addressability, not mutation or cleanup authority.
- No arbitrary adapter callback may be required to establish lifecycle-port ownership.
- A caller must not be able to substitute a second adapter instance after Browser Session lifecycle binding.
- Create/destroy requests must remain non-caller-constructible and usable only through the bound aggregate composition.
- Sequential aggregate recreation must not make a retained stale authority valid again.
- Known remote identities from partial creation, duplicate output, or unproven destruction must be retained as recovery evidence without becoming command authority.
- Ownership recovery and transport liveness remain orthogonal.
- Browser Session remains the domain authority; WebDriver BiDi, CDP, MCP, and LLMs remain adapters or consumers.

## Decision

Introduce and retain `originweave-browser-session` as an independent Rust bounded context. ADR status remains `Proposed` until protected-main and real-browser acceptance exist.

1. `BrowserSession::start` allocates a process-local, monotonically non-reused `BrowserSessionIncarnation` before browser I/O. Allocation fails closed before `u64` wrap.
2. Presentation authority is intentionally non-serializable. Within one process, `BrowserSessionIncarnation` prevents sequential ABA when a later aggregate reuses the same external session, isolation, context, and local epoch values.
3. Browser Session uses a **linear lifecycle-port binding**. `BrowserSession::bind_lifecycle_port` consumes both the aggregate and one concrete adapter value into `BoundBrowserSession<P>`. Binding performs no adapter callback.
4. `BoundBrowserSession<P>` does not expose mutable port access and its public create/destroy methods accept no alternate port argument. The exact adapter instance is therefore structural composition rather than a caller-selected or self-asserted scalar identity.
5. `DisposableContextPort` has no `port_id()` preflight method. `DisposableContextPortId` is removed. A second adapter cannot claim equality by choosing the same scalar.
6. `DisposableContextCreateRequest` and `DisposableContextDestroyRequest` remain opaque, have no public constructor, and are created only inside the bound Browser Session path after aggregate state or exact presentation authority has been validated. They carry Browser Session addressability and incarnation; the destroy request additionally carries the exact stored handle.
7. The adapter is part of the reviewed lifecycle anti-corruption boundary. A malicious adapter implementation that internally delegates an authorized request is outside what a Rust trait can prevent without inverting the dependency boundary; protocol-specific pending/accepted/quarantine ownership remains the responsibility of the separately reviewed BiDi ACL adapter in ADR 0115.
8. A context enters the owned set only after the bound port returns a `DisposableContextHandle`. Raw `BrowsingContextId` input never creates ownership.
9. `PresentationMutationAuthority` binds browser session, Browser Session incarnation, disposable isolation, browsing context, and context epoch. All fields must match current aggregate ownership before destruction I/O.
10. `DisposableContextCreateError::CreateFailedClean` is valid only when no remote boundary exists. `CreateFailedUncertain(Option<DisposableIsolationId>)` enters `RecoveryRequired`; a known browser-issued isolation identity is preserved exactly.
11. Duplicate browsing-context or isolation output enters `RecoveryRequired` and stores the complete offending `DisposableContextHandle` as recovery evidence. OriginWeave does not auto-destroy ambiguous output.
12. `BrowserSessionRecoveryEvidence` records only reconciliation evidence: `PartialCreationIsolation`, `DuplicateAdapterHandle`, and `UnprovenDestruction`. It grants no browser command authority.
13. Destruction validates exact authority before I/O and passes the current incarnation and stored handle in `DisposableContextDestroyRequest`. `DisposableContextDestroyError` moves the record and aggregate into recovery and retains the exact failed handle.
14. Transport liveness is stored separately from ownership state. The first `record_transport_loss()` records the fact even after `RecoveryRequired`; repeated reports are idempotent.
15. `RecoveryRequired`, `TransportLost`, and `Ended` reject active-only creation, authority issuance/advance, destruction, and normal end. Reconciliation is a later, separately authorized design.
16. Context epochs remain monotonic authority identities within one aggregate. They invalidate older authority after navigation or another lifecycle boundary but are not a substitute for session incarnation.

## Alternatives considered

### Adapter-supplied numeric port id

Rejected. A public scalar is caller-selectable and replayable by a distinct adapter. Making the callback side-effect-free by documentation is also insufficient because Rust `&self` permits interior mutation and delegated effects.

### Pointer-address identity

Rejected. Object addresses are implementation details, can change when values move, and can be reused after destruction. Pointer equality would replace one ABA surface with another.

### Session-owned wrapper with the concrete port

Selected. Ownership is represented by Rust move semantics and private fields. No identity probe is required, the caller cannot swap a second adapter into public lifecycle methods, and opaque requests remain confined to the bound call path.

### Persist authority generations globally

Deferred. Presentation authority is not durable across process restart; restart reconciliation belongs to evidence and browser observation, not silent authority resurrection.

### Automatically clean duplicate or partial state

Rejected. When ownership is ambiguous, cleanup itself can become a cross-owner destructive action.

## Consequences

Browser Session no longer asks an adapter to prove its own identity before authority. The aggregate and exact lifecycle port become one composed runtime object, while adapter-specific remote identifiers remain outside the Browser Session domain model.

The API change is intentionally breaking on the active stack: consumers must call `BrowserSession::bind_lifecycle_port(port)` and then perform lifecycle operations through `BoundBrowserSession`. ADR 0115/#316 must be non-force restacked and adapt its WebDriver BiDi lifecycle adapter to this composition before adoption.

This binding closes ordinary caller substitution and self-selected-id replay. It does not claim that an adversarial implementation of the trusted `DisposableContextPort` trait cannot internally forward calls; such an implementation already executes inside the reviewed adapter TCB. The BiDi ACL still must prove pending → accepted/quarantined remote ownership, complete recovery tuples, and live-target validation independently.

## Security and governance impact

No page-controlled value, raw browser-session id, raw browsing-context id, user-context string, adapter-selected scalar, provider/model decision, or LLM output can mint lifecycle requests or presentation authority. Browser Session performs no arbitrary adapter callback while establishing the lifecycle-port binding.

Unknown or duplicate remote state is quarantined rather than destroyed speculatively. This does not replace Chromium sandboxing, EgressWeave, Keyverse, Wardnet, or central workflow security.

## Tests and exact evidence

The suite retains recovery, sequential ABA, epoch, foreign-authority, destruction, transport-loss, and normal-end coverage. `lifecycle_binding_invokes_no_adapter_callback_before_authorized_create` proves that binding performs no adapter callback before the aggregate-issued create request. `distinct_adapter_cannot_be_substituted_for_create_after_binding` and `distinct_adapter_cannot_be_substituted_for_destroy_after_binding`, together with repository source contracts, require lifecycle methods to use only the consumed port and prohibit reintroduction of public `DisposableContextPortId`/`port_id()` or arbitrary-port Browser Session lifecycle methods.

The prior hostile RED was captured on exact `d43a4d86c8487ebdb9db9f1c4650fb7ee6225afc` in CI `34524654914`: the pre-authority callback fixture observed one identity callback where zero was required. This decision replaces that self-asserted identity design rather than suppressing the test.

Repository contracts, canonical formatting, locked Rust tests, strict Clippy, rustdoc/API docs, exact function/line/region/branch coverage, current review findings, and applicable central checks remain required on the successor exact head. Predecessor GREEN never transfers.

## Buyer acceptance still open

This slice does not yet prove real WebDriver BiDi `browser.createUserContext`/`browsingContext.create`/`browser.removeUserContext` integration, pending/accepted/quarantined remote binding, browser-observed destruction, Browser Session→BiDi private-witness conversion, pinned Chromium presentation post-conditions, crash/restart cleanup, #299 3/3 Agent Task replay, or protected-main release/SBOM/provenance/reproducibility/rollback.

## Migration and rollback

Consumers on the active stack replace `session.create_disposable_context(&mut port)` / `session.destroy_disposable_context(..., &mut port)` with one `let mut bound = session.bind_lifecycle_port(port)` followed by bound lifecycle calls. The wrapper exposes read-only access to the aggregate and adapter for policy validation and diagnostics but does not return mutable adapter access or an unbound session.

Rollback returns to the predecessor active-PR API only if the lifecycle-port authority finding is rejected with stronger evidence; it must not restore self-reported scalar identity as a security boundary.

## Open follow-ups

- Restack #316 onto the verified Browser Session successor and adapt the WebDriver BiDi lifecycle ACL to `BoundBrowserSession` without exposing a second lifecycle side door.
- Implement protocol-specific pending → accepted/quarantined creation and complete recovery tuples in the BiDi ACL owner.
- Define separately authorized reconciliation for `BrowserSessionRecoveryEvidence`, including browser/process restart.
- Replay #299 historical pinned Chromium evidence after the canonical sandbox/runtime repair, then run a separate current-Stable qualification.

## Supersession / reversal conditions

Supersede this ADR if the browser platform provides a complete, queryable, generation-safe ownership primitive with exact destruction evidence, or if OriginWeave adopts another isolation primitive with equivalent guarantees. Do not regress to raw context identity or adapter-selected identity as authority.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
