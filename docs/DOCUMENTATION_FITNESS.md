# OriginWeave Documentation Fitness Assessment

- **Assessment date:** 2026-08-11
- **Assessment scope:** protected `main`, every current OriginWeave implementation lane relevant to canonical product truth, and durable product decisions that must be reconstructable without chat history
- **Assessment type:** semantic fitness, not file-presence inventory
- **Current verdict:** **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**

## 1. Verdict

**DESIGN-SUFFICIENT** means the repository has a coherent product, technical, architecture, decision, diagram, data-model, security, testing, operability, protocol and release graph sufficient to implement and review OriginWeave without reconstructing product intent from chat history.

**PROTECTED-MAIN-PARTIAL** now means something narrower than it did at the beginning of this reconciliation: this PR has repaired the known PRD/TRD/traceability/index semantic drift on its own branch, but protected `main` does not receive those repairs until this documentation line passes exact-head governance and integrates. Active feature PRs also remain non-shipped evidence. The branch therefore must not relabel protected main as documentation-closed before integration.

File existence alone is never sufficient. An artifact can exist and still be stale, contradictory, overclaiming, underclaiming, or disconnected from executable evidence.

## 2. Fitness matrix

| Documentation family | Fitness | Current evidence / remaining boundary |
|---|---|---|
| PRD | **REPAIRED IN THIS CHANGE with active-lane follow-up** | HTTP names active replacement PR #37 while retaining `Planned` protected-main status; historical PR #11 is predecessor lineage. Purpose-bound sensitive disclosure is an Implemented protected-main policy kernel while broker/storage/lifecycle remain Planned under issue #10; active PRs #45/#46 add bounded non-shipped lifecycle/use-state evidence. Resolution freshness is active in #47 with first-party network consumption stacked in #50; neither is protected-main truth. Browser task telemetry in #51 is an active value-object prerequisite, not shipped sampling. MV3 remains a Planned complete compatibility program while protected-main and active-PR evidence are separately identified. |
| TRD | **REPAIRED IN THIS CHANGE with active-lane follow-up** | Implementation inventory separates protected-main status from active/non-shipped evidence instead of composite labels. Session/node, route, sensitive-data, HTTP, proxy/PAC, MV3 and broker boundaries remain reconciled to protected-main truth. #47/#50 tighten the resolution-to-socket freshness boundary only on active work; #48 is a revocation-material freshness primitive without any protected-main revocation claim; #51 is active telemetry structure without OS/Chromium sampling. |
| Root Architecture | **PRESENT-CURRENT with follow-up** | Correct Chromium compatibility-kernel + Rust control-plane direction, explicit authority stack and protected-main truth rule. #47/#50 tighten the already-governed ADR 0004 destination/rebinding authority rather than creating a new service/topology. #45/#46 remain within ADR 0007. #48 adds no revocation fetch/cache/service boundary and #51 adds no sampler/runtime component. Reconcile implementation-facing details after these lanes integrate or the real Chromium vertical slice stabilizes. |
| ADR index/lifecycle | **REPAIRED IN THIS CHANGE** | Indexes Accepted ADRs 0001-0008 and 0010 plus Proposed 0009, 0013, 0014 and 0100-0109 without promoting Proposed decisions. Identifier allocation is treated as a cross-main-and-active-work reservation problem. No new ADR is justified solely by #45/#46, #47/#50, #48 or #51 because those lanes remain inside existing authority decisions and introduce no new durable service, trust domain, persistence owner or externally versioned protocol. |
| Individual ADRs | **PARTIAL BY LIFECYCLE** | Accepted ADRs remain governing design authority. Proposed ADR 0013 covers MV3 compatibility vs extension-to-Agent authority; Proposed ADR 0014 covers ADR acceptance governance. Their presence does not imply Acceptance. HTTP ADRs in PR #37 remain active-PR evidence. #47/#50 are implementation evidence under ADR 0004, #45/#46 under ADR 0007, and #48/#51 do not by themselves create a new accepted architecture boundary. |
| UML / control-flow diagrams | **PRESENT-CURRENT with follow-up** | Component, network authority, observation/action, delegated-task state, deployment, evidence, secret-fill, approval, resource-pressure/GPU fallback and hourly automation flows already exist. This PR adds `uml/extension-authority.md`. The network-authority sequence should be reconciled once #50's executable consumer signature is stable because the sequence changes from untimed resolution to fresh resolution authority plus trusted monotonic use time; that does not require a fictitious new component. No new UML is justified for #45/#46/#48/#51 until a real runtime/service/transaction boundary appears. A detailed real-Chromium vertical-slice sequence remains deferred until issue #28 contracts stabilize. |
| Conceptual ERD/domain model | **PRESENT-CURRENT** | Explicitly conceptual unless an adapter/schema is separately implemented; distinguishes current value/evidence concepts from planned durable records and adapter-owned representations. #45/#46/#47/#48/#50/#51 introduce no physical persistence owner, so adding tables merely to represent active in-memory/value primitives would be false architecture. |
| Traceability | **REPAIRED IN THIS CHANGE with volatile evidence refresh required** | Separates `IMPLEMENTED_ON_PROTECTED_MAIN`, `IMPLEMENTED_ON_ACTIVE_PR`, `PARTIAL`, `ACCEPTED_ARCHITECTURE`, `PLANNED`, `RESEARCH_ONLY`, `SUPERSEDED`, and `OUT_OF_SCOPE`. Current active evidence includes #37/#40/#43/#45/#46/#47/#48/#49/#50/#51 and cannot be promoted to protected-main truth merely because an individual PR is green. |
| Threat model / Security | **PRESENT-CURRENT with follow-up** | Covers major untrusted-content, network, secret, provenance and extension risks. #47/#50 narrow the DNS-rebinding/TOCTOU interval only when one fresh authority chain reaches socket planning; #48 proves only signed-window freshness for already-verified revocation material and does not prove an unrevoked certificate; #46 narrows stale-count/replay risk only within one in-process mutable state object. Update threat/runtime claims only after the corresponding active paths integrate. |
| Test strategy / quality gates | **PRESENT-CURRENT** | Exact owned-code function/line/region/branch coverage, rustdoc and realistic boundary testing are explicit. Active lanes use intentional RED boundaries followed by narrow production changes and exact-head proof. Real browser and MV3 evidence remain pinned-browser executable evidence rather than source-text claims, and pending/failed predecessor evidence never transfers to a later head. |
| Operability / incident response | **PRESENT-CURRENT with follow-up** | Failure, readiness, quarantine and recovery concepts exist. #45/#46/#47/#48/#51 intentionally add no new durable service or daemon, so no runbook/SLO/RPO/RTO is fabricated. #50 changes first-party socket authority composition but not deployment topology. Protected-main runtime/scheduled evidence remains required for operational closure where applicable. |
| API / protocol contracts | **PRESENT-CURRENT as target contracts** | OriginWeave Protocol and adapter boundaries are documented; much browser adapter implementation remains Planned. External protocol identifiers are never durable authority by themselves. On active #50, the ordinary public untimed network planner is being structurally removed so first-party consumers must cross fresh-resolution authorization; until integration that is active-PR evidence only. #46/#51 are internal reusable control-plane/value APIs, not new external wire/schema contracts. |
| Release / rollback / provenance | **PRESENT-CURRENT** | Feature-branch green checks cannot become release readiness. Release remains bound to one exact integrated protected head and applicable CI/security/coverage/package/provenance/recovery/compatibility/review evidence. Active stacks such as #47→#50 and #43→#49 must preserve dependency order; predecessor-head success cannot satisfy a later head. |
| Data governance / privacy | **PRESENT-CURRENT architecture / PARTIAL runtime** | Purpose-bound policy/evidence foundations exist. Active PR #45 adds credential-free handle lifecycle evidence; active PR #46 adds a bounded in-process reservation counter that removes caller-supplied prior-use state from the reservation operation. #47/#48/#50/#51 add timestamps/measurements or authority metadata without protected values. Trusted broker storage, cross-process transactionality, revocation, protected-value resolution/fill, compensation, encryption/KMS and model-disclosure lifecycle remain open under issue #10. |
| Standards / doctoring | **PRESENT-CURRENT with continuous watch** | Primary browser/protocol/standards evidence and APA 7 references are kept in doctoring documents with draft/experimental status distinguished from final normative standards. |

