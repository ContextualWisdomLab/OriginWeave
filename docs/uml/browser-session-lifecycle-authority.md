# Browser Session lifecycle authority

This diagram describes the active-PR domain contract for issue #312. It is not evidence that a WebDriver BiDi or Chromium adapter already implements the port.

```mermaid
sequenceDiagram
    autonumber
    participant C as Application service
    participant S as BrowserSession aggregate
    participant BS as BoundBrowserSession
    participant P as DisposableContextPort / AuthorizedContextOperationPort
    participant B as Browser adapter (planned)

    C->>S: start(valid BrowserSessionId)
    S->>S: allocate BrowserSessionIncarnation
    C->>S: bind_lifecycle_port(port by value)
    S-->>C: BoundBrowserSession owns aggregate + exact port
    Note over S,P: binding invokes no adapter callback; no public raw port accessor

    C->>BS: create_disposable_context()
    BS->>S: require Active + reserve monotonic epoch
    S->>S: mint DisposableContextCreateRequest(session, incarnation, attempt epoch)
    S->>P: create_disposable_context(request)
    P->>B: create/stage isolation boundary + browsing context
    B-->>P: unique isolation id + BrowsingContextId or typed create error
    P-->>S: DisposableContextHandle (still pending in adapter)

    alt domain handle accepted
        S->>S: validate no isolation/context alias
        S->>S: mint DisposableContextCreateCompletion(Accepted, exact attempt)
        S->>P: complete_disposable_context_creation(completion)
        P->>P: pending exact attempt → accepted
        S->>S: register exact handle + Active epoch
        S-->>C: PresentationMutationAuthority(session, incarnation, isolation, context, epoch)
    else domain handle rejected
        S->>S: retain duplicate handle + exact rejected attempt
        S->>S: retain every other Active sibling as RecoveryRequiredOwnedHandle
        S->>S: mint DisposableContextCreateCompletion(Rejected, exact attempt)
        S->>P: complete_disposable_context_creation(completion)
        P->>P: pending exact attempt → quarantined/non-authorizing
        S->>S: RecoveryRequired
    else completion cannot be proven
        S->>S: retain UnsettledAdapterHandle + exact unsettled attempt
        S->>S: retain every other Active sibling as RecoveryRequiredOwnedHandle
        S->>S: RecoveryRequired
    end

    C->>BS: execute_authorized_context_operation(authority, operation)
    BS->>S: validate session/incarnation/isolation/context/epoch
    alt authority current
        S-->>BS: exact stored handle
        BS->>BS: mint private AuthorizedContextOperationRequest
        BS->>P: execute_authorized_context_operation(request)
        P->>B: adapter-owned presentation/reconciliation command
        B-->>P: typed adapter result
        P-->>C: output or AuthorizedContextOperationError::Adapter
    else stale or foreign authority
        S-->>C: AuthorizedContextOperationError::BrowserSession
        Note over BS,P: adapter I/O = 0
    end

    Note over C,S: Raw ids, adapter-selected values, diagnostic references, and a second adapter cannot mint lifecycle or presentation authority.

    C->>BS: advance_context_epoch(context_id)
    BS->>S: replace epoch; old authority becomes stale
    S-->>C: new opaque authority carrying same incarnation + isolation

    C->>BS: destroy_disposable_context(authority)
    BS->>S: validate exact session/incarnation/isolation/context/epoch before I/O
    S->>S: mint DisposableContextDestroyRequest with exact stored handle + validated epoch
    S->>P: destroy_disposable_context(request)
    P->>B: remove exact owned isolation boundary
    B-->>P: observed destruction post-condition or DisposableContextDestroyError
    alt destruction proved
        P-->>S: success
        S->>S: remove live hot-ownership record
        Note over S: epoch allocator remains monotonic; retained predecessor authority stays stale
    else destruction unproven
        S->>S: retain UnprovenDestruction(handle, validated epoch)
        S->>S: retain each other Active sibling as RecoveryRequiredOwnedHandle
        S->>S: keep failed record as Uncertain
        S->>S: RecoveryRequired; all active siblings become Uncertain
    end

    C->>BS: finish()
    alt no live or uncertain ownership remains
        BS->>S: end()
        S-->>C: Ended
    else ownership remains
        BS->>S: end()
        S-->>C: ActiveContextRemains
        Note over C,P: same BoundBrowserSession + exact adapter remain available for cleanup/retry
    end
```

