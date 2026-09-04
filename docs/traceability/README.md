# OriginWeave Product and Decision Traceability

- **Status:** Proposed authoritative traceability baseline
- **Scope:** Product requirements, Accepted architecture, protected-main implementation, active-PR implementation, planned adapters, conversation-derived decisions, standards, and verification evidence

This file prevents two opposite errors:

1. an implemented safety boundary becoming undiscoverable because it exists only in code/tests; and
2. a product-design conversation, issue, or active pull request being presented as if it already shipped.

## 1. Evidence precedence

For current behavior, use this precedence order:

1. exact protected-main code and executable tests;
2. Accepted ADRs governing that code;
3. current root `ARCHITECTURE.md` and authoritative PRD/TRD aligned to protected main;
4. active-PR code/tests as explicitly labeled non-shipped evidence;
5. roadmap and issue plans;
6. conversation-derived product decisions and research notes.

Lower layers may define future direction but cannot override current protected implementation or an Accepted ADR. Active-PR behavior is never protected-main truth.

### 1.1 Active freshness-authority dossiers

Transient implementation evidence that materially tightens an existing authority boundary is kept in explicit active-PR traceability rather than silently changing protected-main maturity:

- [`resolution-freshness-authority.md`](resolution-freshness-authority.md) — PR #47 bounds the lifetime of validated destination-resolution authority; the direct socket consumer still must require that fresh authority before the overall DNS-rebinding/TOCTOU interval can be called implemented on protected main.
- [`tls-revocation-freshness-authority.md`](tls-revocation-freshness-authority.md) — PR #48 classifies independently verified revocation material for freshness only; it does not fetch or authenticate OCSP/CRL material and does not create an unrevoked-certificate claim.

These dossiers are evidence indexes, not substitute ADRs. A new ADR is required only when a durable architecture/trust/deployment decision changes.

## 2. Capability maturity vocabulary

Capability maturity uses exactly one of these values:

- **IMPLEMENTED_ON_PROTECTED_MAIN** — present on protected `main` with executable evidence.
- **IMPLEMENTED_ON_ACTIVE_PR** — implemented and testable on an active PR, but not shipped/protected-main truth.
- **PARTIAL** — material foundations are implemented, while a named runtime, lifecycle, integration, or acceptance boundary remains incomplete.
- **ACCEPTED_ARCHITECTURE** — governing reviewed direction; implementation may be incomplete.
- **PLANNED** — accepted product backlog or target architecture without current implementation evidence.
- **RESEARCH_ONLY** — exploratory evidence that does not define a product commitment.
- **SUPERSEDED** — replaced by later implementation or architecture authority.
- **OUT_OF_SCOPE** — intentionally excluded from the current product boundary.

ADR lifecycle is separate and remains `Proposed`, `Accepted`, `Superseded`, `Deprecated`, or `Rejected`. An Accepted ADR is design authority, not implementation proof.

## 3. Product-level decision trace

