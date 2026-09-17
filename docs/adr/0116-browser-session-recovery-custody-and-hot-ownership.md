# ADR 0116: Browser Session recovery custody and bounded hot ownership

- Status: Proposed
- Date: 2026-09-15
- Last amended: 2026-09-16
- Extends: ADR 0114
- Owning bounded context: `originweave-browser-session`

## Context

ADR 0114 establishes Browser Session as the owner of disposable-context lifecycle authority. One concrete lifecycle adapter is consumed into `BoundBrowserSession<P>`; raw protocol identifiers never mint create, presentation, navigation, cleanup, or recovery authority.

Recovery introduces three additional constraints.

First, unresolved ownership must retain the exact adapter instance that observed the remote state. Reconstructing a second adapter from identifiers breaks lifecycle custody, while exposing raw `P` or an unrestricted callback creates an authority escape.

Second, recovery command execution and recovery completion are different operations. A browser or driver command ACK is not proof that remote ownership has been reconciled. Browser Session therefore needs a proof-bearing settlement transition that retires exactly one current recovery fact only after independently qualified evidence has been verified by the retained adapter.

Third, even while recovery remains open, a generic recovery command must not receive authority or evidence broader than the fact that justifies that command. Passing the full recovery ledgers to every adapter operation discloses unrelated sibling recovery facts and lets an operation run without presenting the Browser Session-issued fact it is meant to reconcile. Recovery operations therefore require one opaque current `RecoveryFact`, are validated before adapter I/O, and receive only the selected fact.

WebDriver BiDi pending/accepted/quarantined tuple truth, protocol event correlation, replay qualification, remote-liveness interpretation, and concrete proof semantics remain #316 responsibilities. Durable cross-process persistence also remains outside the in-memory Browser Session aggregate.

## Decision drivers

- Preserve the exact consumed adapter across unresolved ownership without making it ambient.
- Require one current Browser Session-issued `RecoveryFact` for each generic recovery operation.
- Reject foreign, stale, replayed, shifted, or out-of-range operation facts before adapter I/O.
- Expose only the selected recovery fact to the adapter command path; do not disclose sibling ledgers.
- Keep adapter command success/failure separate from independent recovery proof.
- Revoke generic recovery I/O after final proof-bearing settlement reaches `Ended`.
- Do not mint recovery custody from ownership-clean `TransportLost`.
- Bind settlement to one opaque current fact, not a raw vector index or protocol identifier.
- Retire only the exact independently verified fact; preserve sibling uncertainty.
- Keep identity-oriented and create-attempt recovery facts independently addressable.
- Never restore ordinary create, navigation, presentation, cleanup, or generic recovery-command authority after reconciliation.
- Preserve failed-destroy ownership and epoch evidence until independently qualified reconciliation proves it gone.
- Keep command-authority hot state bounded to live or uncertain ownership.

## Authority boundaries

Browser Session owns lifecycle identity, ownership state, context epochs, ordinary lifecycle/presentation admission, navigation-generation custody, one-way transition into recovery custody, opaque `RecoveryFact` issuance, current-fact validation, deterministic exact-fact retirement, recovery-revision advancement, and terminal revocation of recovery-command authority.

`BoundBrowserSessionRecovery<P>` owns the same concrete adapter instance but is not a protocol-specific recovery engine. It exposes read-only recovery evidence and opaque fact issuance. It does not expose raw `P`, the inner `BoundBrowserSession`, ordinary Browser Session methods, or a caller-provided callback over the adapter.

When `P: RecoveryContextOperationPort`, `execute_recovery_context_operation(fact, operation)` is available only while the aggregate remains `RecoveryRequired` or evidence-bearing `TransportLost`. Before adapter I/O Browser Session validates:

1. the fact belongs to the exact Browser Session id and process-local incarnation;
2. the fact was issued at the current recovery revision;
3. the selected ledger/index still addresses a current recovery fact.

`RecoveryContextOperationRequest<O>` is then privately constructed with Browser Session id, incarnation, unresolved state, the **one selected** identity-oriented or create-attempt fact, and the adapter-defined operation. It does not contain full recovery vectors. Foreign facts return `RecoveryContextOperationError::AuthorityMismatch`; stale or no-longer-current facts return `RecoveryContextOperationError::StaleFact`; both fail before adapter I/O. Adapter success or `RecoveryContextOperationError::Adapter(E)` leaves Browser Session evidence unchanged. Reusing the same current fact for retries is allowed until a successful settlement advances the revision.

When `P: RecoverySettlementPort`, `settle_recovery_fact(fact, proof)` applies the same exact-fact identity/revision/address validation. The retained adapter verifies independently qualified proof carried by `RecoverySettlementRequest<P>`. Browser Session, not the adapter, commits retirement of the selected fact and then advances the monotonic recovery revision. Every previously issued `RecoveryFact`, including unrelated sibling handles, becomes stale after that mutation.

Once both recovery ledgers and uncertain hot ownership are empty, the aggregate becomes terminal `Ended`. `execute_recovery_context_operation` then returns `RecoveryContextOperationError::RecoveryClosed` before adapter I/O. Complete recovery never recreates `Active` or ordinary browser authority.

