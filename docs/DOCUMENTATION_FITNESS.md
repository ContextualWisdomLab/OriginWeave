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
| PRD | **REPAIRED IN THIS CHANGE with active-lane follow-up** | Protected-main requirements remain distinct from active evidence. #37 is the current bounded-HTTP replacement; #45→#46→#53→#55 narrows sensitive-handle lifecycle, reservation, revocation and audience admission without creating the trusted broker; #47→#50→#54 narrows resolution freshness through the socket-use boundary; #51/#52/#57 are browser-runtime prerequisites, not a real Chromium adapter; #43/#49/#56 are MV3 compatibility evidence, not Agent authority. |
| TRD | **REPAIRED IN THIS CHANGE with active-lane follow-up** | Implementation inventory uses one protected-main status plus separate active/non-shipped evidence. Session/node, route, sensitive-data, HTTP, proxy/PAC, MV3 and broker boundaries remain reconciled to protected-main truth. Active value objects, semantic matching and compatibility fixtures do not imply deployed services or complete runtime paths. |
| Root Architecture | **PRESENT-CURRENT with follow-up** | Chromium compatibility kernel + Rust authority-bearing control plane remains correct. #47/#50/#54 tighten ADR 0004; #45/#46/#53/#55 remain inside ADR 0007; #40/#52/#57 remain inside browser authority/structured-observation/action-boundary decisions; #43/#49/#56 remain compatibility work under issue #27. No later lane introduces a new trust domain, persistence owner or deployed component. |
| ADR index/lifecycle | **REPAIRED IN THIS CHANGE** | Accepted ADRs 0001-0008 and 0010 plus Proposed 0009, 0013, 0014 and 0100-0109 are discoverable without promoting Proposed decisions. Repository-scoped identifiers are reserved across protected main and active work. |
| Individual ADRs | **PARTIAL BY LIFECYCLE** | Accepted ADRs remain governing design authority. Proposed ADR 0013 separates MV3 compatibility from extension-to-Agent authority; Proposed ADR 0014 governs ADR acceptance. #53/#54/#55/#56/#57 refine existing decisions and do not independently justify new ADRs. |
| UML / control-flow diagrams | **PRESENT-CURRENT with follow-up** | Component, network authority, observation/action, delegated-task state, deployment, evidence, secret-fill, approval, resource-pressure/GPU fallback and hourly automation flows already exist. `uml/extension-authority.md` closes the permission-vs-Agent-authority gap. Network freshness should reflect the socket-use recheck after #54 integrates; detailed real-Chromium sequence remains deferred until issue #28 stabilizes. |
| Conceptual ERD/domain model | **PRESENT-CURRENT** | The ERD remains explicitly conceptual unless a real persistence owner/schema is implemented. #45/#46/#47/#48/#50/#51/#52/#53/#54/#55/#56/#57 add no OriginWeave-owned durable store, so manufacturing tables for in-memory/value/query/fixture primitives would be false architecture. |
| Traceability | **REPAIRED IN THIS CHANGE with volatile evidence refresh required** | Uses `IMPLEMENTED_ON_PROTECTED_MAIN`, `IMPLEMENTED_ON_ACTIVE_PR`, `PARTIAL`, `ACCEPTED_ARCHITECTURE`, `PLANNED`, `RESEARCH_ONLY`, `SUPERSEDED`, and `OUT_OF_SCOPE`. The dated maturity appendix carries volatile exact heads for current active lanes through #57. |
| Threat model / Security | **PRESENT-CURRENT with follow-up** | Untrusted content, network, secret, provenance and extension risks are covered. #54 narrows resolution-to-socket TOCTOU only on active work; #48 is revocation-material freshness only; #53/#55 narrow handle revocation/audience misuse only in process; #52 binds semantic relationships to exact browser authority; #57 performs bounded semantic matching without minting selector or action authority. None creates a broader authority grant. |
| Test strategy / quality gates | **PRESENT-CURRENT** | Exact owned-code function/line/region/branch coverage, rustdoc and realistic boundary testing remain explicit. Active work uses intentional RED evidence followed by narrow production changes. #52's relationship boundary and #57's typed-query boundary were compile-time RED before production implementation; #56 proved a stale compatibility contract against a real browser workflow. |
| Operability / incident response | **PRESENT-CURRENT with follow-up** | Failure, readiness, quarantine and recovery concepts exist. Later active lanes add no new daemon/service, so no SLO/RPO/RTO or runbook is fabricated. First-party socket timing and browser fixture cleanup remain implementation concerns within existing runtime boundaries. |
| API / protocol contracts | **PRESENT-CURRENT as target contracts** | OriginWeave Protocol and adapter boundaries are documented. #52 is an internal authority-bound semantic-observation API and #57 is an internal bounded typed-query API over those observations; neither is a BiDi/CDP/WebMCP wire protocol or action executor. #55 is an in-process policy API, not authenticated service identity; #56 is a compatibility fixture, not a product protocol. |
| Release / rollback / provenance | **PRESENT-CURRENT** | Release remains bound to one exact integrated protected head. Active stacks #40→#52→#57, #47→#50→#54, #45→#46→#53→#55, and #43→#49/#56 must preserve dependency order; predecessor-head success cannot satisfy a later head. |
| Data governance / privacy | **PRESENT-CURRENT architecture / PARTIAL runtime** | Purpose-bound policy/evidence foundations exist. #53 adds first-revocation-wins in-process state and #55 adds audience binding, but authenticated workload identity, durable broker storage, protected-value resolution/fill, KMS, cross-process transactionality, compensation, retention and model-disclosure lifecycle remain open under issue #10. |
| Standards / doctoring | **PRESENT-CURRENT with continuous watch** | Primary browser/protocol/standards evidence and APA 7 references distinguish draft/experimental material from final normative standards. |

