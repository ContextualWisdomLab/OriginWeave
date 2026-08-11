# Active pull-request maturity evidence — 2026-08-11 closure

- **Protected-main anchor:** `67af7c87589edc2039545af335c95064d9b8391c`
- **Canonical documentation verdict:** **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**
- **Scope:** exact-current reconciliation for PR #73 and PR #74 after their 2026-08-11 defect/review corrections

Protected `main` remains the only shipped-code authority. This appendix records volatile exact-head evidence for active work and must never be read as protected-main implementation or release evidence.

## Exact-current active lanes

| PR | Scope | Maturity | Exact evidence / authority boundary |
|---|---|---|---|
| #73 | Bounded Chromium root-plus-descendant RSS evidence in the controlled pinned-browser fixture | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `e5fabfd57387ec7d2db692961eda93c95cf8d886`, stacked on unchanged #72 head `1a7186085abe926c1d0e5b22c36760965d6e237b`, restores the focused optional-RSS regression and implements `_parse_linux_proc_status_optional_rss_bytes`. Absent or zero `VmRSS` remains representable as nonresident sampled evidence; one positive field becomes bounded bytes; duplicate, malformed, or overflowed evidence fails closed rather than being normalized to absence. CI run `31464241922` succeeds, including Rust contracts job `93693702956` with exact-head checkout and Production coverage job `93693703022`; Manifest V3 Compatibility run `31464241924` also succeeds on the exact head. The PR remains Draft because #72/#71/#70/#65 are active prerequisites. This is controlled Linux CI evidence, not product task/process attribution, cgroup authority, per-tab ownership, GPU/VRAM attribution, or cross-platform resource telemetry. |
| #74 | Separation of extension proposal-grant evaluation from ordinary Agent action policy | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `0d492564aa61c9094f1315ee4e234b46a1e63a6c` is based directly on protected main. A predecessor-head CodeRabbit review correctly identified that no production adapter currently converts an extension proposal into an `ActionRequest`; the former naming therefore implied a composition path that did not exist. The exact current head renames and documents the tests as independent boundaries: `evaluate_extension_access` may authorize the exact extension/session/context `ProposeTypedAction` grant while ordinary user-sourced requests remain independently fail-closed for cross-origin mutation, missing write authority, crawler mutation, execution-mode/purpose mismatch, robots evidence, non-delegable R5 consent, and Human mode. CI run `31464388199`, Security Scan run `31464388200`, and SAST Semgrep run `31464388210` all succeed on this exact head. The branch adds no extension-proposal adapter, action-source transformation, execution API, or new authority and therefore does not prove a real extension → Agent action composition path or close issue #27. |

## Documentation-fitness reconciliation

The repository-wide documentation verdict remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**.

- **ADR:** no additional ADR is warranted by these corrections. #73 is an evidence-integrity refinement inside existing browser/resource contracts. #74 corrects a test/evidence overclaim and reinforces the already documented separation between extension permission and Agent authority. Proposed ADR 0013 remains Proposed and is not promoted by branch presence, tests, or CI.
- **PRD/TRD/Architecture:** current contracts already require resource evidence to remain distinct from trusted attribution and require browser/extension permission not to mint Agent capability, origin, approval, secret, or execution authority. The new exact heads strengthen evidence without changing the governing architecture.
- **UML:** the existing extension-authority view remains sufficient because #74 introduces no new actor, adapter, trust boundary, or execution edge. A detailed production adapter → semantic observation → typed policy/action → post-condition/recovery/resource sequence remains deferred until the production Chromium composition boundary exists; #73's CI `/proc` sampler is not that product boundary.
- **ERD/data model:** neither lane introduces OriginWeave-owned durable persistence, ownership/cardinality changes, migrations, or rollback state. The conceptual ERD remains the truthful artifact; physical process-sample or extension-policy tables would be invented architecture.
- **Security/test/release:** #73 now fails closed on ambiguous Linux RSS evidence and has exact-current repository/coverage/pinned-browser GREEN proof. #74 removes a semantic overclaim exposed by review and has exact-current CI/Security/SAST GREEN proof. Neither active branch is protected-main or release evidence.
- **Traceability:** the earlier #73 `PARTIAL` classification and #74 predecessor-head wording are superseded by this dated appendix only for their exact-current active-PR state. If either head moves, this evidence becomes historical immediately.

## Truth boundary

`IMPLEMENTED_ON_ACTIVE_PR` means the exact branch contains the stated behavior and exact-current proof; it does not mean shipped. A controlled Chromium runner is not the product browser adapter. A sampled Chromium process tree is not trusted whole-task ownership. An extension proposal grant is not an Agent action grant, and no current production adapter composes the two evaluators. Protected-main maturity changes only after dependency-ordered integration and fresh protected-main acceptance.
