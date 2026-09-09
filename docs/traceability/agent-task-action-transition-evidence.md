# Agent Task action-transition evidence

- **Status:** Active-PR evidence; not protected-main shipped truth
- **Canonical browser-domain owner:** OriginWeave
- **Active PR:** #288
- **Protected base:** `main@87c4daa1830bac5a5228b6036752ad5633232085`
- **Initial causal observation repair:** `8b7aa28ecf7acb1e3f3b2dcadd4cb3cbf59ea01f`
- **Initial causal acceptance repair:** `e1dd50999cd3a52977179047e8d5d77a2e85aef9`
- **Immediate pre-click RED:** `07540f0cdb3178998d305382d9176cccaeabce57`
- **Immediate pre-click repair:** `942e4c1a44119384d01ee4c7ec4168e6c5ab38b5`
- **Accepted-outcome URL RED:** `4306f38c45f65a7fba7ccd842ae42b2ed4646e29`
- **Accepted-outcome URL repair:** `cac13ab79cd41ca55315622c3148126bee5643f8`
- **Typed-input observation RED:** `ac78a46dd550e64993e48571d72eee2d8bf2d0f5`
- **Typed-input observation repair:** `dfbc0f13e0e7c4f381fc333f95c41127583bfc7a`
- **Clear post-condition RED:** `7081d021def0d740a86771b5b1b77ff36885e433`
- **Clear post-condition repair:** `1fa16927a33cb0a151aa479587bb54654e48b538`
- **Successful evidence-double alignment:** `f6c43aaad5b1c0acf7e3ca04a3f9ec872d08c83b`
- **Workflow/sandbox owner:** #212
- **WebDriver protocol-diagnostic owner:** #148

## Problem

The controlled Agent Task lane already used browser-computed role/name evidence, native WebDriver clear/type/click commands, page-observed transition state, exact synthetic text echo, URL stability, and profile cleanup. The first causal repair added an `idle` / `idle` observation before the input sequence, preventing a fixture that was already successful at navigation time from satisfying the buyer gate.

That was still insufficient to attribute the final transition specifically to the submit click. A fixture regression could remain idle at the first observation, mutate `#task-result` while the WebDriver value command types the synthetic input, and then present `submitted` plus the expected echo before the click. The later post-condition would still look successful even though the click did not cause the transition.

A separate timing gap remained after that repair: the runner sampled `post_submit_url` immediately after click, before it verified the submitted state and exact echo. A click handler could synchronously satisfy those post-conditions, schedule a navigation, let the early URL sample remain unchanged, and still be accepted without proving that the controlled URL remained stable through the accepted outcome boundary.

The typing path also needed an independent browser observation. The runner issued the WebDriver element value command and later checked the submitted result echo, but a controlled-fixture regression could hard-code the expected output and allow a failed or partial send-keys operation to be accepted from command acknowledgement alone. The typed-value repair therefore reads the input element's `value` property after typing and before click.

One earlier interaction in the same sequence remained acknowledgement-only. The controlled fixture deliberately starts `#task-text` with the non-empty synthetic value `synthetic order 42`. The runner issued Element Clear and immediately issued Element Send Keys without observing the value after clear. A no-op or incomplete clear could therefore be accepted from command completion even though the fixture provides a deterministic post-condition that can prove the clear action itself.

## Decision

The controlled fixture has one canonical unsuccessful result state: `#task-result` is `data-state="idle"` and rendered text `idle`. The runner observes that state twice through WebDriver: once before clear/type and again after typing plus submit-target semantic verification, immediately before the native click. Both observations reuse `_validate_agent_task_pre_action_state`, so unexpected page-controlled values fail closed without being serialized into diagnostics.

The controlled text input also has explicit action post-conditions. Immediately after Element Clear, the runner uses Get Element Property on `value` and requires the browser-observed value to be the empty string before any send-keys command is issued. `_validate_agent_task_cleared_value` emits only `Agent Task clear verification failed` on mismatch; the observed input value is not serialized. Successful evidence carries `clear_value_verified: true`, and surface completeness requires that witness in every successful trial.

After Element Send Keys, the runner again uses Get Element Property for the same controlled input and requires the browser-observed `value` to equal `AGENT_TASK_INPUT_VALUE`. `_validate_agent_task_typed_value` emits only `Agent Task typed input verification failed` on mismatch. Successful evidence carries `input_value_verified: true`, and repeatability surface completeness requires that witness independently of the clear witness.

Only after the empty-after-clear witness, typed-value witness, and second browser-observed idle baseline succeed does the runner issue the WebDriver click. The runner retains the immediate post-click URL check, then verifies `data-state="submitted"` and the exact synthetic echo, and finally reads the browser URL again before returning successful evidence. `url_unchanged` is true only when both post-click URL observations equal the original controlled fixture URL. The final mismatch uses the closed diagnostic `Agent Task URL changed before accepted outcome` and does not serialize the observed page URL.

Successful trial evidence therefore carries `pre_action_baseline_verified: true`, `clear_value_verified: true`, `input_value_verified: true`, and `pre_click_baseline_verified: true`; `_agent_task_surfaces_complete` requires all four witnesses in every successful trial. The accepted observed sequence is `idle before input → native clear → browser-observed empty value → native type → browser-observed typed value → idle immediately before click → click → immediate URL check → submitted/exact echo → accepted-outcome URL check`, rather than inferring success from command acknowledgement, from a post-condition that may already have been true, or from a URL sample that precedes the accepted outcome.

