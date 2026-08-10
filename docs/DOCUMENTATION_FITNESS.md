# OriginWeave Documentation Fitness Assessment

- **Assessment date:** 2026-08-10
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
| PRD | **REPAIRED IN THIS CHANGE** | HTTP names active replacement PR #37 while retaining `Planned` protected-main status; historical PR #11 is predecessor lineage. Purpose-bound sensitive disclosure is an Implemented protected-main policy kernel while broker/storage/lifecycle remain Planned under issue #10; active PR #45 adds credential-free lifecycle evidence and active PR #46 adds bounded in-process authoritative use reservation without turning the broker runtime into shipped behavior. MV3 remains a Planned complete compatibility program while protected-main and active-PR evidence are separately identified. |
| TRD | **REPAIRED IN THIS CHANGE** | Implementation inventory separates protected-main status from active/non-shipped evidence instead of composite labels. Session/node, route, sensitive-data, HTTP, proxy/PAC, MV3 and broker boundaries remain reconciled to protected-main truth; #45/#46 are active partial broker-lifecycle evidence only. |
| Root Architecture | **PRESENT-CURRENT with follow-up** | Correct Chromium compatibility-kernel + Rust control-plane direction, explicit authority stack and protected-main truth rule. No topology change is required for #45/#46 because they implement already-governed ADR 0007 evidence/policy primitives rather than introducing broker persistence or a new service boundary. Reconcile implementation-facing details after #37/#40 or the real Chromium vertical slice integrate. |
| ADR index/lifecycle | **REPAIRED IN THIS CHANGE** | Indexes Accepted ADRs 0001-0008 and 0010 plus Proposed 0009, 0013, 0014 and 0100-0109 without promoting Proposed decisions. Identifier allocation is treated as a cross-main-and-active-work reservation problem. No new ADR is required for #45/#46 because both stay within Accepted ADR 0007's existing sensitive-data authority/broker-lifecycle direction. |
| Individual ADRs | **PARTIAL BY LIFECYCLE** | Accepted ADRs remain governing design authority. Proposed ADR 0013 covers MV3 compatibility vs extension-to-Agent authority; Proposed ADR 0014 covers ADR acceptance governance. Their presence does not imply Acceptance. HTTP ADRs in PR #37 remain active-PR evidence. ADR 0007 already specifies caller-unforgeable state and atomic broker reservation; #45/#46 are partial implementation evidence, not a new architecture decision. |
| UML / control-flow diagrams | **PRESENT-CURRENT with follow-up** | Component, network authority, observation/action, delegated-task state, deployment, evidence, secret-fill, approval, resource-pressure/GPU fallback and hourly automation flows already exist. This PR adds `uml/extension-authority.md`. No new UML is justified for #45/#46 until a real trusted broker/storage/fill runtime changes the deployed component or transaction boundary. A detailed real-Chromium vertical-slice sequence is deferred until issue #28 contracts stabilize. |
| Conceptual ERD/domain model | **PRESENT-CURRENT** | Explicitly conceptual unless an adapter/schema is separately implemented; distinguishes current value/evidence concepts from planned durable records and adapter-owned representations. #45/#46 introduce no physical persistence, so adding tables merely to represent active in-memory primitives would be false architecture. |
| Traceability | **REPAIRED IN THIS CHANGE** | Separates `IMPLEMENTED_ON_PROTECTED_MAIN`, `IMPLEMENTED_ON_ACTIVE_PR`, `PARTIAL`, `ACCEPTED_ARCHITECTURE`, `PLANNED`, `RESEARCH_ONLY`, `SUPERSEDED`, and `OUT_OF_SCOPE`; #37/#40/#43/#45/#46 are active evidence only and cannot be promoted to protected-main truth. |
| Threat model / Security | **PRESENT-CURRENT with follow-up** | Covers major untrusted-content, network, secret, provenance and extension risks. ADR 0007 already identifies stale caller-count/concurrent replay risk; #46 narrows that risk only within one in-process mutable state object and does not claim cross-process atomicity/revocation. Update when a real broker/storage/runtime attack surface integrates. |
| Test strategy / quality gates | **PRESENT-CURRENT** | Exact owned-code coverage, rustdoc and realistic boundary testing are explicit. #46 follows real RED at the unresolved production API, then exact-head workspace/test/Clippy/rustdoc and 100% function/line/region/branch coverage. Real browser and MV3 evidence remain pinned-browser executable evidence rather than source-text claims. |
| Operability / incident response | **PRESENT-CURRENT with follow-up** | Failure, readiness, quarantine and recovery concepts exist. #45/#46 intentionally add no durable broker operation, so no new runbook/SLO/RPO/RTO is fabricated. Protected-main runtime/scheduled evidence remains required for operational closure where applicable. |
| API / protocol contracts | **PRESENT-CURRENT as target contracts** | OriginWeave Protocol and adapter boundaries are documented; much browser adapter implementation remains Planned. External protocol identifiers are never durable authority by themselves. #46's Rust policy API is internal reusable control-plane state, not a new external wire/schema contract. |
| Release / rollback / provenance | **PRESENT-CURRENT** | Feature-branch green checks cannot become release readiness. Release remains bound to one exact integrated protected head and applicable CI/security/coverage/package/provenance/recovery/compatibility/review evidence. |
| Data governance / privacy | **PRESENT-CURRENT architecture / PARTIAL runtime** | Purpose-bound policy/evidence foundations exist. Active PR #45 adds credential-free handle lifecycle evidence; active PR #46 adds a bounded in-process reservation counter that removes caller-supplied prior-use state from the reservation operation. Trusted broker storage, cross-process transactionality, revocation, protected-value resolution/fill, compensation, encryption/KMS and model-disclosure lifecycle remain open under issue #10. |
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

