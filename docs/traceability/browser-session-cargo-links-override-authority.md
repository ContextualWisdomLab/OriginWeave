# Browser Session Cargo links-override authority

## Problem

Cargo target configuration can replace the output of a dependency build script when that dependency declares a `package.links` value. A `[target.<triple>.<links>]` table prevents the build script from running and supplies its metadata directly. Cargo documents override keys including `rustc-link-lib`, `rustc-link-search`, `rustc-flags`, `rustc-cfg`, `rustc-env`, and `rustc-cdylib-link-arg`.

That table is compiler and native-input authority. It can inject `-l`/`-L` inputs, conditional-compilation values, compile-time environment, and cdylib linker arguments while bypassing the production build-script boundary already reviewed by OriginWeave. The predecessor Cargo compiler-authority scanner inspected `target.<triple-or-cfg>` linker, runner, rustflags, and rustdocflags but ignored nested target tables, allowing this build-script replacement path to remain outside exact-tree provenance.

## Authority and constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source and build-script topology. `tests/test_browser_session_cargo_compiler_authority_contract.py` owns repository-selected compiler/linker/rustdoc input and execution authority. The links-override contract consumes that owner and does not re-scan workspace topology.

Until a versioned, dependency-specific override contract proves the exact `package.links` owner, native artifacts, search paths, cfg/env values, linker arguments, and replacement semantics, any nested Git-owned `target.<triple>.<links>` table is fail-closed. Ordinary target tables without a nested links override remain governed by the existing linker/runner/rustflags/rustdocflags rules.

## RED → repair evidence

RED `2f0cbdb69033e2d2f2e49393a5eef6b5a9bf6125` adds `tests/test_browser_session_cargo_links_override_contract.py`. Hostile fixtures cover `rustc-link-lib`, `rustc-link-search`, `rustc-cfg`, `rustc-env`, and `rustc-cdylib-link-arg`; a normal target `rustflags` table remains an allowed control when it does not widen the existing authority classifier.

Repair `a0b85bf55a7736ea99cad1c56c53b7df583647cd` minimally extends the existing parsed target-settings owner. Nested target tables are recorded as links build-script overrides and fail through the same Browser Session provenance error path. No second Cargo configuration or production-topology scanner is introduced.

## Security and reproducibility effect

A checked-in Cargo config can no longer replace a linked dependency's build-script outputs with unreviewed native-library/search-path, cfg, environment, or cdylib-linker metadata. This closes a direct bypass around both the build-script boundary and the previously modeled rustc/linker external-input surfaces.

This source contract does not prove ambient Cargo configuration, command-line `--config`, `$CARGO_HOME`, ancestor configuration, runner/toolchain state, or the contents of future explicitly approved native artifacts. Those remain execution/release provenance surfaces.

## Primary references

- The Rust Project. (2026). *The Cargo Book: Configuration — `target.<triple>.<links>`*. https://doc.rust-lang.org/cargo/reference/config.html#targettriplelinks
- The Rust Project. (2026). *The Cargo Book: Build Scripts — Overriding Build Scripts*. https://doc.rust-lang.org/cargo/reference/build-scripts.html#overriding-build-scripts

Cargo documents that a target links sub-table prevents the linked package's build script from running and substitutes the listed metadata. It also documents that `rustc-link-lib` maps to rustc `-l`, `rustc-link-search` maps to `-L`, `rustc-cfg` controls compile-time cfg, and the other override keys replace build-script-produced compiler/linker metadata.
