# OriginWeave Product and Decision Traceability

- **Status:** Proposed authoritative traceability baseline
- **Scope:** Product requirements, Accepted architecture, implemented kernels, planned adapters, conversation-derived decisions, standards, and verification evidence

This file prevents two opposite errors:

1. an implemented safety boundary becoming undiscoverable because it exists only in code/tests; and
2. a product-design conversation or pull-request proposal being presented as if it already shipped.

## 1. Evidence precedence

For current behavior, use this precedence order:

1. exact protected-main code and executable tests;
2. Accepted ADRs governing that code;
3. current root `ARCHITECTURE.md` and authoritative PRD/TRD aligned to protected main;
4. roadmap and issue/PR plans;
5. conversation-derived product decisions and research notes.

Lower layers may define future direction but cannot override current protected implementation or an Accepted ADR.

## 2. Status vocabulary

- **Implemented** — present on protected `main` with executable evidence.
- **Accepted architecture** — governing reviewed direction, though the complete runtime path may be unfinished.
- **Proposed** — candidate product/design decision requiring reviewed adoption.
- **Open** — intentionally unresolved.

A change can move from Proposed -> Accepted architecture -> Implemented, but never skips evidence merely because the idea is compelling.

## 3. Product-level decision trace

| Product decision | Origin/status | Authoritative artifact | Protected implementation/evidence |
|---|---|---|---|
| Chromium remains the compatibility kernel rather than rewriting Blink/V8 | Accepted architecture | ADR 0001; `ARCHITECTURE.md`; PRD-COMP-001 | Architecture/repository contract tests; Chromium adapter itself remains Planned |
| `Browse. Act. Prove.` provenance-native product identity | Accepted product framing | `README.md`; `docs/PRD.md`; roadmap | Evidence/provenance foundation implemented; full buyer Evidence Trail Planned |
| Human / Assist / Agent Task / Crawler execution modes | Accepted architecture | `ARCHITECTURE.md`; `docs/PRD.md`; ADR 0002 | Core mode/purpose and policy foundation implemented; browser-session integration Planned |
| Page content is data, never instruction authority | Implemented foundation | ADR 0002; `ARCHITECTURE.md`; `docs/TRD.md` | `originweave-core` + `originweave-policy` tests |
| Typed actions instead of default arbitrary JavaScript | Accepted architecture | PRD-ACT-001..004; ADR 0002 | Typed core/policy foundation implemented; full browser action adapter Planned |
| logical origin != resolved destination | Implemented | ADR 0004; TRD-INV-002 | `originweave-destination`; destination governance tests |
| resolved destination != TCP peer | Implemented | ADR 0005; TRD Section 6 | `originweave-network`; loopback/peer tests |
| TCP peer != TLS service identity | Implemented | ADR 0006; TRD Section 6 | `originweave-tls`; rustls integration tests |
| Proxy/PAC route authority must be explicit | Accepted architecture / active development | PRD-NET-005; TRD Section 6.3 | Protected-main direct-only boundary exists; complete proxy execution not yet shipped |
| HTTP semantics require an authenticated governed connection and resource bounds | Accepted architecture / active development | PRD-NET-006; TRD Section 6.6 | Not yet a protected-main product capability in this baseline |
| Node handles bind session/context/origin/document lifetime | Proposed/active development | PRD-OBS-001/002; TRD Section 5 | Not treated as shipped until protected integration |
| Raw secrets never enter model context | Accepted architecture / implemented policy foundation | PRD-DATA-001; ADR 0002; TRD Section 9 | Core secret-delivery policy implemented; trusted broker runtime Planned |
| Sensitive disclosure is purpose-bound and classification-bound | Proposed/active development | PRD-DATA-002; TRD Section 9 | Do not claim complete broker/service until protected integration |
| Evidence/provenance are product outputs, not debug leftovers | Accepted / foundation implemented | ADR 0003; PRD Section 9.6 | `originweave-evidence`; evidence governance tests |
| Human interaction outranks inference/background collection | Accepted architecture / foundation implemented | `ARCHITECTURE.md`; PRD-RES-002 | Deterministic resource mitigation foundation implemented; platform telemetry Planned |
| Structured observation precedes raw HTML/screenshot fallback | Accepted architecture | PRD-OBS-003; TRD Section 7 | Observation adapter Planned |
| WebDriver BiDi / CDP / WebMCP / MCP are adapters, not internal authority | Accepted architecture | PRD Section 9.8; TRD Section 12 | Adapter implementations Planned |
| Manifest V3 compatibility is preserved upstream where practical | Accepted architecture | ADR 0001; PRD Section 9.9 | Chromium compatibility program Planned |
| WARC/PROV-oriented durable evidence adapters | Accepted architecture / Planned | ADR 0003; PRD-EVD-005 | Source/provenance kernel foundation exists; persistence adapters Planned |
| Origin Map visualizes value/action provenance | **conversation-derived Proposed** product UX | PRD-EVD-004; this traceability record | No shipped UI claim |
| Browser / Runtime / Observe / Capture / Governor / Policy / Evidence / Protocol / SDK product surfaces | **conversation-derived Proposed product taxonomy**, aligned to existing architecture | PRD Section 6 | Some foundations exist under crates; named commercial surfaces are not all shipped artifacts |
| Constrained GPU phase scheduling for browser rendering vs local inference | **conversation-derived Accepted architecture direction**, implementation Planned | PRD-RES-005; TRD Section 10 | Deterministic resource plan exists; real GPU scheduler/telemetry Planned |
| Enterprise SSO/SCIM/residency/audit/procurement package | Planned | PRD Section 9.11; roadmap Phase 5 | Not shipped in pre-alpha baseline |

