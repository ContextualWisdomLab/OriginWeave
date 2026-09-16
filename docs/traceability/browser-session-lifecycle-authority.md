# Browser Session lifecycle authority trace

- Status: IMPLEMENTED_ON_ACTIVE_PR
- Owning bounded context: `originweave-browser-session`
- Governing proposals: ADR 0114; ADR 0116
- Requirement owner: issue #312
- Integration prerequisites: #229 presentation-ownership witnesses; #314/#316 WebDriver BiDi ACL after this foundation is exact-head GREEN

## Problem and invariant

Browser-session, user-context/isolation, browsing-context, adapter-selected identifiers, navigation ids, and recovery evidence are addresses or evidence. They are not proof that the current Browser Session aggregate owns lifecycle or presentation mutation, and they are not self-authenticating recovery settlement authority.

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
→ uncertain/rejected create paths retain exact attempt and identity facts as non-authorizing recovery evidence
→ aggregate records accepted exact handle + epoch
→ opaque PresentationMutationAuthority(session, incarnation, isolation, context, epoch)
→ exact authority validation before ordinary lifecycle or purpose-bounded adapter I/O
→ lifecycle destruction uses private DisposableContextDestroyRequest(handle, validated epoch)
→ presentation work uses private AuthorizedContextOperationRequest<O>(handle, validated epoch, operation)
→ exact consumed adapter only
→ failed/unproven destruction retains exact handle + epoch and enters RecoveryRequired
→ RecoveryRequired|evidence-bearing TransportLost may consume the same bound owner into BoundBrowserSessionRecovery<P>
→ recovery command path privately builds RecoveryContextOperationRequest and uses the same adapter
→ adapter command success/failure leaves unresolved Browser Session state/evidence unchanged
→ recovery custody issues opaque current-revision RecoveryFact values for exact recovery facts
→ caller supplies independently qualified adapter proof with one RecoveryFact
→ settle_recovery_fact validates session/incarnation/revision/exact fact before proof I/O
→ exact retained adapter verifies RecoverySettlementRequest through RecoverySettlementPort
→ verifier failure leaves both ledgers unchanged
→ verifier success retires exactly one fact and advances the recovery revision
→ predecessor/replayed/sibling handles issued under the old revision become stale
→ exact uncertain owned context is removed only for ownership evidence that names that same handle
→ all facts + uncertain hot ownership gone → terminal Ended; ordinary authority is never restored
→ proven ordinary destruction removes live hot ownership; failed destruction retains Uncertain ownership
→ BoundBrowserSession::finish() validates normal completion without consuming the owner on rejection
```

`BoundBrowserSession` is the linear lifecycle-port binding. Public create/destroy methods accept no arbitrary port argument, and there is no public raw port accessor. `BoundBrowserSessionRecovery` preserves the same adapter but is a reduced-capability owner, not an alternate ordinary lifecycle path.

`DisposableContextCreateRequest`, `DisposableContextCreateCompletion`, `DisposableContextDestroyRequest`, `AuthorizedContextOperationRequest<O>`, `RecoveryContextOperationRequest<O>`, `RecoveryFact`, and `RecoverySettlementRequest<P>` all have private construction fields. Epochs, revisions, protocol ids, and evidence are correlation/provenance; none is standalone bearer authority.

## Transactional remote creation

A protocol adapter may stage a successful remote create result as pending when it receives the create request. It must not make that result authorizing until Browser Session accepts that exact attempt through `DisposableContextCreateCompletion`.

`DisposableContextCreateRecoveryEvidence` preserves the transaction dimension that raw handle evidence cannot represent. `FailedUncertain` stores the exact aggregate-issued attempt epoch even without a complete handle; `DuplicateCandidate` binds an aliased candidate to its exact rejected attempt; `CompletionUnsettled` binds exact attempt, disposition, and returned handle when settlement cannot be proven.

Identity-oriented `BrowserSessionRecoveryEvidence` is independently useful for ownership reconciliation. The two ledgers are deliberately separate. A later or rejected same-valued candidate cannot erase a previously accepted ownership fact merely because remote values alias.

Protocol pending/accepted/quarantined tuple storage remains #314/#316 responsibility. Browser Session owns attempt identity, domain accept/reject, current command authority, non-authorizing recovery facts, and deterministic exact-fact retirement after proof verification.

## Same-bound-adapter ordinary authorized operations

`AuthorizedContextOperationPort` extends the lifecycle port for post-create presentation work. Browser Session validates session incarnation, isolation identity, browsing-context identity, current presentation state, and context epoch before creating `AuthorizedContextOperationRequest<O>` and routing it to the same `port: P` already consumed by `BoundBrowserSession`.

Stale or foreign authority fails before adapter I/O as `AuthorizedContextOperationError::BrowserSession`. Adapter failures remain typed as `AuthorizedContextOperationError::Adapter`. No raw `P`, second adapter, or unrestricted callback is exposed.

## Recovery-only custody and command path

`BoundBrowserSession::into_recovery(self)` is one-way. It succeeds from `RecoveryRequired`, or from `TransportLost` only if exact unresolved recovery/create-attempt evidence remains. Ownership-clean transport loss cannot mint recovery capability.

Recovery custody exposes `state()`, both evidence ledgers, and—when `P: RecoveryContextOperationPort`—`execute_recovery_context_operation(operation)`. It exposes neither raw `P`, inner `BoundBrowserSession`, inner `BrowserSession`, ordinary create/presentation/navigation/cleanup authority, nor normal finish.

`RecoveryContextOperationRequest<O>` snapshots exact Browser Session id, incarnation, unresolved state, both evidence ledgers, and the adapter-defined operation immediately before I/O. `BoundBrowserSession::dispatch_recovery_operation` is `pub(crate)`. `RecoveryContextOperationError::Adapter` preserves adapter failure. Both success and failure leave Browser Session uncertainty unchanged: command ACK is not destruction or reconciliation proof.

## Proof-bearing exact-fact recovery settlement

Recovery completion is a separate path. `recovery_fact(index)` and `create_attempt_recovery_fact(index)` issue opaque `RecoveryFact` values only for currently addressable facts. Each handle binds exact Browser Session id, process-local incarnation, ledger kind, index, and the current monotonic recovery revision.

`settle_recovery_fact(fact, proof)` validates in this order:

1. exact Browser Session id and incarnation;
2. current recovery revision;
3. exact current ledger/index fact and next-revision capacity;
4. retained-adapter proof verification through `RecoverySettlementPort::verify_recovery_settlement(RecoverySettlementRequest)`;
5. exact one-fact retirement;
6. monotonic revision advance.

`AuthorityMismatch`, `StaleFact`, and `RevisionExhausted` are pre-I/O failures. `RecoverySettlementError::Adapter(E)` is post-verifier/pre-mutation. A failed proof does not consume evidence. A successful settlement invalidates every fact handle issued under the previous revision, including sibling handles, so the caller must reread current custody after mutation.

The two recovery ledgers remain independently consumable. Retiring `BrowserSessionRecoveryEvidence` never implicitly erases matching `DisposableContextCreateRecoveryEvidence` and vice versa.

Ownership retirement is alias-safe. Only `UnprovenDestruction`, `RecoveryRequiredOwnedHandle`, or `TransportLossOwnedHandle` may remove an exact matching `Uncertain` hot ownership record. `PartialCreationIsolation`, `DuplicateAdapterHandle`, and `UnsettledAdapterHandle` retire evidence only; a rejected candidate that reuses remote values cannot delete a distinct accepted owner.

When both ledgers are empty and no uncertain hot ownership remains, Browser Session reaches `Ended`. Recovery custody never recreates `Active`, create authority, `PresentationMutationAuthority`, navigation authority, or ordinary cleanup authority.

#316 remains canonical owner of WebDriver BiDi proof qualification, pending/accepted/quarantined tuple truth, remote liveness, event correlation, replay qualification, and concrete recovery commands. A protocol event is evidence, not Browser Session policy authority. #316 maps independently qualified evidence into `RecoverySettlementPort::Proof`; Browser Session only validates and consumes its own exact fact.

## Lossless evidence and bounded hot ownership

`CreateFailedClean` is valid only when no disposable browser state exists. Uncertain create, duplicate output, completion failure, failed destroy, sibling invalidation, and transport loss preserve exact facts without granting browser authority.

Hot command-authority state contains only live or uncertain ownership. Proven ordinary destruction removes the current hot record. Failed destroy retains the exact record as `Uncertain` plus `UnprovenDestruction { context, context_epoch }`.

The 258-generation hostile fixture proves that the same raw isolation/context values may be reused after proven destruction while BrowserContextEpoch remains monotonic. Immediately after destruction a retained predecessor authority fails `ContextNotOwned`; after recreation it fails `AuthorityMismatch`. Across aggregate recreation, `BrowserSessionIncarnation` rejects stale authority even when raw ids and local epochs alias.

Removing a proven-destroyed record is not durable history deletion. Cross-process recovery and buyer audit history remain separate persistence concerns.

## Abandonment and lifecycle completion

`BoundBrowserSession<P>` and recovery custody are `#[must_use]` linear owners. `Drop` never performs browser I/O. Dropping unresolved custody increments only the process-local `abandoned_bound_session_count()` signal.

