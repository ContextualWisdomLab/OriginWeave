# Browser Session recovery settlement boundary

Status: active-PR implementation evidence for #317. Source implementation exists, but this document does not claim protected-main adoption, executable GREEN, browser acceptance, or release readiness until the exact head passes repository gates.

## Problem

`BoundBrowserSessionRecovery<P>` preserves the exact consumed adapter without exposing raw `P`. Recovery command execution and recovery completion are deliberately separate: an adapter return or browser command ACK is not proof that remote ownership is absent or reconciled.

The settlement boundary therefore lets a protocol owner such as #316 independently qualify browser evidence, submit it against one Browser Session-issued `RecoveryFact`, and retire only that exact uncertainty after the retained adapter verifies the proof.

Two follow-on capability gaps were found while hardening this boundary.

First, the final settlement could move the aggregate to terminal `Ended` while `BoundBrowserSessionRecovery<P>` still exposed generic recovery I/O. That route is now closed by a pre-I/O lifecycle-state gate returning `RecoveryContextOperationError::RecoveryClosed`.

Second, the still-open recovery command path originally needed only the aggregate recovery state and an adapter-defined operation. `RecoveryContextOperationRequest` then copied **both complete recovery ledgers** into the adapter request. That meant one recovery command could reach the retained adapter without naming the exact unresolved fact that justified it and could observe unrelated sibling facts. The current repair requires a Browser Session-issued current `RecoveryFact` for every operation and sends only that selected fact to the adapter.

## Constraints and invariants

Browser Session owns deterministic lifecycle state and exact fact validation/consumption. WebDriver BiDi remains an adapter and evidence source; protocol ids, tuple state, event ordering, command ACKs and liveness conclusions do not become Browser Session policy authority.

The implementation preserves these invariants:

- one opaque `RecoveryFact` addresses exactly one current identity-oriented or create-attempt recovery fact;
- the handle binds Browser Session id, process-local incarnation, ledger kind, index and current recovery revision;
- every generic recovery operation supplies a current `RecoveryFact` plus adapter-defined operation;
- foreign operation facts fail as `RecoveryContextOperationError::AuthorityMismatch` before adapter I/O;
- stale, replayed, shifted or out-of-range operation facts fail as `RecoveryContextOperationError::StaleFact` before adapter I/O;
- `RecoveryContextOperationRequest` contains only the selected fact. It never snapshots sibling recovery vectors;
- operation success/failure leaves lifecycle state and both recovery ledgers unchanged;
- the same current fact may authorize retry attempts until settlement advances the revision;
- settlement uses the same session/incarnation/revision/exact-address validation before proof-verifier I/O;
- successful settlement increments the revision, invalidating every handle issued before that mutation;
- failed proof verification leaves lifecycle state and both ledgers unchanged;
- identity-oriented `BrowserSessionRecoveryEvidence` and transaction-oriented `DisposableContextCreateRecoveryEvidence` remain separately addressable and retired;
- owned-context settlement retires only the exact matching uncertain hot record; candidate/partial-create evidence cannot consume an independently owned context merely because values alias;
- partial settlement preserves unrelated sibling facts;
- when both recovery ledgers and uncertain ownership are empty, the aggregate reaches terminal `Ended` and never recreates ordinary or generic recovery-command authority;
- terminal operation attempts return `RecoveryClosed` before the retained adapter is called.

## Alternatives rejected

Treating `RecoveryContextOperationPort` success as settlement is rejected because transport/protocol command completion is not independent proof of remote destruction or reconciliation.

Allowing `execute_recovery_context_operation(operation)` without a fact is rejected because aggregate recovery state alone is too broad to authorize retained-adapter I/O.

Copying both recovery ledgers into every `RecoveryContextOperationRequest` is rejected because it violates least authority and purpose limitation: the adapter learns sibling uncertainty that the selected operation does not need.

