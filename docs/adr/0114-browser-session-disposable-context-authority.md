# ADR 0114: Browser Session disposable-context authority

- Status: Proposed
- Date: 2026-09-10
- Last code-current review: 2026-09-12

## Context

OriginWeave's Browser Session bounded context is the domain authority for disposable browser lifecycle ownership and presentation mutation. WebDriver BiDi session ids, user-context ids, browsing-context ids, and adapter-selected values are protocol addressability, not authorization.

The active implementation has to satisfy four constraints at once. First, `BoundBrowserSession<P>` must consume the one concrete lifecycle adapter without later exposing raw `&P`/`&mut P` or a replacement-port path. Second, one Browser Session incarnation can issue multiple remote creates, so each result requires an aggregate-issued per-create transaction identity before it may become authorizing. Third, dependent WebDriver BiDi presentation and reconciliation work still needs to reach the same consumed adapter after exact `PresentationMutationAuthority` validation; retaining a second adapter or generic raw callback would recreate the capability-substitution defect. Fourth, uncertain lifecycle outcomes must preserve every exact non-authorizing owned handle needed for recovery, including siblings invalidated indirectly by another context's failure, and a failed `finish()` must not discard the same bound adapter needed to repair the rejected completion.

Lifecycle failures require lossless evidence while the aggregate remains available. A BiDi adapter can successfully create a user context before later browsing-context creation or verification becomes uncertain. Duplicate adapter output can expose an offending handle that must not be silently discarded or automatically destroyed. Destruction can fail without proving that the exact isolation boundary is gone. A failure on one owned context can force all other active siblings into uncertainty, so those sibling handles also have to remain enumerable. Transport liveness remains orthogonal to ownership certainty.

The latest W3C-published WebDriver BiDi Working Draft verified on 2026-09-12 is the 9 September 2026 publication (`WD-webdriver-bidi-20260909`), with 3 September 2026 as the previous version. It defines `browser.UserContext` as `text`, `browser.createUserContext`, `browsingContext.create`, and `browser.removeUserContext`. The specification requires the user-context id to uniquely identify a user context but does not define a 4096-byte identifier limit. These identifiers and commands remain adapter capabilities/addressability rather than OriginWeave policy authority, and command ACK alone is not destruction proof. Runtime-qualified protocol/browser revisions remain separately controlled and are not repinned by this standards-trace update.

## Decision drivers

- Raw WebDriver/BiDi identifiers and adapter-chosen values are addressability, not mutation or cleanup authority.
- Browser-issued protocol identity needed for exact lifecycle ownership and recovery must remain losslessly representable; an uncited implementation constant must not silently redefine `browser.UserContext` semantics.
- No arbitrary adapter callback may be required to establish lifecycle-port ownership.
- A caller must not be able to substitute or recover the concrete adapter after Browser Session binding.
- Browser Session create/destroy capabilities remain non-caller-constructible.
- Every successful remote create result must be correlated to one exact Browser Session-issued attempt before it can become authorizing.
- Browser Session, not the adapter, decides whether a returned domain handle is accepted or rejected.
- Protocol-specific pending/accepted/quarantined tuples remain the WebDriver BiDi ACL owner's truth.
- Presentation/reconciliation I/O must use the exact consumed adapter only after current aggregate authority validation.
- Diagnostic formatting must not invoke adapter-owned `Debug` or expose adapter-internal state.
- Silent loss of active/uncertain ownership on ordinary `BoundBrowserSession` drop must be observable without performing browser I/O from `Drop`.
- A rejected `finish()` must retain the exact bound lifecycle owner so cleanup/reconciliation and a later retry remain possible.
- Sequential aggregate recreation must not make retained stale authority valid again.
- Recovery evidence and transport liveness remain orthogonal.
- Browser Session remains the domain authority; WebDriver BiDi, CDP, MCP, and LLMs remain adapters or consumers.

## Decision

Introduce and retain `originweave-browser-session` as an independent Rust bounded context. ADR status remains `Proposed` until protected-main and real-browser acceptance exist.

