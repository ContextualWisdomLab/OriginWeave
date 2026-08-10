# OriginWeave Documentation Fitness Assessment

- **Assessment date:** 2026-08-10
- **Assessment scope:** protected `main`, current open OriginWeave work, and durable product decisions that must be reconstructable without chat history
- **Assessment type:** semantic fitness, not file-presence inventory
- **Current verdict:** **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**

## 1. Meaning of the verdict

**DESIGN-SUFFICIENT** means the repository now contains enough coherent product, technical, architecture, decision, diagram, data-model, security, testing, operability, protocol and release material to implement and review OriginWeave without reconstructing the original design from conversation history.

**PROTECTED-MAIN-PARTIAL** means the canonical graph still contains stale implementation-status references and incomplete current-state reconciliation. The documentation set is broad enough, but several documents lag protected-main code and active replacement work. Therefore documentation is not yet a release-quality source of current implementation truth.

File existence alone is never sufficient. A document can be present and still be stale, contradictory, overclaiming, underclaiming, or disconnected from code and evidence.

## 2. Fitness matrix

| Documentation family | Fitness | Evidence / current gap |
|---|---|---|
| PRD | **PARTIAL** | Strong whole-product requirements, modes, product family and buyer outcomes. Some implementation notes are stale: HTTP still points at historical PR #11 rather than the current replacement line; sensitive-data policy text still refers to work that has since integrated on protected main; recent real MV3 compatibility evidence is underrepresented. |
| TRD | **PARTIAL** | Strong authority stack, lifecycle, action, observation, network, secret and resource contracts. Current implementation inventory lags protected-main additions and uses composite phrases such as `Planned / active development` even though the document defines a single controlled status vocabulary. |
| Root Architecture | **PRESENT-CURRENT with follow-up** | Correct Chromium-compatibility-kernel + Rust-control-plane direction and explicit authority layers. Must continue to be reconciled when the browser registry, HTTP replacement and Chromium vertical slice integrate. |
| ADR index/lifecycle | **REPAIRED IN THIS CHANGE** | Previous index omitted Accepted ADRs 0007, 0008 and 0010, omitted Proposed ADR 0009, and described 0100-series ADRs as being `in this change` even after the documentation baseline reached protected main. This branch reconciles discoverability and status categories without promoting Proposed ADRs. |
| Individual ADRs | **PARTIAL** | Core Accepted decisions 0001-0008 and 0010 are durable. Proposed 0009 and 0100-0109 remain explicitly Proposed. HTTP feature ADRs remain active-PR evidence until the replacement merges. A dedicated accepted extension/MV3 authority decision is still required before closing the extension compatibility issue. |
| UML / control-flow diagrams | **PRESENT-CURRENT with follow-up** | The product-wide pack already contains component, network authority, observation/action, delegated-task state, deployment, evidence, secret-fill, approval, resource-pressure/GPU fallback and hourly deterministic-gate/model/publication flows. This branch adds [`uml/extension-authority.md`](uml/extension-authority.md) so Chromium MV3 permission and OriginWeave Agent capability cannot be visually conflated. The real Chromium vertical-slice sequence remains incomplete until issue #28 stabilizes. |
| Conceptual ERD/domain model | **PRESENT-CURRENT with follow-up** | Correctly distinguishes conceptual persistence and includes session/context, action/policy/approval, network/TLS/HTTP, sensitive authority, resources, provenance, downloads and extension grants. Must be updated only when persistence ownership or new durable entities actually change; do not invent a database merely to increase diagram count. |
| Traceability | **PARTIAL** | Requirement/decision/standard/module/test mapping exists but must be reconciled with protected-main MV3 evidence, the HTTP replacement, browser registry work and issue-driven buyer gaps. Active PRs must remain visibly distinct from protected-main implementation. |
| Threat model / Security | **PRESENT-CURRENT with follow-up** | Covers major untrusted-content, secret, network, provenance and extension risks. Continue adding executable mitigations when HTTP/browser/runtime boundaries integrate. |
| Test strategy / quality gates | **PRESENT-CURRENT** | Exact owned-code coverage, rustdoc and realistic boundary testing are explicit. Real Chromium and MV3 compatibility evidence is now growing and must remain release-bound to pinned browser evidence. |
| Operability / incident response | **PRESENT-CURRENT with follow-up** | Failure, readiness, quarantine and recovery concepts exist. Protected-main evidence for the hourly model-backed development path remains an operational closure requirement rather than a documentation-only claim. |
| API / protocol contract | **PRESENT-CURRENT as target contract** | The typed OriginWeave Protocol boundary is documented but much of the browser adapter implementation remains Planned. Keep adapter identifiers non-authoritative and versioned. |
| Release / rollback / provenance | **PRESENT-CURRENT** | Correctly prevents feature-level green checks from becoming release readiness. Formal release remains blocked by missing full browser/runtime product evidence. |
| Data governance / PII | **PRESENT-CURRENT as architecture; PARTIAL implementation** | Correctly rejects blanket masking and ambient raw propagation in favor of purpose-bound authorization, opaque handles, encryption, retention and audit. Trusted broker/storage/lifecycle completion remains open work. |
| Standards / doctoring | **PRESENT-CURRENT with continuous watch** | Primary standards and APA 7 doctoring exist. Experimental/draft browser interfaces must remain explicitly separated from final normative standards. |

## 3. Concrete stale/current discrepancies discovered

### 3.1 Historical HTTP PR is still named as active product evidence

