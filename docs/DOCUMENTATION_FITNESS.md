# OriginWeave Documentation Fitness Assessment

- **Assessment date:** 2026-08-11
- **Assessment scope:** protected `main`, every current OriginWeave implementation lane relevant to canonical product truth, and durable product decisions that must be reconstructable without chat history
- **Assessment type:** semantic fitness, not file-presence inventory
- **Current verdict:** **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**

## 1. Verdict

**DESIGN-SUFFICIENT** means the repository has a coherent product, technical, architecture, decision, diagram, data-model, security, testing, operability, protocol and release graph sufficient to implement and review OriginWeave without reconstructing product intent from chat history.

**PROTECTED-MAIN-PARTIAL** means the design graph is sufficient, while protected `main` still lacks the canonical reconciliation and several active implementation slices. Active pull requests are implementation evidence only. Neither a green feature branch nor a Proposed ADR becomes shipped truth through documentation wording.

File existence alone is never sufficient. An artifact can exist and still be stale, contradictory, overclaiming, underclaiming, or disconnected from executable evidence.

## 2. Fitness matrix

| Documentation family | Fitness | Current evidence / remaining boundary |
|---|---|---|
| PRD | **PRESENT-CURRENT on this branch / protected-main follow-up required** | Protected-main requirements remain distinct from active evidence. #37 is the bounded-HTTP replacement; #45→#46→#53→#55 narrows sensitive-handle authority without creating the trusted broker; #47→#50→#54 narrows resolution freshness through socket use; #40→#52→#57→#58 provides browser authority/semantic prerequisites; #43→#56→#59→#60→#61 plus #49 provide active MV3 compatibility evidence only. |
| TRD | **PRESENT-CURRENT on this branch / protected-main follow-up required** | One protected-main implementation state is kept separate from volatile active/non-shipped evidence. Value objects, fixtures and compatibility tests do not imply deployed services, browser adapters or completed runtime paths. |
| Root Architecture | **PRESENT-CURRENT** | The Chromium compatibility kernel plus Rust authority-bearing control plane remains correct. #47/#50/#54 refine ADR 0004; #45/#46/#53/#55 refine ADR 0007; #40/#52/#57/#58 refine browser observation/action boundaries; #43/#49/#56/#59/#60/#61 remain compatibility work under issue #27. No later lane introduces a new trust domain, persistence owner or deployed component. |
| ADR index/lifecycle | **PRESENT-CURRENT on this branch** | Accepted ADRs remain distinct from Proposed decisions. ADR 0013 separates MV3 compatibility from Agent authority; ADR 0014 governs architecture-decision lifecycle. Their branch presence or later integration cannot silently promote them to Accepted. |
| Individual ADRs | **SUFFICIENT BY LIFECYCLE** | Existing Accepted decisions cover current material trust boundaries. #59/#60/#61 refine compatibility evidence and do not independently justify new ADRs. |
| UML / control-flow diagrams | **PRESENT-CURRENT with one legitimate deferral** | Component, network authority, observation/action, delegated-task state, deployment, evidence, secret-fill, approval, resource-pressure/GPU fallback and hourly automation flows exist. `uml/extension-authority.md` closes the permission-vs-Agent-authority gap. Detailed real-Chromium adapter/input/post-condition UML remains deferred until issue #28 executable contracts stabilize. |
| Conceptual ERD/domain model | **PRESENT-CURRENT** | The ERD remains explicitly conceptual until a real persistence owner/schema exists. #45/#46/#47/#48/#50/#51/#52/#53/#54/#55/#56/#57/#58/#59/#60/#61 add no OriginWeave-owned durable store. Manufacturing tables for in-memory state, value objects, query/action-target primitives or compatibility fixtures would be false architecture. |
| Traceability | **PRESENT-CURRENT on this branch** | Uses explicit protected-main, active-PR, partial, accepted-architecture, planned, research-only, superseded and out-of-scope maturity vocabulary. Volatile exact-head evidence lives in the dated maturity appendix, now through #61. |
| Threat model / Security | **PRESENT-CURRENT with implementation follow-up** | Untrusted content, network, secret, provenance and extension risks are covered. Active browser semantics remain descriptive until policy and execution. #59/#60/#61 mutate only controlled compatibility state and add no Agent authority. |
| Test strategy / quality gates | **PRESENT-CURRENT** | Exact owned production function/line/region/branch coverage, rustdoc and realistic boundary testing are explicit. Active compatibility work uses exact RED→GREEN evidence, pinned real Chromium and repeated trials rather than source-text claims alone. |
| Operability / incident response | **PRESENT-CURRENT** | Failure, readiness, quarantine, cleanup and recovery concepts exist. The active fixture lanes add no daemon/service or persistence owner, so new SLO/RPO/RTO claims would be fabricated. |
| API / protocol contracts | **PRESENT-CURRENT as target contracts** | #52 is an internal semantic-observation value API, #57 a bounded typed-query API, and #58 an authority-bound action-target bridge; none is a BiDi/CDP/WebMCP wire protocol, browser input executor, business-risk classification, policy approval or post-condition proof. |
| Release / rollback / provenance | **PRESENT-CURRENT** | Release remains bound to one exact integrated protected head. Active stacks #40→#52→#57→#58, #47→#50→#54, #45→#46→#53→#55, and #43→#56→#59→#60→#61 plus parallel #49 preserve dependency order; predecessor-head success cannot satisfy a later head. |
| Data governance / privacy | **PRESENT-CURRENT architecture / PARTIAL runtime** | Purpose-bound policy/evidence foundations exist. Authenticated workload identity, durable trusted-broker storage, protected-value resolution/fill, KMS, cross-process transactionality, compensation, retention and model-disclosure lifecycle remain open under issue #10. |
| Standards / doctoring | **PRESENT-CURRENT with continuous watch** | Primary browser/protocol/standards evidence and APA 7 references distinguish living/vendor/experimental material from final normative standards. Exact browser release evidence stays pinned to executable Chromium evidence rather than documentation alone. |

