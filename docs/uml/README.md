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

## 7. Secret-fill sequence

```mermaid
sequenceDiagram
    autonumber
    participant Goal as Trusted User / Enterprise Authority
    participant Session as Governed Session / Task
    participant Model as Model / Planner
    participant Policy as Deterministic Policy
    participant Broker as Secret Broker
    participant Adapter as Trusted Browser Adapter
    participant Page as Untrusted Page
    participant Evidence as Credential-free Evidence

    Goal->>Session: authorize purpose, origin, capability, task scope
    Model->>Policy: propose secret_fill(opaque_handle, target, action_intent)
    Note over Model,Policy: Model never receives the raw secret.
    Policy->>Policy: validate mode, purpose, capability, origin, risk, approval, exact scope
    alt authority mismatch / stale approval / invalid handle scope
        Policy-->>Model: deny without broker resolution
        Policy-->>Evidence: denial + scope references
    else policy permits broker use
        Policy->>Broker: resolve opaque handle under exact authorized scope
        Broker->>Broker: validate tenant/task, destination, expiry, revocation, use policy
        alt broker validation fails
            Broker-->>Policy: fail closed
            Policy-->>Evidence: broker denial without raw value
        else broker validation succeeds
            Broker-->>Adapter: minimum secret value on trusted delivery path
            Adapter->>Page: fill only the authorized field/action
            Page-->>Adapter: resulting observable state
            Adapter-->>Evidence: delivery reference + post-condition; no raw secret
        end
    end
```

Page content, model output, logs, and evidence can reference an opaque handle or redacted fingerprint but cannot request broker authority or receive the durable raw value merely by describing it.

## 8. Read/write risk approval flow

```mermaid
flowchart TD
    intent[Typed action intent] --> classify[Classify mutability + risk]
    classify --> read{Read-only and within declared capability?}
    read -- yes --> readpolicy[Validate purpose, session, origin, destination, resource policy]
    readpolicy -->|pass| execute_read[Execute bounded read]
    readpolicy -->|deny| denied[Denied / unsupported]
    read -- no --> writepolicy[Validate write capability, exact target, intent digest, risk tier]
    writepolicy --> approval{Exact approval required?}
    approval -- no --> execute_write[Execute typed action]
    approval -- yes --> approvalcheck{Fresh in-scope approval supplied by authorized authority?}
    approvalcheck -- no --> waiting[Await approval / deny on expiry or rejection]
    approvalcheck -- yes --> execute_write
    execute_read --> verify[Verify declared result]
    execute_write --> verify
    verify --> ok{Post-condition established?}
    ok -- yes --> evidence[Record separate observation, policy, approval, action, post-condition evidence]
    ok -- no --> quarantine[Failed or quarantined; never mark success]
    page[Untrusted page content] -. cannot approve .-> approvalcheck
    model[Model proposal] -. cannot approve .-> approvalcheck
```

A page, model, comment, status check, or other observation cannot synthesize approval. Approval is an independent authority bound to the action contract, and a valid approval does not substitute for post-condition verification.

## 9. Resource-pressure and fallback flow

```mermaid
flowchart TD
    task[Task/session admission request] --> budget[Evaluate CPU, RAM, GPU, VRAM, network, storage and concurrency budgets]
    budget --> fits{Fits bounded budget?}
    fits -- no --> reject[Reject or queue before unsafe launch]
    fits -- yes --> run[Run browser + optional model under governor]
    run --> pressure{Resource pressure detected?}
    pressure -- no --> preserve[Continue within budget]
    pressure -- yes --> browserneed{Browser resources required for current observation/action verification?}
    browserneed -- yes --> modeldegrade[Degrade optional model first]
    modeldegrade --> reduce[Reduce model concurrency / batch]
    reduce --> fallback{Policy permits CPU or remote-model fallback?}
    fallback -- yes --> modelalternate[Use governed alternate model path]
    fallback -- no --> modelpause[Pause or fail model-backed work]
    browserneed -- no --> boundeddegrade[Apply documented bounded capture/model degradation]
    modelalternate --> browserok{Browser still verifiable and inside hard limits?}
    modelpause --> browserok
    boundeddegrade --> browserok
    browserok -- yes --> preserve
    browserok -- no --> stop[Pause/fail task before state-changing action or verification loss]
    preserve --> evidence[Record resource decision and resulting evidence]
    stop --> evidence
```

