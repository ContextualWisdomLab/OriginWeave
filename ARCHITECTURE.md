# OriginWeave Architecture

## 1. Product definition

OriginWeave is an enterprise agentic web runtime and provenance-native browser control plane. Chromium remains the compatibility kernel; Rust owns new governance, resource, evidence, and agent-facing contracts. This separation minimizes the Chromium patch surface and allows the same Rust modules to operate in a desktop browser, headless service, naruon module, or external agent runtime.

## 2. Architectural principles

1. **Compatibility before reinvention.** Blink, V8, Skia, Viz, Dawn, Site Isolation, sandboxing, and Manifest V3 remain upstream-compatible.
2. **Authority is explicit.** No ambient browser state or page content implicitly grants a capability.
3. **Actions are typed.** Production agents do not receive unrestricted JavaScript evaluation as a default tool.
4. **Observe before acting; verify after acting.** A command is successful only when its expected post-condition is observed.
5. **Secrets stay outside model context.** Models receive opaque handles; a broker resolves values directly into a trusted browser process.
6. **Human interaction wins resource contention.** Rendering, input, and active-tab work outrank inference and background collection.
7. **Evidence is a product output.** Extracted data and actions carry source locators, hashes, verification state, and policy decisions.
8. **Adapters are replaceable.** WebDriver BiDi, CDP, WebMCP, MCP, and future protocols map to internal versioned contracts.

## 3. Context

```text
┌──────────────────────────────────────────────────────────┐
│ User / enterprise administrator / external agent         │
└──────────────────────────┬───────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────┐
│ OriginWeave browser, headless runtime, SDK, MCP server   │
├──────────────────────────────────────────────────────────┤
│ Rust control plane                                      │
│ session | policy | observation | action | resource       │
│ secret  | evidence | audit | persistence                 │
├──────────────────────────────────────────────────────────┤
│ Chromium compatibility kernel                            │
│ Blink | V8 | Skia | Viz | Dawn | Network | Extensions    │
└──────────────────────────┬───────────────────────────────┘
                           │
                    Websites and APIs
```

## 4. Execution modes

| Mode | Purpose | Default authority |
|---|---|---|
| Human | ordinary browsing | person-controlled profile and extensions |
| Assist | summaries and reversible preparation | read actions; governed writes |
| Agent Task | bounded delegated workflow | isolated profile and explicit origin capabilities |
| Crawler | public collection and monitoring | read-only; robots and rate policy required |

An Agent Task session must not automatically share the default human profile. Future session adapters will support ephemeral profiles, managed profiles, origin-scoped authentication delegation, and explicit attachment to one current tab.

## 5. Current crate boundaries

### `originweave-core`

Owns stable value contracts without I/O:

- browser-equivalent normalized `Origin` values that reject ambiguous numeric hosts
- immutable `ActionIntentDigest` values
- `SessionMode` and `ExecutionPurpose`
- `InstructionSource` and `SecretDelivery`
- `RiskClass`, `Capability`, and `ActionKind`
- exact action, target-origin, and intent-bound `ApprovalScope` and `ApprovalEvidence`
- `ActionRequest` and `PolicyContext`

### `originweave-policy`

Owns a pure decision function. It denies human-mode agent control, untrusted instruction promotion, missing capabilities, unauthorized origins, crawler mutation, cross-origin mutation, absent robots evidence, unsafe secret delivery, R5 actions, and mismatched action, target-origin, or intent approvals.

### `originweave-resource`

Owns validated task budgets and deterministic cumulative mitigation plans. Platform-specific telemetry and scheduling remain adapter concerns. A plan can independently spill observation cache, reduce the next batch, offload inference to CPU, pause the active agent, and reject new work. Hard RAM or VRAM pressure always stops the active agent and rejects admission; simultaneous pressures never collapse into one lossy enum value.

### `originweave-evidence`

Owns universally value-redacted network evidence and source-bound provenance records. Generic network records retain only bounded method, canonical origin, unambiguous bounded path, and bounded field names. Body capture, typed metadata values, WARC serialization, object storage, retention, encryption, and legal policy remain future bounded modules.

## 6. Planned modules

```text
originweave-session       isolated browser contexts and checkpoints
originweave-observation   AX + DOM + layout + network semantic snapshots
originweave-action        typed browser actions and post-condition verification
originweave-secret        opaque secret broker and trusted fill channel
originweave-bidi          WebDriver BiDi adapter
originweave-cdp           versioned Chromium DevTools Protocol adapter
originweave-mcp           external MCP server
originweave-protocol      Browser Agent Protocol schemas and compatibility
originweave-warc          ISO 28500 WARC persistence
originweave-prov          W3C PROV-O serialization
originweave-benchmark     Mind2Web, WebArena, and hostile-page evaluation
```