The only bridge receiving `&mut P` remains crate-private `BoundBrowserSession::dispatch_recovery_operation`. `RecoveryContextOperationRequest`, `RecoveryFact`, and `RecoverySettlementRequest` have no public construction path.

#316 owns WebDriver BiDi proof qualification. A `browsingContext.contextDestroyed` event, session-loss observation, liveness conclusion, or tuple transition is not automatically proof merely because it came from the protocol. #316 decides which observations can satisfy `RecoverySettlementPort::Proof`; Browser Session consumes only its already-qualified proof under the deterministic exact-fact contract.

## Options considered

### Return raw `P` or expose an unrestricted callback

Rejected. Either form recreates ambient adapter capability outside Browser Session authority.

### Clone or reconstruct the adapter for recovery

Rejected. Equal endpoint, credentials, or identifiers do not prove same lifecycle instance or pending protocol state.

### Treat any `TransportLost` as recovery authority

Rejected. Transport loss proves liveness loss, not unresolved remote ownership. Ownership-clean transport loss cannot mint a recovery command path.

### Execute a recovery operation without a `RecoveryFact`

Rejected. State-level recovery custody is too broad to authorize an arbitrary operation. It allows the caller to reach the retained adapter without identifying the exact unresolved fact that justifies the command.

### Pass both full recovery ledgers to every recovery operation

Rejected. It violates purpose limitation and least authority by disclosing sibling recovery facts unrelated to the selected command. The operation request carries exactly one selected fact.

### Treat successful command execution as reconciliation proof

Rejected. Command completion is not a browser-observed post-condition. Recovery commands never mutate Browser Session uncertainty directly.

### Keep generic recovery I/O callable after complete settlement

Rejected. Once unresolved facts are gone, the purpose that justified retained-adapter access is gone. Terminal `Ended` closes that route before another adapter call.

### Let the adapter delete recovery evidence directly

Rejected. Protocol data must not rewrite Browser Session ownership truth.

### Identify a recovery fact by raw vector index

Rejected. Retiring one fact shifts indices. `RecoveryFact` carries a monotonic revision; successful settlement invalidates all prior handles.

### Keep previously issued sibling facts valid after another fact settles

Rejected. That would make index-shift replay ambiguous. Callers must reread current custody after mutation.

### Clear both recovery ledgers when one condition is proven

Rejected. Identity/ownership uncertainty and create-attempt transaction uncertainty are independent facts and retire independently.

### Restore an ordinary `BoundBrowserSession<P>` after reconciliation

Rejected. Recovery is a one-way boundary. Complete reconciliation reaches `Ended`, never `Active`.

### Keep every proven-destroyed context as a permanent hot tombstone

Rejected. Authorization state and durable audit history have different retention requirements. Proven ordinary destruction removes hot ownership while monotonic epochs reject stale authority.

## Decision

1. `BoundBrowserSession::into_recovery(self)` is the only transition into recovery-only custody.
2. It succeeds from `RecoveryRequired`, or from `TransportLost` only while exact unresolved recovery/create-attempt evidence remains.
3. Handoff moves the existing `BoundBrowserSession<P>` and exact same adapter instance without browser I/O.
4. Recovery custody exposes lifecycle state, read-only evidence, and opaque current-revision `RecoveryFact` issuance; it exposes no raw adapter or ordinary Browser Session authority.
5. `RecoveryContextOperationPort` is the generic recovery-command boundary. Every call supplies a `RecoveryFact` plus adapter-defined operation.
6. Operation validation rejects foreign or stale facts before adapter I/O and sends only the selected current fact in `RecoveryContextOperationRequest`.
7. Operation success/failure does not settle, erase, or mutate Browser Session recovery state.
8. `RecoverySettlementPort` is the generic proof-verification boundary. Protocol-specific proof vocabulary remains adapter-owned.
9. `settle_recovery_fact` validates session/incarnation/revision/exact current fact before proof I/O, checks next-revision capacity, verifies proof through the exact retained adapter, then retires only the selected fact.
10. Verifier failure is non-mutating. Successful retirement advances the recovery revision and invalidates all previously issued handles.
11. Identity-oriented and create-attempt ledgers retire independently.
12. `UnprovenDestruction`, `RecoveryRequiredOwnedHandle`, and `TransportLossOwnedHandle` may retire the exact matching `Uncertain` hot ownership record. Candidate/partial-create evidence cannot consume a distinct accepted owner merely because remote values alias.
13. Partial settlement preserves every unrelated sibling fact and keeps recovery open.
14. When both ledgers and uncertain ownership are empty, Browser Session reaches terminal `Ended`; ordinary and generic recovery-command authority stay closed.
15. Proven ordinary destruction removes the live hot record. Failed destruction retains `Uncertain` ownership plus exact `UnprovenDestruction { context, context_epoch }` evidence.
16. Proven destruction may release raw browser identities for later reuse only under a new monotonic `BrowserContextEpoch`; predecessor authority cannot revive after ABA reuse.
17. `Drop` performs no browser I/O. Unresolved custody preserves process-local abandonment accounting only.
18. Browser-observed navigation remains a separate generation-qualified authority machine; recovery settlement cannot recreate navigation or presentation authority.
19. ADR 0116 remains `Proposed` until this complete slice reaches protected `main` with exact-head gates and independently observed real-browser recovery/destruction/navigation post-conditions.

