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
        S->>S: retain duplicate handle as recovery evidence
        S->>S: retain every other Active sibling as RecoveryRequiredOwnedHandle
        S->>S: mint DisposableContextCreateCompletion(Rejected, exact attempt)
        S->>P: complete_disposable_context_creation(completion)
        P->>P: pending exact attempt → quarantined/non-authorizing
        S->>S: RecoveryRequired
    else completion cannot be proven
        S->>S: retain UnsettledAdapterHandle
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
    S->>S: mint DisposableContextDestroyRequest with exact stored handle
    S->>P: destroy_disposable_context(request)
    P->>B: remove exact owned isolation boundary
    B-->>P: observed destruction post-condition or DisposableContextDestroyError
    alt destruction proved
        P-->>S: success
        S->>S: context = Destroyed
    else destruction unproven
        S->>S: retain UnprovenDestruction for failed handle
        S->>S: retain each other Active sibling as RecoveryRequiredOwnedHandle
        S->>S: RecoveryRequired; all active siblings become Uncertain
    end

    C->>BS: finish()
    alt every owned context Destroyed
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
    Active --> Active: exact owned isolation destruction proved
    Active --> Active: DisposableContextCreateError::CreateFailedClean
    Active --> Active: failed finish / retain same bound owner
    Active --> RecoveryRequired: CreateFailedUncertain / retain known partial isolation
    Active --> RecoveryRequired: duplicate output + exact Rejected completion + sibling RecoveryRequiredOwnedHandle
    Active --> RecoveryRequired: completion unproven / retain UnsettledAdapterHandle + sibling RecoveryRequiredOwnedHandle
    Active --> RecoveryRequired: DisposableContextDestroyError / cleanup unproven + sibling RecoveryRequiredOwnedHandle
    Active --> Ended: all owned contexts Destroyed + finish()
    Active --> TransportLost: browser transport lost / retain TransportLossOwnedHandle / mark uncertain
    RecoveryRequired --> RecoveryRequired: transport_lost = true / preserve recovery evidence
    Ended --> [*]
    RecoveryRequired --> [*]
    TransportLost --> [*]

    note right of RecoveryRequired
      BrowserSessionRecoveryEvidence retains known
      partial identity, duplicate/unsettled handle,
      exact unproven-destruction handle, and
      RecoveryRequiredOwnedHandle for indirect siblings.
      It grants no I/O.
    end note
```

```mermaid
sequenceDiagram
    autonumber
    participant C as Application service
    participant BS as BoundBrowserSession
    participant P as exact bound adapter
    participant O as Operability / recovery observer

    C->>BS: create accepted remote ownership
    alt premature finish
        C->>BS: finish()
        BS-->>C: ActiveContextRemains; wrapper retained
        C->>BS: destroy exact authority
        BS->>P: proven remote destruction
        C->>BS: finish()
        BS-->>C: Ended
    else ordinary wrapper abandonment
        C-xBS: drop without proven cleanup
        Note over BS,P: Drop performs no browser I/O
        BS->>O: increment abandoned_bound_session_count()
        Note over O: process-local signal only; not destruction proof or durable exact-handle storage
    end
```

The abandonment signal is deliberately weaker than durable recovery. Exact crash/process-restart reconciliation remains open until a canonical recovery owner persists `BrowserSessionRecoveryEvidence` before process termination.

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

`RecoveryRequired` and `TransportLost` remain terminal for normal authority in this slice. Later reconciliation may inspect recovery evidence, but it must not reconstruct cleanup authority from raw identifiers or treat command ACK as proof of destruction.
