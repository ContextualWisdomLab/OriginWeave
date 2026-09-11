# Browser Session lifecycle authority trace

- Status: IMPLEMENTED_ON_ACTIVE_PR
- Owning bounded context: `originweave-browser-session`
- Governing proposal: ADR 0114
- Requirement owner: issue #312
- Integration prerequisites: #229 presentation-ownership witnesses; #314/#316 WebDriver BiDi ACL after this foundation is exact-head GREEN

## Problem and invariant

Browser-session, user-context/isolation, browsing-context, and adapter-selected identifiers are addresses. They are not evidence that the current Browser Session aggregate exclusively owns lifecycle or presentation mutation.

The active implementation establishes this chain:

```text
validated BrowserSessionId
→ BrowserSession::start allocates non-reused BrowserSessionIncarnation
→ BrowserSession::bind_lifecycle_port consumes one concrete DisposableContextPort
→ BoundBrowserSession<P> owns aggregate + exact port; no public raw port accessor exists
→ aggregate validates Active + reserves monotonic BrowserContextEpoch
→ aggregate privately constructs DisposableContextCreateRequest(session, incarnation, attempt epoch)
→ exact owned port creates a remote candidate but must keep it non-authorizing
→ aggregate validates returned isolation/context against current ownership
→ aggregate privately constructs DisposableContextCreateCompletion(attempt, Accepted|Rejected)
→ accepted candidate may become adapter-authorizing; rejected candidate remains quarantined
→ aggregate records accepted exact handle + epoch
→ opaque PresentationMutationAuthority(session, incarnation, isolation, context, epoch)
→ exact authority validation before any lifecycle or purpose-bounded adapter I/O
→ lifecycle destruction uses private DisposableContextDestroyRequest
→ presentation/reconciliation uses private AuthorizedContextOperationRequest<O>
→ exact consumed adapter only
→ proven destruction for every context
→ BoundBrowserSession::finish() consumes the normal lifecycle
```

`BoundBrowserSession` is the lifecycle composition boundary. Public create/destroy methods accept no arbitrary port argument, and there is **no public raw port accessor**. Application code cannot recover `&P`, `&mut P`, or a generic callback that would recreate unrestricted adapter authority.

The wrapper has a manual redacted `Debug` implementation. Formatting exposes inert Browser Session summary fields only and never calls `P::fmt`, so a side-effecting or secret-bearing adapter `Debug` cannot become a diagnostic capability escape.

`DisposableContextCreateRequest`, `DisposableContextCreateCompletion`, `DisposableContextDestroyRequest`, and `AuthorizedContextOperationRequest<O>` have private construction paths. The create request carries the already-reserved context epoch as a **per-create transaction** identity. Purpose-bounded operations are constructed only after exact `PresentationMutationAuthority` validation.

## Transactional remote creation

A protocol adapter may stage a successful remote create result as pending when it receives the create request. It must not make that result authorizing yet.

Browser Session examines the returned `DisposableContextHandle`:

- if ownership validation succeeds, `DisposableContextCreateCompletion::Accepted` settles that exact attempt before normal presentation authority is returned;
- if the handle aliases an existing isolation or browsing context, `Rejected` settles that exact attempt and the aggregate enters `RecoveryRequired`;
- if exact completion cannot be proven, Browser Session stores the complete handle as `UnsettledAdapterHandle`, enters recovery, and mints no normal authority.

Protocol-specific tuple contents and pending/accepted/quarantined storage remain #314/#316 responsibilities. Browser Session owns only attempt identity, domain validation, accept/reject decision, and current authority validation.

## Same-bound-adapter authorized operations

`AuthorizedContextOperationPort` extends the lifecycle port for adapters that need post-create presentation or reconciliation work. The operation/output/error vocabulary remains adapter-owned. Browser Session validates session incarnation, isolation, browsing-context identity, and context epoch before creating `AuthorizedContextOperationRequest<O>` and routing it to the same `port: P` already consumed into `BoundBrowserSession`.

Stale or foreign authority returns `AuthorizedContextOperationError::BrowserSession` before adapter I/O. An operation attempted by the exact bound adapter can return `AuthorizedContextOperationError::Adapter`. No raw `P` reference, second adapter, or unrestricted `FnOnce(&mut P)` is exposed.

## Lossless recovery evidence while retained

`CreateFailedClean` is valid only when no disposable browser state exists. `CreateFailedUncertain(Some(isolation))` retains the exact known isolation identity. Duplicate output stores the complete offending handle. Completion failure retains an unsettled complete handle. Failed or unproven destruction records the exact owned handle. Transport loss records each previously active exact handle as `TransportLossOwnedHandle` before marking it uncertain. None of this evidence grants browser command authority.

Repeated transport-loss reports are idempotent, so exact transport-loss evidence is not duplicated by repeated notification.

## Abandonment and lifecycle completion

`BoundBrowserSession<P>` is `#[must_use]`. The normal consuming path is `finish()`, which succeeds only after all owned contexts have proven destruction. `Drop` never performs browser I/O and never treats object destruction as browser destruction proof.