1. `BrowserSession::start` allocates a process-local, monotonically non-reused `BrowserSessionIncarnation` before browser I/O. Exhaustion fails closed.
2. Presentation authority is intentionally non-serializable. `BrowserSessionIncarnation` prevents sequential ABA within one process.
3. Browser Session uses a **linear lifecycle-port binding**. `BrowserSession::bind_lifecycle_port` consumes both aggregate and one concrete `DisposableContextPort` into `BoundBrowserSession<P>` without invoking adapter code.
4. `BoundBrowserSession<P>` has **no public raw port accessor** and no lifecycle method that accepts an alternate port. Tests observe adapter behavior through independently retained inert counters/ledgers rather than extracting `&P`.
5. `DisposableContextPort` has no identity-preflight method. Adapter identity is structural composition, not a self-asserted scalar.
6. `DisposableContextCreateRequest` remains opaque and carries the already-reserved `BrowserContextEpoch` as an exact per-create transaction identity. The `(session, incarnation, attempt epoch)` tuple is unique for create attempts in one live aggregate and is not caller-constructible as a request.
7. After `create_disposable_context` returns a handle, Browser Session validates isolation and browsing-context ownership before granting authority.
8. Browser Session privately issues `DisposableContextCreateCompletion` for that exact attempt with `Accepted` or `Rejected`.
9. The adapter must keep a successful remote create result non-authorizing until the matching `Accepted` completion. `Rejected` results remain non-authorizing recovery/quarantine state. A completion that cannot be proven for the exact pending attempt fails closed and sends the aggregate to `RecoveryRequired`.
10. Protocol-specific remote tuple contents are not copied into Browser Session. #314/#316 owns WebDriver BiDi pending/accepted/quarantined storage and remote-liveness validation.
11. `DisposableIsolationId` preserves the browser-issued isolation identity exactly for create/destroy/recovery addressability. It must not truncate, normalize, hash, or reject an otherwise protocol-valid `browser.UserContext` solely because of an arbitrary local identifier-length constant. Resource-exhaustion limits, where required, belong at a cited protocol/frame/runtime boundary or an explicit deployment policy that still preserves lossless recovery evidence.
12. `DisposableContextDestroyRequest` remains opaque and is created only after exact presentation-authority validation. It carries Browser Session addressability, incarnation, and the exact stored handle.
13. `PresentationMutationAuthority` binds browser session, Browser Session incarnation, disposable isolation, browsing context, and context epoch. All fields must match current aggregate ownership before adapter I/O.
14. `DisposableContextCreateError::CreateFailedClean` is valid only when no remote boundary exists. `CreateFailedUncertain(Option<DisposableIsolationId>)` enters `RecoveryRequired`; any known isolation identity is preserved exactly.
15. Duplicate browsing-context or isolation output enters `RecoveryRequired`, stores the complete offending `DisposableContextHandle`, and sends a `Rejected` completion for the exact attempt. OriginWeave does not auto-destroy ambiguous output.
16. `BrowserSessionRecoveryEvidence` includes partial-creation identity, duplicate handle, unsettled complete adapter handle, exact unproven-destruction handle, `RecoveryRequiredOwnedHandle` for every still-active sibling made uncertain by a recovery transition, and `TransportLossOwnedHandle` for each active handle whose remote liveness becomes uncertain on transport loss. Cause-specific evidence is not duplicated as generic sibling evidence. These values are explicit **unproven destruction** evidence rather than cleanup proof and grant no browser command authority. Repeated recovery/loss observation must not duplicate exact-handle evidence.
17. Destruction validates exact authority before I/O. `DisposableContextDestroyError::DestroyFailed` means destruction was not proven; the owned record becomes uncertain, the exact failed handle is retained as `UnprovenDestruction`, and the aggregate enters recovery rather than treating command acknowledgement or bookkeeping as cleanup proof. Any other active sibling is projected as `RecoveryRequiredOwnedHandle` before it becomes uncertain.
18. Transport liveness is stored separately from ownership state. The first `record_transport_loss()` records exact previously active handles as non-authorizing recovery evidence, marks them uncertain, and records the transport fact. If ownership is already `RecoveryRequired`, the stronger lifecycle state is preserved.
19. `RecoveryRequired`, `TransportLost`, and `Ended` reject normal active-only lifecycle and authority operations.
20. `AuthorizedContextOperationRequest<O>` is non-caller-constructible. `AuthorizedContextOperationPort` lets a dependent adapter define a narrow operation vocabulary while Browser Session first validates current `PresentationMutationAuthority`, binds the exact stored handle, and routes the request through the same consumed adapter instance. `AuthorizedContextOperationError::BrowserSession` is returned before adapter I/O for stale/foreign authority; adapter execution errors remain separately typed. Browser Session does not own WebDriver BiDi command semantics.
21. `BoundBrowserSession<P>` implements a manual redacted `Debug` projection over inert Browser Session fields only. Formatting never calls `P::fmt` and never renders adapter-internal state.
22. `BoundBrowserSession<P>` is `#[must_use]`. `finish(&mut self)` admits normal completion only after all owned contexts have proven destruction. A failed `finish()` returns the domain error without consuming or dropping the wrapper, so the same exact bound adapter and ownership ledger remain available for cleanup/reconciliation and retry. After success the aggregate is `Ended`, and later wrapper destruction is inert. `Drop` never performs browser I/O; if unresolved remote ownership remains, it increments the process-local `abandoned_bound_session_count()` operability signal.
23. The abandonment counter is deliberately not destruction proof and is not durable cross-process recovery storage. Exact recovery handles must be persisted by the separately authorized recovery owner before process termination. Until that owner path is integrated, crash/process-restart reconciliation remains an explicit buyer-acceptance gap rather than an implicit guarantee.
24. Context epochs remain monotonic authority identities within one aggregate and also provide the create-attempt correlation allocated before remote create I/O.