Keeping generic recovery I/O callable after complete settlement is rejected because the retained adapter would remain an ambient browser-I/O capability after its recovery purpose ended.

Passing a raw vector index is rejected because removal of one fact can make an old index address a different sibling. A current-revision opaque handle makes index-shift replay fail closed.

Allowing the adapter to delete Browser Session evidence directly is rejected because it moves domain ownership truth into an adapter and makes protocol data authoritative over policy state.

Returning from recovery custody to ordinary `BoundBrowserSession<P>` is rejected because reconciliation must not resurrect create, presentation, navigation or cleanup authority after uncertainty crossed the recovery boundary.

## Test-first contract and source repair

`recovery_exact_fact_settlement.rs` established proof-bearing exact-fact settlement: sibling preservation, stale replay rejection, foreign-fact pre-I/O rejection, verifier-failure non-mutation, independent identity/create-attempt ledgers, and terminal `Ended` without authority resurrection.

`recovery_operation_terminal_closure.rs` established that generic recovery I/O works while a current recovery purpose exists, but final exact-fact settlement closes the command route before another adapter call.

The current hardening adds `recovery_operation_exact_fact_scope.rs`. Its structural RED required the command API to accept `RecoveryFact`, disclose only the selected fact to `RecoveryContextOperationPort`, reject a fact issued before another settlement as `StaleFact` before operation I/O, and reject a foreign Browser Session fact as `AuthorityMismatch` before operation I/O.

Production `recovery.rs` now shares exact-fact selection logic between command execution and settlement. `RecoveryContextOperationRequest` uses optional selected identity/create-attempt evidence fields rather than full vectors. `recovery_same_adapter_operation.rs` covers both ledger kinds while preserving the rule that command success or failure is non-settling. The terminal fixture now passes the same exact fact to the command before settling it and confirms `RecoveryClosed` takes precedence after terminal closure.

Repository execution remains the next gate. Until the current exact head actually runs and passes repository contracts, rustfmt, locked tests, strict Clippy, rustdoc and production coverage, this is source-level repair evidence rather than executable GREEN.

## Validation order

For `execute_recovery_context_operation(fact, operation)`:

1. recovery custody must still be `RecoveryRequired` or `TransportLost`; otherwise `RecoveryClosed`;
2. exact Browser Session id and incarnation must match;
3. recovery revision must still match;
4. the fact must still address a current entry in its ledger;
5. only then is `RecoveryContextOperationRequest` built and the retained adapter invoked.

Steps 2–4 map to `AuthorityMismatch` or `StaleFact` and occur before adapter I/O. The request carries only the selected fact. Adapter return does not mutate recovery state.

For `settle_recovery_fact(fact, proof)`:

1. exact Browser Session id and incarnation;
2. current recovery revision and exact current ledger/index fact;
3. next-revision capacity;
4. retained-adapter proof verification;
5. exact evidence retirement and, for owned-context evidence, exact uncertain ownership retirement;
6. monotonic revision advance and terminal `Ended` only when no uncertainty remains.

No proof-verifier call occurs before the exact-fact checks succeed. A failed verifier consumes nothing. A successful mutation invalidates all previously issued fact handles.

## Ownership handoff

#317 owns generic same-adapter recovery custody, exact-fact-scoped recovery commands, proof-bearing exact-fact settlement and terminal closure. #316 remains responsible for deciding what WebDriver BiDi observation constitutes acceptable proof, pending/accepted/quarantined tuple correlation, event replay handling, remote liveness and concrete recovery-command semantics.

The dependency order remains #317 exact-head executable GREEN → #318/#321 acceptance → ordinary non-force #316 adoption → pinned-Chromium recovery and browser-observed post-condition evidence. Any implementation that copies #316 protocol tuple state into Browser Session, treats command ACK as proof, leaks sibling recovery facts into unrelated commands, or preserves generic adapter I/O after terminal recovery violates this boundary.