Protected-main PRD currently describes bounded HTTP semantics as Planned with `Active PR #11`. PR #11 is historical and intentionally non-integration-ready; current executable replacement work is PR #37. Canonical requirements must not use the historical PR as current implementation evidence after replacement lineage is established.

**Required repair:** after the current HTTP replacement reaches a stable exact head or protected main, update PRD/TRD/traceability to point to the current lineage and then to protected-main implementation. Never transfer predecessor checks or reviews.

### 3.2 Sensitive-data implementation status lags protected main

Protected main has integrated purpose-bound sensitive disclosure foundations, while PRD/TRD still contain `active PR` language for the policy slice. The broader trusted broker, storage, selective model disclosure, revocation and lifecycle issue remains open, so the correct representation is **implemented policy foundation + planned broker/runtime**, not either `all shipped` or `all planned`.

### 3.3 MV3 compatibility evidence has moved beyond the original roadmap language

Protected main now has executable pinned-Chromium MV3 evidence covering restart persistence and additional core extension APIs, including bookmarks and history. Issue #27 remains open because the complete declared compatibility matrix, downloads/native-messaging/enterprise-policy boundaries and release integration are not finished.

**Required repair:** PRD/TRD/traceability should say **partial protected-main compatibility evidence**, while keeping the full product-surface requirement Planned/Open until the issue acceptance criteria are met.

### 3.4 Browser authority is transitioning from value types to an adapter registry

Protected main already contains session/context/document/node authority foundations. PR #40 is adding the bounded session-scoped registry that prevents raw BiDi/CDP identifiers from becoming durable OriginWeave authority. The architecture and ERD are directionally correct, but PRD/TRD/UML must be reconciled after that branch reaches a stable integration state.

### 3.5 ADR lifecycle discoverability had drifted

The previous ADR index listed only 0001-0006 as current protected-main decisions even though Accepted ADRs 0007, 0008 and 0010 were present on protected main. Proposed ADR 0009 was also absent from both accepted and proposed tables. This created an architecture-discovery defect. The current documentation branch repairs the index while preserving each ADR's own Accepted/Proposed status.

### 3.6 UML assessment itself was initially stale

The first pass of this assessment incorrectly called resource-pressure and hourly automation flows missing. A direct re-read of protected-main `docs/uml/README.md` showed both already exist. This branch corrects the matrix instead of preserving the mistaken audit claim, and adds only the genuinely missing extension-permission-to-Agent-authority view.

## 4. Durable conversation decisions that must remain represented

The following product decisions are durable architecture input and may not live only in chat, scheduler prompts, PR bodies, or implementation plans:

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

## 5. Architecture views requiring follow-through

### 5.1 Extension authority and compatibility sequence — added in this branch

[`uml/extension-authority.md`](uml/extension-authority.md) now separates:

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

Protected-main `docs/uml/README.md` already models browser/model resource pressure and fallback. Future edits should refine it only when the platform telemetry/admission implementation changes, rather than creating a duplicate diagram merely to satisfy a checklist.

### 5.3 Hourly autonomous-development authority flow — already present

Protected-main `docs/uml/README.md` already models deterministic early gates, conditional model-credential use, pristine attempts, bounded validation, publication authority, protected merge and protected-main operational acceptance. Its implementation/evidence status must continue to be reconciled against the actual workflow rather than inferred from the diagram.

### 5.4 Real Chromium vertical slice — incomplete until issue #28 stabilizes

Once issue #28 begins integrating, diagram and trace:

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

### Execute now

- Keep this fitness assessment discoverable from `docs/README.md`.
- Reconcile the ADR index with every protected-main ADR and its own status.
- Add machine-checkable documentation fitness contracts so ADR discoverability/status drift is caught automatically.
- Add the missing extension-permission-to-Agent-authority UML without duplicating already-present resource/automation views.
- Continue the existing HTTP replacement, browser-registry and MV3 compatibility work without using documentation as a reason to stop.

### Defer to stable implementation state

- Replace historical/current PR references in PRD/TRD/traceability immediately after the relevant active branch reaches a stable exact head or protected merge, so documentation does not race source writers.
- Add the detailed real-Chromium vertical-slice UML when its executable contracts are stable enough that the diagram will not encode temporary protocol/field names.
- Promote Proposed ADRs only through an explicit reviewed status change; do not infer Acceptance from file presence on `main`.

## 7. Completion criteria for documentation fitness

The whole documentation graph becomes **PROTECTED-MAIN-SUFFICIENT** only when:

1. PRD and TRD implementation inventories agree with current protected-main crates, APIs and executable browser/extension evidence;
2. no canonical document identifies a superseded/historical PR as current active implementation evidence;
3. the ADR index discovers every ADR and its status agrees with the file metadata;
4. UML covers all current material authority flows, including extension/Agent isolation and the real Chromium vertical slice once implemented;
5. ERD/domain models accurately distinguish conceptual, in-memory, persisted, adapter-owned and external entities;
6. traceability maps every material requirement and Accepted decision to current implementation/test/evidence or an explicit open issue;
7. machine-checkable documentation tests catch stale status/index/link/ownership terminology;
8. security, test, operability, data-governance and release docs agree with the same shipped-vs-planned boundary; and
9. protected-main checks/review/governance for the documentation reconciliation itself pass.

Until then, OriginWeave is **design-documented but not documentation-closed**. That is a product-quality finding, not a release blocker that prevents unrelated safe implementation work.