The baseline values, input properties, and observed URLs are used only for local comparison. Unexpected page-controlled state, text, input values, or URLs are never serialized into CI diagnostics.

## Test-first evidence

The regression sequence is intentionally non-destructive:

- `b9707975a605347b573b992cfe178150feda6a95` introduced the original causal-transition contract.
- `651d7fe89a1e6ebd811607683eb50f3b17a5822e` pinned the fixture's actual `idle` / `idle` baseline.
- `4d61c2f82a82048727980f6638b7e95e93699fd6` first required the baseline witness to propagate into per-trial evidence without changing the existing gate.
- `8b7aa28ecf7acb1e3f3b2dcadd4cb3cbf59ea01f` added the first runner observation repair.
- `42a9a129ff80edc698c3c097a049246d538c63bc` strengthened the regression so a trial lacking that baseline witness cannot satisfy surface completeness.
- `ed6af6ebf825a1571f16aaf1bc1d1bfdea4327a4` aligned the then-current successful-trial doubles.
- `e1dd50999cd3a52977179047e8d5d77a2e85aef9` made `pre_action_baseline_verified is True` mandatory in `_agent_task_surfaces_complete`.
- `07540f0cdb3178998d305382d9176cccaeabce57` added the second-baseline RED after the WebDriver value command and before click.
- `942e4c1a44119384d01ee4c7ec4168e6c5ab38b5` repaired that gap by re-observing the existing result element immediately before click and requiring `pre_click_baseline_verified`.
- `3d3166ec1e3a7c5aaee1f2dae92f09a7acc294f4` and `efca7d69c3bbc459bc218b9142a6dd4e58828076` aligned successful evidence doubles with the stronger two-baseline contract instead of weakening the predicate.
- `4306f38c45f65a7fba7ccd842ae42b2ed4646e29` added the accepted-outcome URL-order RED: after the exact echo is accepted, a browser URL read must still occur before successful evidence is returned.
- `cac13ab79cd41ca55315622c3148126bee5643f8` performed the minimum URL repair by retaining the immediate post-click URL check and adding a second URL equality observation after submitted-state/exact-echo verification.
- `ac78a46dd550e64993e48571d72eee2d8bf2d0f5` added the typed-input RED: command acknowledgement is insufficient until the browser-observed input `value` property is read and validated before click, and surface completeness requires the resulting witness.
- `dfbc0f13e0e7c4f381fc333f95c41127583bfc7a` implemented the typing repair with Get Element Property, a closed/non-echoing validator, `input_value_verified`, trial propagation, and mandatory completeness.
- `7081d021def0d740a86771b5b1b77ff36885e433` added the clear-action RED: the controlled fixture's preloaded input makes Element Clear independently observable, so a browser-observed empty `value` must occur after `/clear` and before `/value`, and surface completeness must require `clear_value_verified`.
- `1fa16927a33cb0a151aa479587bb54654e48b538` implemented the minimum clear repair with Get Element Property, a closed/non-echoing validator, trial propagation, and mandatory completeness without changing the fixture or browser action sequence.
- `f6c43aaad5b1c0acf7e3ca04a3f9ec872d08c83b` aligned existing successful-trial doubles with both already-required `input_value_verified` and the new `clear_value_verified` witness rather than weakening the acceptance predicate.

These commits do not change browser version, trial denominator, semantic-target checks, click/post-condition semantics, cleanup, workflow, sandbox configuration, extension-isolation semantics, or #148 protocol-diagnostic authority.

Because #288 is Draft, CI and Manifest V3 Compatibility may skip before executing this exact lineage. A source-semantic/test-first RED or code inspection is not a substitute for a fresh pinned-Chromium run. Browser acceptance still requires the #212 workflow/sandbox owner path to execute the unchanged three-trial lane on the exact successor head.

## Standards traceability

The latest published WebDriver draft remains **W3C Working Draft, 2 July 2026** as of 9 September 2026. It defines WebDriver as an out-of-process browser-control protocol and separately defines Get Element Property, Element Clear, Element Send Keys, element-state retrieval, and current-URL retrieval commands. The controlled Agent Task lane uses those commands as transport-level observation and interaction mechanisms; OriginWeave's stronger causal acceptance rule is a product evidence invariant layered above the protocol. WebDriver command completion alone does not establish OriginWeave task success.

For the clear boundary, the runner performs Element Clear and then separately performs Get Element Property for `value`; it does not infer an empty value from successful completion of the clear command. It repeats Get Element Property after Element Send Keys to prove the controlled typed value before click. Get Current URL is observed before action, immediately after click, and again after the accepted state/echo observation. These reads bind interaction evidence to browser-observed state rather than to command-completion instants.

### APA 7th

World Wide Web Consortium. (2026, July 2). *WebDriver* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver2-20260702/

## Remaining acceptance

This repair does not claim a production Agent Task adapter, WebDriver BiDi authority translation, semantic-node authority, policy authorization, or sandbox-enabled Chromium GREEN. Those remain separate bounded responsibilities. The next browser acceptance must preserve the protected workflow generation, least-privilege sandbox setup, Chrome/ChromeDriver `150.0.7871.129`, three independent trials, both idle baselines, the browser-observed empty-after-clear witness, browser-observed typed-value witness, both post-click URL observations, page-derived diagnostic redaction, session/profile cleanup, and exact-head evidence.