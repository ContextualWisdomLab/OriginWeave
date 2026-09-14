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
→ uncertain/rejected create paths retain exact aggregate-issued attempt identity as non-authorizing recovery evidence
→ aggregate records accepted exact handle + epoch
→ opaque PresentationMutationAuthority(session, incarnation, isolation, context, epoch)
→ exact authority validation before any lifecycle or purpose-bounded adapter I/O
→ lifecycle destruction uses private DisposableContextDestroyRequest(handle, validated epoch)
→ presentation/reconciliation uses private AuthorizedContextOperationRequest<O>(handle, validated epoch, operation)
→ exact consumed adapter only
→ failed/unproven destruction retains exact handle + validated epoch as non-authorizing recovery evidence
→ proven destruction for every context
→ BoundBrowserSession::finish() validates normal completion without consuming the owner on rejection
```

`BoundBrowserSession` is the lifecycle composition boundary. Public create/destroy methods accept no arbitrary port argument, and there is **no public raw port accessor**. Application code cannot recover `&P`, `&mut P`, or a generic callback that would recreate unrestricted adapter authority.

The wrapper has a manual redacted `Debug` implementation. Formatting exposes inert Browser Session summary fields only and never calls `P::fmt`, so a side-effecting or secret-bearing adapter `Debug` cannot become a diagnostic capability escape.

`DisposableContextCreateRequest`, `DisposableContextCreateCompletion`, `DisposableContextDestroyRequest`, and `AuthorizedContextOperationRequest<O>` have private construction paths. The create request carries the already-reserved context epoch as a **per-create transaction** identity. Destroy and purpose-bounded operation requests carry the exact context epoch that Browser Session validated immediately before adapter I/O. That epoch is correlation/provenance only: it does not authorize a command independently from the private aggregate-issued request. Purpose-bounded operations are constructed only after exact `PresentationMutationAuthority` validation.

## Transactional remote creation

A protocol adapter may stage a successful remote create result as pending when it receives the create request. It must not make that result authorizing yet.

Browser Session examines the returned `DisposableContextHandle`:

- if ownership validation succeeds, `DisposableContextCreateCompletion::Accepted` settles that exact attempt before normal presentation authority is returned;
- if the handle aliases an existing isolation or browsing context, `Rejected` settles that exact attempt and the aggregate enters `RecoveryRequired`;
- if exact completion cannot be proven, Browser Session stores the complete handle as `UnsettledAdapterHandle`, enters recovery, and mints no normal authority.

`DisposableContextCreateRecoveryEvidence` preserves the transaction dimension that raw handle evidence cannot represent. `FailedUncertain` stores the exact aggregate-issued `attempt_epoch` even when no complete handle exists; `DuplicateCandidate` binds an aliased returned handle to its exact rejected attempt; `CompletionUnsettled` binds the exact attempt, `Accepted|Rejected` disposition, and complete returned handle when settlement cannot be proven. This evidence grants no browser authority.

Identity-oriented `BrowserSessionRecoveryEvidence` remains separately useful for exact remote reconciliation. Candidate evidence such as `DuplicateAdapterHandle(H)` or `UnsettledAdapterHandle(H)` is not treated as proof that an already-owned same-valued `H` has been recorded: the existing owner is retained independently as `RecoveryRequiredOwnedHandle(H)`. Equal remote values therefore cannot collapse distinct lifecycle facts from different create attempts.

Protocol-specific tuple contents and pending/accepted/quarantined storage remain #314/#316 responsibilities. Browser Session owns only attempt identity, domain validation, accept/reject decision, current authority validation, and non-authorizing recovery facts.

## Same-bound-adapter authorized operations

`AuthorizedContextOperationPort` extends the lifecycle port for adapters that need post-create presentation or reconciliation work. The operation/output/error vocabulary remains adapter-owned. Browser Session validates session incarnation, isolation, browsing-context identity, and context epoch before creating `AuthorizedContextOperationRequest<O>` and routing it to the same `port: P` already consumed into `BoundBrowserSession`. The request exposes that already-validated epoch so adapter execution logs and protocol correlation cannot collapse distinct authority generations that happen to reuse the same external identifiers.

Stale or foreign authority returns `AuthorizedContextOperationError::BrowserSession` before adapter I/O, so no request and no epoch provenance reaches the adapter on rejection. An operation attempted by the exact bound adapter can return `AuthorizedContextOperationError::Adapter`. No raw `P` reference, second adapter, or unrestricted `FnOnce(&mut P)` is exposed.

## Lossless recovery evidence while retained

`CreateFailedClean` is valid only when no disposable browser state exists. `CreateFailedUncertain(Some(isolation))` retains both the exact known isolation identity and the exact aggregate-issued create-attempt epoch; `CreateFailedUncertain(None)` still retains the exact attempt epoch. Duplicate output stores the complete offending handle and its exact rejected attempt. Completion failure retains the complete handle, attempt epoch, and aggregate disposition. Failed or unproven destruction records both the exact owned handle and the exact validated `BrowserContextEpoch` that was sent on the destroy request. When any such failure moves the aggregate to `RecoveryRequired`, every other still-active sibling is projected exactly once as `RecoveryRequiredOwnedHandle` before becoming uncertain. Transport loss records each previously active exact handle as `TransportLossOwnedHandle`. None of this evidence grants browser command authority.

Cause-specific candidate evidence is retained separately from generic sibling ownership evidence. A same-valued candidate from a later failed attempt cannot erase a previously accepted ownership fact. Repeated transport-loss reports are idempotent, so exact transport-loss evidence is not duplicated by repeated notification.

Operation/destroy/create-attempt provenance is implemented on the active #317 lineage: `AuthorizedContextOperationRequest::context_epoch` and `DisposableContextDestroyRequest::context_epoch` are copied only after exact authority validation; stale authority remains zero-I/O; `UnprovenDestruction` preserves that same epoch; create uncertainty and completion failure retain the reserved `attempt_epoch` in `DisposableContextCreateRecoveryEvidence`. The remaining recovery-boundary gap is a purpose-bounded `RecoveryRequired`/`TransportLost` handoff that keeps the exact same adapter and exact evidence instead of reconstructing a second adapter or restoring ordinary mutation authority.

## Abandonment and lifecycle completion

`BoundBrowserSession<P>` is `#[must_use]`. `finish(&mut self)` succeeds only after all owned contexts have proven destruction. If it returns `ActiveContextRemains`, the wrapper, exact bound adapter, and private ownership ledger remain intact. The same owner can therefore destroy or reconcile the remaining context and retry `finish()` without introducing a second adapter or ambient cleanup capability.

