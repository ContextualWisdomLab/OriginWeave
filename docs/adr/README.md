# OriginWeave Architecture Decision Index

This directory contains durable architecture decisions for OriginWeave. A pull-request body, chat transcript, roadmap bullet, or implementation plan may motivate a decision but does not replace an ADR when the decision changes a governing product or authority boundary.

## Status vocabulary

- **Proposed** — under review; not binding and not a shipped claim.
- **Accepted** — governing decision for protected `main`.
- **Superseded** — replaced by a later Accepted ADR; retained for history.
- **Deprecated** — still discoverable but no longer recommended for new work.
- **Rejected** — evaluated and intentionally not adopted.

An ADR becomes Accepted only through normal protected-branch review and merge. Conversation-derived ideas remain Proposed/Open in PRD/TRD/traceability until that process is complete.

## Current protected-main decisions

| ADR | Decision | Protected-main status | Governs |
|---|---|---|---|
| [0001](0001-chromium-compatibility-kernel.md) | Retain Chromium as the compatibility kernel | Accepted | Blink/V8/graphics/extensions boundary; Rust control-plane integration |
| [0002](0002-agent-safety-kernel.md) | Agent safety kernel | Accepted | mode, capability, origin, risk, crawler, secret and approval policy |
| [0003](0003-provenance-native-observation.md) | Provenance-native observation | Accepted | evidence/provenance as a first-class product output |
| [0004](0004-resolved-destination-policy.md) | Logical origin and resolved destination safety | Accepted | SSRF/rebinding/special-purpose address and redirect authority |
| [0005](0005-direct-socket-binding.md) | Exact direct TCP peer binding | Accepted | explicit socket authority and operating-system peer proof |
| [0006](0006-tls-server-identity.md) | TLS service identity over the verified peer | Accepted | WebPKI identity, roots, time, ALPN and stream binding |

## Proposed target-architecture decisions in this change

The following ADRs make the product-wide target architecture reviewable without promoting it to shipped behavior. They remain **Proposed** until their exact branch is reviewed and merged under protected-main policy. Existing feature PRs may independently carry lower-numbered Proposed ADRs; the `0100` range avoids claiming or conflicting with those active decisions.

| ADR | Decision | Status | Governs |
|---|---|---|---|
| [0100](0100-rust-control-plane-boundary.md) | Rust control-plane boundary | Proposed | Rust-owned product authority versus Chromium compatibility kernel |
| [0101](0101-isolated-execution-profile-modes.md) | Isolated execution/profile modes | Proposed | Human, Assist, Agent Task and Crawler session/profile isolation |
| [0102](0102-typed-actions-and-arbitrary-js.md) | Typed actions over arbitrary JavaScript authority | Proposed | action API, script escape hatches, risk/policy semantics |
| [0103](0103-semantic-observation-and-stale-node-identity.md) | Semantic observation precedence and stale-node identity | Proposed | WebMCP/structured/accessibility/DOM/layout/visual precedence and document epochs |
| [0104](0104-prompt-injection-and-secret-authority.md) | Prompt-injection and secret authority separation | Proposed | untrusted page data, opaque secret handles and broker boundaries |
| [0105](0105-resource-governor-priority.md) | Resource governor and browser-over-model priority | Proposed | CPU/RAM/GPU/VRAM admission, fallback and tenant fairness |
| [0106](0106-provenance-evidence-model.md) | Provenance-native evidence model | Proposed | WARC/PROV-style evidence identities, integrity and disclosure |
| [0107](0107-browser-protocol-adapter-strategy.md) | Versioned browser and agent protocol adapters | Proposed | WebDriver BiDi, CDP, WebMCP, MCP and OriginWeave Protocol boundaries |
| [0108](0108-crawler-policy.md) | Policy-bound crawler mode | Proposed | robots, rate/resource policy, read-only collection and no-evasion behavior |
| [0109](0109-hourly-automation-operational-closure.md) | Hourly automation secret ordering and operational closure | Proposed | deterministic gates, model secret boundary, retries and protected-main proof |

Active feature PRs may contain additional Proposed ADRs. Those ADRs are not described as Accepted until their exact changes merge. When an ADR becomes protected-main architecture, update this index in the same protected change or an immediately coupled documentation repair.

## Decisions that require a dedicated ADR

A new or superseding ADR is required when a change materially alters any of the following:

1. Chromium compatibility-kernel ownership or patch strategy;
2. execution modes or autonomous profile/session isolation;
3. trusted-instruction, untrusted-observation or protected-secret boundaries;
4. capability, action, risk or approval semantics;
5. logical origin, resolved destination, route/proxy, TCP peer, TLS identity or HTTP authority;
6. browser session/context/document/node lifetime and stale-reference semantics;
7. sensitive-data authority, opaque-handle broker or disclosure evidence;
8. observation hierarchy or arbitrary-script policy;
9. resource-governor priority, telemetry or GPU/CPU fallback semantics;
10. evidence/provenance identity, retention or persistence boundaries;
11. WebDriver BiDi, CDP, WebMCP, MCP or OriginWeave Protocol authority/version boundaries;
12. Manifest V3 extension-to-agent authorization;
13. tenant, privacy, residency, audit, deployment or enterprise-control ownership;
14. release acceptance, rollback/recovery or protected-main operational-proof requirements.

## Required ADR structure

New material ADRs should contain the following sections unless a section is demonstrably inapplicable:

```text
# ADR NNNN: Decision title

- Status: Proposed | Accepted | Superseded | Deprecated | Rejected
- Date: YYYY-MM-DD
- Supersedes: optional ADR reference
- Superseded by: optional ADR reference

## Context
## Decision drivers
## Assumptions and authority boundaries
## Options considered
## Decision
## Consequences
## Failure and degraded behavior
## Security / privacy / governance impact
## Tests and acceptance evidence
## Migration and rollback
## Open follow-ups
## Supersession / reversal conditions
## References
```

Material external standards or research belong in APA 7th format in [`../doctoring.md`](../doctoring.md) and may also be repeated in the ADR when the citation is necessary to understand the decision.

## Relationship to product documents

- [`../PRD.md`](../PRD.md) defines buyer and product requirements.
- [`../TRD.md`](../TRD.md) translates them into technical invariants and implementation-status boundaries.
- [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) defines system topology and bounded contexts.
- [`../uml/README.md`](../uml/README.md) visualizes component, sequence, state and deployment relationships.
- [`../erd/README.md`](../erd/README.md) defines the conceptual durable domain model.
- [`../traceability/README.md`](../traceability/README.md) maps requirements and decisions to implementation and evidence.

If these artifacts disagree, protected code plus an Accepted ADR define the current implemented truth; the disagreement is a documentation defect that must be repaired rather than silently rationalized from conversation history.
