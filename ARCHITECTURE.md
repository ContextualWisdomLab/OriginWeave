# OriginWeave Architecture

## 1. Product definition

OriginWeave is an enterprise agentic web runtime and provenance-native browser control plane. Chromium remains the compatibility kernel; Rust owns new governance, sensitive-data authority, destination, direct network, TLS identity, resource, evidence, and agent-facing contracts. This separation minimizes the Chromium patch surface and allows the same Rust modules to operate in a desktop browser, headless service, naruon module, or external agent runtime.

## 2. Architectural principles

1. **Compatibility before reinvention.** Blink, V8, Skia, Viz, Dawn, Site Isolation, sandboxing, and Manifest V3 remain upstream-compatible.
2. **Authority is explicit.** No ambient browser state or page content implicitly grants a capability.
3. **Actions are typed.** Production agents do not receive unrestricted JavaScript evaluation as a default tool.
4. **Observe before acting; verify after acting.** A command is successful only when its expected post-condition is observed.
5. **Secrets and protected values stay outside model context by default.** Models receive opaque handles; a broker resolves values directly into a trusted browser process only after an explicit purpose-bound disclosure decision.
6. **Logical origin, resolved destination, TCP peer, TLS service identity, and sensitive-data disclosure are separate.** An origin grant never implies permission to connect to every resolver result; an approved address is not a transport proof until the operating-system peer is checked; an exact TCP peer is not an authenticated HTTPS service until WebPKI identity is verified; and none of those authorities grants raw protected-value access.
7. **Human interaction wins resource contention.** Rendering, input, and active-tab work outrank inference and background collection.
8. **Evidence is a product output.** Extracted data and actions carry source locators, hashes, verification state, and policy decisions.
9. **Adapters are replaceable.** HTTP, proxy/PAC, WebDriver BiDi, CDP, WebMCP, MCP, and future protocols map to internal versioned contracts.

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
│ session | policy | sensitive data | destination | network│
│ TLS | observation | action | resource | secret | evidence│
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

- browser-equivalent normalized `Origin` values that reject ambiguous numeric hosts;
- immutable `ActionIntentDigest` values;
- `SessionMode` and `ExecutionPurpose`;
- `InstructionSource` and `SecretDelivery`;
- `RiskClass`, `Capability`, and `ActionKind`;
- exact action, target-origin, and intent-bound `ApprovalScope` and `ApprovalEvidence`;
- `ActionRequest` and `PolicyContext`.

### `originweave-policy`

Owns deterministic pure policy decisions without I/O. Its action evaluator denies human-mode agent control, untrusted instruction promotion, missing capabilities, unauthorized origins, crawler mutation, cross-origin mutation, absent robots evidence, unsafe secret delivery, R5 actions, and mismatched action, target-origin, or intent approvals.

The crate also contains the first purpose-bound sensitive-data authority kernel. A `SensitiveDataRequest` and `DisclosureScope` carry no protected value bytes and must match exactly on tenant, task, field, business purpose, canonical destination `Origin`, and classification before the configured disclosure outcome is returned; any mismatch denies access. `SensitiveValueHandleScope` and `HandleUseRequest` bind proposed opaque-handle use to the same canonical scope plus exclusive expiry and maximum use count. `evaluate_handle_use` is a pure admission evaluation over trusted-time and authoritative-state inputs; it does not consume a use, mutate state, resolve a handle, or release a protected value. A later trusted broker must own caller-unforgeable handle state and trusted time, atomically reserve or increment the use count before resolution, and recheck the reserved scope and lifecycle state immediately before disclosure. This is a preparatory policy kernel, not the final independently versioned sensitive-data service, storage, broker, evidence, or lifecycle layer described by issue #10 and ADR 0007.

### `originweave-destination`

Owns pure resolved-address and redirect authority without DNS or socket I/O:

- IPv4 and IPv6 special-purpose classification;
- IPv4-mapped IPv6 canonicalization;
- default public-only and explicitly managed destination policies;
- non-empty, origin-bound approved resolution snapshots;
- concrete connection-address pinning;
- DNS answer contraction and rebinding-expansion detection;
- per-hop redirect origin, resolution, secure-scheme, complete-target cycle, and hop-limit authorization;
- credential-free connection and redirect evidence.

A managed adapter can explicitly permit a local address class, but no local-network exception is ambient or inferred from an origin grant.

### `originweave-network`

Owns the direct-only TCP authority boundary without DNS, proxy, TLS, HTTP, or Chromium integration:

- validation of one explicit canonical `SocketAddr` against a `ResolutionSnapshot`;
- port, per-attempt timeout, and attempt-count bounds;
- a non-cloneable plan consumed by one connection sequence;
- an exact `TcpStream::connect_timeout` call with no hostname re-resolution;
- `peer_addr` verification of the remote IP and port before stream exposure;
- credential-free requested-peer, observed-peer, class, attempt, and timeout evidence;
- standard error chains that preserve destination-policy and operating-system errors.

This crate proves the exact operating-system peer for a direct TCP stream. It does not prove TLS identity, HTTP safety, proxy routing, or that Chromium used the stream.

### `originweave-tls`

Owns authenticated TLS service identity over one existing `DirectTcpConnection` without DNS, reconnect, proxy, HTTP, or Chromium integration:

- exact equality between the canonical HTTPS origin and the origin recorded in TCP evidence;
- RFC 9525 DNS and literal-IP reference identity derived only from the canonical origin;
- DNS SNI only for DNS identities and no invented SNI for IP literals;
- explicit bounded trust-root bundles with canonical SHA-256 identifiers;
- a caller-supplied fixed trusted verification time;
- TLS 1.2 and TLS 1.3 only;
- disabled resumption, early data, secret extraction, key logging, client authentication, certificate compression, and dangerous custom verifier hooks;
- bounded total handshake time, ALPN input, trust roots, and server-presented certificate evidence;
- operating-system peer revalidation before, during, and after the handshake;
- typed protocol, cipher, ALPN, certificate, SPKI, root-bundle, validity, revocation-configuration, and timing evidence;
- deterministic public errors that preserve rustls and I/O sources without retaining credentials or certificate bodies.

This crate proves that the same exact direct TCP stream completed WebPKI authentication for the canonical origin. It does not parse HTTP, authorize a proxy, fetch revocation data, acquire system roots, control Chromium, or claim that server-presented certificate hashes are a reconstructed validation path.

### `originweave-resource`

Owns validated task budgets and deterministic cumulative mitigation plans. Platform-specific telemetry and scheduling remain adapter concerns. A plan can independently spill observation cache, reduce the next batch, offload inference to CPU, pause the active agent, and reject new work. Hard RAM or VRAM pressure always stops the active agent and rejects admission; simultaneous pressures never collapse into one lossy enum value.

### `originweave-evidence`

Owns universally value-redacted network evidence and source-bound provenance records. Generic network records retain only bounded method, canonical origin, unambiguous bounded path, and bounded field names. Body capture, typed metadata values, WARC serialization, object storage, retention, encryption, and legal policy remain future bounded modules.

## 6. Planned modules

```text
originweave-session        isolated browser contexts and checkpoints
originweave-sensitive-data independently versioned field authority/service contracts
originweave-proxy          separately approved proxy and final-target routing
originweave-http           request, response, redirect, and elapsed-time budgets
originweave-observation    AX + DOM + layout + network semantic snapshots
originweave-action         typed browser actions and post-condition verification
originweave-secret         opaque secret/data broker and trusted fill channel
originweave-bidi           WebDriver BiDi adapter
originweave-cdp            versioned Chromium DevTools Protocol adapter
originweave-mcp            external MCP server
originweave-protocol       Browser Agent Protocol schemas and compatibility
originweave-warc           ISO 28500 WARC persistence
originweave-prov           W3C PROV-O serialization
originweave-benchmark      Mind2Web, WebArena, and hostile-page evaluation
```

Each module must be usable alone, through the OriginWeave runtime, and as a module imported by naruon or another CWL product.

## 7. Observation hierarchy

Observation should prefer the most structured trustworthy source available:

1. site-provided typed tool or WebMCP contract;
2. JSON-LD, Microdata, RDFa, Open Graph, tables, forms, and ARIA;
3. universally value-redacted XHR, Fetch, or GraphQL request metadata plus separately authorized typed response capture;
4. accessibility tree combined with DOM and layout;
5. screenshot or vision fallback for canvas and inaccessible custom interfaces.

Raw HTML is not the default model input. Full snapshots are followed by incremental semantic diffs, versioned by document epoch. Node references become invalid after navigation or epoch change.

## 8. Action lifecycle

