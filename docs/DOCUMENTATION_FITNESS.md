# OriginWeave Documentation Fitness Assessment

- **Assessment date:** 2026-08-10
- **Assessment scope:** protected `main`, current open OriginWeave work, and durable product decisions that must be reconstructable without chat history
- **Assessment type:** semantic fitness, not file-presence inventory
- **Current verdict:** **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**

## 1. Meaning of the verdict

**DESIGN-SUFFICIENT** means the repository contains enough coherent product, technical, architecture, decision, diagram, data-model, security, testing, operability, protocol and release material to implement and review OriginWeave without reconstructing the original design from conversation history.

**PROTECTED-MAIN-PARTIAL** means the canonical graph still contains stale implementation-status references and incomplete current-state reconciliation. The documentation set is broad enough, but PRD/TRD and some implementation-facing views lag protected-main code or current active replacement work. Therefore documentation is not yet a release-quality source of current implementation truth.

File existence alone is never sufficient. A document can be present and still be stale, contradictory, overclaiming, underclaiming, or disconnected from code and evidence.

## 2. Fitness matrix

| Documentation family | Fitness | Evidence / current gap |
|---|---|---|
| PRD | **PARTIAL** | Strong whole-product requirements, modes, product family and buyer outcomes. Some implementation notes are stale: HTTP still names historical PR #11 rather than current replacement PR #37; sensitive-data policy text still references already-integrated work; real pinned-Chromium MV3 evidence is underrepresented. |
| TRD | **PARTIAL** | Strong authority stack, lifecycle, action, observation, network, secret and resource contracts. Current implementation inventory lags protected-main additions and still uses composite implementation language in places where protected-main, active-PR and planned maturity should be separated. |
| Root Architecture | **PRESENT-CURRENT with follow-up** | Correct Chromium-compatibility-kernel + Rust-control-plane direction and explicit authority layers. Reconcile again when the browser registry, HTTP replacement and Chromium vertical slice integrate. |
| ADR index/lifecycle | **REPAIRED IN THIS CHANGE** | The previous index omitted Accepted ADRs 0007, 0008 and 0010, omitted Proposed ADR 0009, and retained change-local wording. This branch repairs discoverability/status categories and adds Proposed ADRs 0013/0014 without promoting them to Accepted. |
| Individual ADRs | **PARTIAL** | Accepted decisions 0001-0008 and 0010 remain design authority. Proposed 0009, 0013, 0014 and 0100-0109 remain explicitly non-binding. HTTP feature ADRs remain active-PR evidence until PR #37 integrates. Extension/MV3 authority now has Proposed ADR 0013; issue #27 cannot close merely because the proposal exists. |
| UML / control-flow diagrams | **PRESENT-CURRENT with follow-up** | Product-wide component, network authority, observation/action, delegated-task state, deployment, evidence, secret-fill, approval, resource-pressure/GPU fallback and hourly automation flows already exist. This branch adds [`uml/extension-authority.md`](uml/extension-authority.md), separating Chromium MV3 permission from OriginWeave Agent capability. The real Chromium vertical-slice sequence remains incomplete until issue #28 stabilizes. |
| Conceptual ERD/domain model | **PRESENT-CURRENT with follow-up** | Correctly distinguishes conceptual persistence and includes session/context, action/policy/approval, network/TLS/HTTP, sensitive authority, resources, provenance, downloads and extension grants. Update only when persistence ownership or durable entities actually change; do not invent a database to increase diagram count. |
| Traceability | **REPAIRED IN THIS CHANGE / PARTIAL BY DESIGN** | This branch now separates `IMPLEMENTED_ON_PROTECTED_MAIN`, `IMPLEMENTED_ON_ACTIVE_PR`, `PARTIAL`, accepted architecture and planned work. It records #37 as active HTTP evidence, protected-main node authority plus #40 registry work, partial protected-main MV3 evidence plus #43 downloads, and protected-main sensitive-data foundations plus issue #10 broker lifecycle. PRD/TRD still need the same reconciliation. |
| Threat model / Security | **PRESENT-CURRENT with follow-up** | Covers major untrusted-content, secret, network, provenance and extension risks. Continue adding executable mitigations when HTTP/browser/runtime boundaries integrate. |
| Test strategy / quality gates | **PRESENT-CURRENT** | Exact owned-code coverage, rustdoc and realistic boundary testing are explicit. Real Chromium/MV3 evidence must remain release-bound to pinned browser evidence rather than source-text assertions. |
| Operability / incident response | **PRESENT-CURRENT with follow-up** | Failure, readiness, quarantine and recovery concepts exist. Protected-main evidence for model-backed hourly development remains an operational-closure requirement rather than a documentation-only claim. |
| API / protocol contract | **PRESENT-CURRENT as target contract** | Typed OriginWeave Protocol boundaries are documented but much browser adapter implementation remains Planned. Keep adapter identifiers non-authoritative and versioned. |
| Release / rollback / provenance | **PRESENT-CURRENT** | Correctly prevents feature-level green checks from becoming release readiness. Formal release remains blocked by missing full browser/runtime product evidence. |
| Data governance / PII | **PRESENT-CURRENT as architecture; PARTIAL implementation** | Correctly rejects blanket masking and ambient raw propagation in favor of purpose-bound authorization, opaque handles, encryption, retention and audit. Trusted broker/storage/revocation/lifecycle completion remains open under issue #10. |
| Standards / doctoring | **PRESENT-CURRENT with continuous watch** | Primary standards and APA 7 doctoring exist in [`doctoring.md`](doctoring.md) and [`doctoring/browser-agent-protocols.md`](doctoring/browser-agent-protocols.md). Experimental/draft browser interfaces remain explicitly separated from final normative standards. |

