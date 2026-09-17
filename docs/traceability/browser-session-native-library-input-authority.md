# Browser Session native-library input authority

## Problem

The Browser Session repository contract already constrains Git-owned Cargo settings that replace Rust tools, target runners/linkers, compiler-driver executables, linker plugins, response files, linker scripts, and rustc-managed native tools. That boundary did not constrain top-level rustc `-L` and `-l` flags supplied through repository-owned Cargo `rustflags`.

`-L` changes the search path for external crates and libraries, including native libraries. `-l` asks rustc to link a named native library and supports static archives, dynamic libraries, frameworks, and modifiers such as `+whole-archive`. A reviewed Rust source closure therefore did not prove the final native input closure when a Git-owned `.cargo/config.toml` or `.cargo/config` could add either flag.

## Constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. The compiler-authority contract may consume that topology and constrain Git-owned compiler/input authority, but it must not reimplement workspace/package discovery.

This slice applies only to repository-owned Cargo `rustflags`. It does not claim authority over environment-injected `RUSTFLAGS`, direct `cargo rustc -- ...` arguments, build-system flags outside the repository, or external toolchain configuration. Those require their canonical CI/supply-chain owner or a separate reviewed contract.

## RED → repair

RED `657428dd607c2a384f95865ff59caba704a899d9` adds hostile contract cases for:

- `-L native=tools/review-bypass-native`, which widens native-library search to a repository-selected path; and
- `-l static:+whole-archive=review_bypass_native`, which asks rustc to link a native static archive as a complete archive.

The predecessor exact allowed both settings.

Repair `d3a1790c50c393e0328854a2dd7fe4d9aaf3d5c4` keeps production topology ownership unchanged, normalizes the existing Cargo flag representation once, and extends the compiler-authority contract so Git-owned `rustflags` fail closed on `-L`/`-l` in both `[build]` and `[target.<...>]` settings. Unrelated codegen flags remain allowed.

## Decision

Until external/native input provenance is modeled as a versioned reviewed contract, repository-owned Cargo configuration must not widen rustc external-library search paths or request additional native libraries for Browser Session production packages.

This is an input-provenance rule, not a claim that `-L` or `-l` are unsafe Rust features. They are rejected here because their resolved artifacts are outside the current exact-head source and artifact review closure.

## Residual surfaces

The following remain separate review surfaces and are not pre-authorized by this decision:

- `--extern` and other direct precompiled-Rust dependency injection;
- positional object/archive inputs forwarded through `-C link-arg` / `link-args`;
- environment `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS` and direct `cargo rustc` trailing arguments;
- sysroot/rustup/toolchain composition and custom target specifications;
- non-GNU platform-specific native input/control-file mechanisms.

A future allowlist must identify the exact artifact path, digest/provenance, producer, target triple, linkage kind, and reproducible build evidence on the same reviewed exact tree. A path-only allowlist is insufficient.

## Primary evidence

Rust Project. (2026). *Command-line arguments: `-L` and `-l`*. The rustc book. https://doc.rust-lang.org/nightly/rustc/command-line-arguments.html

The rustc documentation states that `-L` adds a path searched for external crates and libraries and can be scoped to `dependency`, `crate`, `native`, `framework`, or `all`. It also states that `-l` links the generated crate to a specified native library and supports static archives, dynamic libraries, frameworks, and linking modifiers including `+whole-archive`.

Rust Project. (2026). *Build scripts*. The Cargo book. https://doc.rust-lang.org/cargo/reference/build-scripts.html

Cargo documents the corresponding native-library and search-path concepts through `cargo::rustc-link-lib` and `cargo::rustc-link-search`, which are passed to rustc as `-l` and `-L` semantics. Build-script authority itself remains separately fail-closed in the Browser Session Cargo build-surface contract.

## Verification state

The RED and repair commits are structurally present on the active #317 lineage. This dossier does not promote the branch to executable GREEN: current-head hosted repository/security workflows and independent current-head review must still complete on the reconciled lineage before merge or release readiness can be claimed.