Each module must be usable alone, through the OriginWeave runtime, and as a module imported by naruon or another CWL product.

## 7. Observation hierarchy

Observation should prefer the most structured trustworthy source available:

1. site-provided typed tool or WebMCP contract
2. JSON-LD, Microdata, RDFa, Open Graph, tables, forms, and ARIA
3. universally value-redacted XHR, Fetch, or GraphQL request metadata plus separately authorized typed response capture
4. accessibility tree combined with DOM and layout
5. screenshot or vision fallback for canvas and inaccessible custom interfaces

Raw HTML is not the default model input. Full snapshots are followed by incremental semantic diffs, versioned by document epoch. Node references become invalid after navigation or epoch change.

## 8. Action lifecycle

```text
user intent
→ canonical complete intent
→ immutable intent digest
→ typed request
→ instruction-source check
→ capability and browser-equivalent origin check
→ crawler / robots / secret checks
→ risk and exact action + target + intent approval check
→ trusted input execution
→ observed post-condition
→ evidence and audit record
```

State-changing actions remain same-origin by default. Cross-origin workflows require decomposition into separately granted steps rather than one ambient action.

## 9. Resource architecture

The resource governor receives adapter telemetry and emits one cumulative mitigation plan; it does not own OS scheduling. Planned telemetry includes process RSS, JavaScript heap, decoded image cache, semantic snapshot bytes, GPU allocations, frame time, model residency, batch size, and task priority.

Fixed worker pools must prevent oversubscription between Chromium, Rust compute, model runtimes, and numerical libraries. Local model inference and browser rendering must use phase scheduling on constrained GPUs. CPU fallback is required before sacrificing visible interaction. A hard-limit plan must reduce the active consumer and reject new admission rather than merely block future work.

## 10. Network destination policy

Canonical origin parsing establishes identity but does not establish destination safety. Before any real browser navigation, the Chromium adapter must resolve and pin approved address sets; reject or explicitly authorize loopback, private, link-local, multicast, unspecified, and cloud-metadata ranges; detect rebinding; re-evaluate redirects; define proxy and PAC behavior; and bound connection, header, body, redirect, download, and elapsed-time resources.

A destination-policy decision and its resolved-address evidence must remain separate from the logical origin grant. No browser adapter may treat syntactic origin validation as an SSRF defense.

## 11. Persistence and database naming

Persistent objects use two-or-more-word `snake_case` names. Examples include `agent_session`, `browser_profile`, `page_snapshot`, `semantic_node`, `network_exchange`, `action_event`, `policy_decision`, `provenance_record`, `resource_budget`, and `extension_grant`.

WARC stores source exchanges and resources; relational storage holds sessions, policy, and audit metadata; object storage holds screenshots and large artifacts; PROV-JSON-LD represents derivation and responsibility.

## 12. Security boundaries

- Chromium sandbox and Site Isolation are retained.
- Renderer compromise is assumed possible; privileged validation occurs outside the renderer.
- Browser content is data, never authority.
- Secrets are never included in model prompts, traces, or provenance values.
- Generic header and query values are never retained by the evidence kernel.
- Arbitrary script evaluation is absent from the standard action interface.
- Crawler policy is not treated as access authorization.
- High-risk actions fail closed when context, canonical intent, or approval evidence is incomplete.
- Every protocol adapter must validate messages at its process boundary.

## 13. Deployment topology

The intended modular topology is:

```text
OriginWeave desktop browser ─┐
OriginWeave headless service ├─ Browser Agent Protocol ─ agent orchestrator
naruon embedded module       ┘                    └─ contextual-orchestrator
```

No deployment mode may depend on an in-process singleton. Session, policy, evidence, and persistence interfaces must remain tenant-scoped and transport-neutral.

## 14. Quality attributes

| Attribute | Required evidence |
|---|---|
| correctness | contract, property, hostile-input, and post-condition tests |
| safety | prompt-injection, secret, origin, destination, approval, and renderer-boundary tests |
| reliability | crash recovery, checkpoint, retry, and idempotency tests |
| performance | input latency, frame time, task RSS, VRAM, transfer, and token metrics |
| interoperability | BiDi/CDP/MCP/WARC/PROV and Manifest V3 compatibility suites |
| accessibility | WCAG 2.2 AA UI, keyboard flow, and exact-value alternatives |
| reproducibility | locked dependencies, pinned tools, SBOM, provenance, and release attestations |

## 15. Change control

Changes to the compatibility-kernel boundary, risk taxonomy, secret model, origin model, canonical intent model, evidence semantics, destination policy, resource mitigation semantics, or protocol versioning require a new ADR. The current baseline decisions are recorded under `docs/adr/`.