| Product decision | Capability maturity | Authoritative artifact | Protected-main / active-PR evidence boundary |
|---|---|---|---|
| Chromium remains the compatibility kernel rather than rewriting Blink/V8 | ACCEPTED_ARCHITECTURE | ADR 0001; `ARCHITECTURE.md`; PRD-COMP-001 | Architecture/repository contracts exist; complete branded browser distribution remains Planned |
| `Browse. Act. Prove.` provenance-native product identity | ACCEPTED_ARCHITECTURE | `README.md`; `docs/PRD.md`; roadmap | Evidence/provenance foundations exist; complete buyer Evidence Trail remains Planned |
| Human / Assist / Agent Task / Crawler execution modes | PARTIAL | `ARCHITECTURE.md`; `docs/PRD.md`; ADR 0002 | Core mode/purpose/policy foundations exist; browser-session/profile integration remains incomplete |
| Page content is data, never instruction authority | IMPLEMENTED_ON_PROTECTED_MAIN | ADR 0002; `ARCHITECTURE.md`; `docs/TRD.md` | `originweave-core` + `originweave-policy` tests |
| Typed actions instead of default arbitrary JavaScript | PARTIAL | PRD-ACT-001..004; ADR 0002 | Typed core/policy foundations are on main; complete browser action adapter remains Planned |
| logical origin != resolved destination | IMPLEMENTED_ON_PROTECTED_MAIN | ADR 0004; TRD-INV-002 | `originweave-destination`; destination governance tests |
| Bounded resolution freshness is explicit before destination authority is consumed | IMPLEMENTED_ON_ACTIVE_PR | ADR 0004; [`resolution-freshness-authority.md`](resolution-freshness-authority.md) | PR #47 implements the deterministic freshness primitive; protected-main socket planning can still bypass it, so the overall resolution-to-socket TOCTOU boundary remains PARTIAL |
| resolved destination != TCP peer | IMPLEMENTED_ON_PROTECTED_MAIN | ADR 0005; TRD Section 6 | `originweave-network`; loopback/peer tests |
| TCP peer != TLS service identity | IMPLEMENTED_ON_PROTECTED_MAIN | ADR 0006; TRD Section 6 | `originweave-tls`; rustls integration tests |
| Revocation-material freshness is separate from revocation authenticity/non-revocation | IMPLEMENTED_ON_ACTIVE_PR | ADR 0006/0008 boundary; [`tls-revocation-freshness-authority.md`](tls-revocation-freshness-authority.md) | PR #48 adds a freshness classifier only; protected main still records revocation as NotConfigured and makes no unrevoked claim |
| Proxy/PAC route authority must be explicit | PARTIAL | PRD-NET-005; TRD Section 6.3 | Protected-main direct-route authority exists; PAC evaluation/proxy transport/CONNECT remain incomplete |
| Bounded HTTP semantics require an authenticated governed connection and resource bounds | IMPLEMENTED_ON_ACTIVE_PR | PRD-NET-006; issue #9; active PR #37 | `originweave-http` replacement exists on active PR #37; historical PR #11 is SUPERSEDED implementation lineage and is not current evidence; no protected-main HTTP claim yet |
| Node handles bind session/context/origin/document lifetime | PARTIAL | ADR 0010; PRD-OBS-001/002; TRD Section 5 | Core opaque session/context/document/node authority and `SameDocumentMutationKind` epoch rotation are on this lane; live browser mutation observation remains planned; active PR #40 owns the protocol-ID registry and remains non-shipped evidence |
| Semantic observations retain OriginWeave node authority and explicit source-channel provenance | IMPLEMENTED_ON_ACTIVE_PR | PRD-OBS-001/003/005; ADR 0010; structured-observation architecture | Active PR #52, stacked on #40, implements a bounded `SemanticNodeObservation` value primitive that rejects missing evidence-channel provenance. It is not a browser observation adapter; channels and advertised node actions are descriptive evidence and grant no execution authority |
| Raw secrets never enter model context | PARTIAL | PRD-DATA-001; ADR 0002; TRD Section 9 | Core secret-delivery policy exists; trusted broker/runtime completion remains Planned |
| Sensitive disclosure is purpose- and classification-bound | PARTIAL | ADR 0007; PRD-DATA-002; issue #10 | Purpose-bound policy/evidence foundations are on protected main; active PR #45 adds credential-free handle-lifecycle evidence and #46 adds bounded in-process authoritative use reservation, while trusted storage/revocation/value resolution/cross-process lifecycle/model-disclosure remain open |
| Evidence/provenance are product outputs, not debug leftovers | PARTIAL | ADR 0003; PRD Section 9.6 | `originweave-evidence` foundations exist; complete durable Evidence Trail/WARC/PROV adapters remain Planned |
| Human interaction outranks inference/background collection | PARTIAL | `ARCHITECTURE.md`; PRD-RES-002 | Deterministic resource mitigation/CPU-worker admission foundations exist; platform telemetry/actuation remain Planned |
| Structured observation precedes raw HTML/screenshot fallback | ACCEPTED_ARCHITECTURE | PRD-OBS-003; TRD Section 7 | Active PR #52 supplies a non-shipped bounded semantic value primitive; real browser observation and fallback adapters remain Planned |
| WebDriver BiDi / CDP / WebMCP / MCP are adapters, not internal authority | ACCEPTED_ARCHITECTURE | PRD Section 9.8; TRD Section 12 | Protocol adapter implementation remains Planned/active under issue #28; active PR #40 may not be called shipped |
| Manifest V3 compatibility is preserved upstream where practical | PARTIAL | ADR 0001; issue #27; Proposed ADR 0013 | Protected main has pinned real-Chromium compatibility evidence for service worker/content script/storage/DNR/tabs/windows/scripting/commands/side panel/bookmarks/history/restart/repeatability; active PR #43 adds real bounded downloads evidence; full issue #27 matrix remains incomplete |
| Extension permission does not imply OriginWeave Agent capability | PARTIAL | protected-main extension authority kernel; Proposed ADR 0013 | Core extension-to-Agent authority isolation exists on protected main; complete managed-extension/native-messaging/enterprise release policy remains incomplete |
| WARC/PROV-oriented durable evidence adapters | PLANNED | ADR 0003; PRD-EVD-005 | Source/provenance kernel foundation exists; persistence/export adapters remain Planned |
| Origin Map visualizes value/action provenance | PLANNED | PRD-EVD-004; this traceability record | No shipped UI claim |
| Browser / Runtime / Observe / Capture / Governor / Policy / Evidence / Protocol / SDK product surfaces | PARTIAL | PRD Section 6 | Some foundations exist under crates; named commercial surfaces are not all shipped artifacts |
| Constrained GPU phase scheduling for browser rendering vs local inference | PARTIAL | PRD-RES-005; TRD Section 10 | Deterministic resource plan exists; real GPU scheduler/telemetry remains Planned |
| Enterprise SSO/SCIM/residency/audit/procurement package | PLANNED | PRD Section 9.11; roadmap Phase 5 | Not shipped in pre-alpha baseline |

