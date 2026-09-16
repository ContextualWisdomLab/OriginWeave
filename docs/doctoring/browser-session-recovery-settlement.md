# Browser Session recovery settlement boundary

Status: active-PR implementation evidence for #317. The source implementation now exists, but this document does not claim protected-main adoption, executable GREEN, browser acceptance, or release readiness until the exact head passes repository gates.

## Problem

`BoundBrowserSessionRecovery<P>` already preserved the exact consumed adapter and permitted purpose-bounded recovery I/O without exposing raw `P`. That solved custody and dispatch, but not recovery completion: adapter success or command acknowledgement deliberately left `RecoveryRequired` and both recovery ledgers unchanged because an I/O return value is not proof that remote browser ownership was destroyed or reconciled.

The settlement repair gives a protocol owner such as #316 a second-stage path: independently qualify browser evidence, submit it against one Browser Session-issued recovery fact, and retire only that exact uncertainty after the retained adapter verifies the proof.

## Constraints

Browser Session owns deterministic lifecycle state and exact fact consumption. WebDriver BiDi remains an adapter and evidence source; its navigation ids, user-context ids, event ordering and liveness rules do not become Browser Session policy authority.

The implemented settlement boundary preserves these invariants:

- one opaque Browser Session-issued `RecoveryFact` addresses exactly one current recovery fact;
- the handle is bound to the exact Browser Session id, process-local incarnation, ledger kind, index and current recovery-ledger revision;
- foreign session/incarnation handles fail before adapter proof-verification I/O;
- a successful settlement increments the revision, so every handle issued before that mutation becomes stale before adapter I/O;
- proof verification runs through the exact retained adapter via `RecoverySettlementPort`, but successful verification retires only the fact named by the already validated handle;
- failed proof verification leaves lifecycle state and both recovery ledgers unchanged;
- identity-oriented `BrowserSessionRecoveryEvidence` and transaction-oriented `DisposableContextCreateRecoveryEvidence` remain separately addressable and separately retired;
- settling an owned-context fact retires only the exact matching uncertain hot-ownership record; candidate/partial-create evidence never consumes an independently owned context merely because remote values alias;
- partial settlement preserves every unrelated sibling fact;
- when both recovery ledgers and uncertain ownership are empty, the aggregate reaches terminal `Ended`; it never recreates `Active`, `PresentationMutationAuthority`, navigation authority, normal create authority, or an ordinary lifecycle owner.

## Alternatives rejected

Treating `RecoveryContextOperationPort` success as settlement is rejected because transport/protocol command completion is not independent proof of remote destruction or reconciliation.

Passing a raw vector index is rejected because removal of one fact can make an old index address a different sibling fact. A current-revision opaque handle makes stale replay fail closed.

Allowing the adapter to delete Browser Session evidence directly is rejected because it moves domain ownership truth into an adapter and makes protocol data authoritative over policy state.

Returning from recovery custody to ordinary `BoundBrowserSession<P>` is rejected because reconciliation must not resurrect create, presentation, navigation or ordinary cleanup authority after uncertainty has crossed the recovery boundary.

## Test-first contract and source repair

Commit `738ec7d9a6635a8b4b0b9324026c2f0b433b9c77` introduced `crates/originweave-browser-session/tests/recovery_exact_fact_settlement.rs` as a structural RED. It requires:

1. two uncertain owned contexts: settling A preserves B; replay of A and a sibling handle issued before the revision change fail before proof I/O; rereading B permits proof verification; a wrong proof does not mutate B; exact B settlement closes only after no recovery facts remain;
2. a fact issued by another Browser Session cannot reach the target session's proof verifier even when the caller possesses the opaque value;
3. uncertain create evidence carried in the identity and create-attempt ledgers requires two independent settlements rather than one broad erase.

The source repair adds the opaque fact/request/error contract and verifier port in `recovery.rs`, plus crate-private aggregate mutation hooks that retire exact evidence and exact matching uncertain ownership only after proof verification. It does not weaken the hostile fixture and does not reinterpret generic recovery-operation success as proof.

Repository execution remains the next gate. Until the current exact head actually runs and passes repository contracts, rustfmt, locked tests, strict Clippy, rustdoc and production coverage, this is source-level GREEN intent rather than executable GREEN evidence.

## Validation order

`settle_recovery_fact` evaluates in this order:

1. exact Browser Session id and incarnation;
2. current recovery-ledger revision;
3. exact current ledger/index fact and next-revision capacity;
4. retained-adapter proof verification;
5. exact evidence retirement and, for owned-context evidence, exact uncertain ownership retirement;
6. monotonic revision advance and terminal `Ended` only when no uncertainty remains.

No adapter call occurs before the first three checks succeed. A failed verifier does not consume the fact. A successful mutation invalidates all previously issued fact handles before another settlement can be accepted.

## Ownership handoff

#317 owns this generic settlement boundary. #316 remains responsible for deciding what WebDriver BiDi observation constitutes acceptable proof, for pending/accepted/quarantined tuple correlation, event replay handling and remote liveness. The dependency order is #317 exact-head executable GREEN → #318/#321 acceptance → ordinary non-force #316 adoption → pinned-Chromium recovery and post-condition evidence.

Any implementation that copies #316 protocol tuple state into Browser Session, consumes mutable sibling source, or treats command ACK as proof violates this boundary.