`Drop` never performs browser I/O and never treats object destruction as browser destruction proof. Dropping a wrapper with active/uncertain ownership increments the process-local `abandoned_bound_session_count()` signal. This makes ordinary abandonment observable to operability/recovery code without reviving adapter authority. The counter is not durable storage and contains no exact handle payload. Exact durable crash/process-restart recovery therefore remains open until a canonical recovery owner persists Browser Session recovery evidence before process termination.

The abandonment counter and Browser Session incarnation allocator use `AtomicU64::try_update` with the same memory-ordering and closure semantics as the predecessor `fetch_update` calls. This removes the pinned-nightly deprecation without weakening overflow behavior or synchronization semantics.

## Orthogonal transport liveness

Transport liveness is tracked independently from ownership recovery. A first transport loss from `Active` preserves exact active handles as recovery evidence, moves those records to uncertain, and moves the aggregate to `TransportLost`. If transport loss occurs after `RecoveryRequired`, the stronger ownership-recovery state remains while `transport_lost = true` records the orthogonal fact. Repeated loss reports are idempotent.

## Sequential ABA safety

Aggregate A may create `(S,U,C,epoch=1)`, prove destruction, and end. Aggregate B can later start with the same external values and also begin at epoch 1. A's retained authority still fails because B has a different `BrowserSessionIncarnation`. The bound port receives the incarnation inside aggregate-issued lifecycle capabilities.

## Browser-issued user-context identity

`DisposableIsolationId` maps one-to-one to the browser-issued WebDriver BiDi `browser.UserContext` identity. That value is addressability and recovery evidence, not command authority. OriginWeave preserves a protocol-valid browser identity losslessly so the exact remote boundary can later be destroyed or reconciled. The previous arbitrary 4096-byte parser ceiling has been removed; the hostile 4097-byte fixture and internal round-trip test now require lossless preservation. No truncation or normalization is acceptable for a browser-issued identity.

## Standards trace

The latest immutable W3C WebDriver BiDi Working Draft directly verified on 2026-09-15 is the **9 September 2026** publication (`WD-webdriver-bidi-20260909`), with 3 September 2026 as the previous published version. The mutable `/TR/webdriver-bidi/` index can lag this dated publication and is not used to erase immutable provenance. `browser.createUserContext` creates a user context, `browsingContext.create` can create a browsing context inside it, and `browser.removeUserContext` removes the selected user context after closing its navigables. `browser.UserContext` is defined as `text`; the published protocol defines no 4096-byte domain ceiling.

Standards freshness and runtime qualification are separate controls. Updating this citation does not repin the separately qualified Chromium/WebDriver BiDi runtime revision.