## 3. Reconciliation findings and resolution state

### 3.1 HTTP lineage

Protected-main PRD previously named historical PR #11 as active HTTP evidence. Current executable replacement work is PR #37, while protected main still does not ship bounded HTTP semantics.

**Resolved on this documentation branch:** PRD, TRD and traceability name #37 only as active/non-shipped evidence, retain protected-main `Planned`, and treat #11 as predecessor lineage. Old-head checks/reviews do not transfer.

### 3.2 Sensitive-data authority and broker lifecycle

Protected main contains a purpose-bound sensitive-data policy/evidence foundation governed by Accepted ADR 0007, while the complete trusted broker remains unimplemented.

Two newer active lanes provide narrower non-shipped evidence:

- PR #45 records credential-free handle lifecycle evidence in `originweave-evidence` without storing protected values; and
- PR #46 adds `SensitiveHandleUseState`, an in-process authoritative reservation count in `originweave-policy` that increments only after existing exact-scope/classification/expiry/use-limit admission and does not trust a reservation caller to supply the prior-use count.

PR #46's mutable-borrow serialization is deliberately **not** described as durable or cross-process atomic broker enforcement. It stores neither the opaque token nor protected data and does not implement revocation, transactionally durable reservation, value resolution/fill, compensation, encryption/KMS, retention, or model/provider/region disclosure policy.