## Alternatives considered

### Adapter-supplied identity or preflight callback

Rejected. A scalar can be replayed and a shared-reference callback can still have side effects before Browser Session authority exists.

### Public read-only `&P` after binding

Rejected. Rust `&P` forbids an ordinary mutable borrow but does not prohibit interior mutation or remote effects from `&self` methods. The concrete adapter would remain a capability escape.

### `#[derive(Debug)]` over `BoundBrowserSession<P>`

Rejected. Derived formatting delegates to `P::fmt`; a side-effecting or secret-bearing adapter `Debug` becomes an authority/data-exposure escape. Manual redacted formatting is selected.

### Generic `FnOnce(&mut P)` callback

Rejected. Although it would reach the exact consumed adapter, it hands unrestricted adapter authority back to callers and defeats the anti-corruption boundary. A typed `AuthorizedContextOperationPort` operation is selected instead.

### Second retained adapter or shared client outside `BoundBrowserSession`

Rejected. It recreates same-key/different-adapter target redirection and lets presentation/reconciliation work escape the exact lifecycle instance Browser Session accepted.

### Adapter-local create sequence number

Rejected as authority. It may be useful internally, but Browser Session could not prove which pending remote tuple it was accepting. Correlation must originate in the aggregate-issued request.

### Reserved BrowserContextEpoch as create-attempt identity

Selected. Browser Session already reserves the epoch before create I/O, it is non-caller-constructible, monotonic within the aggregate, and the same value becomes the accepted context's first mutation epoch.

### Hard-coded maximum length for `browser.UserContext`

Rejected unless an authoritative protocol/runtime bound is cited and versioned. The current WebDriver BiDi WD defines `browser.UserContext` as `text` and does not define a 4096-byte identifier ceiling. OriginWeave therefore must not convert a browser-issued, otherwise valid identity into ownership loss because of an implementation-chosen domain constant.

### Consuming `finish(self)` before validation

Rejected. An expected `ActiveContextRemains` would destroy the only wrapper that owns the accepted adapter and private lifecycle ledger. Validation therefore occurs through `finish(&mut self)`; only successful completion changes the aggregate to `Ended`.

### Browser I/O from `Drop`

Rejected. Rust destruction is synchronous and cannot prove remote cleanup. `Drop` is restricted to non-I/O abandonment observability; normal completion is explicit through proven destruction plus `finish()`.

### Automatically clean duplicate or rejected state

Rejected. Ambiguous ownership makes speculative cleanup a potential cross-owner destructive action.

## Consequences

The active stack receives a breaking trait extension for presentation/reconciliation adapters: implementations that need post-create authorized operations implement `AuthorizedContextOperationPort` and keep their protocol-specific command vocabulary in the adapter. #316 must restack non-force and map this boundary into its pending/accepted/quarantined BiDi state.

The bound adapter is not publicly recoverable from `BoundBrowserSession`. Application and test code that needs observability retains inert metrics or diagnostic projections separately. Manual `Debug` exposes only Browser Session domain summary fields and a redacted port marker.

Entering `RecoveryRequired` now preserves exact handles for active siblings before marking them uncertain. Cause-specific evidence for the triggering context remains distinct, so recovery can enumerate every potentially live boundary without reconstructing command authority from identifiers.

Transport loss preserves exact previously active handles as non-authorizing recovery evidence. Completion or destruction failure remains ownership uncertainty and does not mint normal authority.

A failed `finish()` leaves the same `BoundBrowserSession` usable for cleanup/reconciliation and retry. Ordinary unresolved wrapper abandonment is process-locally observable, but exact crash/restart recovery still requires a canonical persistence/handoff path. This ADR does not claim that the in-memory counter is durable recovery.

## Security and governance impact

No page-controlled value, raw browser identifier, adapter-selected scalar, diagnostic reference, provider/model decision, or LLM output can mint lifecycle completion or presentation authority. Remote creation stays non-authorizing until the aggregate validates ownership and accepts that exact attempt. Post-create adapter I/O is admitted only through current aggregate authority and the exact consumed adapter instance.

Lossless retention of browser-issued user-context identity is an ownership requirement, not authority delegation. Resource controls must not create an untracked remote isolation boundary by discarding or rewriting the only exact addressability needed for recovery.

This decision does not replace Chromium sandboxing, EgressWeave, Keyverse, Wardnet, or central workflow security.

