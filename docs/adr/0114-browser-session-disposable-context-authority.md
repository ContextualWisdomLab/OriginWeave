# ADR 0114: Browser Session disposable-context authority

- Status: Proposed
- Date: 2026-09-10

## Context

OriginWeave's Browser Session bounded context is the authority for disposable browser lifecycle ownership and presentation mutation. WebDriver BiDi session ids, user-context ids, browsing-context ids, and adapter-selected values are protocol addressability, not authorization.

Two active-PR findings refine the lifecycle-port boundary. First, `BoundBrowserSession::lifecycle_port(&self) -> &P` exposed the concrete adapter after binding. Rust shared references do not prove purity: interior mutability, synchronization primitives, or an internally synchronized client can still mutate local state or perform remote I/O. A "read-only" label therefore does not create a security boundary.

Second, one Browser Session incarnation can issue multiple remote create attempts. A create request carrying only `(BrowserSessionId, BrowserSessionIncarnation)` does not identify which returned protocol tuple Browser Session later accepted or rejected. A WebDriver BiDi adapter needs an exact **per-create transaction** so it can stage remote state as pending, promote only the accepted candidate, and quarantine the rejected candidate without relying on call order or adapter-local counters as authority.

Lifecycle failures still require lossless evidence. A BiDi adapter can successfully create a user context before later browsing-context creation or verification becomes uncertain. Duplicate adapter output can expose an offending handle that must not be silently discarded or automatically destroyed. Destruction can fail without proving that the exact isolation boundary is gone. Transport liveness remains orthogonal to ownership certainty.

The 9 September 2026 WebDriver BiDi Working Draft defines `browser.createUserContext`, `browsingContext.create`, and `browser.removeUserContext`. These commands remain adapter capabilities rather than OriginWeave policy authority, and command ACK alone is not destruction proof.

## Decision drivers

- Raw WebDriver/BiDi identifiers and adapter-chosen values are addressability, not mutation or cleanup authority.
- No arbitrary adapter callback may be required to establish lifecycle-port ownership.
- A caller must not be able to substitute or recover the concrete adapter after Browser Session binding.
- Browser Session create/destroy capabilities remain non-caller-constructible.
- Every successful remote create result must be correlated to one exact Browser Session-issued attempt before it can become authorizing.
- Browser Session, not the adapter, decides whether a returned domain handle is accepted or rejected.
- Protocol-specific pending/accepted/quarantined tuples remain the WebDriver BiDi ACL owner's truth.
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
6. `DisposableContextCreateRequest` remains opaque and gains the already-reserved `BrowserContextEpoch` as an exact create-attempt identity. The `(session, incarnation, attempt epoch)` tuple is unique for create attempts in one live aggregate and is not caller-constructible as a request.
7. After `create_disposable_context` returns a handle, Browser Session validates isolation and browsing-context ownership before granting authority.
8. Browser Session then privately issues `DisposableContextCreateCompletion` for that exact attempt with `Accepted` or `Rejected`.
9. The adapter must keep a successful remote create result non-authorizing until the matching `Accepted` completion. `Rejected` results remain non-authorizing recovery/quarantine state. A completion that cannot be proven for the exact pending attempt fails closed and sends the aggregate to `RecoveryRequired`.
10. Protocol-specific remote tuple contents are not copied into Browser Session. #314/#316 owns WebDriver BiDi pending/accepted/quarantined storage and remote-liveness validation.
11. `DisposableContextDestroyRequest` remains opaque and is created only after exact presentation-authority validation. It carries Browser Session addressability, incarnation, and the exact stored handle.
12. `PresentationMutationAuthority` binds browser session, Browser Session incarnation, disposable isolation, browsing context, and context epoch. All fields must match current aggregate ownership before destruction I/O.
13. `DisposableContextCreateError::CreateFailedClean` is valid only when no remote boundary exists. `CreateFailedUncertain(Option<DisposableIsolationId>)` enters `RecoveryRequired`; any known isolation identity is preserved exactly.
14. Duplicate browsing-context or isolation output enters `RecoveryRequired`, stores the complete offending `DisposableContextHandle`, and sends a `Rejected` completion for the exact attempt. OriginWeave does not auto-destroy ambiguous output.
15. `BrowserSessionRecoveryEvidence` includes partial-creation identity, duplicate handle, an unsettled complete adapter handle when completion itself cannot be proven, and exact unproven-destruction handle. Recovery evidence grants no browser command authority.
16. Destruction validates exact authority before I/O. Unproven destruction moves the aggregate into recovery and retains the exact failed handle.
17. Transport liveness is stored separately from ownership state. The first `record_transport_loss()` records the fact even after `RecoveryRequired`; repeated reports are idempotent.
18. `RecoveryRequired`, `TransportLost`, and `Ended` reject normal active-only lifecycle and authority operations.
19. Context epochs remain monotonic authority identities within one aggregate and also provide the create-attempt correlation allocated before remote create I/O.

