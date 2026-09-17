# Browser Session external-crate input authority

## Problem

The Browser Session repository contract already derives production Cargo package/source topology from `tests/test_browser_session_trusted_adapter_boundary.py` and constrains repository-owned Cargo compiler/linker execution plus native-library input expansion. That boundary did not constrain rustc `--extern` supplied through Git-owned Cargo `rustflags`.

rustc documents `--extern` as specifying the name and optional location of a direct external crate. `--extern CRATENAME=PATH` names an exact precompiled crate artifact, while pathless `--extern CRATENAME` makes the crate a candidate from rustc's external-library search path. The crate name is also added to the extern prelude. Repository-owned Cargo configuration could therefore add a precompiled Rust dependency outside the reviewed production source/dependency closure without modifying `Cargo.toml`, the canonical Cargo topology, or the nominal compiler/linker selection.

This is a provenance gap even when the injected crate is not ultimately linked: the compiler is allowed to resolve and expose an additional direct dependency candidate whose producer, source tree, digest, feature set, target, and build evidence are not represented by the exact reviewed tree.

## Constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. The Cargo compiler-authority contract may consume that topology and constrain repository-owned rustc input authority, but it must not create a second workspace/package/dependency resolver.

This slice applies only to Git-owned Cargo `rustflags` in `.cargo/config.toml` and `.cargo/config`. Environment `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS`, direct `cargo rustc -- ...` trailing flags, rustup/sysroot/toolchain composition, and external build-system injection remain separate owner surfaces.

## RED → repair

RED `dbd9faa623f01652c2e75904af8bec614c2e6fc9` adds hostile contract cases for:

- split `--extern review_bypass=tools/libreview_bypass.rlib` in `[build].rustflags`;
- equals-form `--extern=review_bypass=tools/libreview_bypass.so` in target-scoped `rustflags`; and
- pathless `--extern review_bypass`, which can resolve from the external-library search path.

The predecessor exact `a370bad3ce3f56b9ba7f0ff0ded589e97608dcdf` allowed all three forms because external-input classification covered `-L` / `-l` and linker-forwarded native inputs but not rustc external-crate injection.

Repair `098a596029c0cf339c070604a265593540822956` keeps production topology ownership unchanged and introduces one rustc-level external-input classifier. It delegates native-library/search-path forms to the existing linker-input classifier and additionally fails closed on `--extern` and `--extern=...`. The linker-driver parser remains unchanged because `--extern` is rustc input authority rather than a linker-driver option. An unrelated `--check-cfg` control remains allowed.

## Decision

Until precompiled external-crate provenance is modeled as a versioned reviewed artifact contract, repository-owned Cargo configuration must not use rustc `--extern` to widen Browser Session production-package dependency inputs outside the canonical Cargo dependency/source closure.

This is not a claim that `--extern` is unsafe. It is rejected at this boundary because neither a pathname nor a crate name proves the artifact's source, producer, target compatibility, features, digest, reproducibility, or review ancestry.

A future allowlist must bind at minimum the crate identity, exact artifact digest, producer/source revision, rustc/toolchain identity, target triple, crate type, enabled features/configuration, reproducible-build evidence, and consumer contract on the same reviewed exact tree. Path-only approval is insufficient.

## Residual surfaces

The following remain separate review surfaces and are not pre-authorized by this decision:

- positional object/archive inputs forwarded through `-C link-arg` / `link-args`;
- environment `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS` and direct `cargo rustc` trailing arguments;
- rustup/sysroot/toolchain composition and custom target specifications;
- target-specific external toolchain scripts and non-GNU native control/input mechanisms.

## Primary evidence

Rust Project. (2026). *Command-line arguments: `--extern`*. The rustc book. https://doc.rust-lang.org/nightly/rustc/command-line-arguments.html

The rustc documentation states that `--extern` specifies the name and location of an external crate for a direct dependency. It accepts `CRATENAME=PATH` and pathless `CRATENAME`, adds the name to the extern prelude, and allows multiple external artifacts for the same crate name.

Rust Project. (2026). *Extern crate declarations*. The Rust Reference. https://doc.rust-lang.org/reference/items/extern-crates.html

The Rust Reference defines external-crate dependencies and explains that external crates participate in compile-time resolution and linkage semantics. This supports treating precompiled external-crate injection as dependency/input provenance rather than inert compiler metadata.

## Verification state

The RED and repair commits are structurally present on the active #317 lineage. This dossier does not promote the branch to executable GREEN. Current-head hosted repository/security workflows and independent current-head review remain required after lineage reconciliation before merge or release readiness can be claimed.
