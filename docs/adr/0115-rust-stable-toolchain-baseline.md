# ADR 0115: Rust stable toolchain baseline

- **Status:** Proposed; becomes binding only after independent review and protected merge
- **Date:** 2026-09-09

## Context

OriginWeave keeps the supported Rust compiler on an exact patch-version pin so local verification, GitHub Actions, and release evidence do not silently move with the upstream `stable` channel. Protected main currently governs Rust `1.97.1`. Dependabot PR #301 proposes `1.98.1`, but its predecessor exact head changed only `rust-toolchain.toml`; native CI run `34320596205` then failed at the repository contracts that intentionally still required `1.97.1`.

Rust `1.98.1` is the current stable point release as of 2026-09-09. The Rust project states that it fixes a vtable-generation miscompilation that can create a null function pointer and undefined behavior. Advancing the exact baseline is therefore preferable to retaining a stale compiler, but the update must remain a governed compiler-authority change rather than a manifest-only dependency bump.

Branch coverage is a separate compiler concern. OriginWeave currently uses the independently date-pinned `nightly-2026-08-18` because Rust branch coverage through `cargo-llvm-cov` remains nightly-only. Moving the stable compiler does not authorize changing that nightly pin in the same change without separate evidence.

## Decision drivers

- keep compiler provenance reproducible and reviewable;
- consume the current stable point release and its upstream miscompilation fix;
- preserve exact repository contracts instead of weakening them when Dependabot moves the manifest;
- keep stable compilation and nightly branch-coverage authority distinct;
- require one coherent exact-head verification story before protected-main promotion.

## Assumptions and authority boundaries

This ADR governs OriginWeave's supported stable Rust compiler version. It does not change browser, network, policy, evidence, LLM-provider, or release authority. It does not authorize product writers to edit canonical workflow-owner code merely to make a leaf PR green. Workflow/toolchain materialization remains an Actions control-plane concern and must be repaired through its authorized owner path when a checked-in workflow hard-codes an older compiler.

Historical plans, specifications, and evidence that truthfully record Rust `1.97.1` at the time they were written remain historical evidence and are not rewritten merely to make repository-wide text search uniform.

## Options considered

### Keep Rust 1.97.1

Rejected as the target baseline. It is no longer the current stable point release and lacks the upstream `1.98.1` vtable-miscompilation fix.

### Use floating `stable`

Rejected. It would make the compiler used by local or automated verification depend on execution time rather than a reviewed repository change.

### Move stable and branch-coverage nightly together

Rejected for this change. Stable compilation and nightly-only branch coverage have different compatibility risks and evidence. Coupling them would introduce an unnecessary second variable.

### Pin Rust 1.98.1 exactly

Selected. The manifest, repository contracts, governing documentation, and exact-head verification are advanced together while the branch-coverage nightly stays unchanged.

## Decision

OriginWeave will use Rust `1.98.1` as its exact supported stable compiler baseline after this Proposed ADR is independently reviewed and merged to protected `main`.

The governed update requires all of the following on one exact candidate lineage:

1. `rust-toolchain.toml` pins `1.98.1` rather than floating `stable`;
2. repository contract tests continue to assert the exact supported patch version;
3. `AGENTS.md`, `README.md`, toolchain doctoring, the ADR index, and `CHANGELOG.md` describe the same proposed baseline;
4. Python repository contracts, canonical formatting, locked workspace tests, strict Clippy, rustdoc, and exact production function/line/region/branch coverage execute successfully on the exact head;
5. workflow materialization does not retain a contradictory stable compiler pin in an authoritative execution path; any such workflow repair is performed by the canonical workflow owner rather than duplicated in product code;
6. the independently date-pinned `nightly-2026-08-18` branch-coverage compiler is unchanged by this ADR.

## Consequences

The exact stable compiler becomes current without sacrificing reproducibility. Dependabot can continue proposing future stable updates, but a manifest-only version change is expected to fail the duplicated baseline contracts until the governed change is intentionally completed.

The change may expose new compiler diagnostics, formatting output, lints, or code-generation differences. Those are compatibility findings to repair or reject explicitly; they are not grounds for weakening Clippy, rustdoc, tests, or coverage gates.

## Failure and degraded behavior

If Rust `1.98.1` causes a reproducible regression in OriginWeave or a required tool, keep the candidate unmerged and record the exact failure. Do not fall back to floating `stable`, skip the failing lane, or reinterpret a predecessor's GREEN result as evidence for the changed compiler.

A workflow that installs or invokes a contradictory stable version makes compiler provenance incomplete even if ordinary Cargo commands happen to honor `rust-toolchain.toml`. That state remains a merge blocker until the authorized workflow owner resolves it and the exact-head gates are rerun.

## Security / privacy / governance impact

No OriginWeave runtime permission or data boundary changes. The material governance effect is compiler supply/provenance control: the project deliberately consumes a point release containing an upstream miscompilation fix while preserving an auditable exact version. This ADR does not claim a CVE or a deployed security incident.

## Tests and acceptance evidence

The predecessor #301 exact head `ddff2b88a8557374237f916cfc3beaabf5629760` is the RED witness: CI `34320596205` failed in `Check Python repository contracts`, and the Rust formatting/test/Clippy/rustdoc steps did not run. Production coverage success on that predecessor does not override the failed Rust-contract job.

Acceptance requires fresh terminal evidence for the repaired exact head, including the repository contracts, formatting, locked workspace tests, strict Clippy, rustdoc, exact 100% production coverage, Security Scan, Semgrep, and all required central checks. Skipped, cancelled, queued-only, predecessor-head, or status-only evidence is not acceptance.

## Migration and rollback

Migration is a single reviewed compiler-baseline change: adopt the `1.98.1` manifest bump together with current contracts and documentation, then integrate the workflow-owner pin repair before protected-main acceptance.

If exact-head compatibility fails for a reason that cannot be causally repaired, revert the entire proposed baseline change back to the last protected exact compiler pin. Do not leave documentation/tests at one version and the manifest/workflow at another.

## Open follow-ups

- Complete the authorized workflow-owner repair for any remaining hard-coded stable `1.97.1` materialization before merging the baseline.
- Re-run all exact-head required gates after that owner change is adopted.
- Revisit the separate date-pinned branch-coverage nightly only with its own compatibility evidence.

## Supersession / reversal conditions

A future stable compiler may supersede `1.98.1` only through another explicit exact-version decision with updated contracts, current primary-source evidence, and complete exact-head verification. A rollback may temporarily restore the last known-good protected baseline when a concrete regression is proven.

## References

Rust Project Developers. (2026, September 3). *Announcing Rust 1.98.1*. Rust Blog. https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/

Rust Project Developers. (2026, September 3). *Version 1.98.1 (2026-09-03)*. Rust release notes. https://doc.rust-lang.org/stable/releases.html#version-1981-2026-09-03
