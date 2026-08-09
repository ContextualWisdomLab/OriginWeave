# OriginWeave Threat Model

- **Status:** Proposed authoritative product threat model
- **Product status:** Pre-alpha
- **Security policy:** [`../SECURITY.md`](../SECURITY.md)
- **Technical requirements:** [`TRD.md`](TRD.md)
- **Architecture:** [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- **Evidence/standards:** [`doctoring.md`](doctoring.md)

## 1. Purpose

OriginWeave assumes the web, browser renderers, model outputs, downloaded content, external agent frameworks, and many integration inputs may be hostile. This threat model identifies assets, trust boundaries, attacker goals, product security properties, and required validation evidence. It is an engineering threat model, not a claim of certification or legal compliance.

## 2. Security objectives

OriginWeave must preserve all of the following properties even when one lower-trust component misbehaves:

1. untrusted web content cannot become trusted instruction authority;
2. a logical origin grant cannot silently expand into unrelated resolved addresses, proxy routes, TCP peers, TLS identities, or redirects;
3. a model cannot create capabilities, approvals, secrets, destination authority, or successful outcomes by assertion;
4. raw secrets and protected personal values do not enter model-visible observations, generic logs, traces, evidence, or provenance;
5. a node observed in one session/context/document cannot be reused as authority in another;
6. state-changing actions require the right capability, origin, risk/approval and current authority immediately before execution;
7. action success requires an observed post-condition;
8. resource exhaustion by agent/model work cannot take priority over interactive safety;
9. tenant or task authority cannot cross a boundary because two objects share an identifier, cache, browser profile, queue, connection, model session, or storage backend;
10. evidence remains sufficient to reconstruct what was authorized, attempted, observed and derived without becoming a new secret store.

## 3. Trust zones

### Zone A — Human and enterprise authority

Contains the authenticated human goal, managed enterprise policy, explicit approvals, tenant configuration and operator break-glass decisions. These are trusted only after identity/session/policy validation; a UI string alone is not authority.

### Zone B — OriginWeave trusted Rust control plane

Contains deterministic policy, canonical origin/destination/route/transport/TLS/HTTP authority, session/node lifetime validation, resource governance, secret brokerage and evidence assembly. Compromise of this zone is a critical product compromise.

### Zone C — Chromium browser process and privileged adapters

Contains trusted browser integration code that translates Chromium/WebDriver/CDP state into OriginWeave contracts and dispatches approved actions. It must validate all renderer-originated or external-protocol messages before converting them into trusted values.

### Zone D — Renderer / web content

Explicitly hostile. **renderer compromise** is in scope. Site Isolation and Chromium sandboxing reduce blast radius but are not substitutes for Rust-side validation.

### Zone E — Model and orchestration providers

Model outputs are proposals/data, not authority. Providers may be unavailable, stale, compromised, prompt-injected, or configured incorrectly. External orchestration is not allowed to bypass OriginWeave policy or secret boundaries.

### Zone F — Persistence and evidence adapters

Relational, object, WARC, PROV and telemetry systems may have separate operational trust. Each adapter receives only the minimum data authorized by its schema and tenant/purpose contract.

### Zone G — External web and enterprise services

Remote sites, proxies, DNS resolvers, PAC sources, model providers, connectors and APIs may be malicious or compromised. Their identities and data require explicit validation at the applicable boundary.

## 4. Assets

| Asset | Why it matters |
|---|---|
| user/enterprise goal and approvals | source of legitimate task authority |
| browser profile/session state | cookies, history, identity and delegated context |
| secret/protected-value store | credentials and operational PII |
| origin/destination/route/TCP/TLS/HTTP authority | prevents SSRF, route confusion and service impersonation |
| semantic observation and node authority | prevents stale/cross-context browser actions |
| resource budgets | protects human interaction and host stability |
| action/post-condition evidence | distinguishes attempt from actual outcome |
| provenance/source artifacts | supports audit and buyer-visible proof |
| tenant policy and residency/retention policy | prevents cross-tenant or out-of-policy disclosure |
| release artifact, SBOM and provenance | prevents supply-chain substitution |

## 5. Threat actors

- malicious web page or compromised origin;
- compromised renderer process;
- malicious/compromised browser extension;
- prompt-injected page/tool/download/email-like content;
- malicious or compromised model/provider;
- malicious external MCP/BiDi/CDP client;
- tenant user attempting privilege escalation;
- insider/support/operator abusing privileged access;
- compromised dependency, build action or release pipeline;
- network attacker or malicious DNS/proxy/PAC infrastructure;
- confused deputy using a legitimately privileged service for an unauthorized destination or tenant;
- availability attacker causing CPU/RAM/VRAM/network/task-queue exhaustion.

## 6. Major threat scenarios and controls

### TM-001 — Indirect prompt injection

**Attack:** page text, hidden content, WebMCP output or downloaded text tells the model to ignore the user goal, exfiltrate cookies, expand the task or invoke unsafe tools.

**Controls:**

- `trusted_instruction`, `untrusted_observation` and `protected_secret` remain separate types/channels;
- page/model content cannot modify deterministic policy or capability sets;
- model receives bounded typed observations rather than ambient raw browser state;
- high-risk actions are re-evaluated by deterministic policy immediately before execution;
- secret values are absent from model context;
- regression corpus includes visible/hidden/multilingual/encoded injection variants.

### TM-002 — SSRF / destination confusion

**Attack:** a public-looking hostname resolves to private, link-local, metadata or platform endpoints; DNS later expands to an unapproved address; redirect changes authority.

**Controls:** implemented destination classification, explicit class grants, origin-bound resolution snapshots, canonical IPv4-mapped handling, non-expanding revalidation and per-hop redirect authorization.

### TM-003 — Route/proxy/PAC bypass

**Attack:** ambient proxy environment, a malicious PAC source or unreviewed proxy selection changes the real intermediary and bypasses the direct-route security assumptions.

**Controls:** route authority is explicit; direct is default; PAC source and selected proxy require independent approval; proxy execution must not infer authority from environment variables or from final-target origin permission.

### TM-004 — TCP/TLS identity confusion

**Attack:** connecting to an approved IP is treated as authenticating the requested HTTPS service, or a TLS implementation reconnects/resolves behind the policy layer.

**Controls:** exact `peer_addr` verification precedes stream exposure; TLS consumes that same stream; reference identity derives only from canonical HTTPS origin; explicit roots/time/SAN validation; no TLS reconnect/resolution.

### TM-005 — HTTP framing and content ambiguity

**Attack:** request/response smuggling, conflicting framing, compression bomb, malformed trailers, MIME confusion, unsafe filenames or redirects cause a different semantic result than the policy reviewed.

**Controls:** bounded HTTP authority is an independent layer with strict framing, byte/count/deadline limits, integrity/MIME/disposition evidence and no automatic redirect follow. This layer is active development and must not be claimed as protected-main capability until merged.

### TM-006 — Stale or cross-context node reuse

**Attack:** an adapter-local node ID collides across sessions, frames, origins or documents; an agent acts on stale UI after navigation.

**Implemented controls:** `originweave-core` already binds `ObservedNodeHandle` to OriginWeave browser-session, browsing-context, canonical-origin, document-epoch and adapter-local-node identities and exposes deterministic validation for cross-session, cross-context, cross-origin and stale-document reuse. These contracts are covered by the current core tests and remain observation-authority primitives rather than proof of a live Chromium dispatch path.

**Planned controls:** the browser adapter must rotate the document epoch for relevant same-document DOM/accessibility mutations, revalidate the same handle authority at the action linearization point immediately before the trusted side effect, abort on any competing mutation, and require re-observation before a new action can proceed. The first real Chromium vertical slice must prove that a pre-mutation handle is rejected while the newly observed handle succeeds.

### TM-007 — Secret/PII exfiltration

**Attack:** password, token or operational PII is placed in prompt text, accessibility labels, logs, traces, screenshots, provenance, support bundle or an unauthorized connector/model.

**Controls:** opaque handles by default; purpose/field/destination/classification authority; trusted broker; field-level disclosure; provider/region/retention policy; generic evidence value-redaction; no blanket assumption that PII can be safely copied because a service is internal.

### TM-008 — Sensitive-handle replay and race

**Attack:** a handle is replayed, transferred to another task/tenant, used concurrently beyond its maximum count, or resolved after expiry/revocation.

**Controls:** planned broker owns trusted time, atomic use reservation, audience/task/tenant/field/destination scope, max-use enforcement, expiry/revocation and compensation semantics. Model-side counters are not trusted.

### TM-009 — Confused deputy

**Attack:** a service with legitimate access is asked to fill/send data to an unauthorized origin, tenant, model or connector.

**Controls:** **confused deputy** checks bind actor/workload identity, task, tenant, field, business purpose, destination and action; service possession of a credential does not grant downstream disclosure authority.

### TM-010 — Cross-tenant contamination

**Attack:** profile/session/cache/queue/model context/object-store/vector/evidence identifiers allow one tenant to read or act on another tenant's data.

**Controls:** all persistent/runtime authority is tenant/task scoped; no global singleton as authority; cache keys include security scope; **cross-tenant** tests cover concurrent sessions, shared infrastructure and support operations; enterprise adapters remain Planned until these proofs exist.

### TM-011 — Malicious/compromised extension

**Attack:** an MV3 extension observes sensitive task state or injects behavior that the agent interprets as user intent.

**Controls:** preserve Chromium extension sandbox/permission model; extension access to OriginWeave agent authority requires a separate signed policy grant; extension data remains untrusted observation; compatibility tests include task-mode isolation.

### TM-012 — Resource exhaustion

**Attack:** page, model or task drives unbounded DOM snapshot, response body, decompression, screenshot, CPU worker, RAM or VRAM consumption and makes the browser unusable.

**Controls:** boundary-specific byte/count/deadline limits; deterministic resource budget; human/compositor priority; cumulative mitigations; model batch/cache reduction; CPU fallback; pause/reject admission at hard limits.

### TM-013 — Evidence poisoning / provenance forgery

**Attack:** attacker supplies source URLs, locators, timestamps or hashes that make a fabricated result appear verified.

**Controls:** source locators are parsed/validated, digests computed inside trusted boundaries, generic values redacted, model claims separated from deterministic verification, trusted time used for authoritative lifecycle decisions, provenance links carry verification state.

### TM-014 — Supply-chain compromise

**Attack:** dependency/action/toolchain/release artifact is substituted or a mutable external action executes unexpected code.

**Controls:** lock dependencies/toolchains, pin security-sensitive actions, SBOM, signed/provenanced releases, dependency/security scanning, artifact verification, reproducible-build evidence where supported, protected-branch review.

### TM-015 — Approval spoofing / governance bypass

**Attack:** comment/status/model output/self-review is treated as qualifying approval, or pending/old checks are treated as current success.

**Controls:** exact current head/base evidence, formal review-state validation when policy requires a counted review, reviewer eligibility, unresolved-thread checks, no synthetic approval, no old-head check promotion, separate publication and merge authority.

### TM-016 — Operator/support abuse

**Attack:** privileged operator silently accesses profiles, raw secrets or tenant evidence.

**Controls:** least privilege, separate support capability, explicit **break-glass** workflow, reason/time limit/approval, heightened audit and post-event review. Full enterprise operation is Planned.

## 7. STRIDE-oriented summary

| STRIDE | OriginWeave examples | Principal mitigation |
|---|---|---|
| Spoofing | TLS service, reviewer, tenant/workload identity | explicit identity + exact evidence |
| Tampering | page/tool output, provenance, redirect, artifact | validation, digests, immutable evidence, protected release |
| Repudiation | agent/action/approval disputes | intent-bound approval + action/post-condition evidence |
| Information disclosure | secrets, PII, headers, screenshots | opaque handles, purpose-bound disclosure, redaction |
| Denial of service | decompression, DOM, GPU/model pressure | byte/count/time/resource budgets |
| Elevation of privilege | prompt injection, extension, confused deputy | typed authority, deterministic policy, tenant/task scope |

## 8. Security acceptance evidence

Before a production release, security evidence must include at least:

- hostile origin/URL/DNS/redirect/proxy/TCP/TLS/HTTP test suites for every shipped boundary;
- prompt-injection regression across structured, DOM, hidden and visual inputs;
- stale/cross-session/cross-context node tests for shipped observation/action adapters;
- secret/PII byte-occurrence scanning across logs, traces, errors, screenshots, WARC/PROV and model requests for the approved test corpus;
- tenant-isolation concurrency tests;
- resource exhaustion and recovery tests;
- extension compatibility/isolation tests for supported APIs;
- dependency/SAST/security scans on the exact release head;
- SBOM/provenance/reproducibility/rollback evidence;
- a review of any Accepted ADR that changes a trust boundary.

## 9. Residual and open risks

- **Open:** real Chromium navigation has not yet proven end-to-end consumption of every Rust authority layer.
- **Open:** complete proxy/PAC and bounded HTTP integration is not yet protected-main behavior in this baseline.
- **Open:** opaque sensitive-data broker persistence/atomic lifecycle is not yet complete protected-main behavior.
- **Open:** enterprise tenant/SSO/SCIM/residency/support controls are not shipped.
- **Open:** WebMCP is experimental upstream and must remain optional/versioned.
- **Open:** product UI and extension compatibility matrices require release-specific evidence.

## 10. Change control

Any protected change that introduces a new trust zone, privileged IPC/protocol, secret/PII path, persistence class, tenant boundary, browser authority, external model/provider, release authority or executable content channel must update this threat model and the governing ADR/TRD in the same reviewed change or an explicit prerequisite.
