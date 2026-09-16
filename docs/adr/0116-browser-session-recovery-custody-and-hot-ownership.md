# ADR 0116: Browser Session recovery custody and bounded hot ownership

- Status: Proposed
- Date: 2026-09-15
- Last amended: 2026-09-16
- Extends: ADR 0114
- Owning bounded context: `originweave-browser-session`

## Context

ADR 0114 establishes that Browser Session owns disposable-context lifecycle authority, binds one concrete lifecycle adapter linearly, validates opaque presentation authority before browser I/O, and retains non-authorizing recovery evidence when remote state is uncertain. Two follow-on architecture questions remained once that contract was implemented.

First, a session that enters `RecoveryRequired` or `TransportLost` still owns the exact adapter instance that observed the unresolved remote state. Reconstructing a second adapter from identifiers would break the same-instance boundary; exposing raw `P`, the inner `BrowserSession`, or ordinary lifecycle methods would instead turn recovery into an authority escape. Recovery nevertheless needs a narrow way to perform protocol-owner-defined reconciliation through that same retained adapter.

Second, retaining a permanent `Destroyed` record for every proven-destroyed context makes command-authority hot state grow with historical activity. That is unnecessary for authority admission once exact destruction has been proven, but deleting an unproven record would lose ownership evidence. Command-authority state and durable audit/history therefore require different retention semantics.

These questions are Browser Session domain concerns. WebDriver BiDi pending/accepted/quarantined tuples and protocol-specific recovery command semantics remain adapter concerns owned by #316. Durable cross-process recovery persistence is also separate from the in-memory hot map.

## Decision drivers

- Preserve the exact consumed adapter across unresolved ownership without making it generally accessible again.
- Permit only purpose-bounded recovery I/O through that retained adapter; never expose raw `P` or an unrestricted callback.
- Keep recovery evidence non-authorizing and unchanged by a mere adapter success/failure.
- Prevent `RecoveryRequired` or `TransportLost` from becoming an alternate normal lifecycle path.
- Preserve exact failed-destroy ownership and epoch evidence until reconciliation proves the boundary gone.
- Keep command-authority admission bounded by current live/uncertain ownership rather than historical throughput.
- Permit browser reuse of the same raw user-context/browsing-context identity only as a new monotonic ownership generation.
- Reject retained stale authority before adapter I/O after both destruction and same-raw-identity recreation.
- Keep durable audit/history and process-restart persistence separate from the hot authorization map.

## Assumptions and authority boundaries

Browser Session owns lifecycle identity, ownership state, context epochs, admission of ordinary lifecycle/presentation authority, navigation-generation custody, and the one-way transition into recovery custody. `BoundBrowserSessionRecovery<P>` owns custody of the same adapter instance but does not become a protocol-specific recovery engine.

The recovery wrapper may execute an adapter-defined operation only when `P: RecoveryContextOperationPort`. Browser Session constructs an opaque `RecoveryContextOperationRequest` immediately before I/O, snapshots the exact `BrowserSessionId`, `BrowserSessionIncarnation`, current unresolved `BrowserSessionState`, `BrowserSessionRecoveryEvidence`, and `DisposableContextCreateRecoveryEvidence`, and routes it through the same retained adapter. The request has no public constructor and is not ordinary create/destroy/presentation authority.

WebDriver BiDi correlation, pending/accepted/quarantined tuples, remote-liveness interpretation, protocol event replay qualification, and protocol recovery command meaning remain #316 responsibilities. #316 may map its recovery vocabulary into `RecoveryContextOperationPort::Operation`; Browser Session does not inspect or own that protocol vocabulary.

Durable crash/process-restart persistence and buyer audit history are not stored in the hot `BrowserSession.contexts` map. `abandoned_bound_session_count()` is process-local operability evidence only. LLM output, page content, raw protocol identifiers, recovery evidence, and a successful recovery adapter call do not grant Browser Session command authority or prove remote destruction.

## Options considered

### Return raw `P` from the failed bound session

Rejected. Raw adapter recovery recreates ambient capability and permits callers to issue protocol commands outside Browser Session authority.

### Expose `&BrowserSession` from recovery custody

Rejected. Even a read-only projection exposes methods that can become an indirect presentation-authority lookup surface as the aggregate evolves. Recovery custody exposes only explicit non-authorizing projections and the purpose-bounded recovery operation surface.

### Expose `FnOnce(&mut P)` or another generic callback

Rejected. A generic callback is equivalent to raw adapter escape. The callback bridge that touches `&mut P` is crate-private and callable only by the recovery wrapper after constructing the opaque request.

### Clone or reconstruct the adapter for recovery