**Resolved on this documentation branch:** the design verdict remains unchanged. PRD/TRD/traceability may identify #45/#46 as `IMPLEMENTED_ON_ACTIVE_PR`/partial evidence while the complete broker/runtime stays `Planned` under issue #10. Root Architecture, UML and ERD do not gain fictitious service/database boundaries before those boundaries actually exist.

### 3.3 Manifest V3 compatibility

Protected main has executable pinned-Chromium evidence for service worker, content script, storage, DNR, tabs, windows, scripting, commands, side panel, bookmarks, history, restart persistence and repeatability. Active PR #43 adds real bounded downloads evidence; stacked PR #49 adds a regression for per-trial ephemeral profile isolation. Issue #27 remains open for the complete compatibility/release matrix and additional managed/native-messaging boundaries.

**Resolved on this documentation branch:** PRD/TRD/traceability represent complete compatibility as Planned while separately naming partial protected-main evidence and active-PR evidence. Proposed ADR 0013 separates compatibility evidence from Agent-authority evidence. #49 must not be represented as protected-main behavior while its #43 prerequisite remains active.

### 3.4 Browser identifier authority

Protected main contains session/context/document/node authority foundations under Accepted ADR 0010. Active PR #40 exact head `9e635e80e9813a1d2a9c408155d52221b76eeed3` owns a bounded registry mapping protocol-local identifiers into that authority model and is active/non-shipped evidence.

**Resolved on this documentation branch:** PRD/TRD/traceability identify the protected-main core foundation separately from #40 active registry evidence. Detailed adapter-sequence UML remains deferred until executable browser adapter contracts stabilize.

### 3.5 ADR discoverability and identifier allocation

The prior index omitted Accepted ADRs 0007, 0008 and 0010 and Proposed ADR 0009. During this reconciliation, candidate ADR numbers 0011/0012 were also found to be reserved by active PR #37.

**Resolved on this documentation branch:** lifecycle indexes are complete for the branch, extension/governance decisions use non-colliding 0013/0014, and repository-scoped collision-sensitive identifiers are reserved across protected main plus active work rather than allocated from main alone.

### 3.6 Documentation contract parser

The first fitness contract accepted only bare lifecycle metadata, while Accepted ADR 0007 legitimately contains a descriptive suffix after `Accepted`.

**Resolved on this documentation branch:** the parser reads the leading supported lifecycle token and accepts repository-valid descriptive suffixes without accepting unknown lifecycle states.

### 3.7 Initial UML audit false positive

The first audit incorrectly called resource-pressure and hourly automation views missing. Protected-main UML already contained them.

**Resolved on this documentation branch:** the assessment recognizes those views and adds only the genuinely missing extension-permission-to-Agent-authority diagram.

### 3.8 Resolution freshness authority

Protected main already validates, pins and non-expansively revalidates destination addresses, but it does not yet require a bounded approval-to-socket-use interval. Active PR #47 exact head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` implements the reusable `FreshResolutionSnapshot` primitive with a repository-owned 30-second maximum validity budget and exact-head gate evidence.

