# Agent Task cleanup-failure provenance

## Problem

Agent Task browser and profile cleanup wrappers preserve an earlier causal failure in Python exception chaining, but predecessor `d329f9e5dfbd5e4200eeec5fc40bf37d526fb77b` serialized only the wrapper `failure_type` into durable browser evidence. A page-observed/action failure followed by WebDriver-session cleanup failure, or a browser-pass failure followed by profile cleanup failure, therefore became indistinguishable from a cleanup-only failure once the process artifact was consumed.

Review `5151447157` records the exact-head finding. Test-first commit `70c710f00ad06a1437d74d10604820fb30272be2` requires the emitted failed-trial record to retain the bounded primary failure type and cleanup failure type while excluding hostile exception detail. Production commit `3f8f1cc15ec6c37c5b85e8cf6959c5d1c039b1fc` adds only closed exception-class metadata to the two cleanup wrappers and the Agent Task failure materializer. Commit `a297811635636942afa8f0d650f0ec3f9183dfca` binds the profile-cleanup regression to the same explicit primary-failure metadata used by production.

## Constraints

The artifact must not serialize `str(error)`, WebDriver response text, browser/page-controlled values, profile paths, ChromeDriver process diagnostics, credentials, or secret-shaped content. ChromeDriver startup/process classification remains owned by PR #148. Sandbox-helper workflow mechanics remain owned by the canonical `.github` path. Browser interaction, presentation apply/reset, URL stability, cleanup semantics, and the three-trial denominator must not change.

## Alternatives

Dropping the original browser failure was rejected because it destroys causal provenance after cleanup wraps the failure. Serializing exception messages or the full exception chain was rejected because remote, page-controlled, filesystem, or secret-bearing detail could cross the CI evidence boundary. Inferring a primary type from arbitrary chained exceptions at serialization time was rejected because cleanup-only and secondary-cleanup cases can have different chaining semantics.

## Decision

Cleanup wrappers retain `cleanup_error_type` and an optional `primary_error_type` captured at the point where the cleanup boundary already knows whether a primary browser failure exists. Failed Agent Task evidence publishes those closed class names as `cleanup_error_type` and `failure_cause_type`. Existing `AgentTaskSessionStartError` keeps its established bounded `failure_cause_type` behavior. No message text is added.

## Risk and effect

The additional fields expose only Python exception class names already used elsewhere in the evidence schema. This improves post-run RCA by distinguishing causal browser failure from secondary cleanup failure without broadening diagnostic authority. The repair does not establish real-browser GREEN; exact-head repository gates and pinned-Chromium execution remain independent acceptance evidence.

## Acceptance

The focused contract must prove both session-cleanup and profile-cleanup wrappers emit `failure_type`, `failure_cause_type`, and `cleanup_error_type`, and that hostile primary/cleanup messages do not appear in JSON output. Exact-head repository CI must pass before this repair is considered GREEN. Pinned Chromium must still complete the existing three-trial causal browser sequence before #299 can claim browser acceptance.
