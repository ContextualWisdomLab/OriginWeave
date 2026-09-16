# Browser Session lifecycle authority trace

- Status: IMPLEMENTED_ON_ACTIVE_PR
- Owning bounded context: `originweave-browser-session`
- Governing proposals: ADR 0114; ADR 0116
- Requirement owner: issue #312
- Integration prerequisites: #229 presentation-ownership witnesses; #314/#316 WebDriver BiDi ACL after this foundation is exact-head GREEN

## Problem and invariant

Browser-session, user-context/isolation, browsing-context, adapter-selected identifiers, navigation ids, recovery evidence, and adapter command results are addresses or evidence. None is self-authenticating lifecycle, presentation, navigation, or recovery authority.

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
→ failed/unproven destruction retains exact handle + epoch and enters RecoveryRequired
→ RecoveryRequired|evidence-bearing TransportLost may consume the same bound owner into BoundBrowserSessionRecovery<P>
→ recovery custody issues opaque current-revision RecoveryFact values
→ recovery command requires one current RecoveryFact + adapter-defined operation
→ session/incarnation/revision/exact-fact validation occurs before recovery adapter I/O
→ RecoveryContextOperationRequest carries only the selected fact, never sibling recovery ledgers
→ adapter command success/failure leaves Browser Session uncertainty unchanged
→ caller supplies independently qualified proof with one current RecoveryFact
→ settle_recovery_fact revalidates session/incarnation/revision/exact fact before proof I/O
→ exact retained adapter verifies RecoverySettlementRequest through RecoverySettlementPort
→ verifier success retires exactly one fact and advances the recovery revision
→ predecessor/replayed/sibling handles issued under the old revision become stale
→ exact uncertain owned context is removed only for ownership evidence that names that same handle
→ all facts + uncertain hot ownership gone → terminal Ended; ordinary and recovery-command authority are closed
→ proven ordinary destruction removes live hot ownership; failed destruction retains Uncertain ownership
```

`BoundBrowserSession` is the linear lifecycle-port binding. `BoundBrowserSessionRecovery` preserves that same adapter as a reduced-capability, one-way owner. `DisposableContextCreateRequest`, `DisposableContextCreateCompletion`, `DisposableContextDestroyRequest`, `AuthorizedContextOperationRequest<O>`, `RecoveryContextOperationRequest<O>`, `RecoveryFact`, and `RecoverySettlementRequest<P>` have private construction fields. Epochs, revisions, protocol ids, and evidence are correlation/provenance rather than standalone bearer authority.

## Transactional remote creation

A protocol adapter may stage a successful remote create result as pending when it receives the create request. It must not promote that result into an authorizing binding until Browser Session accepts the exact attempt through `DisposableContextCreateCompletion`.

`DisposableContextCreateRecoveryEvidence` preserves transaction identity that raw handle evidence cannot represent. `FailedUncertain` stores the aggregate-issued attempt epoch even without a complete handle; `DuplicateCandidate` binds an aliased candidate to its rejected attempt; `CompletionUnsettled` binds attempt, disposition, and returned handle when completion settlement cannot be proven.

Identity-oriented `BrowserSessionRecoveryEvidence` remains independently useful for ownership reconciliation. The two ledgers are separate: a later or rejected same-valued candidate cannot erase a previously accepted ownership fact merely because remote values alias.

Protocol pending/accepted/quarantined tuple storage remains #314/#316 responsibility. Browser Session owns attempt identity, domain accept/reject, current command authority, non-authorizing recovery facts, and deterministic exact-fact retirement after proof verification.

## Same-bound-adapter ordinary authorized operations

`AuthorizedContextOperationPort` extends the lifecycle port for post-create presentation work. Browser Session validates session incarnation, isolation identity, browsing-context identity, current presentation state, and context epoch before creating `AuthorizedContextOperationRequest<O>` and routing it to the same `port: P` already consumed by `BoundBrowserSession`.

Stale or foreign authority fails before adapter I/O as `AuthorizedContextOperationError::BrowserSession`. Adapter failures remain typed as `AuthorizedContextOperationError::Adapter`. No raw `P`, second adapter, or unrestricted callback is exposed.

## Recovery-only custody and exact-fact command path

`BoundBrowserSession::into_recovery(self)` is one-way. It succeeds from `RecoveryRequired`, or from `TransportLost` only if exact unresolved recovery/create-attempt evidence remains. Ownership-clean transport loss cannot mint recovery capability.

Recovery custody exposes `state()`, read-only recovery ledgers, current-fact issuance, and—when `P: RecoveryContextOperationPort`—`execute_recovery_context_operation(fact, operation)`. It exposes neither raw `P`, inner `BoundBrowserSession`, inner `BrowserSession`, ordinary create/presentation/navigation/cleanup authority, nor normal finish.

Before recovery-command I/O, Browser Session validates that the supplied `RecoveryFact` belongs to the exact session/incarnation, was issued at the current recovery revision, and still addresses the current ledger/index fact. Foreign facts fail as `RecoveryContextOperationError::AuthorityMismatch`; stale, replayed, shifted, or out-of-range facts fail as `RecoveryContextOperationError::StaleFact`. Both fail before the adapter is invoked.

`RecoveryContextOperationRequest<O>` snapshots the Browser Session id, incarnation, unresolved state, the selected identity-oriented **or** create-attempt recovery fact, and the adapter-defined operation. Its evidence fields are `Option<...>`, not full vectors. Sibling facts are intentionally withheld from the adapter command path. `BoundBrowserSession::dispatch_recovery_operation` remains `pub(crate)`.

Adapter success or `RecoveryContextOperationError::Adapter(E)` leaves all Browser Session recovery state unchanged. The same current fact may be retried until a successful settlement changes the recovery revision. Command ACK is not destruction or reconciliation proof. After complete settlement reaches `Ended`, the command path returns `RecoveryContextOperationError::RecoveryClosed` before adapter I/O.

## Proof-bearing exact-fact recovery settlement

Recovery completion is separate from recovery command execution. `recovery_fact(index)` and `create_attempt_recovery_fact(index)` issue opaque `RecoveryFact` values only for currently addressable facts. Each handle binds Browser Session id, process-local incarnation, ledger kind, index, and current monotonic recovery revision.

`settle_recovery_fact(fact, proof)` validates:

1. exact Browser Session id and incarnation;
2. current recovery revision and exact current ledger/index fact;
3. next-revision capacity;
4. retained-adapter proof verification through `RecoverySettlementPort::verify_recovery_settlement(RecoverySettlementRequest)`;
5. exact one-fact retirement;
6. monotonic revision advance.

`AuthorityMismatch`, `StaleFact`, and `RevisionExhausted` are pre-I/O failures. `RecoverySettlementError::Adapter(E)` is post-verifier/pre-mutation. A failed proof consumes nothing. A successful settlement invalidates every fact handle issued under the previous revision, including unrelated sibling handles, so callers reread custody after mutation.

The two recovery ledgers remain independently consumable. Retiring `BrowserSessionRecoveryEvidence` never implicitly erases matching `DisposableContextCreateRecoveryEvidence` and vice versa.

Ownership retirement is alias-safe. Only `UnprovenDestruction`, `RecoveryRequiredOwnedHandle`, or `TransportLossOwnedHandle` may remove an exact matching `Uncertain` hot ownership record. `PartialCreationIsolation`, `DuplicateAdapterHandle`, and `UnsettledAdapterHandle` retire evidence only; a rejected candidate that reuses remote values cannot delete a distinct accepted owner.

When both ledgers are empty and no uncertain hot ownership remains, Browser Session reaches `Ended`. Recovery custody never recreates `Active`, create authority, `PresentationMutationAuthority`, navigation authority, ordinary cleanup authority, or generic recovery-command authority.

#316 remains canonical owner of WebDriver BiDi proof qualification, pending/accepted/quarantined tuple truth, remote liveness, event correlation, replay qualification, and concrete recovery commands. A protocol event is evidence, not Browser Session policy authority. #316 maps independently qualified evidence into `RecoverySettlementPort::Proof`; Browser Session validates and consumes only its own exact fact.

## Lossless evidence and bounded hot ownership

`CreateFailedClean` is valid only when no disposable browser state exists. Uncertain create, duplicate output, completion failure, failed destroy, sibling invalidation, and transport loss preserve exact facts without granting browser authority.

Hot command-authority state contains only live or uncertain ownership. Proven ordinary destruction removes the current hot record. Failed destroy retains the exact record as `Uncertain` plus `UnprovenDestruction { context, context_epoch }`.

The 258-generation hostile fixture proves the same raw isolation/context values may be reused after proven destruction while `BrowserContextEpoch` remains monotonic. Immediately after destruction a predecessor authority fails `ContextNotOwned`; after recreation it fails `AuthorityMismatch`. Across aggregate recreation, `BrowserSessionIncarnation` rejects stale authority even when raw ids and local epochs alias.

Removing a proven-destroyed record is not durable history deletion. Cross-process recovery and buyer audit history remain separate persistence concerns.

## Abandonment and lifecycle completion

`BoundBrowserSession<P>` and recovery custody are `#[must_use]` linear owners. `Drop` performs no browser I/O. Dropping unresolved custody increments only the process-local `abandoned_bound_session_count()` signal.