Protected main has executable pinned-Chromium evidence for service worker, content script, storage, DNR, tabs, windows, scripting, commands, side panel, bookmarks, history, restart persistence and repeatability. Active PR #43 adds real bounded downloads evidence. Issue #27 remains open for the complete compatibility/release matrix and additional managed/native-messaging boundaries.

**Resolved on this documentation branch:** PRD/TRD/traceability represent complete compatibility as Planned while separately naming partial protected-main evidence and active-PR downloads evidence. Proposed ADR 0013 separates compatibility evidence from Agent-authority evidence.

### 3.4 Browser identifier authority

Protected main contains session/context/document/node authority foundations under Accepted ADR 0010. Active PR #40 owns a bounded registry mapping protocol-local identifiers into that authority model.

**Resolved on this documentation branch:** PRD/TRD/traceability identify the protected-main core foundation separately from #40 active/non-shipped registry evidence. Detailed adapter-sequence UML is deliberately deferred until its executable contracts stabilize.

### 3.5 ADR discoverability and identifier allocation

The prior index omitted Accepted ADRs 0007, 0008 and 0010 and Proposed ADR 0009. During this reconciliation, candidate ADR numbers 0011/0012 were also found to be reserved by active PR #37.

**Resolved on this documentation branch:** lifecycle indexes are complete for the branch, extension/governance decisions use non-colliding 0013/0014, and repository-scoped collision-sensitive identifiers are reserved across protected main plus active work rather than allocated from main alone.

### 3.6 Documentation contract parser

The first fitness contract accepted only bare lifecycle metadata, while Accepted ADR 0007 legitimately contains a descriptive suffix after `Accepted`.

**Resolved on this documentation branch:** the parser reads the leading supported lifecycle token and accepts repository-valid descriptive suffixes without accepting unknown lifecycle states.

### 3.7 Initial UML audit false positive

The first audit incorrectly called resource-pressure and hourly automation views missing. Protected-main UML already contained them.

**Resolved on this documentation branch:** the assessment recognizes those views and adds only the genuinely missing extension-permission-to-Agent-authority diagram.

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

### 5.2 Real Chromium vertical slice — deferred until issue #28 stabilizes

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

### 5.3 Trusted sensitive-data broker — deferred until issue #10 owns a real runtime boundary

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
- active-sensitive-data evidence reconciliation without promoting #45/#46 to shipped or inventing broker persistence.

### Still required

- exact-head CI/security/review acceptance of this documentation PR;
- integration before any of these branch repairs become protected-main truth;
- re-reconciliation after active PR #37, #40, #43, #45 or #46 integrates, because active-PR status must then move to protected-main evidence;
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