Rejected. Same credentials, endpoint, or identifier do not prove same lifecycle instance. A second adapter can diverge from the pending remote transaction that produced the evidence.

### Treat a successful recovery adapter call as reconciliation proof

Rejected. Command success alone does not prove that remote ownership was destroyed or reconciled. Browser Session state and evidence remain unchanged until a separately reviewed proof-bearing transition exists.

### Keep every proven-destroyed context as a permanent hot tombstone

Rejected. It makes authority-admission state grow with historical throughput and conflates authorization with audit retention. Monotonic epochs plus exact validation are sufficient to reject predecessor capabilities after a proven destroy and same-identity recreation.

### Delete records after any destroy command acknowledgement

Rejected. A command ACK is not proof that the disposable browser boundary is gone. Failed or otherwise unproven destruction must retain uncertain ownership and exact recovery evidence.

### Reset context epochs when raw identifiers are reused

Rejected. Raw identifier reuse is an ABA case. Reusing the predecessor epoch could make retained authority current again.

## Decision

1. `BoundBrowserSession::into_recovery(self)` is the only Browser Session transition into recovery-only custody. It succeeds only from `BrowserSessionState::RecoveryRequired` or `BrowserSessionState::TransportLost`.
2. `Active` and `Ended` sessions are returned unchanged. Recovery custody cannot be selected as an alternate path around ordinary lifecycle policy.
3. A successful handoff moves the exact existing `BoundBrowserSession<P>` and therefore the same non-`Clone` adapter instance. The handoff performs no browser I/O, no create, no destroy, and no implicit cleanup.
4. Recovery custody is represented by `BoundBrowserSessionRecovery<P>`. It exposes lifecycle `state()`, exact `BrowserSessionRecoveryEvidence`, exact `DisposableContextCreateRecoveryEvidence`, and—only when `P: RecoveryContextOperationPort`—`execute_recovery_context_operation(operation)`.
5. `BoundBrowserSessionRecovery<P>` exposes neither raw `P`, `BoundBrowserSession<P>`, nor `BrowserSession`. It provides no ordinary create, presentation-authority lookup, context-epoch advancement, authority-based or owner-based destroy, ordinary authorized operation, navigation transition, or normal-finish surface.
6. `RecoveryContextOperationRequest<O>` is non-caller-constructible. It snapshots exact session id, incarnation, unresolved state, both recovery-evidence ledgers, and the adapter-owned purpose-bounded operation before adapter I/O.
7. The only bridge that receives `&mut P` is `BoundBrowserSession::dispatch_recovery_operation`, which is `pub(crate)` and lives in the Browser Session owner module. External consumers cannot invoke it or supply a closure.
8. `RecoveryContextOperationPort` extends `DisposableContextPort` with adapter-owned `Operation`, `Output`, and `Error` types. Browser Session routes the opaque request but does not interpret protocol semantics.
9. `RecoveryContextOperationError::Adapter(E)` preserves typed adapter failure. Adapter success and failure both leave Browser Session lifecycle state and recovery evidence unchanged; neither is destruction/reconciliation proof.
10. Negative capability boundaries are executable contracts. Rustdoc `compile_fail` examples and repository contracts must fail if recovery custody can regain an ordinary lifecycle or presentation-authority path.
11. Protocol-specific recovery commands are not added to Browser Session. #316 consumes the generic recovery boundary for WebDriver BiDi pending/accepted/quarantined correlation, event replay qualification, and remote-liveness semantics.
12. `BrowserSession.contexts` is hot command-authority state, not durable audit history. It contains only current live or uncertain ownership records.
13. `destroy_disposable_context` validates the exact current authority before adapter I/O. Only after the adapter proves destruction does Browser Session remove the corresponding hot ownership record.
14. If destruction is not proven, the record remains present as `Uncertain`, `UnprovenDestruction { context, context_epoch }` remains exact and enumerable, and normal authority stays closed.
15. Proven destruction releases the raw isolation/context identity for a later create attempt. Recreation reserves the next monotonic `BrowserContextEpoch`; a retained predecessor authority therefore cannot become current again merely because the browser reused the same raw identifiers.
16. Immediately after proven destruction, retained authority for that record fails before adapter I/O as `ContextNotOwned`. If the same raw identity is later recreated, the retained predecessor fails before adapter I/O as `AuthorityMismatch` because its epoch is stale.
17. Removal from hot command-authority state is not deletion of durable business/audit history. Durable process-restart recovery, evidence retention, and audit storage must be implemented by a separately authorized persistence owner and must not infer destruction from record eviction or from `abandoned_bound_session_count()`.
18. `Drop` on ordinary or recovery custody performs no browser I/O. The contained bound owner retains the same process-local abandonment accounting for unresolved remote ownership.
19. Browser-observed navigation is admitted only for the exact active `(BrowserSessionIncarnation, BrowsingContextId, BrowserContextEpoch)` ownership generation. Admission revokes presentation authority with zero adapter I/O and mints an opaque `NavigationSettlementAuthority` using a separate monotonic navigation generation rather than spending a presentation epoch.
20. Commit progress is non-terminal. Positive settlement, typed `Aborted`/`Failed`, and download start share one exactly-once terminal closure. A newer admitted navigation supersedes an older pending witness; stale or superseded witnesses fail closed without changing a newer generation.
21. Terminal closure creates one context-local opportunity for explicit `reestablish_presentation_authority`; the first successful re-establishment consumes the next presentation epoch and returns the context to `Established`. Generic epoch advancement cannot bypass a navigation-invalidated state.
22. Presentation authority and lifecycle cleanup are separate. A navigation-invalidated presentation capability cannot authorize mutation or authority-based cleanup, while the exact bound lifecycle owner may still destroy its retained disposable context through `destroy_owned_disposable_context` without reopening presentation authority.
23. This ADR remains `Proposed` until the change reaches protected `main` and real-browser recovery/destruction/navigation post-conditions are independently evidenced.

