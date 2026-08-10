# OriginWeave Architecture Decision Index

This directory contains durable architecture decisions for OriginWeave. A pull-request body, chat transcript, roadmap bullet, automation prompt, issue, or implementation plan may motivate a decision but does not replace an ADR when the decision changes a governing product or authority boundary.

## Status vocabulary

- **Proposed** — under review; not binding and not a shipped claim.
- **Accepted** — governing design decision on protected `main`; acceptance does not itself prove that every described capability is implemented.
- **Superseded** — replaced by a later Accepted ADR; retained for history.
- **Deprecated** — still discoverable but no longer recommended for new work.
- **Rejected** — evaluated and intentionally not adopted.

Current contributor/review authority is defined by protected-main [`../../AGENTS.md`](../../AGENTS.md) together with live GitHub repository policy. [ADR 0012](0012-architecture-decision-governance.md) records the proposed durable ADR-acceptance model, including reviewer eligibility, the solo-maintainer hold, re-enablement conditions, and the prohibition on synthetic approval. While ADR 0012 is Proposed, it does not override those live authorities. COMMENTED reviews, check/status results, model verdicts, reactions, author approval, predecessor-head approval, or dismissed reviews never substitute for a review that current policy actually requires.

An Accepted ADR is **design authority, not implementation evidence**. Protected-main source, executable tests, built/released artifacts, migrations/configuration, and protected-main operational evidence appropriate to the claim establish current implemented behavior. An ADR may intentionally describe an accepted target that is only partially implemented; product documents must label implementation status separately.

## Accepted protected-main decisions

| ADR | Decision | Status | Governs |
|---|---|---|---|
| [0001](0001-chromium-compatibility-kernel.md) | Retain Chromium as the compatibility kernel | Accepted | Blink/V8/graphics/extensions boundary; Rust control-plane integration |
| [0002](0002-agent-safety-kernel.md) | Agent safety kernel | Accepted | mode, capability, origin, risk, crawler, secret and approval policy |
| [0003](0003-provenance-native-observation.md) | Provenance-native observation | Accepted | evidence/provenance as a first-class product output |
| [0004](0004-resolved-destination-policy.md) | Logical origin and resolved destination safety | Accepted | SSRF/rebinding/special-purpose address and redirect authority |
| [0005](0005-direct-socket-binding.md) | Exact direct TCP peer binding | Accepted | explicit socket authority and operating-system peer proof |
| [0006](0006-tls-server-identity.md) | TLS service identity over the verified peer | Accepted | WebPKI identity, roots, time, ALPN and stream binding |
| [0007](0007-purpose-bound-sensitive-data-authority.md) | Purpose-bound sensitive-data authority | Accepted | tenant/task/field/purpose/destination/classification disclosure authority |
| [0008](0008-leaf-validity-horizon.md) | Delegated-task TLS leaf-validity horizon | Accepted | minimum certificate-validity horizon for bounded delegated tasks |
| [0010](0010-session-context-bound-node-authority.md) | Session/context-bound node authority | Accepted | browser-session, browsing-context, origin, document-epoch and stale-node authority |

## Proposed decisions retained on protected main

Proposed ADR files can live on protected `main` as reviewable target architecture without becoming Accepted or shipped behavior. Their own status metadata remains authoritative until a later reviewed change accepts, supersedes, rejects, or deprecates them.

| ADR | Decision | Status | Governs |
|---|---|---|---|
| [0009](0009-hourly-agent-credential-boundary.md) | Hourly agent credential boundary | Proposed | deterministic gates, NVIDIA credential materialization, local broker and publication separation |
| [0011](0011-manifest-v3-extension-authority.md) | Manifest V3 compatibility and extension-to-Agent authority | Proposed | Chromium extension compatibility evidence, profile separation, extension grants, native-messaging boundary and release claims |
| [0012](0012-architecture-decision-governance.md) | Architecture decision acceptance governance | Proposed | ADR lifecycle authority, reviewer eligibility, solo-maintainer hold and re-enablement conditions |
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

Active feature PRs may contain additional Proposed ADRs. They are not indexed here as protected-main decisions until their files reach protected `main`. Historical PR checks, stale branch state, or chat decisions never transfer ADR acceptance across a changed head.

## Index completeness rule

Every ADR file on protected `main` must be discoverable from this index with a status that agrees with the ADR's own status metadata. A feature ADR in an active PR belongs in that PR's traceability until merge. When an ADR is added, accepted, superseded, deprecated, or rejected, update this index in the same protected change or an immediately coupled documentation reconciliation.

The machine-checkable documentation contract should fail when:

- an ADR file on protected `main` is absent from this index;
- this index claims `Accepted` while the ADR metadata says `Proposed`, or the reverse;
- a superseded ADR lacks a discoverable successor;
- an active-PR ADR is presented as protected-main implementation evidence; or
- a stale PR number, SHA, run ID, automation prompt, or conversation statement is used as timeless architecture authority.

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
12. Manifest V3 extension-to-agent authorization or compatibility evidence policy;
13. tenant, privacy, residency, audit, deployment or enterprise-control ownership;
14. hourly automation credential, writer, continuation or protected-main operational-proof authority; or
15. release acceptance, rollback/recovery or protected-main operational-proof requirements.

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
- [`../DOCUMENTATION_FITNESS.md`](../DOCUMENTATION_FITNESS.md) records semantic completeness and stale/current findings across the graph.

If these artifacts disagree about current implementation, protected-main source, executable tests, built/released artifacts, configuration/migrations, and protected-main operational evidence appropriate to the claim define implementation truth. Accepted ADRs explain governing design decisions; they do not upgrade missing behavior into shipped behavior. The disagreement is a documentation or implementation defect that must be repaired rather than silently rationalized from conversation history.