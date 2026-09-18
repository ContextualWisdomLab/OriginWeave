# Browser Session custom target specification authority

Status: Draft source-semantic contract evidence on PR #317. This document does not claim hosted executable, repository-security, or browser GREEN.

## Problem

Cargo `build.target` accepts a built-in rustc target, `host-tuple`, or a path to a custom target specification. A repository-owned `.cargo/config.toml` or legacy `.cargo/config` can therefore select a JSON target specification without changing Cargo package/source topology or the visible `rustflags`/`rustdocflags` already covered by the Browser Session compiler-authority contract.

That selection is provenance-bearing. Rust custom target specifications describe compiler target behavior rather than merely naming an output directory. Current rustc target metadata exposes linker selection, linker flavor, pre/post link objects, pre/late/post link arguments, link scripts, linker environment changes, and assembler arguments among the target options. A Git-owned custom target can therefore alter native tool execution or native/link inputs while the reviewed Rust source closure is unchanged.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source discovery.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the owner for repository-selected compiler, rustdoc, linker, toolchain, and external-input authority.
- Built-in target triples and Cargo's `host-tuple` remain allowed. The repair is not a blanket `build.target` ban.
- This contract does not authorize a custom target JSON artifact. An approved future target specification needs immutable artifact identity, compiler-version/schema pinning, transitive linker/native-input provenance, SBOM/attestation, rollback, and the same-tree executable evidence.
- `CARGO_BUILD_TARGET`, direct Cargo `--target`, `RUST_TARGET_PATH`, and target specifications resolved from a rustc sysroot are execution-environment or toolchain inputs and remain with the CI/release supply-chain owner.

## RED

Commit `9a7477e6e19d739cead5c90fa63070aa85a01cf6` adds `tests/test_browser_session_custom_target_spec_authority_contract.py`. The hostile fixture selects `targets/review-bypass.json` through build-level `target`; the target JSON names a different linker. A second fixture places the JSON path beside a built-in target in Cargo's array form. The predecessor compiler-authority contract did not classify `build.target`, so both repository-owned custom-target selectors were outside its fail-closed surface. Built-in target and `host-tuple` controls are retained.

## Decision and repair

Commit `8e4db1b2229a6b77d117be8ed2d0595bbd96a1d7` minimally extends the existing Cargo compiler-authority owner with `_configured_custom_target_specs`. Build-level target strings or arrays whose entries end in `.json` are recorded as `target:custom target specification:<path>` and fail through the existing execution-override assertion. Built-in target triples and `host-tuple` do not enter that list.

No second Cargo topology scanner was added. The hostile contract imports and exercises the canonical compiler-authority assertion, while package/source discovery remains delegated to the trusted-adapter boundary.

## Security effect and residual risk

The repair closes Git-owned Cargo configuration that directly selects a JSON rustc target specification through `[build].target`. It prevents an unreviewed target JSON from changing linker/native-input behavior behind an otherwise unchanged Browser Session source/dependency closure.

It does not prove ambient target selection trustworthy. Environment `CARGO_BUILD_TARGET`, direct `cargo ... --target`, `RUST_TARGET_PATH`, sysroot target metadata, runner-installed compiler versions, or schema compatibility remain CI/release supply-chain concerns. Rust documents custom target JSON properties as unstable and recommends pinning the compiler version; any future approved target JSON must therefore be versioned and evidenced together with the exact rustc/toolchain that consumes it.

## Acceptance

The source contract must remain Draft until the reconciled exact #317 head receives hosted repository/security execution and independent current-head review. Parent #229 ancestry and required checks remain prerequisites; source-level RED→repair here does not transfer predecessor executable evidence.

## References

The Cargo Project. (n.d.). *Configuration*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/config.html

The Rust Project Developers. (n.d.). *Custom targets*. *The rustc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/targets/custom.html

The Rust Project Developers. (2026). *rustc_target::spec* (rustc 1.100.0-nightly, 330d31712 2026-09-17). Retrieved September 18, 2026, from https://doc.rust-lang.org/nightly/nightly-rustc/rustc_target/spec/

The Rust Project Developers. (n.d.). *TargetOptions*. *rustc_target::spec*. Retrieved September 18, 2026, from https://doc.rust-lang.org/beta/nightly-rustc/rustc_target/spec/struct.TargetOptions.html