## 4. Requirement-to-module trace

| Requirement family | Current module(s) / lane | Primary tests/docs | Capability maturity |
|---|---|---|---|
| Canonical origin / action / approval | `originweave-core` | crate tests; ADR 0002 | IMPLEMENTED_ON_PROTECTED_MAIN |
| Deterministic action policy | `originweave-policy` | policy/security-review tests | IMPLEMENTED_ON_PROTECTED_MAIN |
| Destination/rebinding/redirect | `originweave-destination` | destination tests; ADR 0004 | IMPLEMENTED_ON_PROTECTED_MAIN |
| Resolution freshness authority | active `originweave-destination` work in PR #47 | [`resolution-freshness-authority.md`](resolution-freshness-authority.md); active exact-head tests/coverage | IMPLEMENTED_ON_ACTIVE_PR |
| Exact direct socket/peer | `originweave-network` | real loopback + error tests; ADR 0005 | IMPLEMENTED_ON_PROTECTED_MAIN |
| TLS identity | `originweave-tls` | real rustls integration; ADR 0006 | IMPLEMENTED_ON_PROTECTED_MAIN |
| TLS revocation-material freshness | active `originweave-tls` work in PR #48 | [`tls-revocation-freshness-authority.md`](tls-revocation-freshness-authority.md); active exact-head tests/coverage | IMPLEMENTED_ON_ACTIVE_PR |
| Resource budgets/mitigations | `originweave-resource` | crate tests | PARTIAL |
| Redacted evidence/provenance | `originweave-evidence` | crate tests; ADR 0003 | PARTIAL |
| Bounded HTTP/1.1 | active `originweave-http` replacement in PR #37 | issue #9; active-PR unit/integration/coverage evidence | IMPLEMENTED_ON_ACTIVE_PR |
| Proxy/PAC | destination/route foundation + future adapter | roadmap/TRD | PARTIAL |
| Session/context/document/node authority | `originweave-core` authority values; active registry work in PR #40 | ADR 0010; roadmap/TRD/UML | PARTIAL |
| Semantic observation value authority/provenance | active `originweave-core` work in PR #52, stacked on #40 | `semantic_node_observation` tests; PRD-OBS-001/003/005; issue #28 | IMPLEMENTED_ON_ACTIVE_PR |
| Manifest V3 compatibility evidence | `scripts/ci/run_mv3_compatibility.py` + controlled MV3 fixture; active downloads lane #43 | issue #27; real-browser contracts | PARTIAL |
| Extension-to-Agent authority | protected-main core authority kernel + Proposed ADR 0013 | issue #27; extension authority UML | PARTIAL |
| Purpose-bound sensitive-data policy/evidence | `originweave-policy` + evidence foundations; active lifecycle/reservation work #45/#46 | ADR 0007; issue #10 | PARTIAL |
| Trusted sensitive-data broker/storage/lifecycle | future bounded service/crate | issue #10; PRD/TRD/data governance | PLANNED |
| BiDi/CDP/WebMCP/MCP | future/versioned adapter crates; registry prerequisite active in #40 | protocol compatibility tests required | PLANNED |
| WARC/PROV persistence | persistence/export adapters | doctoring + future conformance tests | PLANNED |

