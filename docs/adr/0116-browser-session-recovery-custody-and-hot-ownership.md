# ADR 0116: Browser Session recovery custody and bounded hot ownership

- Status: Proposed
- Date: 2026-09-15
- Last amended: 2026-09-16
- Extends: ADR 0114
- Owning bounded context: `originweave-browser-session`

## Context

ADR 0114 establishes Browser Session as the owner of disposable-context lifecycle authority. It binds one concrete lifecycle adapter linearly, validates opaque presentation authority before browser I/O, and retains non-authorizing evidence whenever remote ownership becomes uncertain.

Three follow-on problems are addressed here.

First, a session that enters `RecoveryRequired`, or enters `TransportLost` while unresolved ownership evidence remains, still owns the exact adapter instance that observed the unresolved remote state. Reconstructing another adapter from identifiers would break same-instance custody. Exposing raw `P`, the inner `BrowserSession`, or ordinary lifecycle methods would instead turn recovery into an authority escape.

Second, recovery command success is not recovery completion. A command ACK cannot prove that a remote browser boundary is absent or reconciled. Browser Session therefore needs a second, proof-bearing transition that consumes exactly one current recovery fact only after a protocol owner has independently qualified evidence and the exact retained adapter verifies it.

Third, retaining a permanent `Destroyed` record for every proven-destroyed context would make command-authority hot state grow with historical throughput. Current command admission and durable audit history require different retention semantics.

WebDriver BiDi pending/accepted/quarantined tuple truth, protocol event correlation, replay qualification, remote-liveness interpretation, and the meaning of concrete recovery evidence remain adapter concerns owned by #316. Durable cross-process persistence is also separate from the in-memory Browser Session hot map.

## Decision drivers

- Preserve the exact consumed adapter across unresolved ownership without making it ambient.
- Permit only purpose-bounded recovery I/O; never expose raw `P` or an unrestricted callback.
- Do not mint recovery custody from `TransportLost` when no unresolved ownership fact exists.
- Keep adapter command success/failure separate from independent recovery proof.
- Bind settlement to one opaque Browser Session-issued fact, not a raw vector index or protocol identifier.
- Reject foreign, stale, replayed, or out-of-range recovery facts before proof-verifier I/O.
- Retire only the exact fact that was independently verified; preserve unrelated sibling uncertainty.
- Keep identity-oriented recovery facts and create-attempt facts independently addressable.
- Prevent recovery settlement from restoring ordinary create, navigation, presentation, or cleanup authority.
- Preserve exact failed-destroy ownership and epoch evidence until reconciliation proves that fact gone.
- Keep command-authority hot state bounded to live or uncertain ownership.
- Preserve monotonic stale-authority rejection across raw browser-id reuse.
- Keep durable history and process-restart persistence separate from command admission.

## Authority boundaries

Browser Session owns lifecycle identity, ownership state, context epochs, ordinary lifecycle/presentation admission, navigation-generation custody, the one-way transition into recovery custody, opaque recovery-fact issuance, and deterministic exact-fact retirement.

`BoundBrowserSessionRecovery<P>` owns the same concrete adapter instance but is not a protocol-specific recovery engine. When `P: RecoveryContextOperationPort`, it may execute a purpose-bounded recovery operation through `RecoveryContextOperationRequest<O>`. This path snapshots exact Browser Session identity, incarnation, unresolved state, both recovery-evidence ledgers, and the adapter-defined operation. Adapter success or failure does not mutate Browser Session uncertainty.

When `P: RecoverySettlementPort`, recovery custody may also issue opaque `RecoveryFact` handles and execute `settle_recovery_fact(fact, proof)`. `RecoveryFact` binds the exact Browser Session id, process-local incarnation, ledger kind, current ledger index, and monotonic recovery revision. `RecoverySettlementRequest<P>` is privately constructed only after Browser Session validates that handle against current custody. The exact retained adapter verifies the independently supplied proof. Browser Session, not the adapter, then commits retirement of exactly the selected fact.

`RecoveryContextOperationRequest`, `RecoveryFact`, and `RecoverySettlementRequest` have no public construction path. The crate-private bridge that touches `&mut P` remains `BoundBrowserSession::dispatch_recovery_operation`; external consumers cannot supply arbitrary callbacks or recover raw adapter access.

#316 owns WebDriver BiDi proof qualification. A `contextDestroyed` event, session-loss observation, liveness conclusion, or tuple transition is not automatically proof merely because it came from the protocol. #316 must decide which observations satisfy `RecoverySettlementPort::Proof`; Browser Session consumes only that already-qualified proof under its deterministic exact-fact contract.