### 2.1 Primary-source standards evidence used by this assessment

The protocol names in this assessment are grounded in repository doctoring and primary sources:

| Boundary | Primary evidence | Repository evidence rule |
|---|---|---|
| WebDriver BiDi | [W3C WebDriver BiDi, 1 June 2026 Working Draft](https://www.w3.org/TR/2026/WD-webdriver-bidi-20260601/) | Doctoring records Working Draft status, so it remains adapter-bound. |
| Manifest V3 | [Chrome manifest format](https://developer.chrome.com/docs/extensions/reference/manifest) and [Manifest Version](https://developer.chrome.com/docs/extensions/reference/manifest/manifest-version) | MV3 is the current Chrome extension baseline without a claim of universal Chrome/Web Store/Google-service compatibility. |
| Chrome DevTools Protocol | [Official CDP tip-of-tree documentation](https://chromedevtools.github.io/devtools-protocol/tot/) | Tip-of-tree changes frequently and lacks backwards-compatibility guarantee; OriginWeave pins/versions adapters. |
| WebMCP | [Chrome WebMCP](https://developer.chrome.com/docs/ai/webmcp), [WebMCP tool security](https://developer.chrome.com/docs/ai/webmcp/secure-tools), and [agent security considerations](https://developer.chrome.com/docs/agents/security) | Doctoring records experimental/origin-trial status and untrusted-content/prompt-injection boundaries. |
| Model Context Protocol | [MCP 2026-07-28 specification](https://modelcontextprotocol.io/specification/2026-07-28) and [official release announcement](https://blog.modelcontextprotocol.io/posts/2026-07-28/) | Durable browser state remains in OriginWeave-level handles; MCP is a high-level adapter, not Chromium authority. |
| W3C PROV-O | [PROV-O Recommendation](https://www.w3.org/TR/prov-o/) | Provenance interoperability adapter, not authorization. |
| WARC | [ISO 28500:2017](https://www.iso.org/standard/68004.html) | Evidence/payload preservation format, not a truth or permission escalation mechanism. |

APA 7th references are recorded in doctoring documents rather than duplicated into every architecture assessment.

## 3. Concrete stale/current discrepancies

### 3.1 Historical HTTP PR still appears as active PRD evidence

Protected-main PRD describes bounded HTTP semantics as Planned with `Active PR #11`. PR #11 is historical predecessor lineage; current executable replacement work is PR #37. Active PR #37 itself is not protected-main implementation truth.

**Current reconciliation:** traceability now records HTTP as `IMPLEMENTED_ON_ACTIVE_PR` under #37 and treats #11 as predecessor/superseded implementation lineage. PRD/TRD still require the equivalent protected-main-vs-active-PR wording repair. Never transfer predecessor checks or reviews.

### 3.2 Sensitive-data implementation status lags protected main

Protected main has purpose-bound sensitive-disclosure policy/evidence foundations, while PRD/TRD retain stale active-PR language. The trusted broker, storage, selective model disclosure, revocation and lifecycle remain open.

**Current reconciliation:** traceability now records **PARTIAL**: protected-main policy/evidence foundation + planned broker/runtime under issue #10. PRD/TRD remain to reconcile.

### 3.3 MV3 compatibility evidence moved beyond original roadmap wording

Protected main has executable pinned-Chromium MV3 evidence for service worker, content script, storage, DNR, tabs, windows, scripting, commands, side panel, bookmarks, history, restart and repeatability. Active PR #43 adds real bounded downloads evidence and now preserves only allowlisted download-stage diagnostics in runner failure evidence. Issue #27 remains open because the complete compatibility matrix, native-messaging/enterprise-policy boundaries and release integration are unfinished.

**Current reconciliation:** traceability records protected-main compatibility as **PARTIAL** and #43 as active-PR evidence. PRD/TRD must adopt the same distinction.

### 3.4 Browser authority is transitioning from protected-main value types to an adapter registry

Protected main contains session/context/document/node authority foundations governed by Accepted ADR 0010. PR #40 is adding a bounded session-scoped registry so raw BiDi/CDP identifiers do not become durable OriginWeave authority.

**Current reconciliation:** traceability records the foundation as protected-main and #40 as active non-shipped evidence. Architecture/ERD remain directionally correct; detailed adapter UML should wait for stable protocol contracts rather than encode temporary field names.

### 3.5 ADR lifecycle discoverability drifted

The prior ADR index omitted Accepted ADRs 0007, 0008 and 0010 and Proposed ADR 0009. This branch repairs the index, adds Proposed ADR 0013 for MV3/extension authority and Proposed ADR 0014 for ADR acceptance governance, and preserves their non-Accepted status.

### 3.6 Documentation-contract parser overfit one metadata spelling

The first documentation-fitness contract accepted only bare lifecycle tokens, but existing Accepted ADR 0007 legitimately uses `- Status: Accepted for the first authority kernel`. CI correctly exposed the mismatch. This branch now parses the leading lifecycle token while retaining exact supported lifecycle validation.

### 3.7 UML assessment itself was initially stale

The first audit called resource-pressure and hourly automation flows missing. Protected-main `docs/uml/README.md` already contains both. The branch corrects the audit and adds only the genuinely missing extension-permission-to-Agent-authority view.

## 4. Durable conversation decisions that must remain represented

1. **OriginWeave — Browse. Act. Prove.** is an enterprise agentic web runtime/provenance-native browser platform, not merely a Selenium-style automation library.
2. Chromium remains the compatibility kernel; OriginWeave does not reimplement Blink or V8 for product differentiation.
3. Rust owns new authority-bearing control-plane semantics and remains independently reusable in headless/MSA composition.
4. Human, Assist, Agent Task and Crawler modes have distinct profile/authority semantics.
5. Agent Task Mode must not ambiently inherit a normal human browser profile.
6. Page, extension and WebMCP content are untrusted observations, never policy or goal authority.
7. Structured observation precedes raw HTML and screenshot-only interpretation.
8. Typed actions and observed post-conditions replace arbitrary JavaScript and command-return-as-success.
9. Logical origin, destination, route/proxy, TCP peer, TLS identity and HTTP semantics are separate authorities.
10. Session/context/document epoch/node identity is separate from raw BiDi/CDP identifiers.
11. Extension permissions are not Agent capabilities; Manifest V3 compatibility and Agent authority isolation are tested separately.
12. Raw secrets stay out of model-visible context; sensitive values use purpose-bound disclosure and opaque handles wherever possible.
13. Browser correctness and human interaction outrank optional local-model throughput under resource pressure.
14. Provenance separates source observation, model judgement, policy decision, approval, action and verified outcome; WARC/PROV remain adapters, not collapsed truth.
15. WebDriver BiDi, CDP, WebMCP and MCP remain versioned adapters; no experimental protocol becomes OriginWeave authority by itself.
16. The first product proof is a pinned-stock-Chromium vertical slice before a large Chromium fork.
17. High-risk actions remain approval-bound; Crawler Mode is read-only and does not include CAPTCHA/block-evasion features.
18. Autonomous development uses NVIDIA NIM/OpenCode with deterministic gates and reviewer/publication authority separation; it does not use `COPILOT_GITHUB_TOKEN` as the development-model credential.
19. Documentation, checks, reviews and operational evidence are separate authorities. A green sub-check, model verdict, active PR, chat decision or ADR never silently upgrades missing implementation to shipped behavior.
20. Work-conserving autonomous maintenance continues to another safe lane instead of ending on one merge, one document, one RCA, one queued check or one external approval gap.
21. Repository-scoped collision-sensitive identifiers such as ADR numbers must be reserved across protected main **and active PRs**, not allocated from protected main alone.

## 5. Architecture views requiring follow-through

### 5.1 Extension authority and compatibility sequence — added in this branch

[`uml/extension-authority.md`](uml/extension-authority.md) separates:

```text
Chromium MV3 permission
-> extension runtime
-> untrusted extension observation/message
-> OriginWeave extension policy/grant
-> Agent capability decision
-> typed action proposal
-> deterministic policy
```

It also separates **compatibility evidence** from **Agent-authority isolation evidence**: neither evidence class proves the other.

### 5.2 Resource-pressure state/sequence — already present

Protected-main `docs/uml/README.md` already models browser/model resource pressure and fallback. Refine it only when platform telemetry/admission implementation changes.

### 5.3 Hourly autonomous-development authority flow — already present

Protected-main `docs/uml/README.md` models deterministic early gates, conditional model-credential use, pristine attempts, bounded validation, publication authority, protected merge and protected-main operational acceptance. Its implementation/evidence status must be reconciled against actual workflows rather than inferred from the diagram.

### 5.4 Real Chromium vertical slice — incomplete until issue #28 stabilizes

Once issue #28 contracts stabilize, diagram and trace:

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

## 6. Immediate repository actions

### Completed or materially advanced in this documentation line

- documentation fitness is discoverable from `docs/README.md`;
- ADR indexes are reconciled with protected-main lifecycle status;
- machine-checkable documentation fitness contracts exist and now tolerate repository-valid descriptive status suffixes;
- extension-permission-to-Agent-authority UML exists without duplicating resource/automation diagrams;
- Proposed ADRs 0013 and 0014 make MV3/extension authority and architecture-decision governance explicit without claiming Acceptance;
- traceability distinguishes protected-main, active-PR, partial and planned capability maturity;
- current active lanes #37, #40 and #43 are represented without promoting them to shipped truth.

### Execute next when the relevant file/branch lease is clear

- reconcile PRD HTTP, sensitive-data and MV3 implementation notes to the same maturity model without encoding unstable exact heads;
- reconcile TRD implementation inventory/status language to protected-main vs active-PR vs planned truth;
- after #43 reaches a stable exact head, resolve its addressed diagnostic review finding only after exact-head checks validate the fix;
- after #37/#40 stop moving, re-read their exact contracts before mutating overlapping documentation or source.

### Defer until executable contracts stabilize

- add detailed real-Chromium vertical-slice UML only when issue #28 adapter contracts are stable enough that the diagram will not encode temporary protocol/field names;
- promote Proposed ADRs only through explicit reviewed status changes; file presence or matching code never implies Acceptance.

## 7. Completion criteria for documentation fitness

The whole graph becomes **PROTECTED-MAIN-SUFFICIENT** only when:

1. PRD and TRD implementation inventories agree with current protected-main crates/APIs and executable browser/extension evidence;
2. no canonical document identifies a historical/superseded PR as current implementation evidence;
3. ADR indexes discover every protected-main ADR and status agrees with file metadata;
4. UML covers every current material authority flow, including extension/Agent isolation and the real Chromium vertical slice once implemented;
5. ERD/domain models distinguish conceptual, in-memory, persisted, adapter-owned and external entities accurately;
6. traceability maps every material requirement and Accepted decision to protected-main implementation/test/evidence, explicitly active-PR evidence, or an open issue;
7. machine-checkable documentation tests catch stale status/index/link/ownership/identifier terminology;
8. security, test, operability, data-governance and release docs agree with the same shipped-vs-planned boundary; and
9. exact-head checks/review/governance for this documentation reconciliation pass.

Until then, OriginWeave is **design-documented but not documentation-closed**. This is a product-quality finding, not a reason to stop unrelated safe implementation work.