## 3. Reconciliation findings

### 3.1 HTTP lineage

Protected-main PRD previously named historical PR #11 as active HTTP evidence. Current replacement work is PR #37, while protected main still does not ship the reconstructed bounded HTTP capability.

**Resolution:** #37 is active/non-shipped implementation evidence, #11 is historical predecessor lineage, and integration before any of these branch repairs become protected-main truth remains mandatory. Old-head checks, reviews and mergeability never transfer.

### 3.2 Sensitive-data authority and broker lifecycle

Protected main contains purpose-bound sensitive-data policy/evidence governed by Accepted ADR 0007. The active dependency chain is #45 → #46 → #53 → #55: lifecycle evidence, authoritative in-process use reservation, first-revocation-wins state, then audience binding.

The audience string accepted by the value/policy primitive is **not authentication**. A future trusted broker must derive audience from authenticated workload/service identity rather than caller-controlled input. One-process synchronization is not durable/cross-process atomicity.

**Resolution:** these lanes may be `IMPLEMENTED_ON_ACTIVE_PR`; the complete broker remains Planned under issue #10. They do not justify a fictitious broker process, KMS path, database table, transaction manager, browser-fill adapter, new deployment topology or physical ERD entity.

### 3.3 Manifest V3 compatibility

Protected main already proves a pinned-Chromium baseline for service worker, content script, storage, DNR, tabs, windows, scripting, commands, side panel, bookmarks/history read behavior, restart persistence and repeatability. The active compatibility stack adds:

- #43: controlled downloads;
- #49: per-trial ephemeral profile isolation;
- #56: bookmark create/read/delete cleanup;
- #59: history add/read/delete/absence verification;
- #60: trial-local unpacked-extension `1.0.0` → `1.0.1` update with explicit schema migration; and
- #61: real content-script isolated-world evidence in which the page main world retains a `page` sentinel while the content script independently retains an `extension` sentinel.

#43/#49/#56/#59/#60/#61 are active compatibility evidence only. Chromium permission or browser compatibility success is not an OriginWeave Agent capability, policy grant, approval or protected-value authority. A successful fixture cannot become an OriginWeave Agent history grant, bookmark grant, download grant or arbitrary page-JavaScript bridge.

The supported-capability matrix in `docs/doctoring/mv3-compatibility.md` separates `PROTECTED_MAIN`, `ACTIVE_PR`, `PLANNED`, security-gated and out-of-scope claims. Update migration is intentionally distinct from restart persistence, and isolated-world behavior is intentionally distinct from injection alone.

**Resolution:** complete compatibility remains Planned under issue #27. Proposed ADR 0013 remains the authority separator. #59/#60/#61 are refinements of that decision, not new architecture decisions.

### 3.4 Browser identifier authority

Protected main contains session/context/document/node foundations under Accepted ADR 0010. Active #40 maps protocol-local identifiers into OriginWeave-owned authority and remains non-shipped.

**Resolution:** protocol identifiers remain adapter-local, and detailed adapter sequence UML remains deferred until issue #28 stabilizes executable BiDi/CDP contracts.

### 3.5 ADR discoverability and identifier allocation

The earlier index omitted existing ADRs, and active #37 already reserves ADR identifiers 0011/0012.

**Resolution:** the branch indexes every ADR by lifecycle, uses non-colliding 0013/0014 for new Proposed decisions, and treats collision-sensitive identifiers as reserved across protected main plus active work.

### 3.6 Documentation contract parser

The first fitness contract accepted only bare lifecycle metadata even though repository-valid ADRs can carry descriptive suffixes.

