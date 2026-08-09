# OriginWeave UML and Control-Flow Diagrams

- **Status:** Proposed authoritative diagram pack
- **Notation:** Mermaid diagram-as-code
- **Textual authority:** [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md), [`../TRD.md`](../TRD.md), and Accepted ADRs

These diagrams visualize governing boundaries; they do not imply that every planned adapter is already shipped. Labels use `implemented`, `active`, or `planned` where implementation status matters.

## 1. Component and bounded-context view

```mermaid
flowchart TB
    subgraph UX[User / Enterprise Experience]
        human[Human browser user]
        admin[Enterprise administrator]
        agent[External agent / orchestrator]
    end

    subgraph Product[OriginWeave Product Surfaces]
        browser[OriginWeave Browser\nplanned]
        runtime[OriginWeave Runtime\nplanned]
        observe[OriginWeave Observe\nplanned]
        capture[OriginWeave Capture\nplanned]
        governor[OriginWeave Governor\nfoundation implemented]
        policy[OriginWeave Policy\nfoundation implemented]
        evidence[OriginWeave Evidence\nfoundation implemented]
        protocol[OriginWeave Protocol / SDK\nplanned]
    end

    subgraph Rust[Rust Control Plane]
        core[originweave-core]
        policy_crate[originweave-policy]
        destination[originweave-destination]
        network[originweave-network]
        tls[originweave-tls]
        resource[originweave-resource]
        evidence_crate[originweave-evidence]
        http[HTTP / proxy / session / action / observation\nplanned or active PR work]
    end

    subgraph Chromium[Chromium Compatibility Kernel]
        blink[Blink]
        v8[V8]
        viz[Skia / Viz / Dawn]
        sandbox[Sandbox / Site Isolation]
        extensions[Manifest V3 extensions]
    end

    subgraph Adapters[External Protocol Adapters]
        bidi[WebDriver BiDi\nplanned]
        cdp[CDP\nplanned]
        webmcp[WebMCP\nplanned / experimental upstream]
        mcp[MCP\nplanned]
    end

    human --> browser
    admin --> browser
    agent --> protocol
    browser --> runtime
    protocol --> runtime
    runtime --> observe
    runtime --> capture
    runtime --> governor
    runtime --> policy
    runtime --> evidence
    policy --> core
    policy --> policy_crate
    runtime --> destination --> network --> tls --> http
    governor --> resource
    evidence --> evidence_crate
    runtime --> Adapters
    Adapters --> Chromium
    browser --> Chromium
```

## 2. Network and service-authority sequence

```mermaid
sequenceDiagram
    autonumber
    participant U as User / Task Authority
    participant O as Canonical Origin
    participant D as Destination Policy
    participant R as Route / Proxy Policy
    participant N as Direct TCP Authority
    participant T as TLS Identity
    participant H as HTTP Semantics
    participant B as Browser Adapter
    participant E as Evidence

    U->>O: approve bounded target origin
    O->>D: request origin-bound resolution approval
    D-->>O: approved nonempty address snapshot
    O->>R: select direct/proxy route under explicit policy
    R-->>N: exact authorized connection target
    N->>N: connect exact socket + verify peer_addr
    N-->>T: verified stream + TCP evidence
    T->>T: verify canonical DNS/IP identity, roots, time, ALPN
    T-->>H: authenticated stream + TLS evidence
    H->>H: enforce framing, bytes, integrity, MIME, redirect budgets
    H-->>B: bounded exchange / redirect metadata
    B->>E: bind adapter result to all independent authority evidence
    Note over D,H: A green result at one layer never substitutes for the next layer.
```

## 3. Observation-to-action sequence

```mermaid
sequenceDiagram
    autonumber
    participant Goal as Trusted User Goal
    participant Session as Agent Task Session
    participant Page as Untrusted Web Page
    participant Observe as OriginWeave Observe
    participant Model as Model / Orchestrator
    participant Policy as Deterministic Policy
    participant Broker as Secret Broker
    participant Adapter as Browser Action Adapter
    participant Evidence as Evidence Trail

    Goal->>Session: establish task purpose + capabilities + origins
    Session->>Observe: request bounded observation
    Page-->>Observe: tool / structured / network / AX-DOM-layout / visual data
    Observe-->>Model: typed untrusted observation + node authority
    Model->>Policy: propose typed action intent
    Policy->>Policy: validate session, purpose, capability, origin, risk, approval
    alt secret needed
        Policy-->>Broker: authorize opaque handle use under exact scope
        Broker-->>Adapter: trusted value fill without model disclosure
    end
    Policy-->>Adapter: authorized typed action
    Adapter->>Page: trusted input event / navigation
    Page-->>Adapter: resulting observable state
    Adapter->>Adapter: verify declared post-condition
    Adapter-->>Evidence: action + policy + source + post-condition evidence
    Evidence-->>Model: safe outcome summary / references
```

