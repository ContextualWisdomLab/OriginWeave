# Browser Session Cargo host-config execution authority traceability

## Decision

Repository-owned Cargo configuration must not be able to introduce unreviewed host-side compiler, linker, runner, rustdoc, or build-script-link authority for Browser Session production builds. `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected Cargo compiler/toolchain/execution and external-input authority; this dossier adds no second Cargo package/source topology scanner.

The contract therefore fails closed on authority-bearing `[host]` and `[host.<host-tuple>]` settings: `linker`, `runner`, authority-extending `rustflags` / `rustdocflags`, and nested host build-script `links` overrides. Because Cargo's host configuration reuses `TargetConfig`, a nested table under generic `[host]` cannot be assumed harmless merely because its key is not the current host tuple: that map is also the shape used for `links_overrides`. Harmless host flags such as `-C opt-level=2` and `--document-private-items` remain permitted.

## Problem and buyer/security effect

Cargo's nightly `host-config` feature gives Git-owned configuration a distinct path for artifacts compiled or executed on the build host, including build scripts and other host artifacts. Cargo documents generic `[host]` and host-tuple-specific tables, with host-tuple settings taking precedence, and documents `host.runner` as the wrapper used to execute host build targets such as build scripts. Cargo's current `TargetConfig` is the configuration type for `[target]` or `[host]` and exposes `rustflags`, `rustdocflags`, `linker`, `runner`, and `links_overrides`; a links override suppresses the matching package build script and substitutes configured build output.

Before this generation the Browser Session compiler-authority owner examined `[build]`, `[target]`, repository environment/config inclusion, unstable toolchain selectors, and profile rustflags/codegen backends, but ignored `[host]`. A reviewed repository could therefore pre-position host linker/runner/compiler-input authority that becomes effective when nightly `-Zhost-config` / `-Ztarget-applies-to-host` semantics are selected. Treating the feature as currently inactive would be mutable invocation-state trust rather than source provenance.

The first links-override repair still classified nested maps only after descending into a host-specific table. That left an ambiguity at generic `[host]`: a table such as `[host.review_bypass]` could be interpreted as a build-script `links` override even though `review_bypass` is not a host tuple. Allowing it because the key did not look like the current architecture would make the guard depend on an unstable parser distinction rather than the authority-bearing `TargetConfig.links_overrides` contract.

## RED → repair evidence

- **Structural RED `960d361a37d942937f9d2d88f9cd745265a77a90`** added a focused supplemental contract that calls the canonical compiler-authority owner and proves that generic host linker, host-tuple runner, and authority-extending host rustflags were not rejected, while unrelated host rustflags remain a control.
- **Minimal canonical repair `da6b14de3366c235b1b4c10d340e68477cd41c2a`** added `_configured_host_execution_authority()` to the existing compiler-authority owner. It reused existing target execution keys and rustc/linker/input classifiers rather than rediscovering Cargo package/source topology.
- Review of Cargo's current `TargetConfig` surface exposed two valid omissions in the first repair: host `rustdocflags` share the same target configuration structure, and host target configuration can carry `links_overrides` that replace build-script output. **Review-driven RED `66d8a535befaade759eba6f3f464499792d6bfaf`** added hostile rustdoc external-input and host-tuple links-override cases plus an unrelated rustdoc control.
- **Causal follow-up repair `d2830ddc28e8d6e70af54ae4dec09fd8d40dd3ec`** reused the existing rustdoc classifiers and treated nested host-tuple build-script override tables as explicit authority. The change stayed in the canonical compiler-authority helper.
- A fresh hostile case then exposed the generic-host ambiguity: **RED `6574faa0d7012dec8fad0f0a563599af90e89634`** added `[host.review_bypass]` with `rustc-link-search`, which the depth-gated implementation did not report.
- **Minimal repair `edfbd7402d7653374ad7ab5556b3952f73d0ad89`** removed that depth dependency. Any nested map under the selected `[host]` configuration that is not a known direct execution key or the typed `rustflags` / `rustdocflags` surface now fails closed as potential `links` build-script override authority, while the focused harmless generic host controls continue to pass. The canonical compiler-authority owner remains the only implementation writer.

## Alternatives considered

1. **Ignore `[host]` until `-Zhost-config` appears in repository configuration.** Rejected. Invocation flags and Cargo's unstable-feature activation are separate mutable execution surfaces; Git-owned latent authority must not become pre-authorized merely because the current invocation does not activate it.
2. **Reject every `[host]` table or every host rustflag.** Rejected. This would conflate deterministic optimization/documentation settings with execution/input authority and would make the guard broader than the owned security invariant.
3. **Treat every `[host.<key>]` as a host tuple and allow unknown tuple names.** Rejected. `TargetConfig` itself owns `links_overrides`; an unknown nested map is therefore authority-bearing until Cargo's selected host-configuration interpretation proves otherwise. Fail-open tuple guessing would recreate the bypass.
4. **Create a second Cargo scanner in the focused test.** Rejected. The focused test only constructs hostile/config-control fixtures and delegates the decision to the canonical compiler-authority owner.

## Invariants

- `test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler/rustdoc/toolchain/linker execution and external-input authority.
- `[host]` linker/runner authority is fail closed.
- Authority-extending host `rustflags` and `rustdocflags` are fail closed using the same classifier semantics as `[build]` / `[target]`.
- Nested host configuration maps are fail closed as potential `links` overrides because they can suppress a package build script and inject replacement build output, including native link search/library material.
- Non-authority host flags remain admissible controls; the guard is not a blanket ban on host configuration.

## Remaining evidence and risk

This generation is source-structural evidence until the exact pull-request head receives executable hosted repository/security checks. It does not claim protected-main integration, release readiness, whole-PR review closure, or owned 100% Docstring/rustdoc/Test/Edge Case Coverage. Ambient user/global Cargo configuration, CLI `--config`, environment variables, and toolchain selection are separate invocation/runtime provenance surfaces and are not made trustworthy by this repository-owned config guard. Those surfaces belong in the CI/release execution-environment contract rather than by extending this Git-source topology scanner.

## Primary references

Cargo Team. (2026). *Unstable Features: target-applies-to-host and host-config*. The Cargo Book, nightly documentation. https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#host-config

Cargo Team. (2026). *TargetConfig*. Cargo 1.100.0-nightly rustdoc. https://doc.rust-lang.org/nightly/nightly-rustc/cargo/context/target/struct.TargetConfig.html

Cargo Team. (2026). *Configuration*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/config.html