`BoundBrowserSession` is a linear lifecycle-port binding. It consumes one concrete port, exposes no public raw `&P`, and exposes no lifecycle method that accepts a replacement port. `AuthorizedContextOperationPort` adds a typed, purpose-bounded post-create operation vocabulary without exposing the adapter itself. Tests retain inert observation state separately from the moved adapter.

`DisposableContextCreateRequest`, `DisposableContextCreateCompletion`, `DisposableContextDestroyRequest`, and `AuthorizedContextOperationRequest` are non-caller-constructible. The create request carries the reserved `BrowserContextEpoch` as a per-create transaction id; Browser Session alone decides whether the returned domain handle is accepted or rejected and validates current authority before any later adapter operation.

For WebDriver BiDi, `DisposableIsolationId` maps to the user-context id created by `browser.createUserContext`. Protocol-specific pending/accepted/quarantined remote tuples and command semantics remain in the BiDi ACL boundary rather than this domain model.

## Recovery, transport, and abandonment state

```mermaid
stateDiagram-v2
    [*] --> Active
    Active --> Active: create candidate + exact Accepted completion + authority
    Active --> Active: authorized operation / current authority / exact bound adapter
    Active --> Active: context epoch advanced / prior authority stale
    Active --> Active: exact owned isolation destruction proved / remove hot record
    Active --> Active: DisposableContextCreateError::CreateFailedClean
    Active --> Active: failed finish / retain same bound owner
    Active --> RecoveryRequired: CreateFailedUncertain / retain attempt provenance
    Active --> RecoveryRequired: duplicate output + exact Rejected completion + sibling RecoveryRequiredOwnedHandle
    Active --> RecoveryRequired: completion unproven / retain UnsettledAdapterHandle + exact attempt + sibling evidence
    Active --> RecoveryRequired: DisposableContextDestroyError / keep failed record Uncertain + sibling evidence
    Active --> Ended: hot ownership empty + finish()
    Active --> TransportLost: browser transport lost / retain TransportLossOwnedHandle / mark uncertain
    RecoveryRequired --> RecoveryRequired: transport_lost = true / preserve stronger recovery state
    RecoveryRequired --> RecoveryCustody: into_recovery(self) / move exact adapter + evidence / no I/O
    TransportLost --> RecoveryCustody: into_recovery(self) / move exact adapter + evidence / no I/O
    RecoveryCustody --> [*]: persist or protocol-reconcile elsewhere; no ordinary authority surface
    Ended --> [*]

    note right of RecoveryRequired
      BrowserSessionRecoveryEvidence retains known
      partial identity, duplicate/unsettled handle,
      exact unproven-destruction handle + epoch, and
      RecoveryRequiredOwnedHandle for indirect siblings.
      DisposableContextCreateRecoveryEvidence retains
      create-attempt epoch/disposition separately.
      Neither evidence family grants I/O authority.
    end note

    note right of RecoveryCustody
      BoundBrowserSessionRecovery<P> exposes only
      state + exact non-authorizing evidence.
      It does not expose raw P, BrowserSession,
      create, presentation authority, epoch advance,
      destroy, authorized operation, or finish.
    end note
```

