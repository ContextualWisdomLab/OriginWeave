# Browser Session incremental-cache input authority

## Decision

OriginWeave treats an explicit repository-selected rustc `-C incremental=<path>` value as mutable compiler-input authority. Git-owned Cargo `rustflags`, `rustdocflags`, profile `rustflags`, and rustdoc `--doctest-build-arg` forwarding must fail closed when they select an incremental cache directory.

Cargo's own boolean `build.incremental` / profile `incremental` setting is not rejected by this source contract. Cargo owns its normal target-directory cache placement; the integrity and lifecycle of runner caches, `CARGO_INCREMENTAL`, target directories, toolchain images, and restored CI artifacts remain CI/release supply-chain evidence.

## Problem

`rustc -C incremental=<path>` is not only a compile-speed preference. rustc stores compilation information in the selected directory and reuses it on later compilations. Current Cargo source also constructs the compiler invocation by adding `-C incremental=<path>` for Cargo-managed incremental builds. A repository-selected arbitrary path can therefore make the reviewed Browser Session build consume mutable work products whose producer execution, source state, toolchain, lifetime, and digest are not established by the repository source tree.

A path allowlist is insufficient: a reviewed pathname does not prove the identity or freshness of the cache contents, symlink containment, producer toolchain, or reproducible reconstruction.

## Contract and causal repair

The structural RED is commit `f455739da5953f0c70d4289cef795a967a0348c2`, `tests/test_browser_session_incremental_cache_input_authority_contract.py`. It fixes hostile cases for:

- build `rustflags`: split `-C`, `incremental=<path>`;
- target `rustflags`: compact `-Cincremental=<path>`;
- Cargo profile `rustflags`: `--codegen=incremental=<path>`;
- build `rustdocflags`; and
- rustdoc `--doctest-build-arg` forwarding.

The same test keeps Cargo-managed `[build] incremental = false` as an allowed control.

The minimal repair is commit `6753b717486bd025c97043de7a53323c6dd4d47c`. The canonical compiler-authority owner, `tests/test_browser_session_cargo_compiler_authority_contract.py`, extends `_codegen_option_extends_external_inputs()` to classify `incremental=` alongside PGO profile inputs. Existing build/target rustflags, rustdocflags, profile-rustflags, and doctest-forwarding call sites consume the same classifier; no second Cargo topology scanner or downstream policy copy is introduced.

## Invariants

1. Repository-owned explicit incremental cache paths cannot become unreviewed compiler input.
2. Cargo's ordinary boolean incremental setting is not conflated with a repository-selected arbitrary cache pathname.
3. Browser Session source authority does not claim to attest ambient runner caches or externally restored target directories.
4. Any future exception must identify the cache content immutably and bind it to producer source, exact toolchain, target, compilation options, SBOM/provenance, expiry/invalidation policy, and reproducible fallback before the fail-closed rule is relaxed.

## Residual evidence boundary

This source contract does not prove environment/direct-CLI `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS`, `CARGO_INCREMENTAL`, ancestor or `$CARGO_HOME` configuration, externally restored `target/` contents, runner image state, rustup/sysroot state, or remote build-cache integrity. Those remain canonical CI/release supply-chain responsibilities and must not be represented as closed by this repository-only check.

## Primary references

Rust Project. (2026). *Codegen options: incremental*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/#incremental

Rust Project. (2026). *cargo::core::compiler: add_codegen_incremental*. Cargo API documentation. https://doc.rust-lang.org/stable/nightly-rustc/cargo/core/compiler/index.html

Rust Project. (2026). *CodegenOptions*. rustc_session API documentation, rustc 1.100.0-nightly (923c95cdf, 2026-09-16). https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/options/struct.CodegenOptions.html

Rust Project. (2026). *Profiles: incremental*. The Cargo Book. https://doc.rust-lang.org/nightly/cargo/reference/profiles.html#incremental

Accessed 2026-09-18.