Durable crash/process-restart persistence and buyer audit history do not live in `BrowserSession.contexts`. `abandoned_bound_session_count()` is process-local operability evidence only. LLM output, page content, protocol identifiers, recovery evidence, adapter command success, and model judgment never become deterministic Browser Session policy authority.

## Options considered

### Return raw `P` from the failed bound session

Rejected. Raw adapter recovery recreates ambient capability and permits browser commands outside Browser Session authority.

### Expose `&BrowserSession` or `FnOnce(&mut P)` from recovery custody

Rejected. The first can become an indirect capability-minting escape as the aggregate evolves; the second is equivalent to raw adapter access. The adapter bridge remains crate-private.

### Clone or reconstruct the adapter for recovery

Rejected. Equal credentials, endpoint, or identifiers do not establish same lifecycle instance or pending protocol state.

### Treat any `TransportLost` state as recovery authority

Rejected. Transport loss proves only liveness loss. Before any remote ownership, or after every boundary is proven destroyed, there is no unresolved fact to reconcile.

### Treat a successful recovery command as reconciliation proof

Rejected. Command completion is not a browser-observed post-condition. `execute_recovery_context_operation` always preserves Browser Session uncertainty.

### Let the adapter delete recovery evidence directly

Rejected. That would move domain ownership truth into an adapter and let protocol data rewrite policy state.

### Identify a recovery fact by raw vector index

Rejected. Retiring one fact shifts later indices. An old index could then address a different sibling. `RecoveryFact` therefore carries a monotonic revision; successful settlement advances it and invalidates all previously issued fact handles.

### Keep previously issued sibling facts valid after another fact settles

Rejected. It makes index-shift replay ambiguous. Callers must reread current custody after every successful settlement.

### Clear both recovery ledgers when one remote condition is proven

Rejected. `BrowserSessionRecoveryEvidence` records identity/ownership uncertainty while `DisposableContextCreateRecoveryEvidence` records create-attempt transaction uncertainty. They are independent facts and require independent retirement.

### Restore an ordinary `BoundBrowserSession<P>` after reconciliation

Rejected. Crossing the recovery boundary is one-way. Even complete recovery reaches terminal `Ended`; it never recreates `Active`, create authority, presentation authority, navigation authority, or normal lifecycle cleanup authority.

### Keep every proven-destroyed context as a permanent hot tombstone

Rejected. It conflates authorization state with audit history. Exact destruction plus monotonic epochs are sufficient for stale-authority rejection.

### Delete hot ownership after a destroy command ACK

Rejected. Only proven destruction or proof-bearing recovery settlement may retire uncertainty. Failed/unproven destruction retains exact ownership and evidence.

## Decision