The browser is not unbounded: browser correctness is prioritized over optional model acceleration, but hard host and tenant limits still fail closed. A task cannot be recorded as successfully changed when resource eviction prevents the browser from establishing its post-condition.

## 10. Hourly product-development gate-to-model flow

```mermaid
flowchart TD
    trigger[Hourly / manual trigger on protected workflow definition] --> snapshot[Refetch exact protected main, open PRs/issues, release blockers and writer lease]
    snapshot --> openpr{Open PR exists?}
    openpr -- yes --> openstate[Emit open_pull_request deterministic state]
    openstate --> nostart[Stop before NVIDIA_NIM_API_KEY materialization]
    openpr -- no --> deterministic{Release blocker, dry-run or deterministic product/release gate?}
    deterministic -- yes --> deterministic_result[Handle deterministic state without model credential]
    deterministic -- no --> credential{Conditional credential gate authorized and broker healthy?}
    credential -- no --> credentialdenied[credential denied or broker unavailable]
    credentialdenied --> stopnosecret[stop without secret materialization]
    credential -- yes --> broker[Expose NVIDIA_NIM_API_KEY only to authorized credential/broker path]
    broker --> pristine[Create pristine workspace from exact HEAD]
    pristine --> attempt[Run bounded model attempt]
    attempt --> classify{Attempt result}
    classify -- success --> seal[Seal credential-free bounded change bundle]
    classify -- model_timeout --> retry{Broker healthy and remaining budget feasible?}
    classify -- model_or_tool_failure --> retry
    classify -- credential_broker_unavailable --> stopmodel[Stop model fallback]
    retry -- yes --> pristine
    retry -- no --> stopmodel
    seal --> validate[Independent tests, coverage, security and secret-fingerprint validation]
    validate --> validation{Validation result}
    validation -- failed --> validationfailed[validation failed]
    validationfailed --> failclosed[fail closed without publication]
    validation -- passed --> validationpassed[validation passed]
    validationpassed --> changed{Verified non-empty change?}
    changed -- no --> evidence[Record deterministic no-change / product result]
    changed -- yes --> publish{Publication authority available and live state unchanged?}
    publish -- no --> failclosed
    publish -- yes --> pr[Open/update one reviewed PR]
    pr --> governance[Independent exact-head checks, review and protected branch policy]
    governance --> merge[Protected merge only when all authorities pass]
    merge --> acceptance[Protected-main scheduled/manual operational acceptance]
```

The credential decision is fail closed: `credential denied or broker unavailable` reaches `stop without secret materialization`. The validation decision is also fail closed: `validation failed` reaches `fail closed without publication`; only `validation passed` can proceed to non-empty-change and publication decisions.

This diagram describes the governing workflow architecture and closure contract. It is not evidence that an active incident repair has merged or that protected-main acceptance has already occurred. `COPILOT_GITHUB_TOKEN`, invented PATs, raw-secret rematerialization, synthesized approval, and fail-open publication are outside the design.

## 11. Diagram maintenance rules

Update this pack when a protected change materially alters:

- product/bounded-context ownership;
- authority ordering or trust boundaries;
- task/session lifecycle;
- observation hierarchy or action lifecycle;
- secret-broker delivery or approval semantics;
- resource admission, pressure, or browser/model fallback priority;
- hourly deterministic/model gate ordering or operational-closure evidence;
- deployment boundaries;
- evidence/provenance relationships.

A feature-specific ADR may include a more detailed sequence diagram, but this pack remains the product-wide view and must not require maintainers to reconstruct the complete system from scattered ADR diagrams.