## 4. Delegated-task state machine

```mermaid
stateDiagram-v2
    [*] --> Created
    Created --> Authorized: task purpose + policy accepted
    Created --> Rejected: invalid authority
    Authorized --> Active: isolated/attached context ready
    Active --> Observing
    Observing --> Planning: bounded observation complete
    Planning --> AwaitingApproval: risk requires human/dual control
    AwaitingApproval --> Planning: exact approval supplied
    AwaitingApproval --> Cancelled: rejected / expired
    Planning --> Acting: deterministic gates pass
    Acting --> Verifying
    Verifying --> Observing: post-condition passed, more work remains
    Verifying --> Failed: post-condition absent or unsafe transition
    Observing --> Paused: resource pressure / operator pause
    Planning --> Paused: resource pressure / provider pause
    Paused --> Active: authority and resources revalidated
    Active --> Cancelled: user/operator cancellation
    Observing --> Completed: goal satisfied without mutation
    Verifying --> Completed: final post-condition satisfied
    Failed --> Recovering: safe checkpoint/retry path exists
    Recovering --> Active: exact state revalidated
    Recovering --> Quarantined: ambiguous external side effect
    Completed --> [*]
    Cancelled --> [*]
    Quarantined --> [*]
```

The complete durable runtime state machine is **Planned**. The important contract is that cancellation, resource pause, approval, failure, ambiguous external effects, and post-condition verification are explicit states rather than hidden boolean flags.

## 5. Deployment and trust-boundary topology

```mermaid
flowchart LR
    subgraph ClientBoundary[Client / Human Boundary]
        desktop[OriginWeave Browser\nplanned desktop distribution]
        sdk[OriginWeave SDK / MCP client\nplanned]
    end

    subgraph RuntimeBoundary[OriginWeave Trusted Runtime]
        rt[Runtime / session authority]
        pol[Policy engine]
        net[Destination -> route -> TCP -> TLS -> HTTP]
        sec[Secret broker]
        obs[Observation / action adapters]
        ev[Evidence / provenance]
        gov[Resource governor]
    end

    subgraph BrowserBoundary[Chromium Process Boundary]
        browserproc[Browser process]
        renderer[Renderer processes\nuntrusted/compromise-tolerant]
        gpuproc[GPU process]
        ext[MV3 extension runtime]
    end

    subgraph External[External Services]
        web[Web origins / APIs]
        orch[contextual-orchestrator or other agent]
        model[Model provider]
        store[Relational / WARC / object / provenance stores\nplanned adapters]
    end

    desktop --> rt
    sdk --> rt
    orch --> sdk
    rt --> pol
    rt --> net
    rt --> sec
    rt --> obs
    rt --> ev
    rt --> gov
    obs --> browserproc
    browserproc --> renderer
    browserproc --> gpuproc
    browserproc --> ext
    net --> web
    rt -. bounded model request .-> model
    ev -. credential-free evidence .-> store
    renderer -. page content is data only .-> obs
```

## 6. Evidence authority flow

```mermaid
flowchart TD
    source[source_resource / page / response]
    observation[observation_evidence]
    proposal[model_proposal\nnon-authoritative]
    policy[policy_decision]
    approval[approval_evidence]
    network[network + TLS + HTTP evidence]
    action[action_event]
    post[post_condition_evidence]
    value[extracted_value]
    prov[provenance_record]
    result[task result / audit view]

    source --> observation
    observation --> proposal
    proposal --> policy
    approval --> policy
    network --> action
    policy --> action
    observation --> action
    action --> post
    observation --> value
    value --> prov
    source --> prov
    action --> prov
    post --> prov
    policy --> prov
    prov --> result
```

A model proposal can explain *why an action was proposed* but is not mergeable with `policy_decision`, `approval_evidence`, `network` authority, or `post_condition_evidence` into one undifferentiated success status.

## 7. Diagram maintenance rules

Update this pack when a protected change materially alters:

- product/bounded-context ownership;
- authority ordering or trust boundaries;
- task/session lifecycle;
- observation hierarchy or action lifecycle;
- deployment boundaries;
- evidence/provenance relationships.

A feature-specific ADR may include a more detailed sequence diagram, but this pack remains the product-wide view and must not require maintainers to reconstruct the complete system from scattered ADR diagrams.
