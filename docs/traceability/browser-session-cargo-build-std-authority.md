# Browser Session Cargo `build-std` authority

## Problem

OriginWeave's Browser Session trusted-adapter boundary reviews the repository-owned Cargo production package/source closure and the companion compiler-authority contract reviews Git-owned Cargo execution and input overrides. Cargo also permits `-Z` features to be configured in `.cargo/config.toml` under `[unstable]`. In particular, `build-std` changes a build from consuming the installed pre-built standard library to compiling selected standard-library crates from source as part of the crate graph, while `build-std-features` changes the features used for that standard-library build.

That changes compiler inputs and supply-chain provenance without changing the Browser Session package manifests or Rust production-source closure. Treating it as an ordinary build preference would therefore let repository-owned configuration widen the reviewed toolchain/input boundary.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the owner for Git-owned Cargo-selected compiler, rustdoc, linker, toolchain, and external-input authority.
- This slice does not attempt to attest the ambient Rust toolchain, installed `rust-src`, runner image, or direct command-line `-Z` flags.
- Unrelated unstable Cargo settings are not blanket-banned solely because they live under `[unstable]`; only settings that alter the reviewed standard-library input boundary are classified here.

## Authoritative evidence

The Cargo Book's current unstable-features reference states that anything configurable with a `-Z` flag can also be set in `.cargo/config.toml` under `[unstable]`, and gives `build-std = ["core", "alloc"]` as an example. The same reference states that `build-std` compiles the standard library from source as part of the crate graph, requires the `rust-src` component and nightly Cargo/rustc, and may select the standard-library crates to build. `build-std-features` configures the features enabled for that standard-library build.

Primary references:

- Cargo Book, *Unstable Features — build-std*: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std
- Cargo Book, *Unstable Features — build-std-features*: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std-features

## RED → repair

RED `5a63a00042b9f80fb905167b28ba736b71d65d39` adds a focused contract that routes repository-owned `[unstable] build-std` and `build-std-features` settings through the existing compiler-authority owner. It covers list and boolean `build-std` forms plus `build-std-features`, and keeps an unrelated `mtime-on-use` setting as an allowed control.

Repair `e524af3a2e316a3b124a25aac8392760c25a12ba` adds the minimal classification to the existing parsed-config owner. Active `build-std` and `build-std-features` settings fail closed as `unstable_keys`; explicit `false` or an empty list do not widen the input boundary. No second Cargo topology or configuration scanner is introduced.

## Decision and security effect

Repository-owned Cargo configuration may not select source-built standard-library inputs for the Browser Session production boundary until the same reviewed change supplies a versioned toolchain/source provenance contract. This prevents a Git-owned `.cargo/config*` change from silently replacing the pre-built sysroot assumption with source-built `core`, `alloc`, `std`, `proc_macro`, or feature-modified standard-library artifacts.

The fail-closed rule is intentionally narrower than banning every unstable Cargo feature. A future need for `build-std` must identify the exact Rust toolchain, `rust-src` source identity, selected standard-library crates/features, target specification, resulting artifacts, and reproducible attestation before the rule is relaxed.

## Residual execution/release provenance

This repository contract does not prove ambient execution state. Remaining owner surfaces include direct Cargo CLI `-Z build-std` / `-Z build-std-features`, ancestor or `$CARGO_HOME` configuration, runner/container images, installed nightly Cargo/rustc and `rust-src`, custom target specifications, and other unstable features that can alter compiler/toolchain authority. Those require CI/release environment controls and, where repository-owned configuration gains such authority, additional focused fail-closed contracts.

No hosted repository execution, protected-head GREEN, immutable release, or browser-observed acceptance is claimed by this source-semantic repair alone.