## Alternatives considered

### Adapter-supplied identity or preflight callback

Rejected. A scalar can be replayed and a shared-reference callback can still have side effects before Browser Session authority exists.

### Public read-only `&P` after binding

Rejected. Rust `&P` forbids an ordinary mutable borrow but does not prohibit interior mutation or remote effects from `&self` methods. The concrete adapter would remain a capability escape.

### Adapter-local create sequence number

Rejected as authority. It could be useful internally, but Browser Session could not prove which pending remote tuple it was accepting. Correlation must originate in the aggregate-issued request.

### Reserved BrowserContextEpoch as create-attempt identity

Selected. Browser Session already reserves the epoch before create I/O, it is non-caller-constructible, monotonic within the aggregate, and the same value becomes the accepted context's first mutation epoch.

### Automatically clean duplicate or rejected state

Rejected. Ambiguous ownership makes speculative cleanup a potential cross-owner destructive action.

## Consequences

The active stack receives a breaking trait change: every `DisposableContextPort` implementation must settle successful create results through `complete_disposable_context_creation`. #316 must restack non-force and map this completion into its adapter-local pending/accepted/quarantined state.

The bound adapter is no longer publicly recoverable from `BoundBrowserSession`. Application and test code that needs observability must retain inert metrics or diagnostic projections separately; those projections must not expose adapter command capability.

A completion failure is treated as ownership uncertainty. Browser Session retains the returned handle as recovery evidence and does not mint normal authority.

## Security and governance impact

No page-controlled value, raw browser identifier, adapter-selected scalar, diagnostic reference, provider/model decision, or LLM output can mint lifecycle completion or presentation authority. Remote creation stays non-authorizing until the aggregate validates ownership and accepts that exact attempt.

This decision does not replace Chromium sandboxing, EgressWeave, Keyverse, Wardnet, or central workflow security.

## Tests and exact evidence

Required executable cases include:

- binding invokes no arbitrary adapter callback before an aggregate-issued create request;
- the concrete bound adapter cannot be recovered through a public `lifecycle_port()` accessor;
- a second adapter cannot be substituted for create or destroy after binding;
- two successful remote create candidates in the same session incarnation receive distinct attempt epochs;
- one candidate can be accepted and the other rejected without pending-state collision or overwrite;
- accepted-completion failure and rejected-completion failure both fail closed and preserve exact recovery evidence;
- recovery, sequential-incarnation ABA, epoch exhaustion, foreign authority, destruction failure, transport loss, and normal end remain covered.

The historical exact `9cde981899950b900698a17e7fa739af59f6bb4f` CI `34531025582` passed exact production coverage but failed canonical Rust formatting. That exact head also still exposed raw `&P` and lacked per-create completion. Successor evidence must therefore be fresh: repository contracts, canonical formatting, locked tests, strict Clippy, rustdoc/API docs, and production function/line/region/branch coverage each exactly 100%.

## Buyer acceptance still open

This slice does not prove real WebDriver BiDi lifecycle integration, browser-observed destruction, Browser Session→BiDi private-witness conversion, current Chromium presentation post-conditions, crash/restart reconciliation, #299 3/3 Agent Task replay, or protected-main release/SBOM/provenance/reproducibility/rollback.

## Migration and rollback

Consumers continue to bind once with `BrowserSession::bind_lifecycle_port(port)` and perform lifecycle work through `BoundBrowserSession`. Code must not depend on recovering `&P`. Adapter implementations add exact-attempt staging and completion settlement.

Rollback may return to the predecessor active-PR API only if these authority findings are disproved with stronger executable evidence. It must not restore a raw adapter accessor, self-reported identity, or adapter-local call order as an authorization boundary.

## Open follow-ups

- Restack #316 onto the verified Browser Session successor and implement WebDriver BiDi pending → accepted/quarantined transaction settlement.
- Define separately authorized recovery reconciliation for `BrowserSessionRecoveryEvidence`.
- Replay #299 historical pinned Chromium evidence after the canonical sandbox/runtime repair, then run a separate current-Stable qualification.

## Supersession / reversal conditions

Supersede this ADR if the browser platform provides a complete, queryable, generation-safe ownership primitive with exact destruction evidence, or if OriginWeave adopts another isolation primitive with equivalent guarantees. Do not regress to raw context identity or adapter-selected identity as authority.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