Dropping a wrapper with active/uncertain ownership increments the process-local `abandoned_bound_session_count()` signal. This makes ordinary abandonment observable to operability/recovery code without reviving adapter authority. The counter is not durable storage and contains no exact handle payload. Exact crash/process-restart recovery therefore remains open until a canonical recovery owner persists `BrowserSessionRecoveryEvidence` before process termination.

## Orthogonal transport liveness

Transport liveness is tracked independently from ownership recovery. A first transport loss from `Active` preserves exact active handles as recovery evidence, moves those records to uncertain, and moves the aggregate to `TransportLost`. If transport loss occurs after `RecoveryRequired`, the stronger ownership-recovery state remains while `transport_lost = true` records the orthogonal fact. Repeated loss reports are idempotent.

## Sequential ABA safety

Aggregate A may create `(S,U,C,epoch=1)`, prove destruction, and end. Aggregate B can later start with the same external values and also begin at epoch 1. A's retained authority still fails because B has a different `BrowserSessionIncarnation`. The bound port receives the incarnation inside aggregate-issued lifecycle capabilities.

## Standards trace

The latest W3C-published WebDriver BiDi Working Draft verified on 2026-09-11 is the 24 August 2026 publication. `browser.createUserContext` creates a user context, `browsingContext.create` can create a browsing context inside it, and `browser.removeUserContext` removes the selected user context after closing its navigables. A previously cited 9 September snapshot could not be verified in the W3C latest-published report or publication index and is therefore not treated as authoritative evidence.

OriginWeave does not treat those protocol identifiers as policy authority or assume historical non-reuse after removal. A command ACK is insufficient proof that the disposable boundary is actually gone.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| lifecycle port ownership is structural | `BoundBrowserSession`; `bound_port_is_structural_and_not_swappable` |
| no public raw port accessor | absence of `BoundBrowserSession::lifecycle_port`; repository contract |
| binding performs no arbitrary adapter callback | `BrowserSession::bind_lifecycle_port`; `lifecycle_binding_invokes_no_adapter_callback_before_authorized_create` |
| no self-asserted adapter id authority | absence of `DisposableContextPortId` / `port_id()` |
| create requests are aggregate-issued and attempt-scoped | `DisposableContextCreateRequest::attempt_epoch`; transaction hostile fixture |
| per-create transaction settles accepted/rejected candidates | `DisposableContextCreateCompletion`; `accepted_and_rejected_create_candidates_are_correlated_by_exact_attempt` |
| completion failure fails closed | `UnsettledAdapterHandle`; internal completion-failure tests |
| raw context cannot mint presentation authority | `BrowserSession::presentation_authority`; `bound_creation_is_the_only_raw_context_entry_to_authority` |
| same consumed adapter handles authorized post-create work | `AuthorizedContextOperationPort`; `authorized_operation_uses_exact_bound_port_and_rejects_stale_authority_before_io` |
| stale operation authority fails before adapter I/O | `AuthorizedContextOperationError::BrowserSession`; authorized-operation hostile fixture |
| adapter-owned Debug is not executed or rendered | manual `Debug for BoundBrowserSession<P>`; `bound_session_debug_never_executes_or_exposes_adapter_debug` |
| sequential ABA authority is rejected before I/O | `BrowserSessionIncarnation`; `stale_authority_cannot_cross_sequential_session_incarnations` |
| lossless recovery evidence while aggregate is retained | `BrowserSessionRecoveryEvidence`; recovery tests |
| transport loss preserves exact active handles | `TransportLossOwnedHandle`; `transport_loss_preserves_exact_owned_handle_as_non_authorizing_recovery_evidence` |
| unproven destruction retains exact handle | `destroy_failure_requires_recovery_before_any_new_authority` |
| unresolved wrapper drop performs no browser I/O and is observable | `abandoned_bound_session_count`; `dropping_unresolved_bound_session_is_observable_without_implicit_browser_io` |
| normal consuming completion requires proven destruction | `BoundBrowserSession::finish`; `proven_destruction_can_finish_without_abandonment_path` |
| transport liveness remains orthogonal | `BrowserSession::record_transport_loss` |
| normal end requires proved destruction | `BrowserSession::end` |
| incarnation exhaustion fails closed | `allocate_incarnation` |

Historical exact `9cde981899950b900698a17e7fa739af59f6bb4f` / CI `34531025582` is RED for this successor: production exact coverage passed, but canonical formatting failed, and the raw port accessor plus missing transaction completion remained. Historical exact `729603ae4feadd369eee7819a45d6850604975da` / CI `34541860394` passed production exact coverage but failed the repository contract after the ADR lost the `DisposableContextDestroyError` trace. Historical GREEN never transfers.

Protected-main integration is required before capability maturity can be promoted beyond `IMPLEMENTED_ON_ACTIVE_PR`.

## Buyer acceptance still open

This slice does not yet prove actual WebDriver BiDi lifecycle integration, observed removal post-condition, protocol-specific pending/accepted/quarantined binding, durable crash/process-restart recovery persistence, Browser Session authority conversion into BiDi presentation private witnesses, Chromium post-condition observation, #299 3/3 browser trials, or protected-main release/SBOM/provenance/reproducibility/rollback.

## Reference

Browser Testing and Tools Working Group. (2026, August 24). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260824/