## 3. Reconciliation findings and resolution state

### 3.1 HTTP lineage

Protected-main PRD previously named historical PR #11 as active HTTP evidence. Current executable replacement work is PR #37, while protected main still does not ship bounded HTTP semantics.

**Resolved on this documentation branch:** PRD, TRD and traceability name #37 only as active/non-shipped evidence, retain protected-main `Planned`, and treat #11 as predecessor lineage. Old-head checks/reviews do not transfer.

### 3.2 Sensitive-data authority and broker lifecycle

Protected main contains a purpose-bound sensitive-data policy/evidence foundation governed by Accepted ADR 0007, while the complete trusted broker remains unimplemented.

The current active dependency chain is #45 → #46 → #53 → #55:

- #45 records credential-free handle lifecycle evidence without storing protected values;
- #46 adds an in-process authoritative reservation count and removes caller authority over the prior-use count;
- #53 adds typed first-revocation-wins in-process state; and
- #55 binds handle admission to a bounded non-transferable audience and proves synchronized one-use contention while retaining revocation precedence.

The audience string accepted by the value/policy primitive is **not authentication**. A future trusted broker must derive audience from authenticated workload/service identity rather than caller-controlled input. Likewise, mutable-borrow or externally synchronized one-process serialization is not durable/cross-process atomicity.

**Resolved on this documentation branch:** these lanes may be represented as `IMPLEMENTED_ON_ACTIVE_PR` evidence while the complete broker/runtime stays `Planned` under issue #10. They do not justify a fictitious broker process, KMS path, database table, transaction manager, browser-fill adapter, new deployment topology or physical ERD entity.

### 3.3 Manifest V3 compatibility

Protected main has executable pinned-Chromium evidence for service worker, content script, storage, DNR, tabs, windows, scripting, commands, side panel, bookmarks, history, restart persistence and repeatability. Active #43 adds bounded downloads compatibility; #49 adds per-trial ephemeral-profile isolation; #56 proves a bounded real bookmark mutation lifecycle using `chrome.bookmarks.create` → `get` → `remove` with cleanup while retaining history coverage.

PR #56 also exposed and repaired a repository-contract drift: the real browser workflow already exercised mutation successfully while a Python source contract still required historical read-only `chrome.bookmarks.getTree`.

**Resolved on this documentation branch:** complete compatibility remains Planned under issue #27. #43/#49/#56 are active compatibility evidence only. Proposed ADR 0013 remains the authority separator: Chromium permission or browser compatibility success is not an OriginWeave Agent capability, policy grant, approval or protected-value authority.

### 3.4 Browser identifier authority

Protected main contains session/context/document/node authority foundations under Accepted ADR 0010. Active #40 owns a bounded registry that maps protocol-local identifiers into that authority model and remains active/non-shipped evidence.

**Resolved on this documentation branch:** protected-main foundations and active registry evidence remain separate. Detailed adapter-sequence UML remains deferred until executable browser adapter contracts stabilize.