## 5. Requirement-to-ADR trace

| Requirement | Governing ADR / current decision boundary |
|---|---|
| PRD-COMP-001, Chromium compatibility kernel | ADR 0001 (Accepted) |
| PRD-ACT-001, PRD-ACT-005, PRD-CRAWL-001, trust-source boundary | ADR 0002 (Accepted) |
| PRD-EVD-001, PRD-EVD-002, PRD-EVD-005 | ADR 0003 (Accepted) |
| PRD-NET-001, PRD-NET-002, redirect/rebinding/freshness boundary | ADR 0004 (Accepted); active PR #47 tightens the existing boundary without creating a new deployed component or trust owner |
| PRD-NET-003 | ADR 0005 (Accepted) |
| PRD-NET-004 | ADR 0006 (Accepted); active PR #48 adds revocation-material freshness only and does not define a complete revocation architecture |
| Purpose-bound sensitive-data authority | ADR 0007 (Accepted); trusted broker/storage/lifecycle still issue #10 |
| TLS delegated-task leaf-validity horizon | ADR 0008 (Accepted) |
| Session/context/document/node binding | ADR 0010 (Accepted); active registry implementation #40 remains non-shipped |
| Semantic observation authority/provenance | Existing session/node authority plus structured-observation architecture; active PR #52 narrows the value contract without creating a new service, trust owner, persistence boundary, or external protocol and therefore does not justify a new ADR by itself |
| Manifest V3 compatibility + extension-to-Agent authority | ADR 0013 is Proposed on documentation PR #44; protected-main extension authority code does not auto-Accept the ADR |
| Architecture-decision acceptance governance | ADR 0014 is Proposed on documentation PR #44; protected-main AGENTS + live policy remain authoritative |
| HTTP semantics | active PR #37 contains its feature ADR lineage; it is active-PR evidence until protected merge and index reconciliation |
| Proxy/PAC route execution | current protected-main route authority + future dedicated execution decision as needed |
| Enterprise deployment/privacy | open ADR family before production release |

## 6. Standards-to-decision trace

The canonical APA 7th bibliography is [`../doctoring.md`](../doctoring.md). This matrix points to the decision use; it does not duplicate the bibliography.

| Standard / primary evidence family | OriginWeave use |
|---|---|
| WHATWG URL + Chromium canonicalizer | Browser-compatible origin identity and numeric-host rejection |
| IANA special-purpose registries / RFC 6890 / RFC 8190 / RFC 9637 | Destination classification and fail-closed public-web policy |
| RFC 9293 | Exact TCP endpoint/peer model |
| RFC 5280 / RFC 9525 / RFC 9325 | Certificate path, HTTPS service identity, and the separation between certificate validity and any future revocation policy |
| RFC 9110 / RFC 9112 / RFC 9530 | Bounded HTTP semantics, framing, redirect evidence and digest fields |
| RFC 9309 | Crawler robots evidence, explicitly not access authorization |
| W3C WebDriver BiDi | Versioned browser automation adapter, not core authority |
| Chrome DevTools Protocol | Chromium-specific observation/diagnostic adapter |
| Chrome Manifest V3 documentation | Extension compatibility baseline |
| W3C PROV-O / ISO 28500 WARC | Provenance and web-capture interoperability |
| NIST AI 600-1 / web-agent prompt-injection research | Trust-class separation and model risk testing |
| WCAG 2.2 / ISO/IEC 40500:2025 | Product UI accessibility target |

Material claims should update `docs/doctoring.md` with current primary evidence rather than relying on this summary.

## 7. Diagram-to-requirement trace