## 4. Requirement-to-module trace

| Requirement family | Current module(s) | Primary tests/docs | Implementation status |
|---|---|---|---|
| Canonical origin / action / approval | `originweave-core` | crate tests; ADR 0002 | Implemented |
| Deterministic action policy | `originweave-policy` | policy/security-review tests | Implemented |
| Destination/rebinding/redirect | `originweave-destination` | destination tests; ADR 0004 | Implemented |
| Exact direct socket/peer | `originweave-network` | real loopback + error tests; ADR 0005 | Implemented |
| TLS identity | `originweave-tls` | real rustls integration; ADR 0006 | Implemented |
| Resource budgets/mitigations | `originweave-resource` | crate tests | Implemented foundation |
| Redacted evidence/provenance | `originweave-evidence` | crate tests; ADR 0003 | Implemented foundation |
| HTTP | future/active `originweave-http` work | dedicated design/tests/PR evidence | Planned until protected merge |
| Proxy/PAC | destination foundation + future adapter | roadmap/TRD | Planned/active |
| Session/observation/action | future crates/adapters | roadmap/TRD/UML | Planned/active |
| Secret broker | future bounded service/crate | PRD/TRD | Planned/active |
| BiDi/CDP/WebMCP/MCP | adapter crates | protocol compatibility tests required | Planned |
| WARC/PROV persistence | persistence adapters | doctoring + future conformance tests | Planned |

## 5. Requirement-to-ADR trace

| Requirement | Governing ADR |
|---|---|
| PRD-COMP-001, PRD-COMP-003 | ADR 0001 |
| PRD-ACT-001, PRD-ACT-005, PRD-CRAWL-001, trust-source boundary | ADR 0002 |
| PRD-EVD-001, PRD-EVD-002, PRD-EVD-005 | ADR 0003 |
| PRD-NET-001, PRD-NET-002, redirect/rebinding boundary | ADR 0004 |
| PRD-NET-003 | ADR 0005 |
| PRD-NET-004 | ADR 0006 |
| Session/context/document node binding | Proposed/active decision; index only after dedicated ADR reaches protected main |
| Proxy/PAC route execution | Proposed/active decision; protected-main index updates after merge |
| HTTP semantics | Proposed/active decision; protected-main index updates after merge |
| Sensitive-data broker lifecycle | Proposed/active decision; policy/evidence slices do not equal full broker acceptance |
| Enterprise deployment/privacy | Open ADR family before production release |

