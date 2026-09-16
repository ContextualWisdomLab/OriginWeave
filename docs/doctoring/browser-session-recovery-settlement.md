# Browser Session recovery settlement boundary

Status: active-PR design evidence for #317. This document does not claim protected-main adoption, executable GREEN, browser acceptance, or release readiness.

## Problem

`BoundBrowserSessionRecovery<P>` already preserves the exact consumed adapter and permits purpose-bounded recovery I/O without exposing raw `P`. That solves custody and dispatch, but not recovery completion. Adapter success or command acknowledgement deliberately leaves `RecoveryRequired` and both recovery ledgers unchanged because an I/O return value is not proof that remote browser ownership was destroyed or reconciled.

Without a second-stage settlement transition, a protocol owner such as #316 can independently qualify browser evidence but cannot retire the matching Browser Session uncertainty. The only alternatives are unsafe: erase all evidence on adapter success, reconstruct ordinary authority from raw identifiers, or abandon a recovery owner that can never reach a terminal condition.

## Constraints

Browser Session owns deterministic lifecycle state and exact fact consumption. WebDriver BiDi remains an adapter and evidence source; its navigation ids, user-context ids, event ordering and liveness rules do not become Browser Session policy authority.

A settlement boundary must therefore satisfy all of the following:

- one opaque Browser Session-issued handle addresses exactly one current recovery fact;
- the handle is bound to the exact Browser Session incarnation and a current recovery-ledger revision;
- foreign session/incarnation handles fail before adapter proof-verification I/O;
- settlement of one fact invalidates every previously issued fact handle so index movement or sibling removal cannot redirect a stale handle;
- proof verification happens through the exact retained adapter, but successful verification retires only the fact named by the already validated handle;
- failed proof verification leaves lifecycle state and both recovery ledgers unchanged;
- identity-oriented `BrowserSessionRecoveryEvidence` and transaction-oriented `DisposableContextCreateRecoveryEvidence` remain separately addressable and separately retired;
- partial settlement preserves every unrelated sibling fact;
- complete settlement may close recovery custody, but it never recreates `Active`, `PresentationMutationAuthority`, navigation authority, normal create authority, or an ordinary lifecycle owner.

## Alternatives rejected

Treating `RecoveryContextOperationPort` success as settlement is rejected because transport/protocol command completion is not independent proof of remote destruction or reconciliation.

Passing a raw vector index is rejected because removal of one fact can make an old index address a different sibling fact. A current-revision opaque handle is required to make stale replay fail closed.

Allowing the adapter to delete Browser Session evidence directly is rejected because it moves domain ownership truth into an adapter and makes protocol data authoritative over policy state.

Returning from recovery custody to ordinary `BoundBrowserSession<P>` is rejected because reconciliation must not resurrect create, presentation, navigation or ordinary cleanup authority after uncertainty has crossed the recovery boundary.

## Test-first contract

Exact #317 commit `738ec7d9a6635a8b4b0b9324026c2f0b433b9c77` introduces `crates/originweave-browser-session/tests/recovery_exact_fact_settlement.rs` as a structural RED. The fixture intentionally references settlement types and methods that do not yet exist.

The hostile cases require:

1. two uncertain owned contexts: settling A preserves B; replay of A and a sibling handle issued before the revision change fail before proof I/O; rereading B permits proof verification; a wrong proof does not mutate B; exact B settlement closes only after no recovery facts remain;
2. a fact issued by another Browser Session cannot reach the target session's proof verifier even when the caller possesses the opaque value;
3. uncertain create evidence carried in the identity and create-attempt ledgers requires two independent settlements rather than one broad erase.

The current head is RED by construction. Draft-policy skipped CI is not evidence that the test compiled or failed for the intended reason. Production implementation must not begin by weakening this fixture or by treating a successful generic recovery operation as proof.

## Intended minimal production shape

The implementation should stay inside the Browser Session bounded context and expose only protocol-agnostic concepts: an opaque recovery-fact handle, an opaque proof request passed to a narrow settlement-verification port, typed stale/foreign/adapter failures, and exact current-fact removal after successful verification.

A monotonic recovery-ledger revision is preferable to trying to preserve stable vector indices. Every successful settlement increments the revision, making all previously issued handles stale. Validation order is aggregate/session-incarnation identity, current revision, exact fact identity, adapter proof verification, then mutation. No adapter call occurs before the first three checks succeed.

When one recovery fact represents an owned context, successful settlement must also retire only the matching uncertain hot-ownership record. When no ownership or create-attempt facts remain, the recovery wrapper may reach terminal `Ended`. It must not return an ordinary owner or mint a presentation epoch.

## Ownership handoff

#317 owns this generic settlement boundary. #316 remains responsible for deciding what WebDriver BiDi observation constitutes acceptable proof, for pending/accepted/quarantined tuple correlation, event replay handling and remote liveness. The dependency order is therefore #317 RED → minimal Browser Session settlement implementation → exact-head executable GREEN → #318/#321 acceptance → ordinary non-force #316 adoption → pinned-Chromium recovery and post-condition evidence.

Any implementation that copies #316 protocol tuple state into Browser Session, consumes mutable sibling source, or treats command ACK as proof violates this boundary.