```mermaid
sequenceDiagram
    autonumber
    participant C as Application service
    participant BS as BoundBrowserSession
    participant R as BoundBrowserSessionRecovery
    participant P as exact bound adapter
    participant O as Operability / recovery observer

    C->>BS: create accepted remote ownership
    alt premature finish
        C->>BS: finish()
        BS-->>C: ActiveContextRemains; wrapper retained
        C->>BS: destroy exact authority
        BS->>P: proven remote destruction
        BS->>BS: remove live hot-ownership record
        C->>BS: finish()
        BS-->>C: Ended
    else unresolved ownership enters recovery
        C->>BS: destroy failure or record_transport_loss()
        BS-->>C: RecoveryRequired or TransportLost + exact evidence
        C->>BS: into_recovery(self)
        BS-->>R: move exact non-Clone adapter + evidence; adapter I/O = 0
        Note over R,P: recovery custody cannot regain ordinary lifecycle/presentation authority
    else ordinary wrapper abandonment
        C-xBS: drop without proven cleanup
        Note over BS,P: Drop performs no browser I/O
        BS->>O: increment abandoned_bound_session_count()
        Note over O: process-local signal only; not destruction proof or durable exact-handle storage
    end
```

Recovery custody is deliberately narrower than protocol reconciliation. #316 remains responsible for WebDriver BiDi pending/accepted/quarantined tuple truth and any purpose-bounded protocol recovery operation that uses the exact adapter held by recovery custody. Durable crash/process-restart persistence remains open until a canonical recovery owner stores exact recovery evidence before process termination.

## Same-raw-identity hot-state hostile case

```mermaid
sequenceDiagram
    autonumber
    participant C as Application service
    participant BS as BoundBrowserSession
    participant P as exact lifecycle port

    C->>BS: create U/C
    BS->>P: create(attempt epoch 1) + Accepted
    BS-->>C: authority epoch 1
    C->>BS: destroy(authority epoch 1)
    BS->>P: destroy exact U/C + epoch 1
    P-->>BS: destruction proved
    BS->>BS: remove U/C from hot ownership

    C->>BS: destroy(retained authority epoch 1)
    BS-->>C: ContextNotOwned
    Note over BS,P: stale check rejects before adapter I/O

    C->>BS: create same raw U/C again
    BS->>P: create(attempt epoch 2) + Accepted
    BS-->>C: authority epoch 2
    C->>BS: destroy(retained authority epoch 1)
    BS-->>C: AuthorityMismatch
    Note over BS,P: same raw values cannot resurrect predecessor epoch
```

The hostile acceptance repeats this create → proven destroy → same-handle recreate cycle for 258 ownership generations. Hot command-authority state remains bounded to live/uncertain ownership instead of accumulating proven-destroyed tombstones. Durable audit/history retention is a separate persistence concern.

## Sequential ABA hostile case

```mermaid
sequenceDiagram
    autonumber
    participant A as BoundBrowserSession A
    participant B as BoundBrowserSession B
    participant PA as Lifecycle port A
    participant PB as Lifecycle port B

    A->>A: start(S) => incarnation A; bind PA
    A->>PA: create(request S, incarnation A, attempt 1)
    PA-->>A: U, C pending
    A->>PA: completion Accepted(attempt 1)
    A->>PA: destroy(request S, incarnation A, U/C)
    A->>A: finish()

    B->>B: start(S) => incarnation B; bind PB
    B->>PB: create(request S, incarnation B, attempt 1)
    PB-->>B: same U, same C pending
    B->>PB: completion Accepted(attempt 1)
    Note over A,B: local attempt/epoch may both equal 1, but incarnations differ
    B->>B: validate retained authority A
    B-->>A: AuthorityMismatch before PB adapter I/O
    B->>PB: operate/destroy only with authority B + incarnation B
```

`RecoveryRequired` and `TransportLost` remain closed to normal lifecycle and presentation authority. `into_recovery(self)` is a one-way custody transfer, not a command-authority resurrection path; later protocol reconciliation must stay purpose-bounded and must not infer cleanup authority from raw identifiers or treat command ACK as proof of destruction.