1. `BoundBrowserSession::into_recovery(self)` is the only transition into recovery-only custody. It succeeds from `RecoveryRequired`, or from `TransportLost` only when exact `BrowserSessionRecoveryEvidence` or `DisposableContextCreateRecoveryEvidence` remains.
2. `Active`, `Ended`, and ownership-clean `TransportLost` are returned unchanged. Handoff performs no browser I/O.
3. Handoff moves the exact existing `BoundBrowserSession<P>` and same non-`Clone` adapter instance.
4. Recovery custody exposes lifecycle state and exact non-authorizing evidence. It exposes no raw `P`, inner `BoundBrowserSession`, inner `BrowserSession`, ordinary create, presentation lookup, epoch advance, destroy, authorized operation, navigation transition, or normal finish.
5. `RecoveryContextOperationPort` is the only generic recovery-command path. Its request is private-construction and its success/failure leaves all Browser Session recovery state unchanged.
6. `RecoverySettlementPort` is the only generic proof-verification path. Protocol-specific proof vocabulary remains adapter-owned.
7. `recovery_fact(index)` and `create_attempt_recovery_fact(index)` issue opaque current-revision handles only for facts that currently exist.
8. `settle_recovery_fact` validates exact Browser Session id and incarnation before adapter I/O. A foreign fact returns `RecoverySettlementError::AuthorityMismatch`.
9. It then validates the current recovery revision and exact current ledger/index fact before adapter I/O. Replay, sibling handles issued before another settlement, or out-of-range facts return `RecoverySettlementError::StaleFact`.
10. Revision increment capacity is checked before verifier I/O; exhaustion fails closed as `RevisionExhausted`.
11. Only after those checks does Browser Session construct `RecoverySettlementRequest` and call the exact retained adapter's `verify_recovery_settlement`.
12. Verifier failure returns `RecoverySettlementError::Adapter(E)` and mutates neither recovery ledger nor Browser Session lifecycle state.
13. Verifier success retires exactly the selected fact. Identity-oriented and create-attempt ledgers are independent; one settlement never broad-erases both.
14. For `UnprovenDestruction`, `RecoveryRequiredOwnedHandle`, or `TransportLossOwnedHandle`, retirement may also remove the exact matching `Uncertain` hot-ownership record. `PartialCreationIsolation`, `DuplicateAdapterHandle`, and `UnsettledAdapterHandle` retire evidence only and cannot consume an independently accepted same-valued owner.
15. A successful settlement advances the monotonic recovery revision. Every `RecoveryFact` issued before that mutation becomes stale.
16. Partial settlement preserves every unrelated sibling fact and keeps the aggregate in its unresolved state.
17. When both recovery ledgers are empty and no uncertain hot ownership remains, Browser Session reaches terminal `Ended`. It never transitions back to `Active` or restores ordinary browser command authority.
18. The crate-private `dispatch_recovery_operation` remains the only bridge receiving `&mut P`; callers never receive raw adapter access.
19. Negative capability boundaries remain executable contracts through rustdoc `compile_fail` and repository tests.
20. `BrowserSession.contexts` contains only current live or uncertain ownership. Proven ordinary destruction removes a hot ownership record; failed destruction retains it as `Uncertain` plus exact `UnprovenDestruction { context, context_epoch }`.
21. Proven destruction releases raw isolation/context identities for later reuse only under a new monotonic `BrowserContextEpoch`. Predecessor authority therefore cannot revive after ABA reuse.
22. `Drop` performs no browser I/O. Unresolved custody preserves process-local abandonment accounting.
23. Browser-observed navigation remains generation-qualified and protocol-agnostic. Admission revokes presentation authority without adapter I/O; commit is non-terminal; positive settlement, typed negative terminal, and download start share one exactly-once closure; explicit re-establishment consumes a new presentation epoch.
24. Presentation mutation and lifecycle cleanup remain separate. Navigation-invalidated presentation authority cannot mutate or authorize authority-based cleanup, while the exact ordinary bound lifecycle owner may destroy its retained context without reopening presentation authority.
25. This ADR remains `Proposed` until the complete slice reaches protected `main` with exact-head repository gates and independently observed real-browser recovery/destruction/navigation post-conditions.

## Consequences

Browser Session has two linear owner forms: ordinary `BoundBrowserSession<P>` and one-way `BoundBrowserSessionRecovery<P>`. Recovery custody can issue purpose-bounded adapter commands and can consume independently qualified proof, but neither mechanism exposes the adapter or recreates ordinary Browser Session authority.

Recovery settlement is deliberately revision-coarse. Settling any fact invalidates all fact handles issued under the prior revision, including unrelated siblings. This forces callers to reread the current evidence ledger after mutation and prevents index-shift replay at the cost of extra handle acquisition. Recovery fact counts are expected to be small, and correctness at this security boundary dominates preserving stale handles.

Hot ownership remains proportional to current live/uncertain state rather than historical throughput. Durable history must be retained by a separate authorized persistence owner.

The active #317 lineage keeps ownership generation, navigation witness generation, and recovery revision as separate authority dimensions. Raw browser identifiers never substitute for any of them.

## Failure and degraded behavior

A rejected `into_recovery` performs no I/O and returns the original bound owner. A failed recovery command preserves custody and evidence. A successful recovery command also preserves custody and evidence; it is not proof.

A settlement with a foreign, stale, replayed, or out-of-range `RecoveryFact` fails before verifier I/O. A verifier rejection fails after proof I/O but before domain mutation. In both cases the ledgers remain unchanged.

A failed destruction never retires hot ownership. Transport loss preserves active handles as non-authorizing evidence and does not prove destruction. Transport loss with no unresolved browser state creates no recovery custody.

If monotonic incarnation, context epoch, navigation generation, or recovery revision allocation exhausts, allocation fails closed rather than wrapping authority identity.

Dropping unresolved ordinary or recovery custody performs no browser I/O. Process-local abandonment observability may increase, but it is neither cleanup nor durable recovery.

## Security / privacy / governance impact

Recovery custody is a capability-reduction boundary. Untrusted page data, model output, protocol identifiers, adapter-selected handles, and recovery evidence cannot reconstruct ordinary command authority. Deterministic Browser Session policy is never delegated to an LLM.

The exact adapter remains owned and reachable only through purpose-bounded traits. `RecoveryFact` and `RecoverySettlementRequest` are opaque and non-caller-constructible. Session/incarnation/revision/exact-fact validation occurs before proof-verifier I/O, preventing foreign or replayed handles from turning protocol proof checking into an oracle or mutation channel.

Alias-safe hot-state retirement prevents a rejected or partial create that reuses remote values from deleting a distinct previously accepted owner. Resource bounding therefore cannot convert unresolved ownership into an untracked boundary.

