# OriginWeave Product Requirements Document

- **Document status:** Proposed authoritative product baseline
- **Product status:** Pre-alpha
- **Product:** OriginWeave
- **Tagline:** **Browse. Act. Prove.**
- **Canonical architecture:** [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- **Technical requirements:** [`TRD.md`](TRD.md)
- **Data governance:** [`DATA_GOVERNANCE.md`](DATA_GOVERNANCE.md)
- **Roadmap:** [`product-roadmap.md`](product-roadmap.md)
- **Research and standards:** [`doctoring.md`](doctoring.md)

## 1. Purpose

This PRD defines the product boundary that previously existed across architecture, ADRs, implementation plans, roadmap entries, pull-request descriptions, and product-design conversations. It does not convert planned work into shipped functionality. Shipped implementation truth is determined from the consistent set of protected `main` source, executable tests, produced build/release artifacts, applicable migrations and configuration, and integrated operational evidence. If those authorities disagree, the discrepancy is a release/operability defect that must be resolved before the affected behavior is described as shipped; an Accepted ADR provides design authority but never overrides contrary executable or released evidence.

OriginWeave is an enterprise agentic web runtime and provenance-native browser platform. Chromium remains the compatibility kernel; Rust owns the governance, authority, resource, evidence, and agent-facing control plane.

## 2. Product vision

A person or enterprise can delegate a bounded web task and receive the requested outcome together with evidence sufficient to understand what was observed, authorized, attempted, and verified.

OriginWeave must provide:

1. explicit task and execution-mode authority;
2. explicit browser/session/context/document identity;
3. typed actions instead of ambient arbitrary-script authority;
4. deterministic origin, destination, route, TCP, TLS and HTTP authority;
5. purpose-bound sensitive-data disclosure without model-visible raw secrets;
6. post-condition verification for state-changing work;
7. bounded resource governance that protects interactive browser correctness;
8. provenance that distinguishes source evidence, policy, model judgement, action and outcome; and
9. stable external adapters without making experimental browser protocols the product authority.

## 3. Requirement status vocabulary

Every requirement uses **exactly one** status from this table. Implementation evidence, active-PR detail, non-goal classification and dependency notes belong in separate columns or prose and never create composite status labels.

| Status | Meaning |
|---|---|
| **Implemented** | Present on protected `main` with repository tests and documented authority boundaries. |
| **Accepted architecture** | A reviewed governing design direction; this status does not itself prove the complete runtime path is implemented. |
| **Planned** | In the product roadmap or accepted target architecture, but not a shipped capability. |
| **Proposed** | Product direction still requiring a dedicated reviewed decision or sufficient implementation evidence. |
| **Open** | A decision or acceptance criterion is intentionally unresolved. |

Only `Implemented` may describe shipped behavior. An Accepted ADR is design authority, not implementation proof. Active PR implementation evidence may be named in the evidence column, but it does not change a requirement to `Implemented` until the applicable behavior reaches protected `main`.

## 4. Problem statement

General browser automation often collapses distinct authorities: URL into network permission, DNS into safe destination, socket into authenticated service, selector into durable node identity, model output into executable code, credential possession into disclosure authority, command return into success, and browser logs into provenance. These shortcuts are unacceptable for a governed enterprise runtime because they make failure, security review and acquisition diligence ambiguous.

## 5. Primary users and stakeholders

### 5.1 Enterprise automation owner

Needs delegated web work under explicit tenant, purpose, origin, risk, sensitive-data and resource policies.

### 5.2 Security and governance administrator

Needs least privilege, deterministic network authority, approval evidence, purpose-bound data handling, tenant separation, supply-chain evidence and incident reconstruction.

### 5.3 Agent platform engineer

Needs a stable typed protocol that does not couple the orchestrator to Chromium-local identifiers, raw secrets or one experimental automation surface.

### 5.4 Data and research engineer

Needs structured extraction with exact source locators, hashes, capture time, derivation and model provenance.

### 5.5 Human browser user

Needs Chromium-compatible browsing to remain responsive and understandable while assistance or delegated tasks operate under separate authority.

### 5.6 Operator and procurement reviewer

Needs observable failure modes, operability, rollback, SBOM/provenance, compatibility, accessibility and supportable deployment boundaries.

## 6. Product family

The status applies to the **whole named product surface**, not to every implemented foundation underneath it. Partial protected-main foundations are called out explicitly so a product-family row never promotes an unfinished surface to shipped functionality.

| Surface | Product responsibility | Status | Implementation evidence / note |
|---|---|---|---|
| **OriginWeave Browser** | Chromium-compatible interactive distribution with governed agent entry points | Planned | No protected-main branded browser distribution yet |
| **OriginWeave Runtime** | Headless/embedded governed web-task runtime | Planned | Rust authority kernels exist; browser integration remains incomplete |
| **OriginWeave Observe** | Structured observation from tools, structured data, network, accessibility, DOM/layout and visual fallback | Planned | Session/context/node-authority foundations are on protected main; active PR #52 adds a bounded authority-bound semantic-observation value primitive with explicit evidence-channel provenance, but it is not a browser observation adapter and remains non-shipped |
| **OriginWeave Capture** | Schema-bound extraction, crawler controls, downloads and WARC/PROV-oriented capture | Planned | Evidence foundations and partial real-Chromium extension compatibility evidence exist; complete capture runtime is not shipped |
| **OriginWeave Governor** | CPU, RAM, GPU, VRAM, admission and model/browser priority governance | Accepted architecture | Deterministic resource-budget and CPU-worker admission foundations are implemented; platform telemetry/scheduling adapters remain incomplete |
| **OriginWeave Policy** | Capability, origin, purpose, risk, crawler, approval and sensitive-data authority | Accepted architecture | Capability/origin/purpose/risk/crawler/approval and purpose-bound sensitive-data policy foundations are implemented on protected main; trusted sensitive-data broker/storage/lifecycle remain planned under issue #10 |
| **OriginWeave Evidence** | Credential-free evidence, provenance and task-trail contracts | Accepted architecture | Credential-free network evidence and purpose-bound sensitive-access receipts are implemented; complete Evidence Trail, WARC/PROV adapters and durable enterprise storage remain planned |
| **OriginWeave Protocol** | Stable browser-agent protocol independent of one upstream automation standard | Planned | Contract documented; implementation pending |
| **OriginWeave SDK** | Typed client libraries and adapters | Planned | Not a shipped product surface |
| **OriginWeave Enterprise** | Managed policy, tenancy, SSO/SCIM, residency, audit, deployment and support | Planned | Enterprise persistence/control plane not shipped |

## 7. Execution modes

### 7.1 Human Mode

**Status:** Accepted architecture.

Person-controlled browsing. Autonomous control is denied unless separately activated; ordinary installed extensions remain governed by Chromium and enterprise policy. Human input/rendering has priority over optional agent/model work.

### 7.2 Assist Mode

**Status:** Accepted architecture.

The system can read, summarize and prepare reversible work, but state-changing behavior follows the same typed action, policy and approval path as delegated automation. Runtime integration is tracked as implementation work, not as part of the status label.

### 7.3 Agent Task Mode

**Status:** Accepted architecture.

A delegated task uses a task-scoped isolated browser context/profile policy, explicit capabilities, origins, purposes, sensitive-data authority and resource budget. It must not inherit unrestricted Human Mode authority by convenience.

### 7.4 Crawler Mode

**Status:** Accepted architecture.

Governed public collection is read-only, robots/rate/resource/purpose/retention aware, and does not include CAPTCHA solving, fingerprint impersonation/evasion intended to defeat bot-management, or deliberate access-control circumvention. Privacy-preserving minimization of ambient host fingerprint leakage is a separate presentation-identity boundary and grants no bypass authority.

## 8. Core user journeys

### 8.1 Delegated web task

```text
user goal
-> create isolated session
-> establish task authority
-> navigate through governed network/service boundaries
-> observe structured page state
-> propose typed action
-> evaluate deterministic policy and approval
-> execute through trusted adapter
-> verify expected post-condition
-> record credential-free evidence
-> return result + evidence trail
```

### 8.2 Evidence-first extraction

```text
requested schema
-> typed/site-provided data when available
-> structured metadata
-> bounded network data
-> accessibility + DOM + layout
-> bounded visual fallback
-> schema/value validation
-> source/provenance binding
-> export value + evidence
```

### 8.3 Sensitive form completion

```text
planner identifies field + purpose + destination
-> policy evaluates exact scope
-> planner receives opaque handle only
-> trusted broker revalidates current authority
-> trusted browser path receives minimum required value
-> post-condition is verified
-> disclosure receipt records metadata without protected value
```

The full journey is target architecture until the trusted broker, browser-fill path, and post-condition/evidence path are all protected-main integrated. The purpose-bound sensitive-data policy foundation and access-evidence primitives are already on protected main; those implemented subcomponents do not make the complete broker journey shipped.

### 8.4 Enterprise crawler

```text
public-crawl purpose
-> origin scope
-> robots/rate/retention policy
-> bounded read-only navigation
-> structured extraction
-> evidence/provenance export
```

## 9. Functional requirements

### 9.1 Compatibility

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-COMP-001 | Chromium is the compatibility kernel; OriginWeave does not reimplement Blink or V8 | Accepted architecture | ADR 0001 |
| PRD-COMP-002 | Maintain a Manifest V3 compatibility matrix and representative extension test farm | Planned | Partial protected-main pinned-Chromium evidence covers service worker, content script, storage, DNR, tabs, windows, scripting, commands, side panel, bookmarks, history, restart and repeatability; active PR #43 adds bounded real downloads evidence; issue #27 still owns the complete matrix/release acceptance |
| PRD-COMP-003 | Chromium-specific integrations remain behind versioned adapters | Planned | Adapter strategy ADR 0107 |
| PRD-COMP-004 | Headless runtime remains independently usable without the interactive browser UI | Planned | Modular architecture target |
| PRD-COMP-005 | Governed sessions minimize ambient host fingerprint leakage through a bounded, internally consistent presentation identity | Proposed | Local `originweave-fingerprint` explicit-validation kernel evidence and Proposed ADR 0110; evidence-backed default selection, Chromium application, and real cross-surface evidence remain unshipped |

### 9.2 Session and observation authority

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-OBS-001 | Autonomous observations can carry explicit browser-session, browsing-context, canonical-origin and document-epoch authority | Implemented | `ObservedNodeHandle`, `BrowserSessionId`, `BrowsingContextId` and `DocumentEpoch` are on protected main under Accepted ADR 0010; real browser adapter remains planned |
| PRD-OBS-002 | Actionable semantic-node handles are invalidated by relevant document-epoch changes at the action linearization boundary | Accepted architecture | Core exact-authority validation exists; adapter lifecycle/mutation invalidation and atomic dispatch evidence remain planned; active PR #40 owns the bounded protocol-ID registry and remains non-shipped evidence |
| PRD-OBS-003 | Observation prefers typed/structured evidence before accessibility/DOM/layout and bounded visual fallback | Accepted architecture | ADR 0103; active PR #52 adds a bounded `SemanticNodeObservation` value contract bound to `ObservedNodeHandle`, typed node-local action descriptors and explicit non-empty evidence-channel provenance. It is not a browser observation adapter and remains active/non-shipped evidence |
| PRD-OBS-004 | Observation can use bounded incremental updates rather than full repeated snapshots | Planned | Adapter-specific design needed |
| PRD-OBS-005 | Source channel and trust/provenance remain explicit | Accepted architecture | Evidence model foundations exist; active PR #52 fails closed when a semantic observation has no contributing evidence channel, while channel identity itself grants no execution authority |

### 9.3 Typed action execution

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-ACT-001 | Actions carry typed kinds, risk classes, canonical targets and immutable intent digests | Implemented | `originweave-core` / policy foundations |
| PRD-ACT-002 | Arbitrary JavaScript is not an ordinary autonomous production tool | Accepted architecture | ADR 0102 |
| PRD-ACT-003 | Standard browser actions are exposed through versioned typed contracts | Planned | Browser adapter not complete |
| PRD-ACT-004 | Command completion alone is not success; expected post-condition must be observed | Accepted architecture | Evidence/action design |
| PRD-ACT-005 | High-risk approval is bound to exact intent and target | Implemented | Protected-main safety kernel |

### 9.4 Network and service authority

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-NET-001 | Logical origin is distinct from resolved destination authority | Implemented | `originweave-core` + destination policy |
| PRD-NET-002 | Resolution snapshots are bounded, origin-bound and fail closed on unapproved expansion | Implemented | `originweave-destination` |
| PRD-NET-003 | Direct transport connects only to approved canonical sockets and verifies `peer_addr` | Implemented | `originweave-network` |
| PRD-NET-004 | TLS authenticates service identity over the exact governed transport with explicit roots/time | Implemented | `originweave-tls` |
| PRD-NET-005 | Proxy/PAC route authority is explicit and never ambient | Implemented | Protected-main route-authority foundation; PAC evaluation, proxy transport and CONNECT remain planned |
| PRD-NET-006 | Bounded HTTP semantics operate over authenticated governed transport | Planned | Current implementation evidence is active replacement PR #37; it is not protected-main truth. Historical PR #11 is predecessor lineage and must not be used as current implementation evidence |
| PRD-NET-007 | Real Chromium navigation proves end-to-end consumption of every shipped authority layer | Planned | Issue #28 / release acceptance requirement |

### 9.5 Secret and sensitive-data authority

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-DATA-001 | Raw secret values never enter model-visible context | Accepted architecture | ADR 0104; trusted browser/broker runtime path not fully shipped |
| PRD-DATA-002 | Sensitive disclosure binds tenant/task/field/purpose/destination/classification | Implemented | Protected-main purpose-bound sensitive-data policy kernel governed by Accepted ADR 0007; this status does not claim broker/storage/value resolution |
| PRD-DATA-003 | Trusted broker owns expiry, revocation, atomic use reservation and resolution | Planned | Broker/storage/lifecycle implementation pending under issue #10 |
| PRD-DATA-004 | Privacy controls use purpose-bound authorization, encryption, retention and audit rather than blanket masking | Accepted architecture | `DATA_GOVERNANCE.md` |
| PRD-DATA-005 | Model disclosure additionally binds provider/model/region/retention policy | Planned | Requires orchestrator/provider integration |

### 9.6 Evidence and provenance

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-EVD-001 | Generic network evidence is credential-free and value-redacted | Implemented | Protected-main evidence kernel |
| PRD-EVD-002 | Evidence binds validated source identity and digests | Implemented | Protected-main evidence foundations |
| PRD-EVD-003 | Evidence Trail links source, model judgement, policy, approval, action and verified outcome as distinct authorities | Planned | Conceptual ERD/provenance ADR; complete trail is not shipped |
| PRD-EVD-004 | **Origin Map** provides buyer-visible provenance exploration | Proposed | UX/product-design work still required |
| PRD-EVD-005 | WARC and PROV are separate interoperability/export adapters | Accepted architecture | ADR 0106 |
| PRD-EVD-006 | Sensitive-access evidence records authority without protected value | Implemented | Protected-main purpose-bound sensitive-access receipts |

### 9.7 Resource governance

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-RES-001 | Deterministic resource budgets produce cumulative mitigations | Implemented | `originweave-resource` foundations |
| PRD-RES-002 | Browser/human correctness outranks optional model throughput | Accepted architecture | ADR 0105 |
| PRD-RES-003 | CPU worker saturation participates in deterministic new-work admission | Implemented | Protected-main `ResourceSnapshot`/`ResourceGovernor` CPU-worker admission; platform worker telemetry/actuation remains adapter work |
| PRD-RES-004 | Platform adapters report bounded CPU/RAM/GPU/VRAM/network/storage telemetry | Planned | Platform integration required |
| PRD-RES-005 | Constrained GPU systems shrink/offload/pause model work before sacrificing governed browser correctness | Accepted architecture | ADR 0105 |

### 9.8 External agent interoperability

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-INT-001 | WebDriver BiDi is a versioned adapter, not core authority | Planned | W3C draft tracked |
| PRD-INT-002 | CDP capabilities are pinned/version-gated | Planned | Chromium-specific adapter |
| PRD-INT-003 | WebMCP is optional, experimental and untrusted-content aware | Planned | Adapter required |
| PRD-INT-004 | MCP integrates through the Rust control plane rather than directly owning Chromium | Planned | MCP adapter required |
| PRD-INT-005 | OriginWeave Protocol is the stable internal/external semantic boundary | Planned | API contract defined |

### 9.9 Extensions

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-EXT-001 | Manifest V3 remains the extension compatibility baseline | Accepted architecture | Official Chrome platform baseline; real pinned-Chromium evidence exists on protected main |
| PRD-EXT-002 | Upstream extension APIs are preserved where possible | Accepted architecture | Chromium-kernel strategy; current protected-main compatibility lane exercises multiple real MV3 APIs |
| PRD-EXT-003 | Extension access to agent authority requires separate signed policy grant | Planned | Protected-main extension authority foundation exists, but the complete managed-extension/native-messaging/enterprise runtime contract remains open under issue #27; Proposed ADR 0013 does not itself make this shipped |
| PRD-EXT-004 | Compatibility tests cover install/update, worker lifecycle, scripts, storage, DNR, messaging, download, side panel and isolation | Planned | Protected-main suite already covers worker/content/storage/DNR/tabs/windows/scripting/commands/side panel/bookmarks/history/restart/repeatability; active PR #43 adds downloads; install/update/native messaging/enterprise isolation and release-wide matrix remain open under issue #27 |

### 9.10 Crawler and capture policy

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-CRAWL-001 | Crawler mutation is denied and robots policy is explicit | Implemented | Safety-kernel policy foundation |
| PRD-CRAWL-002 | Rate, depth, count, concurrency, retention, purpose and export controls are explicit | Planned | Crawler runtime work required |
| PRD-CRAWL-003 | CAPTCHA bypass, fingerprint impersonation or evasion intended to defeat bot-management, and deliberate access-control circumvention are excluded | Accepted architecture | ADR 0108 and Proposed ADR 0110; privacy-preserving host-fingerprint minimization does not grant bypass authority |

### 9.11 Enterprise operation

| ID | Requirement | Status | Implementation evidence / note |
|---|---|---|---|
| PRD-ENT-001 | SSO/SCIM, tenant isolation, managed policy, regional residency, encrypted profiles and break-glass controls | Planned | Enterprise control plane not shipped |
| PRD-ENT-002 | OpenTelemetry-compatible observability reports task, stale-action, policy, resource and recovery evidence without protected values | Planned | Operability contract |
| PRD-ENT-003 | Operator workflows support cancellation, quarantine, replay, crash recovery, rollback and controlled upgrade | Planned | Operability contract |
| PRD-ENT-004 | Procurement evidence includes SBOM, provenance, reproducibility and compatibility matrices | Planned | Release contract |

## 10. Non-functional requirements

### 10.1 Correctness and fail-closed semantics

- Rust-owned production behavior uses exact coverage and meaningful contract/property/integration tests.
- Network, browser, secret and evidence authority fail closed on ambiguity.
- State-changing success requires post-condition evidence.
- Old-head, synthetic, status-only or model-only evidence never upgrades the current source state.

### 10.2 Security and privacy

- Renderer compromise is in scope.
- Page/tool/download/model data is untrusted observation.
- Deterministic policy cannot be weakened by model output.
- Generic evidence/logging excludes raw credentials and protected values.
- Sensitive disclosure follows [`DATA_GOVERNANCE.md`](DATA_GOVERNANCE.md).
- Cross-tenant and confused-deputy tests are required before corresponding enterprise claims.

### 10.3 Resource reliability

- All potentially attacker-controlled byte/count/depth/time/concurrency dimensions are bounded at their trust boundary.
- Model work must not make state-changing browser work unverifiable.
- Admission failure is explicit rather than silent overcommit.

### 10.4 Performance

Performance goals are profile- and hardware-specific. Product releases must publish measured baselines rather than invented universal latency targets. Required dimensions include foreground input/frame behavior, task throughput, observation size/compression, peak RAM/VRAM, model fallback cost, extraction accuracy and task success stability.

### 10.5 Accessibility

- Product UI targets WCAG 2.2 AA where applicable.
- Keyboard operation, focus visibility, accessible names/states, non-color-only signals and exact-value/evidence alternatives are release requirements for shipped UI.
- Accessibility must not leak protected values into hidden DOM or accessible labels.

### 10.6 Operability

- Every production failure has a typed or bounded classification and recovery/rollback path.
- Operator evidence excludes raw secrets and uncontrolled page content.
- SLI/SLO/RPO/RTO values are measured deployment-profile evidence, not architecture prose.

### 10.7 Packaging and supply chain

- External actions/dependencies are pinned according to repository policy.
- Release evidence includes dependency/security scanning, SBOM/provenance and artifact identity.
- Reproducibility claims apply only when exact artifact comparison proves them.

## 11. Buyer-visible acceptance

A commercial release is accepted only when the integrated exact protected source head and produced release evidence prove the claims actually included in that release.

Minimum acceptance families:

1. exact CI/security/SAST/coverage/rustdoc evidence;
2. qualifying independent non-author review when current GitHub or explicit operational governance requires it;
3. protected branch/ruleset acceptance with zero valid unresolved findings;
4. browser/version/extension compatibility evidence for claimed surfaces;
5. realistic DNS/route/TCP/TLS/HTTP/navigation security tests for claimed network paths;
6. hostile prompt/node/secret/tenant/resource tests for claimed agent paths;
7. accessibility evidence for shipped UI;
8. package/SBOM/provenance/reproducibility evidence;
9. migration/configuration/rollback/recovery evidence for changed durable/runtime contracts; and
10. post-publication artifact verification against the intended source/release identity.

No emergency or executive path may convert missing required evidence into a successful release.

## 12. Degraded behavior

Failures are scoped to the affected capability:

- unsupported browser adapter -> disable governed capability, never raw-script fallback;
- stale node -> reject and re-observe;
- unavailable broker/policy -> no protected disclosure;
- model/provider failure -> deterministic path continues where independent; model-required task fails or uses only policy-approved fallback;
- resource pressure -> bound/reduce/pause optional work before compromising browser correctness;
- evidence failure for governed state-changing action -> no proved-success claim;
- crawler challenge/rate/robots block -> stop/degrade affected origin without evasion.

## 13. Non-goals

The following are not product capabilities unless a future reviewed product decision explicitly changes the boundary:

- Blink/V8/browser-engine rewrite;
- arbitrary JavaScript as the ordinary autonomous action interface;
- model-visible raw-secret delivery;
- implicit trust from network location, browser profile, extension install or credential possession;
- CAPTCHA solving, fingerprint impersonation or evasion intended to defeat bot-management, residential-proxy rotation, or access-control circumvention;
- blanket PII masking as the only privacy control;
- unbounded raw HTML/screenshot/network retention;
- universal legal/copyright authorization inferred from `robots.txt`;
- self-declared CSAP/SOC 2/ISO certification;
- merging or releasing by bypassing required checks, currently applicable review governance, branch protection, or reproducibility/provenance evidence.

## 14. Product metrics

Release and product evaluation should include:

- repeated task success and variance;
- unauthorized-action rate;
- stale-node action rejection rate;
- injection-induced authority-escalation rate;
- extraction precision/recall and provenance completeness;
- peak RAM/VRAM and browser frame/input degradation;
- recovery success after browser/model/network failure;
- secret/protected-value leakage rate in model/log/evidence corpus;
- browser/extension compatibility pass rate;
- release provenance completeness.

## 15. Ownership and integration

OriginWeave remains independently usable. CWL integrations use explicit versioned APIs/events/artifacts; no sibling service receives direct OriginWeave application-database authority by default. Central `.github`, contextual-orchestrator, EgressWeave, naruon and other products retain their own writer/authority boundaries.

## 16. Change control

A material change to product identity, execution modes, action authority, browser/node lifetime, network authority, secrets/data governance, resource priority, evidence/provenance, crawler behavior, enterprise tenancy, protocol adapters or release acceptance requires the affected PRD/TRD/Architecture/ADR/UML/ERD/threat/test/operability/traceability views to be updated or explicitly proven unaffected.

Status changes occur only from fresh evidence. `Accepted architecture` never automatically becomes `Implemented`; active PR work stays non-shipped until protected integration and exact acceptance evidence exist.
