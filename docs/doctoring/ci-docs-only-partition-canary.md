# Docs-only CI partition canary

This document is an executable canary for issue #279 and its owner repair PR #282. Its semantic delta is intentionally documentation-only so GitHub Actions can prove the exact-head partition without changing Rust, browser, policy, security, release, or workflow authority.

Acceptance is narrow and observable. On the unchanged canary head, `CI` must execute `Classify CI scope` and `Repository and documentation contracts`. The classifier must report a documentation-only change. `Rust contracts` and `Production coverage` must not execute Rust-heavy work for this delta. Queued, skipped predecessor, runner-less, cancelled, or status-only evidence is not a successful canary result.

The canary does not weaken the central required security/review workflows. It does not establish product release readiness or browser-runtime correctness; it only exercises the local CI trigger partition introduced for #279.