**Resolution:** machine checks validate the leading supported lifecycle state and reject unknown states without rejecting valid suffixes.

### 3.7 UML audit correction

An early audit incorrectly called resource-pressure and hourly-automation views missing.

**Resolution:** the existing resource-pressure/GPU fallback and hourly automation flows are recognized. Only the genuinely missing extension-permission-to-Agent-authority view was added.

### 3.8 Resolution freshness authority

Active #47 → #50 → #54 progressively binds approved resolution state to first-party network planning and rechecks freshness immediately before socket I/O under trusted monotonic time.

**Resolution:** this refines Accepted ADR 0004 rather than introducing a resolver service, proxy/PAC authority, wall-clock authority, persistence owner or new deployed component.

### 3.9 TLS revocation-material freshness

Active #48 provides a bounded freshness primitive for already verified revocation material.

**Resolution:** this is not OCSP/CRL acquisition, signature/path validation, cache operation or an unrevoked-certificate claim. No fictitious revocation-service topology is added.

### 3.10 Browser task telemetry

Active #51 provides bounded RSS, observation-byte, action-latency and task-duration values but performs no OS/Chromium sampling.

**Resolution:** value-object availability cannot be promoted to measured runtime evidence or GPU/local-model evidence.

### 3.11 Semantic observation authority

Active #52 carries an OriginWeave-owned node handle, bounded semantic fields, typed advertised actions, provenance channels and bounded relationships. Every relationship must remain inside the same browser session, browsing context, canonical origin and document epoch. Self-parent/self-child relationships and duplicate child handles fail closed. The relationship graph remains descriptive evidence.

**Resolution:** #52 is not a browser observation adapter. Accessibility, DOM, layout, WebMCP, structured-data and visual inputs remain untrusted observations and cannot mint capability.

### 3.12 Typed semantic query authority

Active #57 performs bounded exact role, accessible-name and required-typed-action matching only against already validated semantic observations.

**Resolution:** semantic query success is descriptive selection, not CSS/XPath/raw-DOM authority, arbitrary JavaScript, browser I/O, action dispatch or policy approval.

### 3.13 Authority-bound semantic node action target

Active #58 accepts only an advertised `NodeActionKind`, carries the exact OriginWeave-owned node handle and revalidates session/context/origin/document epoch immediately before later use.

**Resolution:** this remains descriptive execution input. A node advertising `Click` cannot determine business-risk classification: the same click could represent navigation, submit, purchase, delete, permission management or legal consent. Policy intent, approval, browser dispatch and verified success remain separate boundaries under issue #28.

### 3.14 Controlled history mutation compatibility

Active #59 creates one synthetic loopback history entry, requires exact readback, removes it in `finally` and proves its absence afterwards.

**Resolution:** browser history compatibility is not an OriginWeave Agent history grant. No history values are exposed to a model and no human/default profile is used.

### 3.15 Controlled extension update migration

Active #60 copies the checked-in fixture into a trial-local directory, keeps one ephemeral profile and one extension path, transitions only `1.0.0` → `1.0.1`, observes the loaded version and requires schema state 1 → 2 migration.

**Resolution:** this proves one deterministic unpacked-extension version transition. It does not establish Chrome Web Store updates, enterprise rollout, arbitrary downgrade or third-party migration safety.

### 3.16 Content-script isolated-world compatibility

Active #61 gives the page main world and the MV3 content script the same JavaScript global name with different values and requires the page to keep publishing `page` while the content script observes its own `extension` value. If the worlds collapse, the existing compatibility gate fails in real pinned Chromium.

**Resolution:** this is a bounded compatibility proof, not a trusted page-content channel, arbitrary JavaScript bridge or Agent capability.

## 4. Durable product decisions captured by the canonical graph

