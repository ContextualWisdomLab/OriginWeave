# OriginWeave Technical Requirements Document

- **Document status:** Proposed authoritative technical baseline
- **Product status:** Pre-alpha
- **Product requirements:** [`PRD.md`](PRD.md)
- **Canonical system architecture:** [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- **Architecture decisions:** [`adr/README.md`](adr/README.md)
- **Research and standards:** [`doctoring.md`](doctoring.md)

## 1. Purpose and truth model

This TRD defines technical invariants for OriginWeave without describing planned modules as shipped. The implementation-status vocabulary is:

- **Implemented** — present on protected `main` with executable repository evidence.
- **Accepted architecture** — binding design direction represented by reviewed architecture/ADR material, while complete adapters may remain absent.
- **Planned** — roadmap work that has not reached protected `main`.
- **Proposed** — a candidate design that still needs a dedicated reviewed decision or implementation proof.
- **Open** — deliberately unresolved.

Pull-request code is not treated as Implemented until it reaches protected `main` and required acceptance evidence is re-established there. Active-PR implementation may be recorded in a separate evidence note, but it never creates a composite implementation status.

## 2. Current protected-main implementation inventory

The current reusable Rust control plane is intentionally smaller than the final browser product. The status column describes protected `main` only; active PR evidence is kept in the final column.

| Module / boundary | Current responsibility | Protected-main status | Active/non-shipped evidence |
|---|---|---|---|
| `originweave-core` | Canonical origin, typed actions, purpose/mode, capabilities, risk, secret-delivery, approval, session/context/document/node authority values. | **Implemented** | Active origin-bound `ExtensionAgentGrant` evaluation adds canonical-origin matching and exclusive trusted-time expiry; it is not protected-main truth until merge |
| `originweave-policy` | Pure fail-closed action policy including purpose-bound sensitive-data authority. | **Implemented** | Trusted broker/runtime lifecycle remains separate planned work under issue #10 |
| `originweave-destination` | Resolved-address classification, origin-bound snapshots, route authority, connection pinning, rebinding and redirect authority. | **Implemented** | PAC evaluation/proxy transport/CONNECT are still Planned |
| `originweave-network` | Direct single-address TCP connection plan and exact operating-system peer verification. | **Implemented** | — |
| `originweave-tls` | WebPKI service identity over the already verified TCP stream. | **Implemented** | — |
| `originweave-resource` | Deterministic resource budgets, CPU-worker admission and cumulative mitigation plans. | **Implemented** | Platform telemetry/actuation remains Planned |
| `originweave-evidence` | Value-redacted network evidence, provenance foundations and sensitive-access evidence primitives. | **Implemented** | Complete durable Evidence Trail/WARC/PROV persistence remains Planned |
| Browser/session protocol registry | Bind raw BiDi/CDP identifiers to OriginWeave session/context/document authority. | **Planned** | Active PR #40; core lifetime value contracts are already Implemented under ADR 0010 |
| Semantic observation/action browser adapters | Chromium/BiDi/CDP observation, node lifecycle, typed input and post-condition verification. | **Planned** | Issue #28 |
| Bounded HTTP execution | HTTP/1.1 semantics over authenticated governed transport. | **Planned** | Active replacement PR #37; historical PR #11 is predecessor lineage, not current evidence |
| Proxy/PAC execution | Evaluate authorized route selection and perform governed proxy/CONNECT transport. | **Planned** | Protected-main route-authority value foundation already exists |
| Sensitive-data broker persistence/runtime | Atomic opaque-handle lifecycle, revocation/reservation, value resolution and trusted fill. | **Planned** | Protected-main policy/evidence foundations exist; issue #10 owns complete runtime lifecycle |
| Manifest V3 compatibility program | Real pinned-Chromium extension compatibility and release matrix. | **Planned** | Protected main already contains partial real-browser evidence; active PR #43 adds downloads evidence |
| WARC/PROV persistence | Durable capture and provenance serialization. | **Planned** | — |

## 3. Architectural invariants

### TRD-INV-001 — Chromium compatibility kernel

**Accepted architecture.** Blink, V8, Skia, Viz, Dawn/WebGPU, Site Isolation, sandboxing, and the Manifest V3 extension runtime remain Chromium/upstream responsibilities. OriginWeave adds Rust control-plane behavior through narrow adapters or service boundaries.

### TRD-INV-002 — Explicit authority dimensions

The following authorities are separate and may not be inferred from one another:

```text
user / enterprise goal
-> task purpose
-> browser session and browsing context
-> typed action capability
-> logical origin
-> resolved destination
-> proxy/PAC route authority
-> exact TCP peer
-> TLS service identity
-> bounded HTTP semantics
-> document epoch + observed node authority
-> sensitive-data / secret authority
-> risk-specific approval
-> observed post-condition
-> evidence/provenance
```

A **logical origin** is not a **resolved destination** decision. A resolved address approval is not a **TCP peer** proof. A verified TCP peer is not **TLS service identity**. TLS identity is not an HTTP body/resource-budget decision. A valid node handle is not action capability. A policy decision is not evidence that execution succeeded.

### TRD-INV-003 — Untrusted page content

Browser content, rendered text, hidden text, comments, ads, WebMCP output, network bodies, downloads, extension messages, and model-produced summaries are data. They cannot mutate system policy, expand capabilities, authorize destinations, reveal secrets, or redefine the user's goal.

### TRD-INV-004 — Secret separation

Raw credentials and protected values do not enter LLM prompts or untrusted observation channels. Model-visible values are opaque handles or independently safe derived values. Trusted resolution/fill occurs outside model context.

### TRD-INV-005 — Observe/verify action lifecycle

A typed action may be attempted only after exact current authority is validated. Success requires an observed post-condition. Adapter return without the expected state transition is failure or uncertainty, not success.

## 4. Execution-mode requirements

### Human Mode

**Accepted architecture.** Autonomous control is denied unless the human explicitly transitions or delegates into another governed mode. Human input/rendering priority is highest.

### Assist Mode

**Accepted architecture.** Reversible/read behavior may be automated. Irreversible or externally visible state changes re-enter the risk/approval pipeline. The browser adapter path remains Planned.

### Agent Task Mode

**Accepted architecture.** Each delegated task receives an isolated or explicitly attached browser context, scoped capabilities, origins, secrets, policy and resource budgets. The unrestricted default human profile is not ambient task authority. Complete browser adapter/session integration remains Planned.

### Crawler Mode

**Accepted architecture.** The read-only crawler policy foundation is Implemented, while the complete crawler runtime is Planned. Robots evidence, rate controls, purpose, privacy, retention and legal/contract policy are distinct checks.

## 5. Identifier and lifetime contracts

### 5.1 Core identifiers

Protected-main core contracts already define opaque browser-session, browsing-context, document-epoch and observed-node authority values governed by Accepted ADR 0010. External browser identifiers must be translated through scoped registries instead of becoming core authority directly. The protocol-ID registry is active PR #40 evidence until protected integration.

Required browser-lifetime tuple:

```text
browser_session_id
+ browsing_context_id
+ canonical_origin
+ document_epoch
+ adapter_local_node_id
```

An actionable node reference is valid only when every component matches the live task context immediately before use.

### 5.2 Document epochs

Navigation, document replacement, or another adapter-defined actionable-document lifetime change rotates `document_epoch`. A stale node reference must fail deterministically before input dispatch. Core exact-authority validation is Implemented; real browser lifecycle invalidation/linearized dispatch remains adapter work.

### 5.3 Idempotency

Commands that can be replayed through retries or operator recovery require explicit idempotency semantics. Idempotency keys are scoped at least by tenant/task/action contract and cannot turn a semantically different action into the same request.

The active `originweave-bap` lifecycle lane provides only the bounded in-memory
portion of this contract: an immutable command receipt binds a tenant namespace,
key, and task ID to one accepted lifecycle transition. The caller-supplied tenant
namespace scopes retry identity only; it is not authenticated tenant authority.
Durable storage, authenticated tenant binding, concurrent deduplication, and
externally visible side-effect suppression remain unimplemented.

## 6. Network authority stack

### 6.1 Origin

**Implemented.** Canonical origin parsing follows browser-compatible host semantics and rejects dangerous ambiguous numeric-host forms. The origin represents logical web identity only.

### 6.2 Resolution and destination classification

**Implemented.** A trusted adapter supplies a nonempty bounded resolver result. The destination kernel:

- canonicalizes IPv4-mapped IPv6 before classification;
- applies reviewed special-purpose/platform endpoint rules;
- binds approved addresses to the origin;
- requires connection candidates to belong to the approved set;
- permits nonempty DNS contraction;
- rejects newly introduced addresses as possible rebinding;
- reauthorizes redirect targets independently.

The pure destination crate itself does no DNS lookup.

### 6.3 Route/proxy authority

**Protected-main status: Implemented for route-authority foundations. Proxy/PAC execution: Planned.** Direct routing is the default. Proxy and PAC-selected routes require explicit authority. A proxy is an intermediate authority and never replaces final-target authorization. Ambient environment proxy variables cannot silently change the governed route. PAC evaluation, proxy transport and CONNECT require separate execution evidence before release claims.

### 6.4 Direct transport

**Implemented.** `originweave-network` receives an exact authorized canonical socket, applies bounded attempt/timeout policy, connects to that address, and verifies `peer_addr` before exposing the stream. It performs no hostname re-resolution and does not authenticate HTTPS identity.

### 6.5 TLS service identity

**Implemented.** `originweave-tls` consumes the exact verified stream. Reference identity comes only from the canonical HTTPS origin. DNS identities require `dNSName` SAN; IP literals require exact `iPAddress` SAN; Common Name fallback is prohibited. Trust roots and verification time are explicit. The first slice permits TLS 1.3 and TLS 1.2, disables early data/resumption/key logging/secret extraction/client certificates/dangerous custom verifiers, and records bounded credential-free evidence.

### 6.6 HTTP semantics

**Protected-main status: Planned.** Active replacement PR #37 implements bounded HTTP/1.1 semantics but remains non-shipped evidence until protected integration. Historical PR #11 is predecessor lineage and is not current implementation evidence.

HTTP processing must consume an authenticated governed connection and define:

- supported methods and caller-controlled fields;
- syntax/framing rules;
- header, trailer, chunk, decoded-body and elapsed-time budgets;
- digest/integrity handling;
- MIME and content-disposition interpretation;
- redirect metadata and per-hop reauthorization;
- download byte and persistence gates;
- connection-reuse authority if later introduced.

No HTTP adapter may reconnect by hostname behind the authority stack without a new authorization path.

### 6.7 Browser network integration

**Planned and release-critical.** Safe navigation is not a supported claim until the real Chromium/browser adapter demonstrates that its real network path consumes the governed resolution, route, transport, TLS and HTTP authorities without an alternate ambient connection path.

## 7. Observation architecture

Observation order is an **Accepted architecture** requirement:

1. site-provided typed tool / WebMCP when present and policy-allowed;
2. JSON-LD, Microdata, RDFa, Open Graph, HTML table/form/link and ARIA structure;
3. bounded, authorized network data;
4. Accessibility tree + DOM + layout;
5. screenshot/vision fallback for canvas, remote-desktop, image-only or semantically inaccessible UI.

Raw HTML is not the default model payload.

### 7.1 Semantic snapshot

**Planned.** A semantic snapshot includes bounded node identity, role, accessible name, state, canonical origin, visibility/layout evidence, available typed actions and source channels. It must not include raw credentials or unrestricted network headers.

### 7.2 Incremental observation

**Planned.** After a full snapshot, subsequent updates use bounded semantic diffs where possible. Observation cache use is included in task resource accounting.

### 7.3 Source disagreement

**Planned.** If accessibility, DOM/layout, structured data, network data and visual evidence disagree materially, evidence records the disagreement; the runtime does not silently select the most convenient interpretation.

## 8. Action architecture

The standard action vocabulary is **Accepted architecture**; complete real-browser runtime integration remains Planned:

```text
navigate
go_back
query_nodes
click_node
type_text
fill_secret
select_option
set_checkbox
scroll_container
wait_for_state
download_resource
upload_approved_file
extract_schema
capture_evidence
```

Arbitrary script execution is outside the standard production interface.

### 8.1 Pre-execution checks

Immediately before a state-changing action, validate:

- session mode and purpose;
- exact task/action capability;
- node/session/context/origin/document lifetime when a node is used;
- current destination/route/service authority if network activity is implied;
- sensitive-data scope and secret-handle authority;
- risk class and exact current approval evidence;
- resource/admission state.

### 8.2 Post-condition

The adapter declares an observable post-condition contract, such as URL change, dialog appearance, field state, download evidence or bounded network mutation. Failure to observe it is not success.

## 9. Sensitive-data and secret requirements

### 9.1 Purpose-bound authority

**Implemented policy foundation.** Protected disclosure authority binds tenant, task, field, business purpose, canonical destination and data classification under Accepted ADR 0007. This implementation does not imply that trusted value storage, opaque-handle resolution, revocation or browser fill are complete.

### 9.2 Opaque handle broker

**Planned.** The broker, not the model, owns:

- trusted time;
- caller-unforgeable handle state;
- atomic use reservation/increment;
- maximum-use enforcement;
- expiry/revocation;
- concurrent and replay protection;
- value resolution/fill;
- compensation/recovery after reserved-but-failed use.

Issue #10 owns the broader broker/storage/lifecycle completion.

### 9.3 Evidence

Protected-main evidence primitives can record purpose-bound sensitive-access authority without carrying the protected value. Complete broker-use receipts must remain aligned with the runtime lifecycle once that broker exists.

## 10. Resource-governor requirements

### 10.1 Deterministic kernel

**Implemented.** `originweave-resource` validates budgets, includes CPU-worker admission state, and produces a cumulative mitigation plan. It does not sample the operating system or directly schedule processes.

### 10.2 Adapter telemetry

**Planned.** Platform adapters supply bounded observations for process/task RSS, JS heap, observation cache, frame time, GPU/VRAM use, model residency, batch size and CPU-worker use.

### 10.3 Priority order

**Accepted architecture.** Human interaction and compositor/foreground browser health outrank autonomous inference and background capture.

### 10.4 Constrained GPU

**Accepted architecture.** Rendering and local model inference use phase scheduling where necessary. The implementation of platform GPU telemetry/scheduling remains Planned. The mitigation ladder can shrink model batches, release inference caches, offload to CPU, pause the task and reject admission before foreground rendering is sacrificed.

## 11. Evidence and provenance requirements

### 11.1 Evidence identity

Evidence must record the exact authority/result it proves rather than collapse multiple authorities into one green boolean. Examples:

```text
origin_authority
resolution_evidence
route_evidence
connection_evidence
tls_identity_evidence
http_exchange_evidence
observation_evidence
policy_decision
action_event
post_condition_evidence
provenance_record
```

### 11.2 Data minimization

Generic network evidence retains bounded names and canonical locators while values are universally redacted unless a separate schema-specific capture contract authorizes typed values.

### 11.3 Durable adapters

**Planned.** WARC-compatible source capture, relational metadata, object artifacts, and PROV-compatible derivation are independent adapters. The logical ERD in [`erd/README.md`](erd/README.md) is conceptual and does not claim that all entities already have durable tables.

## 12. External protocol adapters

### WebDriver BiDi

**Planned.** WebDriver BiDi is an evolving W3C adapter contract. Its session/user-context/browsing-context identifiers are translated into OriginWeave-scoped internal identities. Core lifetime authority is already Implemented; active PR #40 is non-shipped registry implementation evidence.

### Chrome DevTools Protocol

**Planned.** The **Chrome DevTools Protocol** supplies Chromium-specific observation, diagnostics and experimental capabilities. OriginWeave binds supported protocol versions and does not expose unrestricted Runtime evaluation as a normal agent action.

### WebMCP

**Planned.** **WebMCP** is an experimental external dependency that can provide typed page tools. Tool schemas and outputs remain untrusted page-originated data and cannot grant OriginWeave authority.

### Model Context Protocol

**Planned.** The **Model Context Protocol** adapter exposes a small stable OriginWeave tool surface to orchestrators. MCP providers do not receive direct Chromium credentials or bypass the Rust policy/runtime boundary.

### OriginWeave Protocol

**Planned.** A versioned internal Browser Agent Protocol/OriginWeave Protocol is the product contract. BiDi, CDP, WebMCP and MCP are adapters rather than the core data model.

## 13. Manifest V3 extension requirements

The complete compatibility program is **Planned** under issue #27, while partial real-browser evidence exists on protected main. OriginWeave preserves Chromium's extension implementation rather than rebuilding Chrome APIs in Rust. Agent authority remains separate from ordinary extension permissions. Proposed ADR 0013 documents this separation but is not Accepted design authority until reviewed/integrated accordingly.

Protected-main pinned-Chromium evidence currently exercises service-worker lifecycle, content scripts, storage, declarativeNetRequest, tabs, windows, scripting, commands, side panel, bookmarks, history, restart persistence and repeatability. Active PR #43 adds a bounded real `chrome.downloads` path and allowlisted download-stage failure evidence. Installation/update, native messaging, managed-extension/enterprise policy, broader isolation, Web Store and release-wide compatibility remain outside the current protected-main claim.

## 14. Prompt-injection and model boundary

### 14.1 Trust classes

```text
trusted_instruction
untrusted_observation
protected_secret
```

No automatic transformation may promote an `untrusted_observation` into `trusted_instruction`.

### 14.2 LLM authority

The model may propose plans, classifications or typed actions but cannot:

- create capabilities;
- expand origin/destination/route authority;
- reveal raw secrets;
- manufacture approval;
- change deterministic policy;
- make a failed post-condition successful;
- reinterpret missing required GitHub/release evidence as passing.

### 14.3 Model-backed tests and automation

Live model work uses GitHub Secret **`NVIDIA_NIM_API_KEY`** through a bounded trusted path and preferably `contextual-orchestrator` where appropriate. Scheduled autonomous development uses an immutably pinned OpenCode Agent. **`COPILOT_GITHUB_TOKEN`** is not a development-scheduler credential. Deterministic repository gates run before optional model credentials are materialized.

## 15. Error, retry, timeout and cancellation requirements

### 15.1 Typed errors

Each boundary returns errors that preserve the responsible layer and safe underlying cause without leaking credentials or protected payloads.

### 15.2 Retry

Retry is allow-listed for evidence-classified transient failures. Deterministic validation, permission, origin/destination, identity, malformed-input and policy failures are not retried merely to consume an attempt budget.

### 15.3 Timeouts

Per-layer timeouts do not substitute for an end-to-end deadline. Future composed runtime work must carry an explicit task/deadline budget and allocate remaining time to adapters rather than independently resetting unlimited timers.

### 15.4 Cancellation

Long-running tasks and external model calls require cancellation semantics that preserve evidence of what was attempted, what may have committed externally, and what remains safe to retry.

## 16. Concurrency requirements

- Browser/session state is tenant/task scoped; no in-process global singleton may become authority.
- Node/action validation occurs immediately before execution to close stale-state races.
- Sensitive-handle use becomes atomic in the trusted broker.
- Migration/release/automation writer leases prevent competing repository writers.
- Repository-scoped collision-sensitive identifiers such as ADR numbers, migration IDs and protocol/schema versions are reserved across protected main plus active work before allocation.
- Platform compute pools avoid avoidable oversubscription between Chromium, Rust and model runtimes.

## 17. Persistence and data naming

OriginWeave-owned persistent database objects use descriptive names containing at least two semantic words and `snake_case` by default. Examples:

```text
agent_session
browser_profile
page_snapshot
semantic_node
action_event
policy_decision
provenance_record
resource_budget
extension_grant
task_checkpoint
network_exchange
download_artifact
```

The conceptual model is defined in [`erd/README.md`](erd/README.md). Adapters may use WARC/object storage/relational stores independently; cross-service application database access is not an integration contract. Conceptual ERD entities are not evidence that a physical relational schema exists.

## 18. Security and enterprise controls

The implementation targets evidence useful for CSAP/SOC 2-oriented secure operation without claiming certification. Requirements include least privilege, tenant separation, encrypted sensitive persistence, key lifecycle, purpose limitation, retention/export control, auditable privileged access, incident/vulnerability management, immutable build provenance, dependency review, sandbox/site isolation preservation and explicit egress policy.

Renderer compromise is within the threat model; privileged validation occurs outside renderer-controlled state.

## 19. Accessibility requirements

Product UI targets WCAG 2.2 AA / ISO/IEC 40500:2025-aligned evidence. Approval, evidence, policy, task and resource interfaces require keyboard operation, visible focus, accessible state/error announcements, non-color-only risk cues, safe confirmation flows and exact-value alternatives for diagrams/charts.

## 20. Test strategy

### Implemented kernels

Require deterministic unit/property/integration tests for canonicalization, classification, rebinding, redirects, route authority, direct peers, TLS identity, policy, session/node authority values, resources and evidence.

### Browser vertical slice

Requires real browser integration tests covering isolated contexts, protocol-ID registry binding, stale nodes, iframes/shadow DOM where supported, origin changes, typed actions, post-conditions, crashes, cancellations and governed real network composition.

### Manifest V3 compatibility

Maintain pinned real-Chromium evidence for every claimed extension surface, with restart/repeatability and bounded failure diagnostics. Compatibility evidence and Agent-authority evidence are independent: neither can substitute for the other.

### Security

Cover hostile Unicode/URLs, prompt injection, hidden content, redirect/private-address attempts, DNS rebinding, metadata endpoints, proxy bypass, certificate confusion, request smuggling/framing, decompression/resource exhaustion, MIME/integrity confusion, secret replay and cross-tenant authority.

### Resource

Cover CPU/RAM/VRAM pressure and foreground-interaction fallback on declared hardware profiles.

### Interoperability

Maintain versioned adapter contracts for BiDi/CDP/MCP/WebMCP and a Manifest V3 compatibility farm.

### Quality gate

Owned production code must satisfy the repository's exact function/line/region/statement/branch coverage policy and public rustdoc/docstring contract. Tests must measure real behavior rather than exclude production paths.

## 21. Observability and operability

**Planned.** OpenTelemetry-compatible metrics/traces and durable audit should expose at least:

- task/session state and failure class;
- policy/approval denials;
- stale node and post-condition failures;
- destination/route/TLS/HTTP timing and rejection categories;
- resource pressure and mitigation decisions;
- observation compression/cache size;
- provider/model invocation outcomes without raw secrets;
- provenance completeness;
- crash/cancel/recovery state.

Operator workflows include cancellation, quarantine, bounded retry, evidence inspection, rollback, supported-version upgrade and incident reopening.

## 22. Release requirements

A release is valid only from an exact protected head whose required CI, security, exact owned-code coverage, packaging, SBOM/provenance, reproducibility, supported Chromium/protocol/extension compatibility, accessibility, migration/rollback, independent review and release-acceptance evidence are all satisfied.

Feature-branch green status is not protected-main operational proof. Incident repairs that affect scheduled/runtime behavior require protected-main execution evidence after merge.

## 23. Documentation change control

A material change to any of the following must update the authoritative documentation graph in the same or prerequisite reviewed change:

- product surface or execution mode;
- origin/destination/route/TCP/TLS/HTTP authority;
- session/context/document/node lifetime;
- risk/approval/sensitive-data/secret authority;
- evidence/provenance schema;
- observation hierarchy or action lifecycle;
- resource scheduling/telemetry contract;
- external protocol/version boundary;
- enterprise privacy/security/tenancy contract;
- release acceptance or rollback semantics.

If a decision is not implemented, the documentation must retain `Planned`, `Proposed`, or `Open` status rather than silently describe it as shipped. Active PR evidence remains explicitly non-shipped until protected integration and exact acceptance evidence exist.