## Consequences

Recovery now has two distinct least-authority paths on the same retained adapter: an exact-fact-scoped command path and an exact-fact proof-settlement path. The command path can be retried while its fact remains current but sees no sibling recovery evidence. Settlement changes the revision, forcing all pre-existing handles to be reacquired and preventing replay after index shifts.

The revision is deliberately coarse: settling any fact invalidates every handle from the previous revision. Recovery fact counts are expected to be small, and correctness at this security boundary takes precedence over preserving stale handles.

Hot ownership remains proportional to current live/uncertain state rather than historical throughput. Durable history and cross-process recovery require a separate authorized persistence owner.

## Failure and degraded behavior

A rejected `into_recovery` performs no I/O and returns the original bound owner.

A recovery operation using a foreign or stale fact fails before adapter I/O. Adapter failure preserves custody and evidence. Adapter success also preserves custody and evidence; it is not proof. After final settlement reaches `Ended`, another operation returns `RecoveryClosed` before I/O.

A settlement with a foreign, stale, replayed, or out-of-range fact fails before proof-verifier I/O. Proof rejection occurs after verifier I/O but before domain mutation. In both cases recovery ledgers remain unchanged.

A failed destruction never retires hot ownership. Transport loss preserves active handles as non-authorizing evidence and cannot itself prove destruction. Transport loss with no unresolved browser state creates no recovery custody.

If monotonic incarnation, context epoch, navigation generation, or recovery revision allocation exhausts, allocation fails closed rather than wrapping authority identity.

## Security / privacy / governance impact

Recovery custody is a capability-reduction boundary. Untrusted page data, model output, protocol identifiers, adapter-selected handles, recovery evidence, command acknowledgements, and model judgment cannot reconstruct deterministic Browser Session authority.

Exact-fact command validation prevents a generic recovery operation from using custody alone as authority. Selected-fact-only requests also avoid disclosing unrelated recovery facts to the adapter, reducing purpose-unrelated propagation of browser identifiers or other recovery metadata. Session/incarnation/revision/exact-address validation occurs before adapter I/O, so foreign or stale handles cannot turn adapter command execution into an oracle or mutation channel.

This ADR does not move EgressWeave, Wardnet, Keyverse, contextual-orchestrator, Chromium sandboxing, or WebDriver BiDi protocol truth into Browser Session.

## Tests and acceptance evidence

- `crates/originweave-browser-session/src/recovery.rs`
  - one-way `into_recovery`
  - `RecoveryContextOperationRequest<O>` / `RecoveryContextOperationPort`
  - `RecoveryFact`
  - `RecoverySettlementRequest<P>` / `RecoverySettlementPort`
  - pre-I/O exact-fact validation and terminal `RecoveryClosed`
- `crates/originweave-browser-session/tests/recovery_operation_exact_fact_scope.rs`
  - selected-fact-only operation request
  - stale fact after settlement rejected before adapter I/O
  - foreign fact rejected before adapter I/O
- `crates/originweave-browser-session/tests/recovery_same_adapter_operation.rs`
  - same-adapter command path for identity and create-attempt facts
  - command success/failure is non-settling
- `crates/originweave-browser-session/tests/recovery_operation_terminal_closure.rs`
  - final settlement closes generic recovery I/O before another adapter call
- `crates/originweave-browser-session/tests/recovery_exact_fact_settlement.rs`
  - exact one-fact settlement, replay/sibling/foreign rejection, proof-failure non-mutation, independent ledgers, terminal closure
- `tests/test_browser_session_recovery_operation_contract.py`
  - nonconstructible selected-fact request surface and hostile exact-fact fixtures
- `tests/test_browser_session_recovery_settlement_contract.py`
  - opaque settlement API and validation order
- `docs/traceability/browser-session-lifecycle-authority.md`
- `docs/uml/browser-session-lifecycle-authority.md`

These remain active-PR contracts until the exact head passes repository contracts, canonical rustfmt, locked tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage at 100%, required review, and protected-main integration.

## Migration and rollback

Consumers of the prior active-PR recovery command signature must reacquire a current `RecoveryFact` and pass it to `execute_recovery_context_operation(fact, operation)`. Adapters must treat `RecoveryContextOperationRequest` as one selected fact, not a snapshot of both ledgers.

If the selected-fact command boundary cannot be supported safely, rollback means removing the generic recovery-command surface and retaining read-only recovery custody plus proof-bearing settlement. Rollback must not restore raw adapter access, whole-ledger command requests, caller-constructible fact handles, or command-ACK settlement.

## Follow-up

#316 must consume this boundary without copying Browser Session source. Its next integration slice must bind WebDriver BiDi pending/accepted/quarantined tuple and remote-liveness evidence to the current `RecoveryFact`, qualify protocol-specific proof for `RecoverySettlementPort`, and prove with pinned Chromium that command ACK and browser-observed post-condition remain distinct.
