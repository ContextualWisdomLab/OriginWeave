# Browser Session Cargo cfg-target links-override authority

## Problem

The Browser Session compiler-authority contract treated every nested table below `[target.<key>]` as a Cargo `links` build-script override. That is correct for tuple target tables loaded through Cargo `TargetConfig`, but not for `target.'cfg(...)'` tables.

Cargo's current target loader uses a distinct `TargetCfgConfig` for `target.'cfg(...)'`. That type admits `runner`, `rustflags`, `rustdocflags`, and `linker`; remaining keys are captured as `other` and Cargo warns that they are unused. By contrast, tuple targets are loaded through `TargetConfig`, whose unknown nested tables are parsed as `links_overrides` and can suppress a package build script while substituting configured build output.

Treating the two grammars as identical created a security-contract false positive: a nested table under `target.'cfg(...)'` was rejected as executable provenance authority even though current Cargo does not consume it as a links override.

## Constraint and owner boundary

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the canonical owner for repository-selected Cargo compiler, rustdoc, linker, target and build-script-output authority. The trusted-adapter boundary remains the single writer for production Cargo package/source topology. This repair does not create a second Cargo parser and does not broaden any tuple-target authority.

## RED → repair

- RED `a108eb42996a67db3f8809e17cc3166caa6b197c` adds a control for `[target.'cfg(unix)'.review_bypass]` and a paired tuple-target hostile case. Before the repair, the cfg-target control is misclassified as `links build-script override`.
- Repair `9b6191ae6e699e68d4b25e8298a18ffcc0c611b5` preserves linker/runner/rustflags/rustdocflags classification for cfg targets, but only derives nested `links` override authority when the target key is not `cfg(...)`.
- The tuple-target hostile case remains fail closed, so the repair removes a false positive without weakening build-script-output provenance.

## Primary-source trace

Cargo source revision `8814ead110e36ed8fdcf1fdd4009baf82bd78523`, `src/context/target.rs`:

- `TargetCfgConfig` defines the cfg-target grammar and records unmatched fields in `other`.
- `load_target_cfgs` warns for each `other` key instead of interpreting it as a build-script override.
- `TargetConfig` carries `links_overrides` for tuple targets and host config.
- `load_config_table` calls `parse_links_overrides` only for the selected tuple/host target table.

Cargo Book configuration documentation separately documents `target.<tuple>.<links>` as the build-script override form, while `target.<cfg>` is documented for runner/rustflags/rustdocflags/linker behavior.

## Decision

Selected: model Cargo's two target grammars explicitly at the narrow classification point. `target.'cfg(...)'` retains all typed compiler/linker authority checks, but nested unknown tables are not promoted to `links` authority.

Rejected: fail closed on every nested cfg-target table. It is conservative but semantically incorrect against current Cargo and creates avoidable buyer-facing false positives.

Rejected: remove nested-table detection globally. That would create a real tuple-target provenance bypass because `target.<tuple>.<links>` can replace build-script output.

## Risk and follow-up

This contract is source-semantic evidence, not proof that every future Cargo release preserves the same grammar. Cargo source/documentation revisions must be rechecked before changing the pinned toolchain or admitting a newer Cargo behavior. Hosted exact-head tests and security checks remain required before protected-main integration.