OriginWeave does not treat protocol identifiers as policy authority or assume historical non-reuse after removal. A command ACK is insufficient proof that the disposable boundary is actually gone.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| lifecycle port ownership is structural | `BoundBrowserSession`; `bound_port_is_structural_and_not_swappable` |
| no public raw port accessor | absence of `BoundBrowserSession::lifecycle_port`; repository contract |
| binding performs no arbitrary adapter callback | `BrowserSession::bind_lifecycle_port`; `lifecycle_binding_invokes_no_adapter_callback_before_authorized_create` |
| no self-asserted adapter id authority | absence of `DisposableContextPortId` / `port_id()` |
| create requests are aggregate-issued and attempt-scoped | `DisposableContextCreateRequest::attempt_epoch`; transaction hostile fixture |
| uncertain create preserves exact transaction identity with or without a complete handle | `DisposableContextCreateRecoveryEvidence::FailedUncertain`; internal Some/None recovery tests |
| duplicate candidate retains exact rejected attempt | `DisposableContextCreateRecoveryEvidence::DuplicateCandidate`; `create_recovery_same_handle_distinct_fact.rs` |
| completion failure retains exact attempt + disposition + handle | `DisposableContextCreateRecoveryEvidence::CompletionUnsettled`; internal accepted/rejected completion tests |
| same-valued prior owner and later candidate remain distinct recovery facts | `RecoveryRequiredOwnedHandle`; `create_recovery_same_handle_distinct_fact.rs` |
| per-create transaction settles accepted/rejected candidates | `DisposableContextCreateCompletion`; `accepted_and_rejected_create_candidates_are_correlated_by_exact_attempt` |
| completion failure fails closed | `UnsettledAdapterHandle`; internal completion-failure tests |
| raw context cannot mint presentation authority | `BrowserSession::presentation_authority`; `bound_creation_is_the_only_raw_context_entry_to_authority` |
| same consumed adapter handles authorized post-create work | `AuthorizedContextOperationPort`; `authorized_operation_uses_exact_bound_port_and_rejects_stale_authority_before_io` |
| authorized operation carries exact validated epoch | `AuthorizedContextOperationRequest::context_epoch`; `authorized_operation_uses_exact_bound_port_and_rejects_stale_authority_before_io` |
| stale operation authority fails before adapter I/O | `AuthorizedContextOperationError::BrowserSession`; authorized-operation hostile fixture |
| destroy request carries exact validated epoch | `DisposableContextDestroyRequest::context_epoch`; `destroy_failure_requires_recovery_before_any_new_authority` |
| unproven destroy preserves exact handle + validated epoch | `BrowserSessionRecoveryEvidence::UnprovenDestruction`; `destroy_failure_requires_recovery_before_any_new_authority` |
| adapter-owned Debug is not executed or rendered | manual `Debug for BoundBrowserSession<P>`; `bound_session_debug_never_executes_or_exposes_adapter_debug` |
| sequential ABA authority is rejected before I/O | `BrowserSessionIncarnation`; `stale_authority_cannot_cross_sequential_session_incarnations` |
| lossless recovery evidence while aggregate is retained | `BrowserSessionRecoveryEvidence`; `DisposableContextCreateRecoveryEvidence`; recovery tests |
| `RecoveryRequired` preserves indirectly invalidated siblings | `RecoveryRequiredOwnedHandle`; `recovery_required_projects_exact_handles_for_indirectly_uncertain_siblings` |
| transport loss preserves exact active handles | `TransportLossOwnedHandle`; `transport_loss_preserves_exact_owned_handle_as_non_authorizing_recovery_evidence` |
| unresolved wrapper drop performs no browser I/O and is observable | `abandoned_bound_session_count`; `dropping_unresolved_bound_session_is_observable_without_implicit_browser_io` |
| failed finish retains exact bound owner | `BoundBrowserSession::finish`; `failed_finish_retains_same_bound_owner_for_cleanup_and_retry` |
| normal completion requires proven destruction | `BoundBrowserSession::finish`; `proven_destruction_can_finish_without_abandonment_path` |
| protocol-valid user-context identity is preserved losslessly | `user_context_identity_length.rs`; internal 4097-byte round-trip test |
| atomic lifecycle counters use the non-deprecated API without changing ordering | `ABANDONED_BOUND_SESSIONS.try_update`; `allocate_incarnation` |
| transport liveness remains orthogonal | `BrowserSession::record_transport_loss` |
| normal end requires proved destruction | `BrowserSession::end` |
| incarnation exhaustion fails closed | `allocate_incarnation` |

Historical exact `9cde981899950b900698a17e7fa739af59f6bb4f` / CI `34531025582` is RED for this successor: production exact coverage passed, but canonical formatting failed, and the raw port accessor plus missing transaction completion remained. Historical exact `729603ae4feadd369eee7819a45d6850604975da` / CI `34541860394` passed production exact coverage but failed the repository contract after the ADR lost the `DisposableContextDestroyError` trace. Exact `d5046e76cb7555b448b728ea1bed9ba1ea8de8c3` / CI `34573175780` passed Python repository contracts, then failed canonical formatting; production coverage stopped during measurement because the two intentional hostile lifecycle REDs were still unresolved. Historical GREEN never transfers.

Protected-main integration is required before capability maturity can be promoted beyond `IMPLEMENTED_ON_ACTIVE_PR`.

## Buyer acceptance still open

This slice does not yet prove actual WebDriver BiDi lifecycle integration, observed removal post-condition, protocol-specific pending/accepted/quarantined binding, durable crash/process-restart recovery persistence, Browser Session authority conversion into BiDi presentation private witnesses, Chromium post-condition observation, #299 3/3 browser trials, or protected-main release/SBOM/provenance/reproducibility/rollback.

## Reference

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
