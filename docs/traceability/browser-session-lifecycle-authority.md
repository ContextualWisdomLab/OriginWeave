# Browser Session lifecycle authority trace

- Status: IMPLEMENTED_ON_ACTIVE_PR
- Owning bounded context: `originweave-browser-session`
- Governing proposals: ADR 0114; ADR 0116
- Requirement owner: issue #312
- Integration prerequisites: #229 presentation-ownership witnesses; #314/#316 WebDriver BiDi ACL after this foundation is exact-head GREEN

## Problem and invariant

Browser-session, user-context/isolation, browsing-context, adapter-selected identifiers, and recovery evidence are addresses or evidence. They are not proof that the current Browser Session aggregate exclusively owns lifecycle or presentation mutation.

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
→ exact authority validation before ordinary lifecycle or purpose-bounded adapter I/O
→ lifecycle destruction uses private DisposableContextDestroyRequest(handle, validated epoch)
→ presentation work uses private AuthorizedContextOperationRequest<O>(handle, validated epoch, operation)
→ exact consumed adapter only
→ failed/unproven destruction retains exact handle + validated epoch as non-authorizing recovery evidence
→ RecoveryRequired|TransportLost can consume the same bound owner into BoundBrowserSessionRecovery<P>
→ recovery custody exposes exact non-authorizing evidence and one purpose-bounded RecoveryContextOperationPort path
→ recovery wrapper privately builds RecoveryContextOperationRequest(session, incarnation, state, evidence, operation)
→ the crate-private bridge dispatches that request through the exact retained adapter without exposing raw P
→ adapter success/failure leaves unresolved Browser Session state/evidence unchanged
→ proven destruction removes the live hot-ownership record; failed destruction retains Uncertain ownership
→ BoundBrowserSession::finish() validates normal completion without consuming the owner on rejection
```

`BoundBrowserSession` is the linear lifecycle-port binding. Public create/destroy methods accept no arbitrary port argument, and there is **no public raw port accessor**. Application code cannot recover `&P`, `&mut P`, or a generic callback that would recreate unrestricted adapter authority.

The wrapper has a manual redacted `Debug` implementation. Formatting exposes inert Browser Session summary fields only and never calls `P::fmt`, so a side-effecting or secret-bearing adapter `Debug` cannot become a diagnostic capability escape.

`DisposableContextCreateRequest`, `DisposableContextCreateCompletion`, `DisposableContextDestroyRequest`, `AuthorizedContextOperationRequest<O>`, and `RecoveryContextOperationRequest<O>` have private construction paths. Create attempt epochs and validated context epochs are correlation/provenance, not standalone bearer authority.

## Transactional remote creation

A protocol adapter may stage a successful remote create result as pending when it receives the create request. It must not make that result authorizing yet. Browser Session accepts or rejects the returned domain handle and settles that exact attempt through `DisposableContextCreateCompletion`.

`DisposableContextCreateRecoveryEvidence` preserves the transaction dimension that raw handle evidence cannot represent. `FailedUncertain` stores the exact aggregate-issued `attempt_epoch` even when no complete handle exists; `DuplicateCandidate` binds an aliased returned handle to its exact rejected attempt; `CompletionUnsettled` binds the exact attempt, `Accepted|Rejected` disposition, and complete returned handle when settlement cannot be proven. This evidence grants no browser authority.

Identity-oriented `BrowserSessionRecoveryEvidence` remains separately useful for exact remote reconciliation. A same-valued later candidate does not erase a previously accepted ownership fact represented independently by `RecoveryRequiredOwnedHandle`.

Protocol-specific tuple contents and pending/accepted/quarantined storage remain #314/#316 responsibilities. Browser Session owns only attempt identity, domain validation, accept/reject decision, current authority validation, and non-authorizing recovery facts.

## Same-bound-adapter ordinary authorized operations

`AuthorizedContextOperationPort` extends the lifecycle port for post-create presentation work. Browser Session validates session incarnation, isolation, browsing-context identity, and context epoch before creating `AuthorizedContextOperationRequest<O>` and routing it to the same `port: P` already consumed into `BoundBrowserSession`. Stale or foreign authority returns `AuthorizedContextOperationError::BrowserSession` before adapter I/O; an adapter failure returns `AuthorizedContextOperationError::Adapter`.

No raw `P` reference, second adapter, or unrestricted `FnOnce(&mut P)` is exposed.

## Recovery-only custody and same-adapter recovery operation

`BoundBrowserSession::into_recovery(self)` is the one-way custody boundary for unresolved ownership. It succeeds only from `RecoveryRequired` or `TransportLost`, moves the exact already-consumed non-`Clone` adapter without browser I/O, and returns `BoundBrowserSessionRecovery<P>`. `Active` or `Ended` returns the original bound owner unchanged.

Recovery custody exposes `state()`, `recovery_evidence()`, `create_attempt_recovery_evidence()`, and—when the retained adapter implements `RecoveryContextOperationPort`—`execute_recovery_context_operation(operation)`. It exposes neither raw `P`, the inner `BoundBrowserSession`, nor the inner `BrowserSession`; ordinary create, presentation-authority lookup, epoch advancement, destroy, `AuthorizedContextOperationPort`, navigation authority, and normal finish remain unavailable. Rustdoc `compile_fail` contracts pin those negative capabilities.

The recovery operation does not weaken that boundary. `RecoveryContextOperationRequest<O>` is created only inside recovery custody and snapshots exact `BrowserSessionId`, `BrowserSessionIncarnation`, unresolved `BrowserSessionState`, `BrowserSessionRecoveryEvidence`, `DisposableContextCreateRecoveryEvidence`, and the adapter-defined operation immediately before I/O. `BoundBrowserSession::dispatch_recovery_operation` is `pub(crate)` and lives in the Browser Session owner module; downstream code cannot call it or supply a callback receiving `&mut P`.

`RecoveryContextOperationPort` owns its operation/output/error vocabulary. `RecoveryContextOperationError::Adapter` preserves typed failure. Both adapter success and failure leave Browser Session state and evidence unchanged. A successful call is not destruction proof, not reconciliation proof, and not presentation authority. #316 remains responsible for WebDriver BiDi pending/accepted/quarantined tuple truth, remote liveness, event correlation, replay qualification, and the meaning of concrete recovery commands.

## Lossless recovery evidence while retained

`CreateFailedClean` is valid only when no disposable browser state exists. `CreateFailedUncertain(Some(isolation))` retains both the exact known isolation identity and exact aggregate-issued create-attempt epoch; `CreateFailedUncertain(None)` still retains the exact attempt epoch. Duplicate output and completion failure preserve exact transaction identity. Failed or unproven destruction records the exact owned handle and validated `BrowserContextEpoch`. Entering `RecoveryRequired` projects still-active siblings as `RecoveryRequiredOwnedHandle`; transport loss records active handles as `TransportLossOwnedHandle`. None of this evidence grants browser command authority.

## Bounded hot ownership and Sequential ABA

Hot command-authority state contains only live or uncertain ownership. After exact authority validation and adapter-confirmed destruction, Browser Session removes that context record instead of accumulating a permanent tombstone. A failed destroy retains the exact record as `Uncertain` plus `UnprovenDestruction { context, context_epoch }`.

The 258-generation hostile fixture proves the same raw isolation/context values can be reused after proven destruction while epochs remain monotonic. Immediately after destruction a retained authority fails as `ContextNotOwned`; after same-raw-id recreation it fails as `AuthorityMismatch`. Sequential ABA across independent aggregates is also rejected by `BrowserSessionIncarnation` even when external ids and local epoch values alias.

Removing proven-destroyed command-authority records is not durable history deletion. Durable crash/process-restart recovery and buyer audit history remain a separate persistence concern and must not be reconstructed from the bounded hot map or `abandoned_bound_session_count()`.

## Abandonment and lifecycle completion

`BoundBrowserSession<P>` is `#[must_use]`. A failed `finish()` does not consume the wrapper; the exact bound adapter and ownership ledger remain available for cleanup/retry. `Drop` never performs browser I/O and never treats object destruction as browser destruction proof. Dropping unresolved ordinary or recovery custody increments the process-local `abandoned_bound_session_count()` signal only.