## Consequences

The Browser Session aggregate now has two linear owner forms: ordinary `BoundBrowserSession<P>` and one-way `BoundBrowserSessionRecovery<P>`. Recovery custody can perform only the adapter-owned recovery operations admitted by `RecoveryContextOperationPort`; it cannot expose the adapter, regain ordinary Browser Session authority, or independently decide that protocol recovery is complete.

Proven destruction makes hot ownership proportional to current live/uncertain state rather than the total number of historical context generations. This reduces long-lived session state without weakening stale-authority rejection. Durable history must be captured elsewhere when required; it is not implicitly provided by the command-authority map.

The active #317 production lineage treats ownership generation and current navigation witness as separate authority dimensions. Reusing a raw browsing-context id after proven destruction does not carry predecessor lifecycle or navigation authority forward. Navigation liveness uses its own monotonic generation; presentation epochs advance only when explicit re-establishment succeeds.

## Navigation authority interaction

`record_observed_navigation` is a protocol-agnostic Browser Session transition. It accepts already-qualified adapter evidence only after aggregate trust, exact incarnation, exact live ownership, and current context epoch match. It does not consume a WebDriver BiDi navigation id as policy authority. #316 remains responsible for mapping protocol events and replay qualification into this domain transition.

The returned `NavigationSettlementAuthority` has private fields and no caller constructor. `record_observed_navigation_committed` records first commit progress but does not make presentation re-establishment eligible. `record_observed_navigation_settled`, `record_observed_navigation_terminated`, and `record_observed_navigation_download_started` share the same current-witness terminal closure. `reestablish_presentation_authority` is explicit and single-use. `RecoveryRequired`, `TransportLost`, `Ended`, proven context destruction, stale generations, and superseded witnesses all dominate terminal or re-establishment attempts before adapter I/O.

## Failure and degraded behavior

If `into_recovery(self)` is called while the aggregate is `Active` or `Ended`, no transition occurs and the original `BoundBrowserSession<P>` is returned to the caller. No adapter I/O occurs during either a successful or rejected handoff.

A failed recovery operation returns `RecoveryContextOperationError::Adapter` and preserves the same custody and evidence. A successful recovery operation returns the adapter-defined output but also preserves the same custody and evidence; a separate owner transition is required before uncertainty can be cleared.

A failed destruction never retires the hot ownership record. The exact record becomes or remains `Uncertain`, exact `UnprovenDestruction { context, context_epoch }` evidence is retained, normal authority is closed, and the aggregate enters or remains in recovery. A transport loss preserves owned handles as non-authorizing evidence and does not prove destruction.

Dropping unresolved ordinary or recovery custody performs no browser I/O. Process-local abandonment observability may increase, but this is neither cleanup nor durable recovery. If monotonic epoch/incarnation/navigation-generation allocation is exhausted, allocation fails closed rather than reusing authority identity.

## Security / privacy / governance impact

The recovery wrapper is a capability-reduction boundary. Untrusted page data, model output, protocol identifiers, recovery evidence, and adapter-selected handles cannot reconstruct ordinary Browser Session authority. The exact adapter remains owned and is callable only through the purpose-bounded recovery trait; raw `P` never becomes ambient.

Bounded hot-state retirement occurs only after exact pre-I/O authority validation and proven destruction. Therefore resource-bounding cannot convert uncertain remote ownership into an untracked boundary.

Navigation events are evidence, not deterministic policy authority. Browser Session creates and consumes its own opaque navigation witness only after exact current ownership validation. This prevents raw protocol ids, delayed events, sibling contexts, prior incarnations, or superseded generations from rewriting current presentation authority.

