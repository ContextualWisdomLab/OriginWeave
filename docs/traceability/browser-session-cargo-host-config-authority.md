# Browser Session Cargo host-config execution authority traceability

## Decision

Repository-owned Cargo configuration must not be able to introduce unreviewed host-side compiler, linker, runner, rustdoc, or build-script-link authority for Browser Session production builds. `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected Cargo compiler/toolchain/execution and external-input authority; this dossier adds no second Cargo package/source topology scanner.

The contract therefore fails closed on authority-bearing `[host]` and `[host.<host-tuple>]` settings: `linker`, `runner`, authority-extending `rustflags` / `rustdocflags`, and host-tuple `links` build-script overrides. Harmless host flags such as `-C opt-level=2` and `--document-private-items` remain permitted.

## Problem and buyer/security effect

Cargo's nightly `host-config` feature gives Git-owned configuration a distinct path for artifacts compiled or executed on the build host, including build scripts and other host artifacts. Cargo documents generic `[host]` and host-tuple-specific tables, with host-tuple settings taking precedence, and documents `host.runner` as the wrapper used to execute host build targets such as build scripts. Cargo's current `TargetConfig` also exposes `rustflags`, `rustdocflags`, `linker`, `runner`, and `links_overrides` for `[target]` or `[host]` configurations.

Before this generation the Browser Session compiler-authority owner examined `[build]`, `[target]`, repository environment/config inclusion, unstable toolchain selectors, and profile rustflags/codegen backends, but ignored `[host]`. A reviewed repository could therefore pre-position host linker/runner/compiler-input authority that becomes effective when nightly `-Zhost-config` / `-Ztarget-applies-to-host` semantics are selected. Treating the feature as currently inactive would be mutable invocation-state trust rather than source provenance.

## RED → repair evidence

- **Structural RED `960d361a37d942937f9d2d88f9cd745265a77a90`** added a focused supplemental contract that calls the canonical compiler-authority owner and proves that generic host linker, host-tuple runner, and authority-extending host rustflags were not rejected, while unrelated host rustflags remain a control.
- **Minimal canonical repair `da6b14de3366c235b1b4c10d340e68477cd41c2a`** added `_configured_host_execution_authority()` to the existing compiler-authority owner. It reused existing target execution keys and rustc/linker/input classifiers rather than rediscovering Cargo package/source topology.
- Review of Cargo's current `TargetConfig` surface exposed two valid omissions in the first repair: host `rustdocflags` share the same target configuration structure, and host target configuration can carry `links_overrides` that replace build-script output. **Review-driven RED `66d8a535befaade759eba6f3f464499792d6bfaf`** added hostile rustdoc external-input and host-tuple links-override cases plus an unrelated rustdoc control.
- **Causal follow-up repair `d2830ddc28e8d6e70af54ae4dec09fd8d40dd3ec`** reused the existing rustdoc classifiers and treats nested host-tuple build-script override tables as explicit authority. The change is confined to the canonical compiler-authority helper; no Browser Session runtime, WebDriver BiDi policy, canonical source-topology owner, workflow, ruleset, or external CWL owner is modified.

## Alternatives considered

1. **Ignore `[host]` until `-Zhost-config` appears in repository configuration.** Rejected. Invocation flags and Cargo's unstable-feature activation are separate mutable execution surfaces; Git-owned latent authority must not become pre-authorized merely because the current invocation does not activate it.
2. **Reject every `[host]` table or every host rustflag.** Rejected. This would conflate deterministic optimization/documentation settings with execution/input authority and would make the guard broader than the owned security invariant.
3. **Create a second Cargo scanner in the focused test.** Rejected. The focused test only constructs hostile/config-control fixtures and delegates the decision to the canonical compiler-authority owner.

## Invariants

- `test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler/rustdoc/toolchain/linker execution and external-input authority.
- `[host]` linker/runner authority is fail closed.
- Authority-extending host `rustflags` and `rustdocflags` are fail closed using the same classifier semantics as `[build]` / `[target]`.
- Host-tuple `links` overrides are fail closed because they can suppress a package build script and inject replacement build output, including native link search/library material.
- Non-authority host flags remain admissible controls; the guard is not a blanket ban on host configuration.

## Remaining evidence and risk

This generation is source-structural evidence until the exact pull-request head receives executable hosted repository/security checks. It does not claim protected-main integration, release readiness, whole-PR review closure, or owned 100% Docstring/rustdoc/Test/Edge Case Coverage. Ambient user/global Cargo configuration, CLI `--config`, environment variables, and toolchain selection are separate invocation/runtime provenance surfaces and are not made trustworthy by this repository-owned config guard.

## Primary references

Cargo Team. (2026). *Unstable Features: target-applies-to-host and host-config*. The Cargo Book, nightly documentation. https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#host-config

Cargo Team. (2026). *TargetConfig*. Cargo 1.100.0-nightly rustdoc. https://doc.rust-lang.org/nightly/nightly-rustc/cargo/context/target/struct.TargetConfig.html

Cargo Team. (2026). *Configuration*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/config.html