The abandonment counter and Browser Session incarnation allocator use the non-deprecated `AtomicU64::try_update` API while preserving ordering and overflow behavior.

## Navigation authority interaction

The active #317 lineage admits an observed navigation only for the exact active `(BrowserSessionIncarnation, BrowsingContextId, BrowserContextEpoch)` generation. It revokes presentation authority without adapter I/O, mints an opaque `NavigationSettlementAuthority`, treats commit as non-terminal progress, uses one exactly-once closure for positive settlement / typed negative terminal / download start, and permits one explicit re-establishment opportunity. A newer navigation supersedes an older witness. Aggregate trust and current live ownership dominate settlement and re-establishment.

Presentation mutation and lifecycle cleanup are separate. A navigation-invalidated presentation capability cannot authorize mutation or authority-based cleanup; the exact bound lifecycle owner may still perform `destroy_owned_disposable_context` without reopening presentation authority.

## Browser-issued identity and standards trace

`DisposableIsolationId` maps one-to-one to WebDriver BiDi `browser.UserContext` and preserves protocol text losslessly. OriginWeave does not trim, normalize, impose the removed 4096-byte limit, or treat that address as command authority.

The latest immutable W3C WebDriver BiDi Working Draft directly verified on 2026-09-15 is the **9 September 2026** publication (`WD-webdriver-bidi-20260909`), with 3 September 2026 as the previous published version. The mutable `/TR/webdriver-bidi/` index and the separately qualified Chromium/runtime revision are distinct provenance axes. A command ACK is insufficient proof that a disposable boundary is actually gone.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| lifecycle port ownership is structural; no public raw port accessor | `BoundBrowserSession`; lifecycle binding hostile tests |
| create requests are aggregate-issued and attempt-scoped | `DisposableContextCreateRequest::attempt_epoch`; transaction hostile fixture |
| same-valued prior owner and later candidate remain distinct recovery facts | `RecoveryRequiredOwnedHandle`; `create_recovery_same_handle_distinct_fact.rs` |
| same consumed adapter handles ordinary purpose-bounded work | `AuthorizedContextOperationPort`; `authorized_context_operation.rs` |
| same consumed adapter handles recovery-only work without raw adapter escape | `RecoveryContextOperationPort`; `RecoveryContextOperationRequest`; `recovery_same_adapter_operation.rs` |
| recovery success/failure preserves unresolved state/evidence | `recovery_same_adapter_operation.rs` |
| unproven destroy preserves exact handle + epoch | `BrowserSessionRecoveryEvidence::UnprovenDestruction`; destroy-failure tests |
| `RecoveryRequired` preserves indirectly invalidated siblings | `RecoveryRequiredOwnedHandle`; `recovery_required_sibling_evidence.rs` |
| transport loss preserves exact active handles | `TransportLossOwnedHandle`; `transport_loss_recovery_evidence.rs` |
| unresolved state moves to recovery-only custody without replacing adapter | `BoundBrowserSession::into_recovery`; `recovery_owner_handoff.rs` |
| recovery custody cannot regain ordinary lifecycle or presentation authority | `BoundBrowserSessionRecovery` rustdoc `compile_fail`; repository contract |
| proven destruction releases bounded hot ownership while stale authority remains rejected | `proven_destroy_releases_hot_ownership.rs` |
| unresolved wrapper drop performs no browser I/O and is observable | `abandoned_bound_session_count`; `bound_session_abandonment.rs` |
| failed finish retains exact bound owner | `BoundBrowserSession::finish`; abandonment fixture |
| protocol-valid user-context identity is preserved losslessly | `user_context_identity_length.rs` |
| transport liveness remains orthogonal | `BrowserSession::record_transport_loss` |
| navigation witness is opaque and context-generation-bound | navigation owner tests; `tests/test_browser_session_navigation_owner_surface_contract.py` |

Historical predecessor CI/review receipts do not transfer to the current head. Active-PR source and tests remain non-shipment until exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage at 100%, required security/review workflows, and protected `main` integration are observed.

## Current integration boundary

#317 owns Browser Session domain policy and the generic same-adapter recovery-operation boundary. #318/#321 own stacked hostile acceptance and doctoring only. #316 owns WebDriver BiDi pending/accepted/quarantined state, event correlation, remote liveness, protocol-specific recovery semantics, and real-browser adapter integration. No child may copy Browser Session production source or reconstruct a second adapter from raw identifiers.

Real Chromium navigation, interaction, cleanup, recovery, and browser-observed post-condition evidence remains required before shipment. Immutable release/SBOM/provenance/reproducibility/rollback evidence remains a separate release gate.