This ADR does not move EgressWeave, Wardnet, Keyverse, contextual-orchestrator, Chromium sandboxing, or WebDriver BiDi protocol truth into OriginWeave Browser Session. Recovery evidence may identify remote browser boundaries but does not itself contain page content or create a new purpose for PII processing.

## Tests and acceptance evidence

- `crates/originweave-browser-session/src/recovery.rs`
  - `BoundBrowserSession::into_recovery`
  - `BoundBrowserSessionRecovery<P>`
  - `RecoveryContextOperationRequest<O>`
  - `RecoveryContextOperationPort`
  - `RecoveryContextOperationError<E>`
  - negative `compile_fail` capability contracts
- `crates/originweave-browser-session/tests/recovery_owner_handoff.rs`
  - unproven destroy moves the exact adapter and exact evidence without I/O
  - transport loss moves the exact adapter and exact evidence without I/O
- `crates/originweave-browser-session/tests/recovery_same_adapter_operation.rs`
  - recovery operation executes through the exact retained adapter
  - exact session/incarnation/state and both evidence ledgers reach the opaque request
  - adapter failure preserves custody/evidence
  - adapter success is not treated as cleanup proof
- `crates/originweave-browser-session/tests/proven_destroy_releases_hot_ownership.rs`
  - 258 same-handle ownership generations remain admissible after proven destruction
  - epochs are strictly monotonic
  - retained predecessor authority fails before lifecycle adapter I/O
- destroy-failure tests require `Uncertain` ownership plus exact `UnprovenDestruction` evidence when destruction is not proven.
- navigation owner tests cover revocation, current-witness commit/terminal ordering, supersession, cleanup separation, trust-state precedence, and presentation-epoch conservation.
- `tests/test_browser_session_lifecycle_contract.py` pins recovery wrapper surface, hostile fixtures, ADR 0116, traceability, and UML doctoring.
- `tests/test_browser_session_navigation_owner_surface_contract.py` pins the production-owner navigation API and its opaque witness/epoch-separation contract independently from stacked #318/#321 acceptance.

These are active-PR contracts until the exact head passes repository contracts, canonical rustfmt, locked tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage at 100%, required review, and protected-main integration.

## Migration and rollback

This active-PR change is additive at the ownership-type boundary but changes the internal retention, recovery-operation, and navigation-admission model. Dependents must adopt it by ordinary non-force restack after the parent exact head is verified; they must not copy Browser Session source or infer recovery/navigation authority from adapter identifiers.

Rollback before protected-main adoption is performed by reverting the whole recovery-custody/hot-retirement/recovery-operation/navigation-authority slice together with its hostile fixtures and ADR, not by selectively restoring raw adapter access, keeping record eviction without stale-authority tests, or restoring raw-id navigation authority. After protected-main adoption, rollback requires a policy-compliant change that preserves all unresolved ownership evidence and demonstrates that predecessor capabilities cannot revive.

## Open follow-ups

- #316: adopt the released/verified Browser Session recovery-operation boundary and implement purpose-bounded WebDriver BiDi recovery operations, pending/accepted/quarantined correlation, event replay qualification, and remote-liveness semantics without reconstructing an adapter.
- #318/#321: executable acceptance/review successors for the implemented #317 navigation contract, including same-raw-id ABA and sibling-recreation matrices; they do not own a second production state machine.
- Durable crash/process-restart persistence of exact recovery evidence and buyer-required audit history.
- Real Chromium proof of remote destruction, cleanup, navigation, interaction, recovery, and browser-observed post-conditions.
- `docs/product-technical-gap-baseline.md` and release evidence must stay synchronized with protected-main truth; active-PR implementation is not shipment.
- Protected-main immutable release, SBOM, provenance, reproducibility, and rollback evidence.

## Supersession / reversal conditions

Supersede this ADR only when a later Accepted design provides at least equivalent guarantees for same-instance recovery custody, zero ambient adapter escape, purpose-bounded same-adapter recovery I/O, exact uncertain-ownership retention, bounded command-authority hot state, monotonic stale-authority rejection across raw-id reuse, generation-bound navigation witness custody, single-assignment terminal closure, presentation/lifecycle cleanup separation, and separation of durable history from command admission. A protocol adapter that merely offers different identifiers, reconnects to the same endpoint, or reports command success does not satisfy those guarantees.

Changing the recovery owner or persistence architecture does not by itself require restoring destroyed tombstones to the hot map; the replacement must state how command authority remains bounded and how durable evidence is retained independently.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

Rust Project Developers. (2026). *The Rust Programming Language: Ownership*. https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