Successful exact-fact reconciliation updates underlying aggregate state before eventual drop. If every fact and uncertain owner is retired, the aggregate is `Ended`, so dropping a fully reconciled recovery wrapper is not reported as unresolved abandonment.

## Navigation authority interaction

The #317 lineage admits observed navigation only for the exact active `(BrowserSessionIncarnation, BrowsingContextId, BrowserContextEpoch)` generation. Admission revokes presentation authority without adapter I/O and mints opaque `NavigationSettlementAuthority`. Commit is non-terminal; positive settlement, typed negative terminal, and download start share one exactly-once closure. A newer navigation supersedes an older witness. Explicit re-establishment alone consumes the next presentation epoch.

Presentation mutation and lifecycle cleanup are separate. A navigation-invalidated presentation capability cannot authorize mutation or authority-based cleanup; the exact ordinary bound lifecycle owner may still destroy its retained context without reopening presentation authority.

Recovery settlement is also separate from navigation settlement. Completing remote-ownership reconciliation cannot recreate navigation or presentation authority.

## Browser-issued identity and standards trace

`DisposableIsolationId` maps to WebDriver BiDi `browser.UserContext` and preserves protocol text losslessly. OriginWeave does not normalize that address or treat it as command authority.

The immutable W3C WebDriver BiDi Working Draft verified for this lineage is the **9 September 2026** publication (`WD-webdriver-bidi-20260909`). The mutable `/TR/webdriver-bidi/` index and separately qualified Chromium runtime revision are distinct provenance axes. A command acknowledgement is insufficient proof that a disposable boundary is gone.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| structural lifecycle-port ownership; no public raw adapter | `BoundBrowserSession`; lifecycle hostile tests |
| aggregate-issued create attempt | `DisposableContextCreateRequest::attempt_epoch`; transaction fixture |
| same consumed adapter for ordinary work | `AuthorizedContextOperationPort`; `authorized_context_operation.rs` |
| same consumed adapter for recovery commands | `RecoveryContextOperationPort`; `RecoveryContextOperationRequest`; `recovery_same_adapter_operation.rs` |
| command success/failure does not settle ownership | `recovery_same_adapter_operation.rs` |
| exact proof-bearing recovery fact | `RecoveryFact`; `RecoverySettlementRequest`; `RecoverySettlementPort`; `settle_recovery_fact` |
| replay/sibling/foreign proof cases | `recovery_exact_fact_settlement.rs` |
| recovery settlement surface remains opaque/current | `tests/test_browser_session_recovery_settlement_contract.py` |
| unproven destroy preserves exact handle + epoch | `UnprovenDestruction`; destroy-failure tests |
| ownership-clean transport loss cannot mint recovery | `recovery_handoff_requires_unresolved_ownership.rs` |
| proven destruction bounds hot ownership | `proven_destroy_releases_hot_ownership.rs` |
| recovery custody cannot regain ordinary authority | recovery rustdoc `compile_fail`; repository contracts |
| navigation witness is opaque and generation-bound | navigation owner tests; `tests/test_browser_session_navigation_owner_surface_contract.py` |

Historical predecessor CI/review receipts do not transfer to the current head. Active-PR source remains non-shipment until exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage at 100%, required review/security gates, and protected `main` integration are observed.

## Current integration boundary

#317 owns Browser Session domain policy, same-adapter recovery-command custody, and generic exact-fact settlement. #318/#321 own stacked hostile navigation/ABA acceptance and doctoring only. #316 owns WebDriver BiDi proof qualification, pending/accepted/quarantined state, event correlation, remote liveness, protocol-specific recovery semantics, and real-browser adapter integration.

Real Chromium navigation, interaction, cleanup, recovery, and browser-observed post-condition evidence remains required before shipment. Immutable release, signed artifact, SBOM, provenance, reproducibility, and rollback evidence remain separate release gates.
