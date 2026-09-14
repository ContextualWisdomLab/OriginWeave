# ADR 0115: Browser Session recovery custody and bounded hot ownership

- Status: Proposed
- Date: 2026-09-15
- Extends: ADR 0114
- Owning bounded context: `originweave-browser-session`

## Context

ADR 0114 establishes that Browser Session owns disposable-context lifecycle authority, binds one concrete lifecycle adapter linearly, validates opaque presentation authority before browser I/O, and retains non-authorizing recovery evidence when remote state is uncertain. Two follow-on architecture questions remained once that contract was implemented.

First, a session that enters `RecoveryRequired` or `TransportLost` still owns the exact adapter instance that observed the unresolved remote state. Reconstructing a second adapter from identifiers would break the same-instance boundary; exposing raw `P`, the inner `BrowserSession`, or ordinary lifecycle methods would instead turn recovery into an authority escape.

Second, retaining a permanent `Destroyed` record for every proven-destroyed context makes command-authority hot state grow with historical activity. That is unnecessary for authority admission once exact destruction has been proven, but deleting an unproven record would lose ownership evidence. Command-authority state and durable audit/history therefore require different retention semantics.

These questions are Browser Session domain concerns. WebDriver BiDi pending/accepted/quarantined tuples and protocol-specific recovery commands remain adapter concerns owned by #316. Durable cross-process recovery persistence is also separate from the in-memory hot map.

## Decision drivers

- Preserve the exact consumed adapter across unresolved ownership without making it generally accessible again.
- Keep recovery evidence non-authorizing.
- Prevent `RecoveryRequired` or `TransportLost` from becoming an alternate normal lifecycle path.
- Preserve exact failed-destroy ownership and epoch evidence until reconciliation proves the boundary gone.
- Keep command-authority admission bounded by current live/uncertain ownership rather than historical throughput.
- Permit browser reuse of the same raw user-context/browsing-context identity only as a new monotonic ownership generation.
- Reject retained stale authority before adapter I/O after both destruction and same-raw-identity recreation.
- Keep durable audit/history and process-restart persistence separate from the hot authorization map.

## Assumptions and authority boundaries

Browser Session owns lifecycle identity, ownership state, context epochs, admission of ordinary lifecycle/presentation authority, and the one-way transition into recovery custody. `BoundBrowserSessionRecovery<P>` owns custody of the same adapter instance but does not become a protocol-specific recovery engine.

WebDriver BiDi correlation, pending/accepted/quarantined tuples, remote-liveness interpretation, and any protocol recovery command remain #316 responsibilities. A future protocol recovery operation must be purpose-bounded against recovery custody rather than reconstructing an adapter from raw ids.

Durable crash/process-restart persistence and buyer audit history are not stored in the hot `BrowserSession.contexts` map. `abandoned_bound_session_count()` is process-local operability evidence only. LLM output, page content, raw protocol identifiers, and recovery evidence do not grant Browser Session command authority.

## Options considered

### Return raw `P` from the failed bound session

Rejected. Raw adapter recovery recreates ambient capability and permits callers to issue protocol commands outside Browser Session authority.

### Expose `&BrowserSession` from recovery custody

Rejected. Even a read-only projection exposes methods that can become an indirect presentation-authority lookup surface as the aggregate evolves. Recovery custody exposes only explicit non-authorizing projections.

### Clone or reconstruct the adapter for recovery

Rejected. Same credentials, endpoint, or identifier do not prove same lifecycle instance. A second adapter can diverge from the pending remote transaction that produced the evidence.

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
4. Recovery custody is represented by `BoundBrowserSessionRecovery<P>`. It exposes only lifecycle `state()`, exact `BrowserSessionRecoveryEvidence`, and exact `DisposableContextCreateRecoveryEvidence`.
5. `BoundBrowserSessionRecovery<P>` exposes neither raw `P`, `BoundBrowserSession<P>`, nor `BrowserSession`. It provides no ordinary create, presentation-authority lookup, context-epoch advancement, destroy, authorized-operation, or normal-finish surface.
6. Negative capability boundaries are executable contracts. Rustdoc `compile_fail` examples and repository contracts must fail if recovery custody can regain an ordinary lifecycle or presentation-authority path.
7. Protocol-specific recovery commands are not added to Browser Session. #316 may define purpose-bounded WebDriver BiDi recovery operations that consume the exact adapter held by recovery custody, but protocol tuple truth and command semantics remain outside the Browser Session aggregate.
8. `BrowserSession.contexts` is hot command-authority state, not durable audit history. It contains only current live or uncertain ownership records.
9. `destroy_disposable_context` validates the exact current authority before adapter I/O. Only after the adapter proves destruction does Browser Session remove the corresponding hot ownership record.
10. If destruction is not proven, the record remains present as `Uncertain`, `UnprovenDestruction { context, context_epoch }` remains exact and enumerable, and normal authority stays closed.
11. Proven destruction releases the raw isolation/context identity for a later create attempt. Recreation reserves the next monotonic `BrowserContextEpoch`; a retained predecessor authority therefore cannot become current again merely because the browser reused the same raw identifiers.
12. Immediately after proven destruction, retained authority for that record fails before adapter I/O as `ContextNotOwned`. If the same raw identity is later recreated, the retained predecessor fails before adapter I/O as `AuthorityMismatch` because its epoch is stale.
13. Removal from hot command-authority state is not deletion of durable business/audit history. Durable process-restart recovery, evidence retention, and audit storage must be implemented by a separately authorized persistence owner and must not infer destruction from record eviction or from `abandoned_bound_session_count()`.
14. `Drop` on recovery custody performs no browser I/O. The contained bound owner retains the same process-local abandonment accounting for unresolved remote ownership.
15. This ADR remains `Proposed` until the change reaches protected `main` and real-browser recovery/destruction post-conditions are independently evidenced.