### 3.5 ADR discoverability and identifier allocation

The earlier index omitted existing ADRs, and candidate ADR identifiers 0011/0012 were already reserved by active #37.

**Resolved on this documentation branch:** lifecycle indexes are complete for the branch, extension/governance decisions use non-colliding 0013/0014, and collision-sensitive identifiers are reserved across protected main plus active work.

### 3.6 Documentation contract parser

The first fitness contract accepted only bare lifecycle metadata, while Accepted ADR 0007 legitimately contains a descriptive suffix after `Accepted`.

**Resolved on this documentation branch:** the parser validates the leading supported lifecycle token while accepting repository-valid descriptive suffixes and rejecting unknown lifecycle states.

### 3.7 Initial UML audit false positive

The first audit incorrectly called resource-pressure and hourly automation views missing. Protected-main UML already contained them.

**Resolved on this documentation branch:** those existing views are recognized, and only the genuinely missing extension-permission-to-Agent-authority view was added.

### 3.8 Resolution freshness authority

Protected main validates, pins and non-expansively revalidates destination addresses, but it does not yet ship a bounded approval-to-socket-use interval. Active #47 provides the reusable resolution-freshness primitive; #50 makes first-party connection planning consume freshness; #54 rechecks freshness at `connect_at(current_time)` immediately before socket I/O.

The active stack deliberately does not claim resolver implementation, DNS acquisition, proxy/PAC authority or wall-clock authority. It narrows the plan-to-connect TOCTOU window using trusted monotonic timing and exact socket candidates.

**Documentation consequence:** this remains a refinement of Accepted ADR 0004, not a new deployed component or persistence entity. The network-authority sequence should be reconciled to protected-main truth only after dependency-ordered integration.

### 3.9 TLS revocation-material freshness

Protected main deliberately reports revocation as `NotConfigured`. Active #48 adds a reusable signed-window freshness check for already-verified revocation material.

**Documentation consequence:** #48 is `IMPLEMENTED_ON_ACTIVE_PR` for freshness only. It is not OCSP/CRL acquisition, signature/path validation, cache operation or proof that a certificate is unrevoked. No revocation service/cache topology or physical ERD is invented.

### 3.10 Browser task telemetry

Active #51 adds a bounded `BrowserTaskTelemetry` value object for RSS bytes, observation bytes, action latency and task duration. It does not sample Chromium or the operating system and does not create a GPU/local-model claim.

**Documentation consequence:** this is a reusable active-PR prerequisite for issue #28 resource evidence, not a runtime sampler service, persistence entity or release metric.

### 3.11 Semantic observation authority

Active PR #52 is stacked on browser-registry PR #40 and remains non-shipped. Its current semantic-observation contract carries an OriginWeave-owned `ObservedNodeHandle`, bounded role/accessibility-name/visible-text fields, observed state, typed node-local action descriptors, and a non-empty evidence-channel provenance set.

The latest active refinement adds optional parent and ordered child relationships with at most 128 children. Every relationship must remain inside the same browser session, browsing context, canonical origin and document epoch as the observation handle. Self-parent/self-child relationships and duplicate child handles fail closed. The relationship graph remains descriptive evidence; it cannot create execution capability or cross-document authority.

The evidence channel records how a value was observed; Accessibility, DOM, layout, WebMCP, structured-data and visual content remain untrusted observations. An advertised `NodeActionKind` likewise remains descriptive and grants no execution authority.

**Documentation consequence:** #52 is `IMPLEMENTED_ON_ACTIVE_PR` evidence for a semantic-observation value primitive only. It is not a browser observation adapter, performs no BiDi/CDP/WebMCP I/O or action dispatch, and establishes no service or persistence boundary. No new ADR or physical ERD entity is justified.

### 3.12 Typed semantic query authority

Active PR #57 is stacked on exact semantic-observation authority from #52 and remains non-shipped. Its `SemanticNodeQuery` requires at least one bounded typed selector and matches exact semantic role, accessible name and advertised `NodeActionKind` only against already validated `SemanticNodeObservation` values.

The test-only head intentionally failed compilation on the absent public query/error boundary. The current exact #57 head passes repository contracts, formatting, workspace checks/tests, strict Clippy, rustdoc and exact owned production function/line/region/branch coverage, plus the inherited Manifest V3 compatibility gate.