1. OriginWeave is **Browse. Act. Prove.**: an enterprise agentic web runtime and provenance-native browser platform, not Selenium-style automation.
2. Chromium remains the compatibility kernel; Blink/V8 are not rewritten for differentiation.
3. Rust owns new authority-bearing control-plane semantics and remains independently reusable.
4. Human, Assist, Agent Task and Crawler modes have distinct authority/profile semantics; Agent Task does not ambiently inherit Human authority.
5. Page, extension, WebMCP and model content are untrusted observations, not goal/policy authority.
6. Structured observation precedes raw HTML or screenshot-only interpretation.
7. Typed actions and observed post-conditions replace arbitrary-script and command-return-as-success semantics.
8. Logical origin, destination, route/proxy, TCP peer, TLS identity and HTTP semantics are separate authorities.
9. Session/context/document epoch/node identity is separate from raw BiDi/CDP identifiers.
10. Manifest V3 permission is not an OriginWeave Agent capability; compatibility evidence and Agent-authority evidence are independent.
11. Raw secrets stay outside model-visible context; sensitive values use purpose-bound authority, opaque handles and trusted fill paths.
12. Browser correctness/human interaction outrank optional local-model throughput under pressure.
13. Provenance distinguishes source observation, model judgement, policy, approval, action and verified outcome.
14. WebDriver BiDi, CDP, WebMCP and MCP are versioned adapters, never the product authority model by themselves.
15. The first browser proof uses pinned stock Chromium before any broad fork.
16. High-risk actions remain approval-bound; Crawler Mode remains read-only and excludes CAPTCHA/block-evasion features.
17. Autonomous development uses OpenCode/NVIDIA NIM under deterministic gates and separate review/publication authority, never `COPILOT_GITHUB_TOKEN` as the development-model credential.
18. Documentation, checks, reviews, model judgements and operational evidence are separate evidence authorities.
19. Work-conserving maintenance continues to another safe lane rather than stopping on one merge, document, RCA, queued check or approval gap.
20. Collision-sensitive repository identifiers are reserved across protected main and active work before allocation.
21. In-memory sensitive-handle primitives may narrow replay/revocation risk without claiming the durable trusted broker exists.
22. A validated DNS answer is not sufficient socket authority indefinitely; resolution-to-socket use requires bounded trusted-monotonic freshness.
23. Revocation-material freshness, cryptographic validity, acquisition/cache operation and an unrevoked claim remain separate evidence authorities.
24. Browser telemetry values and actual OS/Chromium measurement adapters are separate maturity claims.
25. Semantic observation provenance and advertised node-local actions are descriptive evidence and never execution authority.
26. Semantic relationships remain bounded within exact session/context/origin/document authority.
27. Sensitive-handle audience must ultimately derive from authenticated workload/service identity.
28. Real browser compatibility fixtures may mutate and clean controlled synthetic state without creating Agent authority.
29. Semantic query success is neither selector authority nor permission to execute an advertised action.
30. Semantic action-target binding preserves exact node authority but remains separate from business-risk classification, policy approval, dispatch and observed success.
31. Update migration, restart persistence, injection and isolated-world behavior are separate compatibility claims and must retain distinct executable evidence.

## 5. Architecture views legitimately deferred

### 5.1 Extension authority — present

`uml/extension-authority.md` captures:

```text
Chromium MV3 permission
-> extension runtime
-> untrusted extension observation/message
-> OriginWeave extension policy/grant
-> Agent capability decision
-> typed action proposal
-> deterministic policy
```

Compatibility evidence cannot substitute for Agent-authority evidence, or vice versa.

### 5.2 Network freshness sequence — reconcile after #47 → #50 → #54 integrates

```text
resolver answer
-> destination policy + origin binding
-> fresh resolution approval
-> connection authorization at trusted monotonic use time
-> socket-use freshness recheck
-> exact socket candidate
-> observed TCP peer
-> TLS/HTTP authority layers
```

### 5.3 Real Chromium vertical slice — deferred until issue #28 stabilizes

```text
isolated profile/context
-> BiDi/CDP adapter
-> OriginWeave registry
-> semantic observation
-> typed semantic query
-> authority-bound semantic action target
-> explicit business intent / deterministic policy
-> real browser input
-> observed post-condition
-> credential-safe evidence
-> teardown/recovery
```

#40/#52/#57/#58 make the authority/value boundaries concrete but do not yet establish browser I/O, typed business policy, native input or post-condition proof. Freezing temporary protocol fields into authoritative UML before those executable contracts exist would create false architecture.

### 5.4 Trusted sensitive-data broker — deferred until issue #10 establishes a real runtime boundary

Protected-main policy/evidence plus #45→#46→#53→#55 do not justify inventing a broker process, durable database, KMS topology, authenticated service-identity mechanism or browser-fill adapter. Add physical ERD/component/transaction views only when executable ownership exists.

## 6. Completion criteria

The graph becomes **PROTECTED-MAIN-SUFFICIENT** only when:

1. PRD/TRD implementation inventories agree with the exact protected-main crates/APIs/browser evidence;
2. no historical/superseded lineage is presented as current implementation evidence;
3. ADR indexes discover every protected-main ADR and match lifecycle metadata;
4. UML covers every implemented material authority flow, with planned diagrams clearly marked;
5. ERD/domain models distinguish conceptual, in-memory, persisted, adapter-owned and external state truthfully;
6. traceability maps each material requirement/Accepted decision to protected-main evidence, explicitly active-PR evidence or an open issue;
7. documentation tests catch stale status/index/link/ownership/identifier/maturity terminology;
8. security, test, operability, privacy and release docs agree on shipped-vs-planned boundaries; and
9. this documentation reconciliation itself reaches protected main through repository governance and is re-evaluated against whatever feature heads actually integrated.

Until then, OriginWeave is **design-documented but not protected-main documentation-closed**. That finding must never be used as an excuse to stop unrelated safe implementation work.
