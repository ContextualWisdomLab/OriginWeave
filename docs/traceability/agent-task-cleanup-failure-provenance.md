# Agent Task cleanup-failure provenance

## Problem

Agent Task browser and profile cleanup wrappers preserve an earlier causal failure in Python exception chaining, but predecessor `d329f9e5dfbd5e4200eeec5fc40bf37d526fb77b` serialized only the wrapper `failure_type` into durable browser evidence. A page-observed/action failure followed by WebDriver-session cleanup failure, or a browser-pass failure followed by profile cleanup failure, therefore became indistinguishable from a cleanup-only failure once the process artifact was consumed.

Review `5151447157` records the first exact-head finding. Test-first commit `70c710f00ad06a1437d74d10604820fb30272be2` requires the emitted failed-trial record to retain the bounded primary failure type and cleanup failure type while excluding hostile exception detail. Production commit `3f8f1cc15ec6c37c5b85e8cf6959c5d1c039b1fc` adds only closed exception-class metadata to the two cleanup wrappers and the Agent Task failure materializer. Commit `a297811635636942afa8f0d650f0ec3f9183dfca` binds the profile-cleanup regression to the same explicit primary-failure metadata used by production. Commit `d31a8d5e54a5b14fbc5e3a17980f97ec40b43c81` corrects the focused harness so cleanup exceptions and the `main()` materializer come from the same `runpy` module instance; otherwise Python class identity would make an artificial cross-module exception miss the production `isinstance` branch.

Review `5151777477` found one remaining nested case: browser/action failure A can be wrapped by WebDriver-session cleanup failure B and then by profile cleanup failure C. The predecessor durable record retained only `BrowserProfileCleanupError`, `BrowserSessionCleanupError`, and C's class name, losing the original A type and B's cleanup type. Test-first `2320dd7f7313b443a87f5735266330ddd61f6053` requires the three-level chain to retain the root failure type plus both cleanup-stage type names while rejecting hostile messages. Production `08133c3166dd899fe5925ce96781023e1dce5ed7` propagates only those already-bounded class names through `BrowserProfileCleanupError` and the existing Agent Task materializer.

## Constraints

The artifact must not serialize `str(error)`, WebDriver response text, browser/page-controlled values, profile paths, ChromeDriver process diagnostics, credentials, or secret-shaped content. ChromeDriver startup/process classification remains owned by PR #148. Sandbox-helper workflow mechanics remain owned by the canonical `.github` path. Browser interaction, presentation apply/reset, URL stability, cleanup semantics, and the three-trial denominator must not change.

## Alternatives

Dropping the original browser failure was rejected because it destroys causal provenance after cleanup wraps the failure. Serializing exception messages or the full exception chain was rejected because remote, page-controlled, filesystem, or secret-bearing detail could cross the CI evidence boundary. Inferring a primary type from arbitrary chained exceptions at serialization time was rejected because cleanup-only and secondary-cleanup cases can have different chaining semantics. Flattening a nested cleanup chain to the wrapper class name was rejected because it discards the already-bounded root and session-cleanup types needed to distinguish A -> B -> C from an unrelated wrapper failure. Keeping separate `runpy` module instances in the contract was rejected because identical-looking exception classes from separate executions are distinct Python class objects and would test an impossible production boundary rather than the runner's real materializer.

## Decision

`BrowserSessionCleanupError` retains `cleanup_error_type` and optional `primary_error_type`. `BrowserProfileCleanupError` retains its own `cleanup_error_type`; when its primary error is the local `BrowserSessionCleanupError`, it also preserves that wrapper's bounded `cleanup_error_type` as `session_cleanup_error_type` and forwards the wrapper's bounded root `primary_error_type` when one exists. Failed Agent Task evidence publishes the root as `failure_cause_type`, the session cleanup as optional `session_cleanup_error_type`, and the outer cleanup as `cleanup_error_type`. Existing one-stage cleanup records remain compatible, and `AgentTaskSessionStartError` keeps its established bounded `failure_cause_type` behavior. No message text is added.

## Risk and effect

The additional field exposes only a Python exception class name already present inside the bounded cleanup wrapper; it does not retain the exception object or any raw diagnostic. This improves post-run RCA by distinguishing the root browser/action failure, WebDriver-session cleanup failure, and profile-cleanup failure without broadening diagnostic authority. The repair does not establish real-browser GREEN; exact-head repository gates and pinned-Chromium execution remain independent acceptance evidence.

## Acceptance

The focused contract must prove single-stage session/profile cleanup still emit `failure_type`, `failure_cause_type`, and `cleanup_error_type`. The nested contract must additionally prove `browser/action A -> session cleanup B -> profile cleanup C` emits the closed root type, `session_cleanup_error_type`, and final `cleanup_error_type`, while hostile primary/session/profile messages remain absent from JSON output. Contracts must exercise the same loaded runner namespace as the failure materializer. Exact-head repository CI must pass before this repair is considered repository GREEN. Pinned Chromium must still complete the existing three-trial causal browser sequence before #299 can claim browser acceptance.