## Tests and exact evidence

Required executable cases include:

- binding invokes no arbitrary adapter callback before an aggregate-issued create request;
- the concrete bound adapter cannot be recovered through a public `lifecycle_port()` accessor;
- a second adapter cannot be substituted for create or destroy after binding;
- two successful remote create candidates in the same session incarnation receive distinct attempt epochs;
- one candidate can be accepted and the other rejected without pending-state collision or overwrite;
- accepted-completion failure and rejected-completion failure both fail closed and preserve exact recovery evidence;
- an otherwise-valid browser-issued `browser.UserContext` longer than the former 4096-byte implementation threshold remains losslessly representable for exact destroy/recovery rather than being rejected by an arbitrary domain cap;
- `DisposableContextDestroyError::DestroyFailed` preserves the exact failed handle, enters `RecoveryRequired`, and never counts a destroy command acknowledgement as proof;
- `RecoveryRequired` preserves each indirectly invalidated active sibling exactly once as `RecoveryRequiredOwnedHandle` while retaining the triggering context's cause-specific evidence;
- transport loss preserves every previously active exact handle as `TransportLossOwnedHandle` without adapter I/O or authority resurrection;
- formatting a bound session does not invoke adapter-owned `Debug` and does not expose adapter-internal state;
- an authorized operation reaches the exact consumed adapter, adapter errors remain typed, and stale authority fails before adapter I/O;
- dropping a bound session with unresolved ownership performs no implicit browser cleanup and increments the abandonment operability signal;
- a failed `finish()` performs no browser I/O or abandonment, retains the same bound owner, permits exact cleanup, and succeeds on retry after proven destruction;
- recovery, sequential-incarnation ABA, epoch exhaustion, foreign authority, destruction failure, transport loss, and normal end remain covered.

The historical exact `9cde981899950b900698a17e7fa739af59f6bb4f` CI `34531025582` passed exact production coverage but failed canonical Rust formatting. The historical `729603ae4feadd369eee7819a45d6850604975da` run `34541860394` passed exact production coverage but failed the repository contract because ADR 0114 had lost the `DisposableContextDestroyError` trace. Exact `d5046e76cb7555b448b728ea1bed9ba1ea8de8c3` / CI `34573175780` passed Python repository contracts but failed canonical formatting; production coverage stopped during measurement because the two intentionally RED hostile lifecycle cases were still unresolved. Historical GREEN never transfers. Successor evidence must be fresh: repository contracts, canonical formatting, locked tests, strict Clippy, rustdoc/API docs, and production function/line/region/branch coverage each exactly 100%.

## Buyer acceptance still open

This slice does not yet prove actual WebDriver BiDi lifecycle integration, browser-observed destruction, protocol-specific pending/accepted/quarantined binding, durable crash/process-restart recovery handoff, Browser Session authority conversion into BiDi presentation private witnesses, current Chromium post-condition observation, #299 3/3 browser trials, or protected-main release/SBOM/provenance/reproducibility/rollback.

## Migration and rollback

Consumers continue to bind once with `BrowserSession::bind_lifecycle_port(port)` and perform lifecycle work through `BoundBrowserSession`. Code must not depend on recovering `&P`. Adapter implementations add exact-attempt staging/completion and, when they need post-create presentation or reconciliation I/O, implement the typed `AuthorizedContextOperationPort` operation vocabulary.

Normal owners destroy every owned context, call `finish()`, and may then release the ended wrapper. If `finish()` rejects, they retain the same wrapper, perform permitted cleanup/reconciliation, and retry. Recovery owners must persist exact recovery evidence before terminating a process that still has unresolved ownership; the abandonment counter is an operability alert, not a persistence mechanism.

Rollback may return to the predecessor active-PR API only if these authority findings are disproved with stronger executable evidence. It must not restore a raw adapter accessor, derived adapter `Debug`, self-reported identity, unrestricted adapter callback, consuming failed-finish path, arbitrary browser-user-context length cap, or adapter-local call order as an authorization boundary.

## Open follow-ups

- Restack #316 onto the verified Browser Session successor and implement WebDriver BiDi pending → accepted/quarantined transaction settlement plus typed presentation/reconciliation operations.
- Define the separately authorized durable recovery persistence/reconciliation owner for `BrowserSessionRecoveryEvidence` and unresolved abandonment.
- Replay #299 historical pinned Chromium evidence after the canonical sandbox/runtime repair, then run a separate current-Stable qualification.

## Supersession / reversal conditions

Supersede this ADR if the browser platform provides a complete, queryable, generation-safe ownership primitive with exact destruction evidence, or if OriginWeave adopts another isolation primitive with equivalent guarantees. Do not regress to raw context identity or adapter-selected identity as authority.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
