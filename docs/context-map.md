# OriginWeave Context Map

This map records bounded-context ownership and allowed dependency direction. Protected `main` remains shipment truth. A context marked **active PR** exists only on the referenced branch until it is integrated.

## Subdomains

| Subdomain | Classification | Current owner | Responsibility |
|---|---|---|---|
| Browser authority contracts | Core | `originweave-core` | Stable no-I/O values for origin identity, action intent, capability, risk, approval scope, action requests, and policy context |
| Policy decision | Core | `originweave-policy` | Deterministic fail-closed authorization over typed action requests and explicit context |
| Destination authority | Core | `originweave-destination` | Resolved-address classification, approved resolution snapshots, redirect authorization, and DNS-rebinding resistance |
| Direct transport authority | Supporting | `originweave-network` | Single-use direct TCP plans and exact operating-system peer proof |
| TLS service identity | Supporting | `originweave-tls` | WebPKI authentication for an already verified direct TCP stream |
| Resource governance | Supporting | `originweave-resource` | Bounded task budgets and deterministic mitigation plans |
| Evidence contracts | Supporting | `originweave-evidence` | Value-redacted network evidence and source-bound provenance records |
| Browser Agent Protocol | Supporting | `originweave-bap` | Browser-agent protocol contracts and lifecycle vocabulary |
| MCP protocol adapter | Generic integration | `originweave-mcp` (**active PR #272**) | MCP version/method/tool discovery, routing integrity, and translation into typed OriginWeave action contracts; grants no execution authority |

Planned contexts such as browser sessions, HTTP, proxy, observation, typed browser execution, secret brokering, WebDriver BiDi/CDP adapters, WARC/PROV persistence, and release benchmarks remain planned until code reaches protected `main`. Their planned names do not grant ownership to unrelated current crates.

## Context relationships

```text
External MCP client
        |
        v
+--------------------+
| MCP adapter        |  generic integration / ACL
| originweave-mcp    |
+---------+----------+
          | route/action equality; mismatch stays adapter-owned
          | accepted routes enter ordinary policy evaluation
          v
+--------------------+        +----------------------+
| Policy decision    |------->| Browser authority    |
| originweave-policy |        | originweave-core     |
+--------------------+        +----------------------+
          ^                           ^
          |                           |
          +------ typed request ------+
                                      |
                  +-------------------+-------------------+
                  |                   |                   |
          +-------+--------+  +-------+--------+  +-------+--------+
          | Destination    |  | Evidence       |  | Resource       |
          | authority      |  | contracts      |  | governance     |
          +-------+--------+  +----------------+  +----------------+
                  |
                  v
          +-------+--------+
          | Direct TCP     |
          | authority      |
          +-------+--------+
                  |
                  v
          +-------+--------+
          | TLS identity   |
          +----------------+
```

The arrows show allowed contract dependence, not runtime authority transfer. A downstream context must still validate its own invariants. In particular, an origin grant does not authorize a DNS result, a destination approval does not prove the connected peer, a verified peer does not prove TLS service identity, and an MCP route does not authorize an action.

## Relationship contracts

### MCP adapter → browser authority contracts

Relationship: **Anti-Corruption Layer / conformist at the protocol edge**.

The MCP adapter may depend on stable OriginWeave action vocabulary from `originweave-core`. External MCP method names, tool names, protocol versions, cursors, and request metadata remain adapter types. They must not be promoted into `originweave-core` domain entities. The adapter maps a bounded, validated MCP request to an existing typed action kind and does not invent capabilities or approvals.

### MCP adapter → policy decision

Relationship: **published contract consumption**.

A route/action mismatch remains adapter-owned as `McpRouteRejection::ActionMismatch`; it is not a policy `DenialReason` and never enters policy evaluation. After the adapter proves MCP route/action equality, it may call the protocol-independent policy API published by `originweave-policy`. Policy owns authorization invariants and depends only on stable OriginWeave domain contracts; it does not import MCP DTOs, `ValidatedMcpToolCall`, MCP-specific denial reasons, or other adapter vocabulary. MCP owns protocol validation and the protocol-to-policy bridge. Neither context may absorb the other's responsibility.

### Destination authority → browser authority contracts

Relationship: **customer/supplier through stable values**.

Destination policy consumes canonical origin identity but owns address classification, resolution snapshots, rebinding checks, and redirect destination decisions. `Origin` is a logical web identity, not an SSRF decision.

### Direct transport authority → destination authority

Relationship: **customer/supplier through an approved snapshot**.

The network boundary accepts only a concrete address authorized by an origin-bound `ResolutionSnapshot`. It performs no DNS resolution and cannot reinterpret an origin grant as socket authority.

### TLS service identity → direct transport and destination authority

Relationship: **customer/supplier through verified transport evidence**.

TLS consumes the already connected direct stream, the canonical HTTPS origin, and explicit trust material. It may not reconnect, re-resolve, or replace exact peer evidence with certificate success.

### Evidence and resource contexts

Relationship: **published contracts, no shared mutable kernel**.

Evidence and resource governance expose bounded value contracts. Browser/protocol adapters may produce telemetry or evidence inputs, but these contexts do not acquire browser, network, policy, or secret authority from those callers.

## Dependency rules

1. Domain/security contracts must not import MCP, BiDi, CDP, Chromium SDK, HTTP client, persistence, UI, or provider DTOs.
2. Protocol adapters may depend inward on stable OriginWeave contracts and policy ports; stable contracts and policy must not depend outward on protocol adapters.
3. Policy authorization remains centralized in `originweave-policy`; adapters cannot duplicate or weaken it. Protocol-integrity failures remain owned by the adapter that can interpret them.
4. Cross-context calls use public crate APIs or explicit application ports. Direct access to another context's internals is forbidden.
5. No context may infer authority from another context's success. Each boundary emits evidence specific to the invariant it proves.
6. Shared Kernel additions require an accepted ADR and must be smaller than the contexts that consume them. `originweave-core` is not a dumping ground for integration DTOs.
7. Planned contexts remain separate responsibilities even before their crates exist. New work must not be parked temporarily in `originweave-core` merely because the final context has not yet been created.

## Machine-checkable fitness

`tests/test_repository_contract.py` enforces the first MCP ownership slice introduced with PR #272: `originweave-mcp` is a workspace package, MCP routing does not live under `originweave-core`, the adapter depends inward on `originweave-core` and `originweave-policy`, and policy depends only on `originweave-core` without MCP protocol or adapter vocabulary. The adapter owns route/action integrity rejection and delegates only matching routes to the ordinary policy evaluator. `tests/test_ddd_documentation_contract.py` binds this Context Map to the same ownership language. Additional context relationships should gain equivalent import/dependency fitness checks when their production boundaries land.
