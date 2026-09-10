# Browser Session lifecycle authority

This diagram describes the active-PR domain contract for issue #312. It is not evidence that a WebDriver BiDi or Chromium adapter already implements the port.

```mermaid
sequenceDiagram
    autonumber
    participant C as Application service
    participant S as BrowserSession aggregate
    participant BS as BoundBrowserSession
    participant P as DisposableContextPort
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
        S->>S: mint DisposableContextCreateCompletion(Rejected, exact attempt)
        S->>P: complete_disposable_context_creation(completion)
        P->>P: pending exact attempt → quarantined/non-authorizing
        S->>S: RecoveryRequired
    else completion cannot be proven
        S->>S: retain UnsettledAdapterHandle
        S->>S: RecoveryRequired
    end

    Note over C,S: Raw ids, adapter-selected values, and diagnostic references cannot mint lifecycle or presentation authority.

    C->>BS: advance_context_epoch(context_id)
    BS->>S: replace epoch; old authority becomes stale
    S-->>C: new opaque authority carrying same incarnation + isolation

    C->>BS: destroy_disposable_context(authority)
    BS->>S: validate exact session/incarnation/isolation/context/epoch before I/O
    S->>S: mint DisposableContextDestroyRequest with exact stored handle
    S->>P: destroy_disposable_context(request)
    P->>B: remove exact owned isolation boundary
    B-->>P: observed destruction post-condition or DisposableContextDestroyError
    P-->>S: success
    S->>S: context = Destroyed
    C->>BS: end()
    BS->>S: require every owned context Destroyed
    S-->>C: Ended
```

`BoundBrowserSession` is a linear lifecycle-port binding. It consumes one concrete port, exposes no public raw `&P`, and exposes no lifecycle method that accepts a replacement port. Tests retain inert observation state separately from the moved adapter.

`DisposableContextCreateRequest` and `DisposableContextCreateCompletion` are non-caller-constructible. The create request carries the reserved `BrowserContextEpoch` as a per-create transaction id; Browser Session alone decides whether the returned domain handle is accepted or rejected.

For WebDriver BiDi, `DisposableIsolationId` maps to the user-context id created by `browser.createUserContext`. Protocol-specific pending/accepted/quarantined remote tuples remain in the BiDi ACL boundary rather than this domain model.

## Recovery and transport state

```mermaid
stateDiagram-v2
    [*] --> Active
    Active --> Active: create candidate + exact Accepted completion + authority
    Active --> Active: context epoch advanced / prior authority stale
    Active --> Active: exact owned isolation destruction proved
    Active --> Active: DisposableContextCreateError::CreateFailedClean
    Active --> RecoveryRequired: CreateFailedUncertain / retain known partial isolation
    Active --> RecoveryRequired: duplicate output + exact Rejected completion
    Active --> RecoveryRequired: completion unproven / retain UnsettledAdapterHandle
    Active --> RecoveryRequired: DisposableContextDestroyError / cleanup unproven
    Active --> Ended: all owned contexts Destroyed + end
    Active --> TransportLost: browser transport lost
    RecoveryRequired --> RecoveryRequired: transport_lost = true / preserve recovery evidence
    Ended --> [*]
    RecoveryRequired --> [*]
    TransportLost --> [*]

    note right of RecoveryRequired
      BrowserSessionRecoveryEvidence retains known
      partial identity, duplicate/unsettled handle,
      or exact unproven-destruction handle.
      It grants no I/O.
    end note
```

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
    A->>A: end()

    B->>B: start(S) => incarnation B; bind PB
    B->>PB: create(request S, incarnation B, attempt 1)
    PB-->>B: same U, same C pending
    B->>PB: completion Accepted(attempt 1)
    Note over A,B: local attempt/epoch may both equal 1, but incarnations differ
    B->>B: validate retained authority A
    B-->>A: AuthorityMismatch before PB destroy I/O
    B->>PB: destroy with authority B + incarnation B
```

`RecoveryRequired` and `TransportLost` remain terminal for normal authority in this slice. Later reconciliation may inspect recovery evidence, but it must not reconstruct cleanup authority from raw identifiers or treat command ACK as proof of destruction.