| Diagram | Requirements represented / maturity |
|---|---|
| UML component/bounded-context view | Product family, Chromium/Rust ownership, adapter boundaries |
| Network authority sequence | PRD-NET-001..007; TRD-INV-002; HTTP remains active-PR until #37 integrates; resolution freshness remains an active lower-layer primitive until the socket consumer requires it |
| Observation/action sequence | PRD-OBS, PRD-ACT, PRD-DATA, trust separation; active #52 makes the bounded semantic-observation value/provenance contract explicit without establishing browser I/O or action dispatch |
| Delegated-task state machine | session lifecycle, approval, resource pause, cancellation/recovery, post-condition truth |
| Deployment topology | renderer trust, orchestrator/model/store boundaries |
| Evidence authority flow | PRD-EVD; proposal/policy/approval/execution/outcome separation |
| Extension authority sequence | MV3 compatibility plane vs explicit OriginWeave extension grant and Agent capability separation |
| Conceptual ERD | session/action/network/sensitive/resource/provenance identity; active freshness and semantic-value primitives introduce no physical persistence |
| Real Chromium vertical-slice sequence | PLANNED until issue #28 implementation stabilizes; active #40/#51/#52 are prerequisites, not proof of the real adapter flow; do not encode temporary adapter fields as shipped architecture |

## 8. Conversation-to-repository capture rule

A **conversation-derived** decision is not binding merely because it was repeated. During maintenance, evaluate whether it affects product identity, public API, authority/security/privacy, data/evidence model, interoperability, resource behavior, lifecycle, non-goals, or release criteria.

If material and absent from GitHub:

1. record it with explicit capability maturity in PRD/TRD/traceability;
2. create or supersede an ADR when it changes a governing architecture decision;
3. update UML/ERD when relationships or lifecycles change;
4. add standards/research to `docs/doctoring.md` when evidence is material;
5. add executable tests before calling production behavior `IMPLEMENTED_ON_PROTECTED_MAIN`;
6. update the protected-main ADR index only after review and merge.

This rule intentionally prevents chat history from becoming a shadow architecture database.

## 9. Documentation drift checks

Repository contracts should fail when canonical PRD/TRD/ADR/UML/ERD/traceability artifacts disappear, lifecycle/index status diverges, an active PR is promoted to protected-main truth, or core maturity/authority vocabulary is removed. Active freshness dossiers must remain discoverable from this index so lower-layer primitives cannot silently become over-broad shipped claims. More semantic checks should be added when a specific drift has caused a real defect; avoid brittle tests that merely duplicate prose.

## 10. Open traceability work

- **Open:** active PR #47 must reach unchanged exact-head CI/security/100% coverage, then the first-party socket consumer must require the fresh resolution authority before the resolution-to-socket TOCTOU interval can become protected-main implemented evidence.
- **Open:** active PR #48 remains freshness classification only; define and review revocation-material acquisition/authenticity/cache/failure/composition before any protected-main revocation-enforcement or unrevoked claim.
- **Open:** after #37 integrates, move bounded HTTP from `IMPLEMENTED_ON_ACTIVE_PR` into protected-main evidence and close historical PR #11 only after unique-work preservation and protected-main verification are proven.
- **Open:** after #43 integrates, move bounded MV3 downloads from `IMPLEMENTED_ON_ACTIVE_PR` into the protected-main compatibility evidence inventory while issue #27 remains open for the complete matrix.
- **Open:** after #40 stabilizes/integrates, map its registry API and tests without presenting raw BiDi/CDP identifiers as durable authority.
- **Open:** after stacked #52 stabilizes/integrates behind #40, reclassify only its bounded semantic-observation value/provenance primitive; keep real browser observation I/O, action dispatch, mutation invalidation and post-condition evidence under issue #28 until implemented.
- **Open:** after #45/#46 integrate, reclassify their narrow lifecycle/reservation primitives while keeping durable trusted-broker storage/revocation/value-resolution/model-disclosure boundaries under issue #10 until implemented.
- **Open:** attach concrete release profiles and quantitative benchmark thresholds after reproducible benchmark evidence exists.
- **Open:** map every future public OriginWeave Protocol operation to risk/capability/authority and conformance tests.
- **Open:** map enterprise controls to exact SOC 2/CSAP-oriented control evidence without claiming certification.
- **Open:** add data-retention and residency lifecycle diagrams when persistence/tenant adapters become concrete.
