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
    Note over S,P: binding invokes no adapter callback

    C->>BS: create_disposable_context()
    BS->>S: require Active + reserve monotonic epoch
    S->>S: mint DisposableContextCreateRequest
    S->>P: create_disposable_context(request)
    P->>B: create fresh isolation boundary + browsing context
    B-->>P: unique isolation id + BrowsingContextId or typed create error
    P-->>S: DisposableContextHandle
    S->>S: register exact handle + Active epoch
    S-->>C: PresentationMutationAuthority(session, incarnation, isolation, context, epoch)

    Note over C,S: Raw ids and adapter-selected scalar identities cannot mint lifecycle or presentation authority.

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

`BoundBrowserSession` is a linear lifecycle-port binding: it consumes one concrete port and exposes no public lifecycle method that accepts a replacement port. `DisposableContextPort` has no identity callback, so Browser Session does not execute arbitrary adapter code merely to establish adapter ownership. `DisposableContextCreateRequest` and `DisposableContextDestroyRequest` are non-caller-constructible capabilities created inside the bound path.

`BrowserSessionIncarnation` separates sequential aggregate lifecycles even when the browser later reuses the same external session, user-context/isolation, browsing-context, and local epoch values. The incarnation is checked by authority validation and reaches the lifecycle port inside the opaque request.

For a WebDriver BiDi adapter, `DisposableIsolationId` maps to the user-context id created by `browser.createUserContext`. That protocol id remains lifecycle addressability rather than OriginWeave policy authority. Protocol-specific pending/accepted/quarantined remote tuples remain in the BiDi ACL boundary rather than this domain model.

## Recovery and transport state

```mermaid
stateDiagram-v2
    [*] --> Active
    Active --> Active: fresh isolation + context / authority minted
    Active --> Active: context epoch advanced / prior authority stale
    Active --> Active: exact owned isolation destruction proved
    Active --> Active: DisposableContextCreateError::CreateFailedClean
    Active --> RecoveryRequired: CreateFailedUncertain / retain known partial isolation
    Active --> RecoveryRequired: duplicate output / retain offending handle
    Active --> RecoveryRequired: DisposableContextDestroyError / cleanup unproven
    Active --> Ended: all owned contexts Destroyed + end
    Active --> TransportLost: browser transport lost
    RecoveryRequired --> RecoveryRequired: transport_lost = true / preserve recovery evidence
    Ended --> [*]
    RecoveryRequired --> [*]
    TransportLost --> [*]

    note right of RecoveryRequired
      BrowserSessionRecoveryEvidence retains known
      partial identity, duplicate handle, or exact
      unproven-destruction handle. It grants no I/O.
    end note

    note right of TransportLost
      Transport liveness is orthogonal to ownership
      recovery. Duplicate loss reports are idempotent.
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
    A->>PA: create(request S, incarnation A)
    PA-->>A: U, C
    A->>PA: destroy(request S, incarnation A, U/C)
    A->>A: end()

    B->>B: start(S) => incarnation B; bind PB
    B->>PB: create(request S, incarnation B)
    PB-->>B: same U, same C
    Note over A,B: both local context epochs may equal 1
    B->>B: validate retained authority A
    B-->>A: AuthorityMismatch before PB destroy I/O
    B->>PB: destroy with authority B + incarnation B
```

`RecoveryRequired` and `TransportLost` remain terminal for normal authority in this slice. A later reconciliation design may inspect `BrowserSessionRecoveryEvidence`, but it must not reconstruct cleanup authority from raw identifiers or treat command ACK as proof of destruction.
