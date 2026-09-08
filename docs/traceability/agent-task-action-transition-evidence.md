# Agent Task action-transition evidence

- **Status:** Active-PR evidence; not protected-main shipped truth
- **Canonical browser-domain owner:** OriginWeave
- **Active PR:** #288
- **Protected base:** `main@87c4daa1830bac5a5228b6036752ad5633232085`
- **Causal observation repair:** `8b7aa28ecf7acb1e3f3b2dcadd4cb3cbf59ea01f`
- **Causal acceptance repair:** `e1dd50999cd3a52977179047e8d5d77a2e85aef9`
- **Workflow/sandbox owner:** #212
- **WebDriver protocol-diagnostic owner:** #148

## Problem

The controlled Agent Task lane already used browser-computed role/name evidence, native WebDriver clear/type/click commands, a page-observed `submitted` state, exact synthetic text echo, URL stability, and profile cleanup. That sequence still admitted one evidence-validity ambiguity: it did not prove that the claimed success surfaces were false before the native action.

A pre-fired fixture or regression could therefore expose `data-state="submitted"` and the expected result text before the click. The existing post-action checks would then confirm a successful-looking state without establishing that the WebDriver action caused the transition. A command acknowledgement was not being treated as success, but the evidence still lacked an observed causal baseline.

## Decision

The controlled fixture has one canonical pre-action state: `#task-result` is `data-state="idle"` and rendered text `idle`. Before typing or clicking, the runner reads both surfaces through WebDriver and requires that exact baseline with `_validate_agent_task_pre_action_state`.

Only after that browser-observed baseline succeeds does the runner continue with the existing native clear/type/click sequence. The post-action acceptance remains unchanged: URL stability, `data-state="submitted"`, and exact echo of the synthetic task input must all be observed. Successful trial evidence carries `pre_action_baseline_verified: true`, and `_agent_task_surfaces_complete` now requires that witness in every successful trial. A post-condition-only record can no longer satisfy the buyer gate.

The baseline values are used only for local comparison. Unexpected page-controlled state or text is never serialized into CI diagnostics; the bounded failure is `Agent Task pre-action baseline was already satisfied`.

## Test-first evidence

The regression sequence is intentionally non-destructive:

- `b9707975a605347b573b992cfe178150feda6a95` introduced the causal-transition contract.
- `651d7fe89a1e6ebd811607683eb50f3b17a5822e` pinned the fixture's actual `idle` / `idle` baseline.
- `4d61c2f82a82048727980f6638b7e95e93699fd6` first required the baseline witness to propagate into per-trial evidence without changing the existing gate.
- `8b7aa28ecf7acb1e3f3b2dcadd4cb3cbf59ea01f` added the minimum runner observation repair: two pre-action observations, one closed validator, and one credential-free evidence field.
- `42a9a129ff80edc698c3c097a049246d538c63bc` strengthened the regression so a trial lacking the baseline witness must fail surface completeness.
- `ed6af6ebf825a1571f16aaf1bc1d1bfdea4327a4` aligned the existing successful-trial test doubles with that explicit witness.
- `e1dd50999cd3a52977179047e8d5d77a2e85aef9` made `pre_action_baseline_verified is True` a mandatory `_agent_task_surfaces_complete` condition.

These commits do not change browser version, trial denominator, native action sequence, URL check, post-condition, cleanup, workflow, sandbox configuration, or #148 protocol-diagnostic authority.

Because #288 is Draft, CI and Manifest V3 Compatibility may skip before executing this exact lineage. A source-semantic/test-first RED or code inspection is not a substitute for a fresh pinned-Chromium run. Browser acceptance still requires the #212 workflow/sandbox owner path to execute the unchanged three-trial lane on the exact successor head.

## Standards traceability

The current published WebDriver 2 draft is **W3C Working Draft, 2 July 2026**. It defines WebDriver as an out-of-process browser-control protocol and separately defines element interaction and element-state retrieval commands. The controlled Agent Task lane uses those commands as transport-level observation and interaction mechanisms; OriginWeave's stronger causal acceptance rule is a product evidence invariant layered above the protocol. WebDriver command completion alone does not establish OriginWeave task success.

The `GET /session/{session id}/element/{element id}/text` endpoint is the standard Get Element Text command. The runner uses element retrieval before and after the native action to demonstrate an observed state transition, rather than inferring success from the click response.

### APA 7th

World Wide Web Consortium. (2026, July 2). *WebDriver* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver2-20260702/

## Remaining acceptance

This repair does not claim a production Agent Task adapter, WebDriver BiDi authority translation, semantic-node authority, policy authorization, or sandbox-enabled Chromium GREEN. Those remain separate bounded responsibilities. The next browser acceptance must preserve the protected workflow generation, least-privilege sandbox setup, Chrome/ChromeDriver `150.0.7871.129`, three independent trials, page-derived diagnostic redaction, session/profile cleanup, and exact-head evidence.