Stacked PR #50 owns first-party network consumption. During this reconciliation run, the parallel-wrapper design was rejected as insufficient because public `ConnectionPlan::new(&ResolutionSnapshot, ...)` remained an untimed bypass. The active branch now structurally hides that untimed planner and is migrating first-party TLS integration helpers to `FreshConnectionPlan` with explicit trusted monotonic approval/use times. This remains **active-PR / PARTIAL** evidence until the exact consumer head is gate-clean and both prerequisite and consumer integrate in dependency order.

**Documentation consequence:** no new ADR, physical ERD entity or deployed component is justified. The change tightens Accepted ADR 0004. The network-authority sequence should be reconciled after the #50 public consumer contract stabilizes so the diagram explicitly shows fresh resolution authority plus trusted monotonic use time before socket authority.

### 3.9 TLS revocation-material freshness

Protected main deliberately reports revocation as `NotConfigured`. Active PR #48 exact head `9bbe12860436027a3b7cd5786775f1dacfbc835d` adds a reusable signed-window freshness check for already-verified revocation material.

**Documentation consequence:** #48 may be represented as `IMPLEMENTED_ON_ACTIVE_PR` for the freshness primitive only. It must not be described as OCSP/CRL acquisition, signature/path validation, cache operation, or proof that a certificate is unrevoked. No new revocation service/cache topology or physical ERD is documented until such a runtime actually exists.

### 3.10 Browser task telemetry

Active PR #51 exact head `1c85b966087191f52b4a709a2822b2a53fb0e2fa` adds a bounded `BrowserTaskTelemetry` value object for RSS bytes, observation bytes, action latency and task duration. It does not sample Chromium or the operating system and does not create a GPU/local-model claim.

**Documentation consequence:** this is a reusable active-PR prerequisite for issue #28 resource evidence, not a new runtime component, persistence entity, sampler service or release metric. Architecture/UML/ERD remain unchanged until an actual measurement adapter establishes those boundaries.

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
14. WebDriver BiDi, CDP, WebMCP and MCP are versioned adapters, never the product's authority model by themselves.
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

## 5. Architecture views still legitimately deferred

### 5.1 Extension authority — present in this branch

`uml/extension-authority.md` shows:

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

### 5.2 Network freshness sequence — existing view requires bounded reconciliation after #50 stabilizes

The durable sequence must make the freshness boundary explicit without encoding temporary branch-only API names:

```text
resolver answer
-> destination policy + origin binding
-> fresh resolution approval [trusted monotonic interval]
-> connection authorization at trusted monotonic use time
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
-> opaque node authority
-> typed policy decision
-> real browser input
-> observed post-condition
-> credential-safe evidence
-> teardown/recovery
```

Do not freeze temporary protocol/field names into authoritative UML before the executable contracts stabilize.

### 5.4 Trusted sensitive-data broker — deferred until issue #10 owns a real runtime boundary

Current protected-main policy/evidence plus active #45/#46 primitives do not justify inventing a broker process, database table, transaction manager, KMS path or browser-fill adapter in timeless Architecture/UML/ERD. When a real broker slice exists, the documentation graph must add the actual component/transaction/data-lifecycle views and mark persisted versus in-memory versus external state from executable evidence.

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
- current browser/protocol standards doctoring;
- active-sensitive-data evidence reconciliation without promoting #45/#46 to shipped or inventing broker persistence;
- explicit freshness/revocation/telemetry maturity boundaries for active #47/#48/#50/#51 without inventing new architecture or persistence.

### Still required

- exact-head CI/security/review acceptance of this documentation PR;
- integration before any of these branch repairs become protected-main truth;
- re-reconciliation whenever active PR #37, #40, #43, #45, #46, #47, #48, #49, #50 or #51 integrates, closes, is superseded or materially changes head, because active-PR status must then move or be removed rather than silently becoming protected-main truth;
- network-authority sequence reconciliation after #50's executable consumer contract is stable and proven;
- detailed real-Chromium vertical-slice UML when issue #28 implementation contracts are stable;
- trusted-broker UML/ERD/operability additions only when issue #10 establishes real runtime/persistence ownership;
- future ERD changes only when persistence ownership/entities actually change;
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
