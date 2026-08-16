# Extension Compatibility and Agent Authority UML

- **Status:** Protected-main architecture visualization with active compatibility work
- **Scope:** Chromium Manifest V3 compatibility plane versus OriginWeave Agent authority
- **Related:** [`README.md`](README.md), [`../PRD.md`](../PRD.md), [`../TRD.md`](../TRD.md), [`../THREAT_MODEL.md`](../THREAT_MODEL.md), issue #27

This diagram makes one security invariant visually explicit:

> **A Chromium extension permission is not an OriginWeave Agent capability.**

A compatible extension can use the Chromium APIs granted by its manifest and managed browser policy. It cannot thereby grant itself OriginWeave task authority, widen an Agent Task origin, resolve a protected secret, approve a high-risk action, or turn extension/page content into a trusted instruction.

## Authority sequence

```mermaid
sequenceDiagram
    autonumber
    participant Admin as Human / Enterprise Policy
    participant Chrome as Chromium MV3 Runtime
    participant Ext as Extension Worker / Content Script
    participant Observe as OriginWeave Observation Adapter
    participant Grant as OriginWeave Extension Grant Policy
    participant Agent as Agent Task / Planner
    participant Policy as Deterministic Action Policy
    participant Broker as Secret / Sensitive Broker
    participant Browser as Trusted Browser Adapter
    participant Evidence as Evidence Trail

    Admin->>Chrome: install/enable extension under Chromium policy
    Chrome-->>Ext: expose manifest-granted Chrome APIs
    Note over Chrome,Ext: Chrome permission is compatibility authority only.

    Ext-->>Observe: extension message / page mutation / tool output
    Observe-->>Agent: bounded untrusted observation + provenance
    Note over Ext,Agent: Extension content cannot become trusted goal or policy.

    Admin->>Grant: admit exact extension id on host-managed Agent Task allow-list
    Note over Grant: Chrome force_installed is not an Agent grant.
    Admin->>Grant: issue explicit OriginWeave extension grant for bounded session/context/capability/origin
    Ext->>Grant: request OriginWeave interaction
    Grant->>Grant: verify extension identity, managed policy, session/context, capability, origin, expiry

    alt no valid OriginWeave grant
        Grant-->>Ext: deny
        Grant-->>Evidence: denial without sensitive value
    else valid grant
        Grant-->>Agent: bounded extension-originated proposal/evidence
        Agent->>Policy: propose typed action under existing Agent Task authority
        Policy->>Policy: revalidate task, action, risk, origin, approval and current browser authority
        alt action requires secret/sensitive value
            Policy->>Broker: authorize exact opaque handle use
            Broker->>Broker: revalidate tenant/task/field/purpose/destination/expiry
            Broker-->>Browser: minimum trusted value delivery
        end
        Policy-->>Browser: authorized typed action
        Browser->>Browser: verify session/context/document epoch immediately before dispatch
        Browser-->>Evidence: action result + observed post-condition
    end
```

## Security state flow

```mermaid
flowchart TD
    manifest[Manifest V3 permissions] --> chromium[Chromium extension authority]
    chromium --> extension[Extension runtime]
    extension --> untrusted[Untrusted observation / message]
    untrusted --> grant{Explicit OriginWeave extension grant?}
    grant -- no --> deny[Deny Agent-control request]
    grant -- yes --> scoped[Bind extension identity + session + context + origin + capability + expiry]
    scoped --> proposal[Typed Agent action proposal]
    proposal --> policy{Agent Task policy passes?}
    policy -- no --> deny
    policy -- yes --> approval{Risk-specific approval required?}
    approval -- missing/invalid --> deny
    approval -- no or valid --> execute[Trusted browser adapter executes]
    execute --> verify{Observed post-condition matches?}
    verify -- no --> fail[Fail / quarantine]
    verify -- yes --> evidence[Credential-safe evidence]

    extension -. cannot mint .-> scoped
    extension -. cannot approve .-> approval
    extension -. cannot resolve .-> secret[Protected secret / sensitive value]
    secret --> execute
```

## Compatibility evidence is separate from authority evidence

```mermaid
flowchart LR
    pinned[Pinned Chromium revision] --> fixture[Controlled MV3 fixture suite]
    fixture --> compat[Compatibility evidence]
    compat --> matrix[Published supported-capability matrix]

    policycode[OriginWeave extension policy] --> isolation[Agent-authority isolation evidence]
    isolation --> release[Release acceptance]
    matrix --> release

    compat -. does not prove .-> isolation
    isolation -. does not prove .-> compat
```

The release claim requires both evidence classes. A passing `downloads`, `bookmarks`, `history`, storage, service-worker, DNR, or content-script compatibility test does not prove extension isolation. Conversely, a correct Rust extension-grant kernel does not prove that a real Chromium extension API works.

## Maturity discipline

Protected main already contains extension-to-Agent authority foundations and pinned-Chromium MV3 compatibility evidence for several surfaces. Issue #27 remains open because the complete declared capability matrix, remaining compatibility surfaces, managed/native-messaging boundaries and release integration are not yet complete. This diagram therefore represents a mixture of implemented foundations and accepted/planned product flow; it must not be read as a claim of full Chrome extension compatibility.