```text
user intent
→ canonical complete intent
→ immutable intent digest
→ typed request
→ instruction-source check
→ capability and browser-equivalent origin check
→ resolved-destination approval and pinning
→ exact direct TCP peer binding
→ authenticated TLS service identity
→ proxy and bounded HTTP adapter checks
→ crawler / robots / secret checks
→ risk and exact action + target + intent approval check
→ purpose-bound sensitive-data decision when protected fields are required
   ├─ DenyAccess → terminate without disclosure
   ├─ HumanApprovalRequired / DualControlRequired
   │  → collect the required independent approval evidence
   │  → re-evaluate the exact tenant/task/field/purpose/destination/classification scope
   └─ permitted disclosure mode → continue only through that exact channel
→ for opaque handles, trusted broker atomically reserves a use from authoritative state
→ trusted broker or input execution
→ observed post-condition
→ evidence and audit record
```

State-changing actions remain same-origin by default. Cross-origin workflows require decomposition into separately granted steps rather than one ambient action. A successful action or network decision cannot be reused as sensitive-data disclosure authority; that decision is evaluated separately for each protected field transition. Approval-required sensitive-data outcomes are not execution permission: the exact scope must be re-evaluated after the required approval evidence exists. Opaque-handle evaluation is not state consumption; the later broker must serialize concurrent reservations against authoritative use-count, expiry, and revocation state before releasing a value.

## 9. Resource architecture

The resource governor receives adapter telemetry and emits one cumulative mitigation plan; it does not own OS scheduling. Planned telemetry includes process RSS, JavaScript heap, decoded image cache, semantic snapshot bytes, GPU allocations, frame time, model residency, batch size, and task priority.

Fixed worker pools must prevent oversubscription between Chromium, Rust compute, model runtimes, and numerical libraries. Local model inference and browser rendering must use phase scheduling on constrained GPUs. CPU fallback is required before sacrificing visible interaction. A hard-limit plan must reduce the active consumer and reject new admission rather than merely block future work.

## 10. Network destination, direct transport, and TLS identity

Canonical origin parsing establishes logical identity but does not establish destination safety. The pure Rust destination kernel provides the reusable policy foundation:

```text
canonical Origin grant
→ resolver-supplied address set
→ special-purpose classification
→ explicit destination policy
→ origin-bound approved snapshot
→ concrete connection-address pin
→ non-expanding DNS revalidation
→ per-hop redirect reauthorization
```

The default web policy admits only addresses classified as public. Loopback, private or unique-local, shared, link-local, metadata, documentation, benchmarking, multicast, broadcast, unspecified, transition, and protocol-reserved destinations fail closed unless a managed caller constructs an explicit class grant. IPv4-mapped IPv6 is canonicalized before classification and set comparison.

Resolution snapshots are non-empty and bound to one logical origin. A connection candidate must appear in the canonical pinned set. A refreshed DNS answer may contract to a non-empty subset but may not add a new address. Every redirect must have a read-origin grant, a target-bound approved resolution, no HTTPS-to-HTTP downgrade, a previously unseen complete-target digest, and remaining hop capacity.

The direct transport kernel consumes the approved decision:

```text
origin-bound ResolutionSnapshot
→ one explicit canonical SocketAddr
→ bounded single-use ConnectionPlan
→ exact operating-system connect call
→ observed peer_addr
→ exact IP-and-port equality
→ verified TCP stream plus credential-free evidence
```

The requested port must be nonzero, each timeout must be in `1ns..=30s`, and one plan permits at most four attempts. IPv4-mapped IPv6 and unmodeled IPv6 flow or scope metadata are rejected at the socket boundary. A stream is never returned before the operating system reports the exact requested peer.

The TLS identity kernel consumes that exact stream:

```text
canonical HTTPS Origin + verified DirectTcpConnection
→ require transport-origin equality
→ derive DNS or IP reference identity
→ explicit trust roots + fixed verification time
→ TLS 1.2/1.3 on the existing stream
→ recheck operating-system peer throughout the deadline
→ require WebPKI SAN identity and explicit ALPN policy
→ authenticated TLS stream plus credential-free evidence
```

Trust-root count and bytes, ALPN count and bytes, server-presented certificate count and bytes, and total handshake time are bounded before evidence leaves the crate. The first slice records revocation as not configured. It neither retrieves revocation material nor claims revocation validation. DNS service identity never falls back to Common Name. IP literal identity requires the exact IP SAN.

The network crate remains direct-only and the TLS crate remains transport-bound. Before OriginWeave claims safe real navigation, the Chromium/BiDi/CDP adapter must prove that its real socket path consumes both authorities; proxy and PAC behavior must separately authorize intermediate and final destinations; HTTP must bound connection, header, body, redirect, download, and elapsed-time resources; and download policy must compare declared and observed MIME without exposing credentials.

