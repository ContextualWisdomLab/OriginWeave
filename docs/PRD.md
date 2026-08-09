# OriginWeave Product Requirements Document

- **Document status:** Proposed authoritative product baseline
- **Product status:** Pre-alpha
- **Product:** OriginWeave
- **Tagline:** **Browse. Act. Prove.**
- **Canonical architecture:** [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- **Technical requirements:** [`TRD.md`](TRD.md)
- **Roadmap:** [`product-roadmap.md`](product-roadmap.md)
- **Research and standards:** [`doctoring.md`](doctoring.md)

## 1. Purpose

This PRD defines the complete OriginWeave product boundary that previously existed across the root architecture, feature ADRs, implementation plans, roadmap entries, pull-request descriptions, and product-design conversations. It does not convert planned work into shipped functionality. Every requirement below is classified through the implementation-status vocabulary in Section 3 and must be validated against protected `main` before a release claim is made.

OriginWeave is an enterprise agentic web runtime and provenance-native browser platform. Chromium remains the compatibility kernel; Rust owns the governance, authority, resource, evidence, and agent-facing control plane.

## 2. Product vision

A person or enterprise can delegate a bounded web task and receive:

1. the requested result;
2. the exact evidence that supports the result;
3. the policy and approval decisions that authorized every sensitive or state-changing step;
4. the network and service-identity evidence that establishes where the runtime connected;
5. a replayable, inspectable task record;
6. a clear distinction between model judgement and deterministic authority.

OriginWeave must achieve this without giving the model ambient browser authority, raw secrets, unrestricted script execution, uncontrolled network reachability, or the ability to promote page text into trusted instructions.

## 3. Requirement status vocabulary

| Status | Meaning |
|---|---|
| **Implemented** | Present on protected `main` with repository tests and documented authority boundaries. |
| **Accepted architecture** | A reviewed governing direction already represented by binding architecture/ADR documents, but the complete production path may not yet exist. |
| **Planned** | In the product roadmap and consistent with accepted architecture, but not yet a shipped capability. |
| **Proposed** | Conversation-derived or issue-derived product direction that still requires a dedicated reviewed ADR/specification or implementation evidence. |
| **Open** | A decision or acceptance criterion is intentionally unresolved. |

No `Planned`, `Proposed`, or `Open` item may be presented to buyers as shipped.

## 4. Problem statement

General browser automation often combines several authorities that OriginWeave must keep separate:

- a URL string is treated as network authorization;
- successful DNS resolution is treated as a safe destination;
- a connected socket is treated as authenticated HTTPS;
- a selector or element reference is treated as indefinitely valid;
- an LLM is allowed to create arbitrary JavaScript or selectors;
- cookies, passwords, and API keys are placed in model context;
- page content can influence the instruction channel;
- a successful command return is treated as proof that the intended state changed;
- raw HTML, screenshots, and network traffic are retained without bounded provenance;
- background agent inference competes with foreground interaction without an enforceable resource policy.

These shortcuts are unacceptable for a governed enterprise runtime because they make authority ambiguous, failures hard to reproduce, and extracted data difficult to defend during audit or acquisition diligence.

## 5. Primary users and stakeholders

### 5.1 Enterprise automation owner

Needs a runtime that can execute delegated browser work under explicit tenant, origin, risk, data, and resource policies.

### 5.2 Security and governance administrator

Needs least-privilege policy, purpose-bound sensitive-data handling, auditable approval, explicit network authority, extension governance, tenant isolation, and evidence suitable for incident investigation.

### 5.3 Agent platform engineer

Needs a stable protocol that does not couple the orchestrator to Chromium-specific selectors, process identifiers, or secret material.

### 5.4 Data and research engineer

Needs structured extraction with exact source locators, hashes, capture times, model provenance, and reproducible export formats.

### 5.5 Human browser user

Needs normal Chromium-compatible browsing to remain responsive and understandable even while an assistant or delegated task is active.

### 5.6 Operator and procurement reviewer

Needs observable failure modes, upgrade/rollback evidence, SBOM/provenance, compatibility matrices, accessibility evidence, and supportable deployment boundaries.

## 6. Product family

The product family is broader than one desktop browser. Naming below is the stable product vocabulary; individual surfaces become shippable only when their acceptance gates pass.

| Surface | Product responsibility | Current status |
|---|---|---|
| **OriginWeave Browser** | Chromium-compatible interactive distribution with governed agent entry points. | Planned |
| **OriginWeave Runtime** | Headless/embedded execution environment for governed web tasks. | Planned |
| **OriginWeave Observe** | Structured observation from typed tools, structured data, network, AX/DOM/layout, and visual fallback. | Planned |
| **OriginWeave Capture** | Schema-bound extraction, crawler controls, downloads, WARC/PROV-oriented capture. | Planned |
| **OriginWeave Governor** | CPU, RAM, GPU, VRAM, admission, model residency, and task-priority governance. | Accepted architecture; deterministic budget kernel implemented |
| **OriginWeave Policy** | Capability, origin, purpose, risk, crawler, approval, and sensitive-data authority. | Implemented foundation; broader runtime integration planned |
| **OriginWeave Evidence** | Credential-free evidence, provenance, task trail, and export contracts. | Implemented foundation; persistence adapters planned |
| **OriginWeave Protocol** | Versioned browser-agent protocol independent of one automation standard. | Planned |
| **OriginWeave SDK** | Typed client libraries and adapters for external agents and CWL services. | Planned |
| **OriginWeave Enterprise** | Managed policy, SSO/SCIM, tenancy, residency, audit, deployment, and support controls. | Planned |

## 7. Execution modes

### 7.1 Human Mode

Purpose: ordinary person-controlled browsing.

- The user's normal profile and explicitly installed extensions may operate.
- Autonomous agent control is denied by default.
- Human rendering and input have the highest resource priority.

Status: **Accepted architecture**.

### 7.2 Assist Mode

Purpose: summarization, search support, reversible preparation, and user-guided form assistance.

- Reading and reversible preparation may be automated under policy.
- State-changing operations require the risk-specific authorization path.
- Page observations remain untrusted data.

Status: **Accepted architecture; runtime integration Planned**.

### 7.3 Agent Task Mode

Purpose: a bounded delegated workflow.

- Uses an isolated task profile or explicitly attached constrained context.
- Receives task-scoped capabilities, origins, purposes, secrets, and resource budgets.
- Must not inherit the unrestricted human profile by default.
- Every actionable node reference is bound to a session/context/document lifetime before execution.

Status: **Accepted architecture; complete browser integration Planned**.

### 7.4 Crawler Mode

Purpose: governed public collection and monitoring.

- Read-only by default.
- RFC 9309 robots evidence is required but is never treated as access authorization.
- Rate, purpose, retention, copyright/terms, and privacy policy are separate controls.
- CAPTCHA bypass and fingerprint-evasion are non-goals.

Status: **Accepted architecture; complete capture runtime Planned**.

## 8. Core user journeys

### 8.1 Delegated web task

```text
user goal
-> create isolated session
-> establish bounded task authority
-> navigate through authorized network/service boundaries
-> observe structured page state
-> propose typed action
-> evaluate deterministic policy and approval
-> execute through trusted adapter
-> observe expected post-condition
-> store credential-free evidence
-> return result + Evidence Trail
```

### 8.2 Evidence-first extraction

```text
requested schema
-> prefer typed/site-provided data
-> structured metadata
-> bounded network response
-> accessibility + DOM + layout
-> visual fallback only when necessary
-> validate field values
-> bind every field to source evidence
-> export value + provenance
```

### 8.3 Sensitive form completion

```text
model proposes field/purpose/destination
-> policy classifies authority
-> model receives opaque secret handle, never raw value
-> trusted broker verifies current scope
-> browser receives value through trusted fill channel
-> post-condition is observed
-> access/disclosure receipt records metadata without protected value
```

### 8.4 Enterprise crawler

```text
public-crawl purpose
-> origin policy
-> robots/rate/retention checks
-> bounded navigation and extraction
-> no state-changing action
-> WARC/PROV-oriented evidence bundle
```

## 9. Functional requirements

### 9.1 Compatibility

- **PRD-COMP-001 — Accepted architecture.** Chromium is the compatibility kernel; OriginWeave does not reimplement Blink or V8.
- **PRD-COMP-002 — Planned.** Maintain a Manifest V3 compatibility matrix and automated extension test farm for supported APIs.
- **PRD-COMP-003 — Planned.** Chromium-specific integrations are isolated behind versioned adapters; core Rust contracts remain usable without Chromium.
- **PRD-COMP-004 — Planned.** External agents can use OriginWeave Runtime without installing the interactive browser UI.

### 9.2 Session and observation authority

- **PRD-OBS-001 — Planned.** Every autonomous task has an explicit browser-session identity and browsing-context identity.
- **PRD-OBS-002 — Planned.** Actionable node handles are invalid after the associated document epoch changes.
- **PRD-OBS-003 — Accepted architecture.** Observation prefers the most structured trustworthy source available: site tool/WebMCP, structured data, bounded network data, Accessibility+DOM+layout, then screenshot/vision fallback.
- **PRD-OBS-004 — Planned.** Full semantic snapshots are followed by bounded incremental diffs rather than repeated complete DOM copies.
- **PRD-OBS-005 — Planned.** Hidden, inaccessible, visually contradictory, or cross-origin observation channels remain distinguishable in evidence.

### 9.3 Typed action execution

- **PRD-ACT-001 — Implemented foundation.** Actions have typed kinds, risk classes, canonical targets, and immutable intent digests.
- **PRD-ACT-002 — Accepted architecture.** Arbitrary JavaScript is not a default production agent tool.
- **PRD-ACT-003 — Planned.** Standard actions include navigation, query, click, text input, approved upload/download, selection, scrolling, waiting, extraction, and evidence capture.
- **PRD-ACT-004 — Accepted architecture.** Command completion is not success; an expected state transition or post-condition must be observed.
- **PRD-ACT-005 — Implemented foundation.** High-risk approvals are bound to the exact action, target origin, and complete intent digest.

### 9.4 Network and service authority

- **PRD-NET-001 — Implemented.** Logical origin is distinct from resolved destination authorization.
- **PRD-NET-002 — Implemented.** Approved resolution snapshots are non-empty, bounded, origin-bound, and fail closed on unapproved address expansion.
- **PRD-NET-003 — Implemented.** Direct transport accepts an exact approved socket and verifies the operating-system peer before stream exposure.
- **PRD-NET-004 — Implemented.** TLS service identity is verified over the same governed transport with explicit roots and trusted time.
- **PRD-NET-005 — Planned.** Proxy/PAC routing authority is explicit and cannot be inherited ambiently.
- **PRD-NET-006 — In progress.** Bounded HTTP semantics operate only over an authenticated governed connection and preserve redirect reauthorization.
- **PRD-NET-007 — Planned.** Chromium's real navigation path must prove it consumes the governed destination, route, transport, TLS, and HTTP authorities before OriginWeave claims safe production navigation.

### 9.5 Secret and sensitive-data authority

- **PRD-DATA-001 — Accepted architecture.** Raw secrets never enter model prompts, page observations, traces, or provenance values.
- **PRD-DATA-002 — In progress.** Sensitive disclosure is purpose-bound by tenant/task/field/destination/classification authority.
- **PRD-DATA-003 — Planned.** The trusted broker owns atomic use reservation, current trusted time, revocation, concurrent/replay protection, value resolution, and compensation semantics.
- **PRD-DATA-004 — Planned.** Enterprise privacy uses purpose limitation, field/record authorization, encryption, bounded retention, export controls, and auditable privileged access rather than blanket masking.

### 9.6 Evidence and provenance

- **PRD-EVD-001 — Implemented foundation.** Generic network evidence is credential-free and universally value-redacted.
- **PRD-EVD-002 — Implemented foundation.** Provenance records bind data to validated source locators and hashes.
- **PRD-EVD-003 — Planned.** Evidence Trail links final result -> extracted value/action -> observation/source -> model judgement when present -> deterministic policy -> approval -> observed outcome.
- **PRD-EVD-004 — Proposed product UX.** **Origin Map** presents this lineage interactively to a user or auditor.
- **PRD-EVD-005 — Planned.** WARC-compatible source capture and PROV-compatible derivation serialization are separate adapters.
- **PRD-EVD-006 — In progress.** Sensitive-access receipts record authorization and lifecycle metadata without the protected value.

### 9.7 Resource governance

- **PRD-RES-001 — Implemented foundation.** Resource budgets produce cumulative deterministic mitigations rather than one lossy pressure enum.
- **PRD-RES-002 — Accepted architecture.** Human input, foreground rendering, and compositor health outrank model inference and background capture.
- **PRD-RES-003 — In progress.** CPU worker admission participates in task admission rather than being dead configuration.
- **PRD-RES-004 — Planned.** Platform adapters report RSS, JS heap, observation cache, frame time, GPU/VRAM use, model residency, and related telemetry.
- **PRD-RES-005 — Planned.** Constrained GPU deployments phase-schedule rendering and local inference, shrink batches, release model caches, and fall back to CPU before sacrificing visible interaction.

### 9.8 External agent interoperability

- **PRD-INT-001 — Planned.** WebDriver BiDi is supported behind a versioned adapter, not used as the internal authority model.
- **PRD-INT-002 — Planned.** Chrome DevTools Protocol capabilities are version-gated and limited to the adapter surface that needs them.
- **PRD-INT-003 — Planned.** WebMCP is treated as a potentially useful site-tool channel but never as the only observation path or as trusted instruction authority.
- **PRD-INT-004 — Planned.** Model Context Protocol exposes bounded OriginWeave tools through the Rust runtime rather than attaching model providers directly to Chromium authority.
- **PRD-INT-005 — Planned.** OriginWeave Protocol/BAP provides an internal stable contract that can be implemented by BiDi, CDP, WebMCP, MCP, desktop, headless, and embedded adapters.

### 9.9 Extensions

- **PRD-EXT-001 — Accepted architecture.** Manifest V3 remains the compatibility baseline.
- **PRD-EXT-002 — Planned.** Existing extension APIs are preserved upstream where possible rather than reimplemented in Rust.
- **PRD-EXT-003 — Planned.** Extension access to agent observation/action authority requires a separate signed policy grant and is not implied by ordinary extension permissions.
- **PRD-EXT-004 — Planned.** Compatibility tests cover install/update, service-worker lifecycle, content scripts, storage, DNR, native messaging, download, side panel, restart, and agent isolation.

### 9.10 Crawler and capture policy

- **PRD-CRAWL-001 — Implemented policy foundation.** Crawler mutation is denied and missing robots evidence fails closed where public-crawl policy requires it.
- **PRD-CRAWL-002 — Planned.** Rate, depth, page count, concurrency, retention, terms, copyright, and personal-data controls are explicit.
- **PRD-CRAWL-003 — Non-goal.** CAPTCHA bypass, fingerprint evasion, or deliberate access-control circumvention are not product capabilities.

### 9.11 Enterprise operation

- **PRD-ENT-001 — Planned.** SSO/SCIM, tenant isolation, managed policy, regional data residency, encrypted profiles, immutable audit, backup/restore, and break-glass workflows.
- **PRD-ENT-002 — Planned.** OpenTelemetry-compatible metrics/traces expose task success, stale-action rate, policy denials, queue/resource pressure, network/TLS/HTTP timing, evidence completeness, and recovery outcomes.
- **PRD-ENT-003 — Planned.** Operator workflows include cancellation, quarantine, replay, crash recovery, rollback, incident evidence, and controlled upgrade.
- **PRD-ENT-004 — Planned.** Procurement evidence includes SBOM, signed provenance, reproducible release evidence, security/change-control documentation, and supported compatibility matrices.

## 10. Non-functional requirements

### 10.1 Correctness

- Rust-owned production behavior requires exact repository coverage gates and meaningful contract/property/integration tests.
- Network and browser authority boundaries fail closed on ambiguity.
- An action result is accepted only after observable post-condition verification.

### 10.2 Security

- Assume a renderer or page may be hostile.
- Page text, WebMCP output, downloads, email-like content, and model-produced data are untrusted observations.
- Deterministic policy cannot be weakened by a model decision.
- Secrets and credentials remain in trusted brokers or adapters.
- Explicit destination and service identity remain separate from application semantics.

### 10.3 Reliability

- Every long-running task must have bounded time/resource policy, cancellation semantics, and recoverable checkpoints where the adapter supports them.
- Retry is limited to evidence-classified transient conditions and never used to obscure deterministic failures.

### 10.4 Performance

- Foreground interaction has priority over autonomous work.
- Observation payloads are bounded and incremental when possible.
- Local inference must not assume unlimited VRAM or memory.

### 10.5 Accessibility

Interactive OriginWeave UI surfaces target WCAG 2.2 AA / ISO/IEC 40500:2025-aligned accessibility evidence. Required UI work includes keyboard operation, visible focus, accessible names/status, non-color-only risk indicators, safe error recovery, and exact-value/evidence alternatives where a visualization is used.

### 10.6 Interoperability

- Public interfaces are versioned.
- Adapter-specific identifiers never become durable core authority identifiers without translation through scoped registries.
- External protocol evolution must be absorbed by adapters where practical.

### 10.7 Reproducibility and supply chain

- Lock dependencies and toolchains.
- Pin security-sensitive automation and record source provenance.
- Produce SBOM/provenance and verify released artifacts before claiming release acceptance.

## 11. Degraded behavior

OriginWeave must remain truthful under partial capability:

| Condition | Required behavior |
|---|---|
| Model provider unavailable | Deterministic browser/runtime paths continue where they do not require the model; model-backed task reports a bounded unavailable state. |
| Secret broker unavailable | Secret-dependent action fails closed; raw secret is not substituted into model context. |
| Unsupported page semantics | Fall through the observation hierarchy; use visual interpretation only when policy allows it and label its evidence source. |
| No safe network authority | Do not connect. |
| TLS identity failure | Do not proceed to application semantics. |
| Post-condition not observed | Report action failure/uncertainty rather than success. |
| Resource hard limit | Pause/deny autonomous work before sacrificing interactive safety. |
| Missing independent release/merge evidence | Do not call the branch or artifact accepted. |

## 12. Buyer-visible acceptance

A commercial milestone is accepted only when a buyer can verify the stated outcome on the exact supported release artifact.

### 12.1 Controlled delegated task

A versioned task suite repeatedly completes navigation, observation, typed action, post-condition verification, and evidence export without raw selectors/scripts/secrets becoming ambient model authority.

### 12.2 Governed networking

For supported navigation, evidence shows logical origin, approved resolution, selected route, observed TCP peer, authenticated TLS service identity, bounded HTTP behavior, redirects, and downloads as separate decisions on the real adapter path.

### 12.3 Provenance-native extraction

A versioned benchmark corpus has explicit field precision/recall and provenance-completeness thresholds. Every accepted field points to source evidence and transformation/model provenance where applicable.

### 12.4 Resource protection

Supported hardware profiles demonstrate bounded task RSS/VRAM and defined fallback behavior while foreground interaction stays within published latency/frame-health targets.

### 12.5 Extension compatibility

A documented Manifest V3 compatibility suite passes for supported Chrome APIs and proves task-mode isolation from extension-sensitive state.

### 12.6 Enterprise readiness

A regulated buyer can inspect access policy, evidence retention, tenant boundaries, incident response, rollback, audit trails, SBOM/provenance, accessibility, and operational SLO evidence without reconstructing undocumented assumptions.

## 13. Explicit Non-goals

- Rewriting Blink or V8 in Rust.
- Presenting the current policy/network/TLS kernels as a complete production browser.
- Arbitrary JavaScript execution as a normal agent tool.
- Sharing the unrestricted default human profile with autonomous tasks.
- CAPTCHA bypass, browser-fingerprint evasion, or access-control circumvention.
- Treating robots.txt as legal or authentication authority.
- Sending raw passwords, cookies, API keys, or protected values to an LLM.
- Treating a green model verdict, comment, status, or synthetic merge as independent release/merge authority.
- Claiming SOC 2, CSAP, privacy, accessibility, or safety certification without the corresponding external evidence.

## 14. Product success metrics

Metrics are release-profile specific and must be captured with versioned fixtures/tasks:

- delegated task success and repeated-run variance;
- unauthorized action rate;
- prompt-injection success rate;
- stale-node action rejection rate;
- extraction precision/recall;
- provenance completeness;
- task peak RSS and VRAM;
- foreground frame/input latency under autonomous load;
- connection, TLS, HTTP, and end-to-end task timing;
- extension compatibility pass rate;
- crash/cancellation recovery rate;
- evidence replay and artifact verification success;
- accessibility conformance evidence for supported UI flows.

Targets are established per release profile; this baseline does not invent numerical claims before benchmark evidence exists.

## 15. Release outcomes

OriginWeave may increment and publish a product version only from an exact protected head whose required CI, security, owned-code coverage, packaging, compatibility, accessibility, SBOM/provenance, reproducibility, rollback/recovery, independent review, and release-acceptance gates are satisfied. The changelog must distinguish shipped features from accepted or proposed architecture.

## 16. Standards and evidence

The authoritative research/standards bibliography and evidence-to-architecture discussion live in [`doctoring.md`](doctoring.md). In particular, OriginWeave treats the current WebDriver BiDi specification as an evolving adapter contract, Manifest V3 as the current Chrome extension baseline, WCAG 2.2 as the accessibility target for product UI, and NIST AI 600-1 as risk-management input rather than certification. Material new product claims must update doctoring and the governing ADR/specification rather than adding uncited claims only to this PRD.