## 6. Standards-to-decision trace

The canonical APA 7th bibliography is [`../doctoring.md`](../doctoring.md). This matrix points to the decision use; it does not duplicate the bibliography.

| Standard / primary evidence family | OriginWeave use |
|---|---|
| WHATWG URL + Chromium canonicalizer | Browser-compatible origin identity and numeric-host rejection |
| IANA special-purpose registries / RFC 6890 / RFC 8190 / RFC 9637 | Destination classification and fail-closed public-web policy |
| RFC 9293 | Exact TCP endpoint/peer model |
| RFC 5280 / RFC 9525 / current TLS guidance | Certificate path and HTTPS service identity |
| RFC 9110 and related HTTP specifications | Redirect and bounded HTTP semantics |
| RFC 9309 | Crawler robots evidence, explicitly not access authorization |
| W3C WebDriver BiDi | Versioned browser automation adapter, not core authority |
| Chrome DevTools Protocol | Chromium-specific observation/diagnostic adapter |
| Chrome Manifest V3 documentation | Extension compatibility baseline |
| W3C PROV-O / ISO 28500 WARC | Provenance and web-capture interoperability |
| NIST AI 600-1 / web-agent prompt-injection research | Trust-class separation and model risk testing |
| WCAG 2.2 / ISO/IEC 40500:2025 | Product UI accessibility target |

Material claims should update `docs/doctoring.md` with current primary evidence rather than relying on this summary.

## 7. Diagram-to-requirement trace

| Diagram | Requirements represented |
|---|---|
| UML component/bounded-context view | Product family, Chromium/Rust ownership, adapter boundaries |
| Network authority sequence | PRD-NET-001..007; TRD-INV-002 |
| Observation/action sequence | PRD-OBS, PRD-ACT, PRD-DATA, trust separation |
| Delegated-task state machine | session lifecycle, approval, resource pause, cancellation/recovery, post-condition truth |
| Deployment topology | renderer trust, orchestrator/model/store boundaries |
| Evidence authority flow | PRD-EVD; separation of proposal/policy/approval/execution/outcome |
| Conceptual ERD | durable session/action/network/sensitive/resource/provenance identity |

## 8. Conversation-to-repository capture rule

A **conversation-derived** decision is not binding merely because it was repeated. During maintenance, evaluate whether it affects product identity, public API, authority/security/privacy, data/evidence model, interoperability, resource behavior, lifecycle, non-goals, or release criteria.

If material and absent from GitHub:

1. record it as `Proposed` or `Open` in PRD/TRD/traceability;
2. create/supersede an ADR when it changes a governing architecture decision;
3. update UML/ERD when relationships or lifecycles change;
4. add standards/research to `docs/doctoring.md` when evidence is material;
5. add executable tests before calling production behavior Implemented;
6. update the protected-main ADR index only after review and merge.

This rule intentionally prevents chat history from becoming a shadow architecture database.

## 9. Documentation drift checks

Repository contracts should fail when the canonical PRD/TRD/ADR index/UML/ERD/traceability files disappear or when core status/authority vocabulary is removed. More semantic checks should be added when a specific drift has caused a real defect; avoid brittle tests that duplicate prose without protecting a contract.

## 10. Open traceability work

- **Open:** attach concrete release profiles and quantitative benchmark thresholds after reproducible benchmark evidence exists.
- **Open:** map every future public OriginWeave Protocol operation to risk/capability/authority and conformance tests.
- **Open:** map enterprise controls to exact SOC 2/CSAP-oriented control evidence without claiming certification.
- **Open:** add data-retention and residency lifecycle diagrams when persistence/tenant adapters become concrete.
- **Open:** after active feature PRs merge, update this matrix from `Proposed/active development` to the exact protected implementation and Accepted ADRs.
