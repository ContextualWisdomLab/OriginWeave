# WebDriver response diagnostic boundary

## Problem

The controlled MV3/Agent Task evidence runner consumes WebDriver Classic HTTP responses from the local ChromeDriver remote end. Before this repair, `_json_request` copied the complete body of an HTTP error and the remote `value.error` / `value.message` fields into `RuntimeError`. `_wait_for_driver` then copied the last exception string into its timeout error. Those values are useful for protocol conformance at the remote boundary, but they are not trustworthy CI log content: the W3C WebDriver error object explicitly contains implementation-defined `message` and `stacktrace` text and may carry additional `data`, including user-prompt text.

The same boundary applies to successful New Session capability data. W3C WebDriver defines `browserVersion` as a standard capability returned by the remote end to identify the user-agent version. OriginWeave must compare that value against the pinned Chrome version, but a mismatched remote value is comparison state rather than CI diagnostic payload.

The browser evidence lane therefore had a provenance mismatch. Remote response data was allowed to decide failure and also become operator-visible diagnostic text without an explicit redaction boundary.

## Constraints

- Keep non-success HTTP status and defensive error-shaped JSON responses fail-closed.
- Keep the existing response-size bound and JSON/object validation.
- Keep exact equality with `PINNED_CHROME_VERSION` mandatory in both MV3 and Agent Task browser passes.
- Do not serialize the remote `browserVersion` value into version-mismatch diagnostics.
- Do not reinterpret remote response text or capabilities as browser policy, product authority, or success evidence.
- Do not add ChromeDriver process/startup reason classification here; PR #148 remains the canonical owner of that diagnostic semantics.
- Do not change Chrome arguments, sandboxing, browser version, workflow activation, retry behavior, or the three-trial denominators.
- Do not weaken browser-observed action, post-condition, URL, session/profile, fixture-cleanup, or success-publication evidence.

## Alternatives

1. Preserve raw WebDriver bodies/messages or mismatched capability values in CI and attempt pattern-based secret filtering. Rejected because remote fields are open-ended data; a denylist cannot establish a closed disclosure boundary.
2. Allowlist W3C error codes and publish them. Rejected for this slice because error-code interpretation overlaps the richer #148 protocol/startup diagnostic authority and is unnecessary to preserve fail-closed behavior.
3. Keep only the bounded local HTTP status for transport-level failures, use fixed diagnostics for remote response/capability failures, and retain the remote values only for local comparison. Selected because it preserves each failure decision while preventing remote payload serialization and keeps this runner generic.

## Decision and test-first evidence

Test-first commit `8faef5967d8df770e9fc84ba358846ecd4dd1062` extends `tests/test_mv3_page_diagnostic_redaction_contract.py` with three hostile-response contracts:

- a non-success WebDriver HTTP body containing `buyer-secret-marker-must-not-reach-ci` must fail as `WebDriver HTTP request failed with status 500` without retaining the body;
- a defensive error-shaped JSON object on a success-status response must fail as `WebDriver command failed` without retaining the remote `error` or `message` fields;
- driver-readiness timeout must be `ChromeDriver did not become ready` even if the last request exception contains the hostile marker.

Exact predecessor production still serialized those values, so this commit is a source-semantic RED. Draft admission prevents representing it as a hosted executed RED. Corrective test-only commit `356764db9ef6fe53c0bba2f02c3f01c58a80d845` makes the second case explicit rather than inaccurately describing an HTTP 200 response as a conforming W3C error response.

Production commit `4f09563520c9ba8565fa2c269db3a3de45fa7ca0` makes the minimum generic response repair: HTTP failures retain only the numeric status, an error-shaped decoded JSON object uses one fixed command-failure diagnostic, and readiness timeout no longer interpolates the last exception. Response bounds, JSON validation and failure behavior remain intact.

Test-first commit `d682b641cfa30e0040c1b445c6aa45d211cdf0a8` then injects the hostile marker as the remote `browserVersion` in both browser-pass variants. The predecessor still performed the required pinned-version comparison but echoed the mismatched capability into `RuntimeError`, establishing the next source-semantic RED. Production commit `32953588084d72eb669ed53b02ccc7d5264dc00e` preserves both exact equality checks and replaces only their diagnostics with `unexpected Chrome version` and `unexpected Agent Task Chrome version`.

## Risks and follow-up

The tighter diagnostic vocabulary intentionally removes remote troubleshooting text from ordinary CI. When ChromeDriver startup/process causality is needed, the bounded diagnostic mechanism owned by PR #148 must supply typed evidence rather than reopening raw remote-text logging here.

This change is not browser acceptance. Exact-head repository gates and sandbox-enabled pinned-Chromium three-trial execution still depend on the current workflow/sandbox owner path in issue #212. A skipped Draft workflow is not GREEN evidence.

## Primary standard

World Wide Web Consortium. (2026, July 2). *WebDriver* (Working Draft). https://www.w3.org/TR/webdriver2/

Section 6.6 defines conforming WebDriver errors as HTTP 4xx/5xx responses with a JSON `value` object containing an error code plus implementation-defined `message` and `stacktrace`, with optional additional `data`. Section 7 defines `browserVersion` as a standard string capability identifying the user-agent version and states that the remote end uses capabilities to describe the session feature set. OriginWeave therefore treats descriptive error fields and remote capability values as untrusted observations: they can cause a failed result but are not serialized into the CI diagnostic surface.