Successful exact-fact reconciliation updates underlying aggregate state before eventual drop. If every fact and uncertain owner is retired, the aggregate is `Ended`, so dropping a fully reconciled recovery wrapper is not reported as unresolved abandonment.

## Navigation authority interaction

The #317 lineage admits observed navigation only for the exact active `(BrowserSessionIncarnation, BrowsingContextId, BrowserContextEpoch)` generation. Admission revokes presentation authority without adapter I/O and mints opaque `NavigationSettlementAuthority`. Commit is non-terminal; positive settlement, typed negative terminal, and download start share one exactly-once closure. A newer navigation supersedes an older witness. Explicit re-establishment alone consumes the next presentation epoch.

Presentation mutation and lifecycle cleanup are separate. A navigation-invalidated presentation capability cannot authorize mutation or authority-based cleanup; the exact ordinary bound lifecycle owner may still destroy its retained context without reopening presentation authority. Recovery settlement remains separate from navigation settlement and cannot recreate navigation or presentation authority.

## Browser-issued identity and standards trace

`DisposableIsolationId` maps to WebDriver BiDi `browser.UserContext` and preserves protocol text losslessly. OriginWeave does not normalize that address or treat it as command authority.

The authoritative W3C TR verified for this lineage on 2026-09-16 identifies **14 September 2026** as the current published WebDriver BiDi Working Draft (`WD-webdriver-bidi-20260914`) and **9 September 2026** (`WD-webdriver-bidi-20260909`) as the previous published version. The mutable Editor's Draft remains separate at https://w3c.github.io/webdriver-bidi/, and the Chromium/runtime compatibility revision is a third, independently qualified provenance axis. A command acknowledgement is insufficient proof that a disposable boundary is gone.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| structural lifecycle-port ownership; no public raw adapter | `BoundBrowserSession`; lifecycle hostile tests |
| aggregate-issued create attempt | `DisposableContextCreateRequest::attempt_epoch`; transaction fixture |
| same consumed adapter for ordinary work | `AuthorizedContextOperationPort`; `authorized_context_operation.rs` |
| same consumed adapter for recovery commands | `RecoveryContextOperationPort`; `RecoveryContextOperationRequest`; `recovery_same_adapter_operation.rs` |
| recovery command requires one current fact | `recovery_operation_exact_fact_scope.rs`; `tests/test_browser_session_recovery_operation_contract.py` |
| foreign/stale operation fact rejected before I/O | `recovery_operation_exact_fact_scope.rs` |
| command success/failure does not settle ownership | `recovery_same_adapter_operation.rs` |
| terminal settlement closes recovery command path | `recovery_operation_terminal_closure.rs` |
| exact proof-bearing recovery fact | `RecoveryFact`; `RecoverySettlementRequest`; `RecoverySettlementPort`; `settle_recovery_fact` |
| replay/sibling/foreign proof cases | `recovery_exact_fact_settlement.rs` |
| recovery settlement surface remains opaque/current | `tests/test_browser_session_recovery_settlement_contract.py` |
| unproven destroy preserves exact handle + epoch | `UnprovenDestruction`; destroy-failure tests |
| ownership-clean transport loss cannot mint recovery | `recovery_handoff_requires_unresolved_ownership.rs` |
| proven destruction bounds hot ownership | `proven_destroy_releases_hot_ownership.rs` |
| recovery custody cannot regain ordinary authority | recovery rustdoc `compile_fail`; repository contracts |
| navigation witness is opaque and generation-bound | navigation owner tests; `tests/test_browser_session_navigation_owner_surface_contract.py` |
| current vs previous W3C publication provenance | `tests/test_browser_session_webdriver_bidi_publication_trace.py`; ADR 0114; this trace |

Historical predecessor CI/review receipts do not transfer to the current head. Active-PR source remains non-shipment until exact-head repository contracts, canonical rustfmt, locked tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage at 100%, required review/security gates, and protected `main` integration are observed.

## Current integration boundary

#317 owns Browser Session domain policy, same-adapter exact-fact recovery-command custody, and generic exact-fact settlement. #318/#321 own stacked hostile navigation/ABA acceptance and doctoring only. #316 owns WebDriver BiDi proof qualification, pending/accepted/quarantined state, event correlation, remote liveness, protocol-specific recovery semantics, and real-browser adapter integration.

Real Chromium navigation, interaction, cleanup, recovery, and browser-observed post-condition evidence remains required before shipment. Immutable release, signed artifact, SBOM, provenance, reproducibility, and rollback evidence remain separate release gates.
