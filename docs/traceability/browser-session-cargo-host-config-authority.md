# Browser Session Cargo host-config execution authority traceability

## Decision

Repository-owned Cargo configuration must not be able to introduce unreviewed host-side compiler, linker, runner, rustdoc, or build-script-link authority for Browser Session production builds. `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected Cargo compiler/toolchain/execution and external-input authority; this dossier adds no second Cargo package/source topology scanner.

The contract fails closed on authority-bearing `[host]` and `[host.<host-tuple>]` settings: `linker`, `runner`, authority-extending `rustflags` / `rustdocflags`, and host build-script `links` overrides. Cargo's host configuration reuses `TargetConfig`; `load_host_triple()` selects `[host.<actual-triple>]` when present and otherwise generic `[host]`, then `load_config_table()` parses the selected table's non-typed keys through `parse_links_overrides()`. A links override is authority even when its table is empty: Cargo constructs `BuildOutput::default()` and inserts the library name into `links_overrides`, so a matching package's build script is skipped despite there being no replacement output fields. Harmless typed host flags such as `-C opt-level=2` and `--document-private-items` remain permitted.

## Problem and buyer/security effect

Cargo's nightly `host-config` feature gives Git-owned configuration a distinct path for artifacts compiled or executed on the build host, including build scripts and other host artifacts. Cargo documents generic `[host]` and host-tuple-specific tables, with host-tuple settings taking precedence, and documents `host.runner` as the wrapper used to execute host build targets such as build scripts. Cargo's current `TargetConfig` is the configuration type for `[target]` or `[host]` and exposes `rustflags`, `rustdocflags`, `linker`, `runner`, and `links_overrides`; a links override suppresses the matching package build script and substitutes configured `BuildOutput`.

The original Browser Session compiler-authority owner examined `[build]`, `[target]`, repository environment/config inclusion, unstable toolchain selectors, and profile rustflags/codegen backends, but ignored `[host]`. Later repairs added host authority and generic nested-map coverage. A subsequent review found that content-shape classification could misclassify a legitimate `[host.<actual-triple>]` table containing only typed host settings as a links override, so the current classifier preserves typed `linker` / `runner` / `rustflags` / `rustdocflags` semantics and recurses into unknown nested maps rather than treating every first-level host table as an override.

That repair exposed a narrower fail-open case. An empty nested table has no scalar payload for the recursive classifier to recognize, but Cargo still inserts an empty `BuildOutput` for that `links` key. Both `[host.review_bypass]` under the generic selected host table and `[host.<actual-triple>.review_bypass]` under a selected host-triple table can therefore suppress a matching build script while the repository guard reports no authority. Empty replacement output is not absence of authority; suppressing the build script is itself a build decision.

## RED → repair evidence

- **Structural RED `960d361a37d942937f9d2d88f9cd745265a77a90`** added a focused supplemental contract that calls the canonical compiler-authority owner and proved that generic host linker, host-tuple runner, and authority-extending host rustflags were not rejected, while unrelated host rustflags remained a control.
- **Minimal canonical repair `da6b14de3366c235b1b4c10d340e68477cd41c2a`** added `_configured_host_execution_authority()` to the existing compiler-authority owner, reusing existing target execution keys and rustc/linker/input classifiers.
- Review of Cargo's `TargetConfig` surface exposed host `rustdocflags` and host `links_overrides`. **Review-driven RED `66d8a535befaade759eba6f3f464499792d6bfaf`** added hostile rustdoc external-input and host-tuple links-override cases; **repair `d2830ddc28e8d6e70af54ae4dec09fd8d40dd3ec`** reused the existing classifiers and treated host build-script override tables as authority.
- **RED `6574faa0d7012dec8fad0f0a563599af90e89634`** exposed a generic `[host.review_bypass]` build-output table. **Repair `edfbd7402d7653374ad7ab5556b3952f73d0ad89`** closed that generic-host path.
- A later focused review found the broad generic-host repair could reject a legitimate host-triple table that contained only typed host settings. **RED `ad71ca1f70ca3e5220d59cdbd4dcab3b60610012`** fixed that false-positive expectation, and **repair `1e4c16d9ad1487f3a101dce699d52913277ad85d`** made the classifier preserve typed host settings while recursively classifying unknown nested build-output payloads.
- Fresh Cargo-source review then found that `parse_links_overrides()` starts each non-typed table with `BuildOutput::default()` and inserts it into `links_overrides` even when the table is empty. **Structural RED `f85408e777480810656d8e161934df8611f4035a`** added empty generic-host and host-triple nested `links` fixtures. The pre-repair classifier accepted both because recursion reached an empty mapping with no scalar payload.
- **Minimal canonical repair `7a72d830dd8e8af8859d676f84e711455e490702`** adds exactly one condition in `_configured_host_execution_authority()`: a non-root empty host mapping is classified as `links build-script override`. Existing typed host-triple controls remain unchanged, non-empty generic/triple hostile cases retain their prior path, and no second Cargo topology/config scanner is introduced.

## Alternatives considered

1. **Ignore `[host]` until `-Zhost-config` appears in repository configuration.** Rejected. Invocation flags and Cargo's unstable-feature activation are separate mutable execution surfaces; Git-owned latent authority must not become pre-authorized merely because the current invocation does not activate it.
2. **Reject every `[host]` table or every host rustflag.** Rejected. This conflates deterministic optimization/documentation settings with execution/input authority and would make the guard broader than the owned invariant.
3. **Guess whether every first-level `[host.<key>]` is a host tuple from spelling.** Rejected. Cargo chooses the actual host-triple prefix at runtime and otherwise parses generic `[host]`; tuple-string heuristics would create a second, drifting parser and previously caused false-positive tension.
4. **Treat an empty `links` table as harmless because it contains no replacement fields.** Rejected. Cargo inserts `BuildOutput::default()` for the library name and skips the package build script. Suppression is itself authority.
5. **Create a second Cargo scanner in the focused test.** Rejected. The focused test only constructs hostile/control fixtures and delegates the decision to the canonical compiler-authority owner.

## Invariants

- `test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler/rustdoc/toolchain/linker execution and external-input authority.
- `[host]` linker/runner authority is fail closed.
- Authority-extending host `rustflags` and `rustdocflags` are fail closed using the same classifier semantics as `[build]` / `[target]`.
- Non-empty and empty host `links` override tables are fail closed because either form can suppress a matching build script; replacement-output fields are not required for that authority to exist.
- Typed non-authority host flags remain admissible controls; the guard is not a blanket ban on host configuration.

## Remaining evidence and risk

This generation is source-structural evidence until the exact pull-request head receives executable hosted repository/security checks. It does not claim protected-main integration, release readiness, whole-PR review closure, or owned 100% Docstring/rustdoc/Test/Edge Case Coverage. Ambient user/global Cargo configuration, CLI `--config`, environment variables, and toolchain selection are separate invocation/runtime provenance surfaces and are not made trustworthy by this repository-owned config guard. Those surfaces belong in the CI/release execution-environment contract rather than by extending this Git-source topology scanner.

## Primary references

Cargo Team. (2026). *Unstable Features: target-applies-to-host and host-config*. The Cargo Book, nightly documentation. https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#host-config

Cargo Team. (2026). *TargetConfig*. Cargo 1.100.0-nightly rustdoc. https://doc.rust-lang.org/nightly/nightly-rustc/cargo/context/target/struct.TargetConfig.html

Cargo Team. (2026). *target.rs*. Cargo source for `load_host_triple`, `load_config_table`, and `parse_links_overrides`. https://doc.rust-lang.org/nightly/nightly-rustc/src/cargo/util/context/target.rs.html

Cargo Team. (2026). *Configuration*. The Cargo Book. https://doc.rust-lang.org/nightly/cargo/reference/config.html