**Documentation consequence:** query matching is descriptive selection only. #57 does not expose CSS/XPath/raw DOM selector languages, arbitrary JavaScript, browser I/O, action dispatch, capability grants, policy approval or persistence. It refines the existing observation/action architecture and therefore does not independently justify a new ADR, deployed component, topology view or physical ERD entity.

## 4. Durable conversation decisions captured in GitHub

The canonical graph must continue to preserve these durable decisions:

1. OriginWeave is **Browse. Act. Prove.** — an enterprise agentic web runtime and provenance-native browser platform, not merely Selenium-style automation.
2. Chromium remains the compatibility kernel; Blink/V8 are not rewritten for product differentiation.
3. Rust owns new authority-bearing control-plane semantics and remains usable independently in headless/MSA composition.
4. Human, Assist, Agent Task and Crawler modes have distinct authority/profile semantics; Agent Task does not ambiently inherit Human Mode authority.
5. Page, extension, WebMCP and model content are untrusted observations, not goal/policy authority.
6. Structured observation precedes raw HTML or screenshot-only interpretation.
7. Typed actions and observed post-conditions replace arbitrary script execution and command-return-as-success.
8. Logical origin, destination, route/proxy, TCP peer, TLS identity and HTTP semantics are separate authorities.
9. Session/context/document epoch/node identity is separate from raw BiDi/CDP identifiers.
10. Manifest V3 permission is not an OriginWeave Agent capability; compatibility evidence and Agent-authority evidence are independent.
11. Raw secrets stay outside model-visible context; sensitive values use purpose-bound authority, opaque handles and trusted fill paths.
12. Browser correctness/human interaction outrank optional local-model throughput under resource pressure.
13. Provenance distinguishes source observation, model judgement, policy, approval, action and verified outcome; WARC/PROV are adapters rather than collapsed truth.
14. WebDriver BiDi, CDP, WebMCP and MCP are versioned adapters, never the product authority model by themselves.
15. The first browser proof is a pinned-stock-Chromium vertical slice before any broad Chromium fork.
16. High-risk actions remain approval-bound; Crawler Mode remains read-only and excludes CAPTCHA/block-evasion features.
17. Autonomous development uses OpenCode/NVIDIA NIM under deterministic gates and separate review/publication authority, never `COPILOT_GITHUB_TOKEN` as the development-model credential.
18. Documentation, checks, reviews, model judgements and operational evidence are separate evidence authorities.
19. Work-conserving maintenance continues to another safe lane rather than stopping on one merge, document, RCA, queued check or external approval gap.
20. ADR numbers, migrations, schema/API/protocol versions and other collision-sensitive repository identifiers are reserved across protected main **and active work** before allocation.
21. A policy primitive may narrow stale-count/replay risk without claiming the durable trusted broker exists; in-memory serialization, cross-process transactionality, lifecycle evidence and protected-value release remain separate maturity claims.
22. A validated DNS answer is not sufficient socket authority indefinitely; the resolution-to-socket interval must be bounded by caller-supplied trusted monotonic freshness and ordinary first-party socket planning must not retain an untimed bypass.
23. Revocation-material freshness, revocation-material cryptographic validity, acquisition/cache operation and an unrevoked-certificate claim are separate evidence authorities.
24. Browser-resource telemetry values and actual OS/Chromium measurement adapters are separate maturity claims; value-object availability cannot be promoted to measured runtime evidence.
25. Every semantic observation must identify at least one contributing evidence channel; observation provenance and advertised node-local actions are descriptive evidence and never grant execution authority.
26. Semantic parent/child relationships must be bounded and remain inside the observation's exact session/context/origin/document authority; relationship metadata cannot mint capability.
27. Sensitive-handle audience must ultimately be derived from authenticated workload/service identity; accepting an audience string in an internal value object is not authentication.
28. A real browser compatibility fixture may mutate and clean up test state, but compatibility success still cannot substitute for OriginWeave Agent-authority evidence.
29. A semantic node query may select bounded reviewed observation evidence, but query success is neither browser-selector authority nor permission to execute the advertised node-local action.

## 5. Architecture views still legitimately deferred

### 5.1 Extension authority — present in this branch

`uml/extension-authority.md` shows the durable separation:

```text
Chromium MV3 permission
-> extension runtime
-> untrusted extension observation/message
-> OriginWeave extension policy/grant
-> Agent capability decision
-> typed action proposal
-> deterministic policy
```

Compatibility evidence cannot substitute for Agent-authority isolation evidence, and vice versa.