## Consequences

The Browser Session aggregate now has two linear owner forms: ordinary `BoundBrowserSession<P>` and one-way `BoundBrowserSessionRecovery<P>`. Recovery custody deliberately cannot complete the protocol-specific recovery by itself; that operation belongs to the WebDriver BiDi ACL/adapter owner and must remain purpose-bounded.

Proven destruction makes hot ownership proportional to current live/uncertain state rather than the total number of historical context generations. This reduces long-lived session state without weakening stale-authority rejection. Durable history must be captured elsewhere when required; it is not implicitly provided by the command-authority map.

A child navigation implementation must treat ownership generation and current navigation witness as separate authority dimensions. Reusing a raw browsing-context id after proven destruction does not carry predecessor lifecycle or navigation authority forward.

## Failure and degraded behavior

If `into_recovery(self)` is called while the aggregate is `Active` or `Ended`, no transition occurs and the original `BoundBrowserSession<P>` is returned to the caller. No adapter I/O occurs during either a successful or rejected handoff.

A failed destruction never retires the hot ownership record. The exact record becomes or remains `Uncertain`, exact `UnprovenDestruction { context, context_epoch }` evidence is retained, normal authority is closed, and the aggregate enters or remains in recovery. A transport loss preserves owned handles as non-authorizing evidence and does not prove destruction.

Dropping unresolved ordinary or recovery custody performs no browser I/O. Process-local abandonment observability may increase, but this is neither cleanup nor durable recovery. If monotonic epoch/incarnation allocation is exhausted, allocation fails closed rather than reusing authority identity.

## Security / privacy / governance impact

The recovery wrapper is a capability-reduction boundary. Untrusted page data, model output, protocol identifiers, recovery evidence, and adapter-selected handles cannot reconstruct ordinary Browser Session authority. The exact adapter remains owned but not ambiently callable.

Bounded hot-state retirement occurs only after exact pre-I/O authority validation and proven destruction. Therefore resource-bounding cannot convert uncertain remote ownership into an untracked boundary.

This ADR does not move EgressWeave, Wardnet, Keyverse, contextual-orchestrator, Chromium sandboxing, or WebDriver BiDi protocol truth into OriginWeave Browser Session. Recovery evidence may identify remote browser boundaries but does not itself contain page content or create a new purpose for PII processing.

## Tests and acceptance evidence

- `crates/originweave-browser-session/src/recovery.rs`
  - `BoundBrowserSession::into_recovery`
  - `BoundBrowserSessionRecovery<P>`
  - negative `compile_fail` capability contracts
- `crates/originweave-browser-session/tests/recovery_owner_handoff.rs`
  - unproven destroy moves the exact adapter and exact evidence without I/O
  - transport loss moves the exact adapter and exact evidence without I/O
- `crates/originweave-browser-session/tests/proven_destroy_releases_hot_ownership.rs`
  - 258 same-handle ownership generations remain admissible after proven destruction
  - epochs are strictly monotonic
  - retained predecessor authority fails before lifecycle adapter I/O
- destroy-failure tests require `Uncertain` ownership plus exact `UnprovenDestruction` evidence when destruction is not proven.
- `tests/test_browser_session_lifecycle_contract.py` pins the recovery wrapper surface, hostile fixtures, ADR 0115, traceability, and UML doctoring.

These are active-PR contracts until the exact head passes repository contracts, canonical rustfmt, locked tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage at 100%, required review, and protected-main integration.

## Migration and rollback

This active-PR change is additive at the ownership-type boundary but changes the internal retention model after proven destruction. Dependents must adopt it by ordinary non-force restack after the parent exact head is verified; they must not copy Browser Session source or infer recovery authority from the new wrapper.

Rollback before protected-main adoption is performed by reverting the whole recovery-custody/hot-retirement slice together with its hostile fixtures and ADR, not by selectively restoring raw adapter access or keeping record eviction without the stale-authority tests. After protected-main adoption, rollback requires a policy-compliant change that preserves all unresolved ownership evidence and demonstrates that predecessor capabilities cannot revive.

## Open follow-ups

- #316: purpose-bounded WebDriver BiDi recovery operations through the exact recovery-held adapter, including pending/accepted/quarantined correlation and remote-liveness semantics.
- #318/#321: production navigation ownership-generation/current-witness state plus same-raw-id ABA acceptance after this foundation receives exact-head verification.
- Durable crash/process-restart persistence of exact recovery evidence and buyer-required audit history.
- Real Chromium proof of remote destruction, cleanup, navigation, interaction, and browser-observed post-conditions.
- `docs/product-technical-gap-baseline.md` and release evidence must stay synchronized with protected-main truth; active-PR implementation is not shipment.
- Protected-main immutable release, SBOM, provenance, reproducibility, and rollback evidence.

## Supersession / reversal conditions

Supersede this ADR only when a later Accepted design provides at least equivalent guarantees for same-instance recovery custody, zero ambient adapter escape, exact uncertain-ownership retention, bounded command-authority hot state, monotonic stale-authority rejection across raw-id reuse, and separation of durable history from command admission. A protocol adapter that merely offers different identifiers, reconnects to the same endpoint, or reports command success does not satisfy those guarantees.

Changing the recovery owner or persistence architecture does not by itself require restoring destroyed tombstones to the hot map; the replacement must state how command authority remains bounded and how durable evidence is retained independently.

## References

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

Rust Project Developers. (2026). *The Rust Programming Language: Ownership*. https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html