This ADR does not move EgressWeave, Wardnet, Keyverse, contextual-orchestrator, Chromium sandboxing, or WebDriver BiDi protocol truth into Browser Session. Recovery evidence may identify remote browser boundaries but creates no new purpose for page-content or PII processing.

## Tests and acceptance evidence

- `crates/originweave-browser-session/src/recovery.rs`
  - `BoundBrowserSession::into_recovery`
  - `BoundBrowserSessionRecovery<P>`
  - `RecoveryContextOperationRequest<O>` / `RecoveryContextOperationPort`
  - `RecoveryFact`
  - `RecoverySettlementRequest<P>` / `RecoverySettlementPort`
  - `RecoverySettlementError<E>`
  - `settle_recovery_fact`
  - negative `compile_fail` capability contracts
- `crates/originweave-browser-session/tests/recovery_owner_handoff.rs`
  - unproven destroy and unresolved transport loss move exact adapter/evidence without I/O
- `crates/originweave-browser-session/tests/recovery_handoff_requires_unresolved_ownership.rs`
  - ownership-clean transport loss cannot mint recovery custody
- `crates/originweave-browser-session/tests/recovery_same_adapter_operation.rs`
  - same-adapter operation; exact provenance; success/failure do not clear uncertainty
- `crates/originweave-browser-session/tests/recovery_exact_fact_settlement.rs`
  - single-fact settlement preserves siblings
  - replay and predecessor sibling handles fail stale before proof I/O
  - foreign session/incarnation fact fails before proof I/O
  - proof failure is non-mutating
  - identity and create-attempt ledgers retire independently
  - complete reconciliation reaches terminal `Ended` without authority resurrection
- `tests/test_browser_session_recovery_operation_contract.py`
  - recovery-operation adapter custody and nonconstructible request surface
- `tests/test_browser_session_recovery_settlement_contract.py`
  - opaque settlement API, pre-I/O validation order, external hostile fixture, and ADR/trace/UML/doctoring currentness
- `crates/originweave-browser-session/tests/proven_destroy_releases_hot_ownership.rs`
  - 258 same-handle generations; monotonic epochs; predecessor rejection
- navigation owner tests and `tests/test_browser_session_navigation_owner_surface_contract.py`
  - current-witness revocation/closure/re-establishment and cleanup separation

These are active-PR contracts until the exact head passes repository contracts, canonical rustfmt, locked tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage at 100%, required review, and protected-main integration.

## Migration and rollback

Dependents must adopt this foundation by ordinary non-force restack after the parent exact head is verified. They must not copy Browser Session source, infer recovery authority from raw identifiers, or reconstruct a second adapter.

Rollback before protected-main adoption reverts the recovery-custody, settlement, hot-retirement, and navigation-authority slice together with its hostile fixtures and ADR. Selectively restoring raw adapter access, evidence-free recovery custody, ACK-as-proof, raw-index settlement, or authority resurrection is not a valid rollback. After protected-main adoption, rollback requires another policy-compliant change that preserves unresolved ownership evidence and demonstrates that predecessor capabilities cannot revive.

## Open follow-ups

- #316: after #317 exact-head executable GREEN, ordinary non-force adoption of the generic recovery boundary; implement WebDriver BiDi proof qualification, pending/accepted/quarantined correlation, event replay qualification, remote liveness, and pinned-Chromium recovery post-condition evidence.
- #318/#321: executable acceptance/review successors for the #317 navigation contract and same-raw-id/sibling-recreation matrices.
- Durable crash/process-restart persistence of exact recovery facts and buyer-required audit history.
- Real Chromium proof of remote destruction, cleanup, navigation, interaction, recovery, and browser-observed post-conditions.
- `docs/product-technical-gap-baseline.md` must distinguish active-PR implementation from protected-main/release evidence.
- Protected-main immutable release, signed artifacts, SBOM, provenance, reproducibility, and rollback evidence.

## Supersession / reversal conditions

A successor may supersede this ADR only if it preserves at least: same-instance recovery custody; no ambient adapter escape; evidence-gated recovery capability; command-ACK/proof separation; opaque exact-fact settlement; pre-I/O foreign/stale rejection; sibling preservation; independent ledger retirement; alias-safe hot ownership; terminal non-resurrection; bounded command-authority state; monotonic ABA rejection; generation-bound navigation custody; presentation/lifecycle cleanup separation; and separation of durable history from command admission.

Changing the recovery owner or persistence architecture alone does not justify weakening those guarantees.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

Rust Project Developers. (2026). *The Rust Programming Language: Ownership*. https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