### 5.2 Network freshness sequence — reconcile after #47 → #50 → #54 integrates

The durable sequence is:

```text
resolver answer
-> destination policy + origin binding
-> fresh resolution approval [trusted monotonic interval]
-> connection authorization at trusted monotonic use time
-> socket-use freshness recheck
-> exact socket candidate
-> observed TCP peer
-> TLS/HTTP authority layers
```

This is a sequence refinement inside the existing network authority model, not a new deployed component or persistence boundary.

### 5.3 Real Chromium vertical slice — deferred until issue #28 stabilizes

The eventual sequence must cover:

```text
isolated profile/context
-> BiDi/CDP adapter
-> OriginWeave registry
-> semantic observation
-> bounded exact-authority relationships
-> typed semantic query
-> opaque node authority
-> typed policy decision
-> real browser input
-> observed post-condition
-> credential-safe evidence
-> teardown/recovery
```

Active #52 and #57 make the semantic-observation and typed-query boundaries more concrete but still do not establish the browser adapter, process topology, policy-to-node action bridge, real input or post-condition verification sequence. Temporary protocol/field names must not be frozen into authoritative UML before executable contracts stabilize.

### 5.4 Trusted sensitive-data broker — deferred until issue #10 owns a real runtime boundary

Protected-main policy/evidence plus active #45→#46→#53→#55 do not justify inventing a broker process, database table, transaction manager, KMS path, authenticated service-identity mechanism or browser-fill adapter. When a real broker slice exists, the documentation graph must add actual component/transaction/data-lifecycle views and mark persisted versus in-memory versus external state from executable evidence.

## 6. What remains before documentation closure

### Completed or materially advanced on this branch

- PRD current-state reconciliation;
- TRD current-state reconciliation;
- requirement/decision/module/evidence traceability reconciliation;
- ADR lifecycle/index repair;
- Proposed ADR 0013 for MV3/extension authority separation;
- Proposed ADR 0014 for architecture-decision acceptance governance;
- extension authority UML;
- conceptual ERD truth discipline;
- documentation fitness and regression contracts;
- browser/protocol standards doctoring;
- sensitive-data evidence reconciliation through #55 without inventing broker persistence or authenticated service identity;
- resolution-freshness reconciliation through socket-use recheck #54 without inventing resolver/proxy authority;
- semantic-observation authority/relationship reconciliation for #52 without promoting it to a browser adapter;
- typed semantic-query reconciliation for #57 without promoting semantic matching to selector/action authority; and
- MV3 compatibility reconciliation through #56 without equating browser permission with Agent authority.

### Still required

- exact-head CI/security/review acceptance of this documentation PR after every documentation mutation;
- integration before any of these branch repairs become protected-main truth;
- re-reconciliation whenever active PR #37, #40, #43, #45, #46, #47, #48, #49, #50, #51, #52, #53, #54, #55, #56 or #57 integrates, closes, is superseded or materially changes head;
- network-authority sequence reconciliation after #47→#50→#54 reaches protected main;
- detailed real-Chromium vertical-slice UML when issue #28 implementation contracts are stable;
- trusted-broker UML/ERD/operability additions only when issue #10 establishes real runtime/persistence ownership;
- future ERD changes only when persistence ownership/entities actually change; and
- ongoing security/operability/release reconciliation as real browser/runtime boundaries integrate.

## 7. Completion criteria

The documentation graph becomes **PROTECTED-MAIN-SUFFICIENT** only when:

1. PRD/TRD implementation inventories agree with protected-main crates/APIs/executable browser evidence;
2. no canonical document identifies historical/superseded PR lineage as current implementation evidence;
3. ADR indexes discover every protected-main ADR and match its lifecycle metadata;
4. UML covers every currently implemented material authority flow, with planned diagrams clearly marked;
5. ERD/domain models accurately distinguish conceptual, in-memory, persisted, adapter-owned and external entities;
6. traceability maps every material requirement/Accepted decision to protected-main evidence, explicitly active-PR evidence, or an open issue;
7. documentation tests catch stale status/index/link/ownership/identifier terminology;
8. security, test, operability, data-governance and release docs agree on shipped-vs-planned boundaries; and
9. this documentation reconciliation itself reaches protected main through live repository governance.

Until then, OriginWeave is **design-documented but not protected-main documentation-closed**. That finding must never be used as an excuse to stop unrelated safe implementation work.