No browser adapter may treat syntactic origin validation as an SSRF defense, resolver success as authorization, TCP peer equality as TLS identity, successful TLS authentication as an HTTP resource-budget decision, or any of those network authorities as permission to disclose a protected field.

## 11. Persistence and database naming

Persistent objects use two-or-more-word `snake_case` names. Examples include `agent_session`, `browser_profile`, `page_snapshot`, `semantic_node`, `network_exchange`, `action_event`, `policy_decision`, `provenance_record`, `resource_budget`, and `extension_grant`. Planned sensitive-data persistence uses names such as `sensitive_data_record`, `sensitive_value_handle`, `sensitive_access_evidence`, `retention_policy_record`, and `encryption_key_reference`; those objects are not implemented by the current pure policy kernel.

WARC stores source exchanges and resources; relational storage holds sessions, policy, and audit metadata; object storage holds screenshots and large artifacts; PROV-JSON-LD represents derivation and responsibility. Protected values require a separately authorized storage and retention design rather than ambient inclusion in these generic stores.

## 12. Security boundaries

- Chromium sandbox and Site Isolation are retained.
- Renderer compromise is assumed possible; privileged validation occurs outside the renderer.
- Browser content is data, never authority.
- Secrets and protected values are never included in model prompts, traces, provenance values, or generic evidence without a separately authorized disclosure contract.
- Generic header and query values are never retained by the evidence kernel.
- Sensitive-data disclosure is separate from session, repository, administrator, origin, destination, transport, TLS, and model-credential authority.
- The current sensitive-data policy API carries authority metadata only, never protected value bytes.
- Disclosure mismatches on tenant, task, field, purpose, canonical destination, or classification deny access; opaque-handle scope mismatch, expiry, or exhausted use count fail closed at evaluation.
- `evaluate_handle_use` does not consume state or release a value; the trusted broker must atomically reserve use count and recheck authoritative expiry/revocation state before resolution.
- Approval-required sensitive-data outcomes must collect approval evidence and re-evaluate the exact scope before execution; `DenyAccess` terminates the disclosure path.
- Logical origin grants, resolved-destination grants, actual peer evidence, and TLS service identity remain distinct.
- DNS answer expansion after approval is denied as a possible rebinding event.
- Direct TCP accepts only a canonical approved socket, never a hostname.
- A TCP stream is exposed only after exact peer verification.
- TLS consumes the verified stream and cannot reconnect or resolve.
- TLS reference identity comes only from the canonical HTTPS origin.
- DNS identity requires SAN and never falls back to Common Name; IP identity requires exact IP SAN.
- TLS uses explicit roots, fixed verification time, TLS 1.2/1.3, and explicit ALPN policy.
- TLS resumption, 0-RTT, key logging, secret extraction, client certificates, and dangerous custom verification are disabled in the first slice.
- Proxy and PAC routing cannot be inherited ambiently by the direct-only or TLS kernels.
- Redirects cannot inherit ambient origin or network authority.
- TCP peer equality does not substitute for TLS server identity, and TLS identity does not substitute for HTTP safety.
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

No deployment mode may depend on an in-process singleton. Session, policy, sensitive-data authority, destination, network, TLS, evidence, and persistence interfaces must remain tenant-scoped and transport-neutral.

## 14. Quality attributes

| Attribute | Required evidence |
|---|---|
| correctness | contract, property, hostile-input, real TCP/TLS, exact sensitive-data scope, and post-condition tests |
| safety | prompt-injection, secret, sensitive-data scope/expiry/use-count, origin, destination, rebinding, redirect, exact-peer, TLS identity, approval, and renderer-boundary tests |
| reliability | crash recovery, checkpoint, retry, timeout, and idempotency tests |
| performance | input latency, frame time, task RSS, VRAM, transfer, connection, handshake, and token metrics |
| interoperability | BiDi/CDP/MCP/WARC/PROV and Manifest V3 compatibility suites |
| accessibility | WCAG 2.2 AA UI, keyboard flow, and exact-value alternatives |
| reproducibility | locked dependencies, fixed trust/time inputs, pinned tools, SBOM, provenance, and release attestations |

## 15. Change control

Changes to the compatibility-kernel boundary, risk taxonomy, secret or sensitive-data authority model, origin model, canonical intent model, evidence semantics, destination taxonomy, DNS pinning semantics, direct socket authority, TLS reference identity, trust-root semantics, trusted-time semantics, ALPN policy, redirect policy, resource mitigation semantics, or protocol versioning require a new ADR. The current baseline decisions are recorded under `docs/adr/`.