# Browser Session Cargo profile rustflags authority

## Problem

Cargo's unstable profile `rustflags` option can be selected from the root workspace manifest with `cargo-features = ["profile-rustflags"]` or from Cargo configuration with the `profile-rustflags` unstable feature enabled. Cargo documents these profile flags as arguments passed directly to `rustc`.

The Browser Session Cargo authority contract already classified build-level and target-level `rustflags`, but did not inspect `rustflags` nested under `[profile.*]`. A Git-owned profile could therefore reintroduce execution or compiler-input authority such as `-C linker=...`, `--extern`, `--sysroot`, a top-level rustc response file, or `-Zcodegen-backend=...` without changing the reviewed production package/source topology.

## Constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. This repair must consume that topology and the existing compiler/linker/input classifiers rather than add another package, dependency, or source scanner.

The `profile-rustflags` capability itself is not prohibited. Profile flags that do not extend execution or external-input authority remain allowed.

## RED and repair

- RED `e7aafa552c923f121ee88e92fcca9b7ad43ca363` adds `tests/test_browser_session_profile_rustflags_authority_contract.py`. It covers root-manifest profile `--extern`, Cargo-config profile linker selection, and an ordinary `opt-level`/`cfg` control.
- Repair `38c1c06fe6942a1024ea73148b0c8b6ccc9f4405` extends only `tests/test_browser_session_cargo_compiler_authority_contract.py`. `_configured_profile_rustflag_authority()` recursively inspects profile tables and reuses `_flags_select_codegen_backend()`, `_flags_select_linker()`, and `_flags_extend_external_link_inputs()`.
- Root `Cargo.toml` profiles and repository `.cargo/config.toml` / `.cargo/config` profiles now fail closed only when profile `rustflags` widen execution or compiler-input authority. Existing direct profile `codegen-backend` handling remains separate.

## Primary evidence

Cargo source and documentation were checked against `rust-lang/cargo@8814ead110e36ed8fdcf1fdd4009baf82bd78523`.

- `doc/book/src/reference/unstable.md`, “Profile `rustflags` option”, describes profile `rustflags` as passed directly to `rustc`, including `[unstable] profile-rustflags = true` with `[profile.release] rustflags = [...]` in Cargo configuration.
- Cargo profile configuration is owned by the root workspace manifest and may also be supplied through Cargo configuration. The contract therefore inspects both Git-owned surfaces while keeping production topology ownership unchanged.

## Decision and security effect

Repository-selected profile `rustflags` are treated as another spelling of the same rustc execution/input authority already governed for build and target flags. The contract rejects profile flags that:

- select a code-generation backend;
- select or reconfigure linker execution, linker plugins, scripts, response files, or positional linker inputs through the existing linker classifier; or
- add opaque/external compiler inputs such as leading `@path`, `--sysroot`, `--extern`, `-L`, or `-l`.

Ordinary profile flags such as optimization level or reviewed `--cfg` values remain permitted. This avoids turning an execution-provenance contract into a blanket Cargo-profile policy.

## Residual authority

This repository-source contract does not claim control over environment or direct-CLI profile injection, including `CARGO_PROFILE_<NAME>_RUSTFLAGS`, ancestor or `$CARGO_HOME` configuration, command-line `--config`, or runner/toolchain mutation outside the reviewed tree. Those are CI/release environment provenance surfaces and require exact runner/configuration evidence rather than source-copying environment policy into OriginWeave.

The repair is source-semantic until the exact PR head receives hosted repository/security execution evidence. Static source inspection is not a substitute for executable GREEN.
