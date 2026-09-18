# Browser Session rustc LLVM plugin authority

## Problem

OriginWeave's Browser Session production closure already fails closed for repository-selected rustc executables, wrappers, code-generation backends, linker plugins, linker scripts, response files, sysroots, external crates, native libraries, and other reviewed Cargo execution/input surfaces. One rustc-native execution surface was still outside that classifier: `-Z llvm-plugins=<path>`.

The current rustc option table exposes `llvm_plugins` as an unstable list option described as “a list LLVM plugins to enable (space separated)”. `rustc_codegen_llvm` copies that list into the LLVM module configuration and passes the joined plugin list to `LLVMRustOptimize`. A Git-owned Cargo `rustflags` value can therefore select code that participates in compiler optimization without changing `Cargo.toml` dependency/source topology, the selected code-generation backend, or the final linker.

This is compiler execution provenance, not an optimization-only preference. A reviewed Browser Session build must not gain a repository-selected LLVM pass plugin through an otherwise innocuous Cargo profile/build/target flag path.

## Decision

The existing Browser Session Cargo compiler-authority contract remains the single owner of Git-owned rustc execution/input selection. Its existing rustflag classifier now treats both compact and split LLVM-plugin forms as code-generation execution extensions:

- `-Zllvm-plugins=tools/review-bypass-pass.so`
- `-Z llvm-plugins=tools/review-bypass-pass.so`

The classifier is already consumed by build-level `rustflags`, target-level `rustflags`, root-manifest profile `rustflags`, and repository Cargo-config profile `rustflags`. No second Cargo topology scanner or Browser Session source-discovery implementation is introduced.

Ordinary flags that do not replace or dynamically extend compiler execution remain outside this prohibition. The focused contract keeps `-C opt-level=2` and a reviewed `--cfg` as controls.

## RED → repair evidence

- RED: `e87cab84490aea41a4bb5f7de20485cbd770642d` adds hostile build, target, Cargo-config profile, and root-profile LLVM-plugin fixtures that the predecessor classifier did not reject.
- Repair: `60f27dfde50b7134ebd43d9d0574e7edc169191d` extends the existing rustc code-generation execution classifier to reject `llvm-plugins=` in compact or split `-Z` form. Existing build/target/profile call sites inherit the repair.
- Exact executable GREEN is not inferred from these source changes. The PR remains Draft and must earn fresh hosted repository/security evidence after the canonical parent lineage is reconciled.

## Security effect

A Git-owned Cargo config or profile can no longer introduce an LLVM pass plugin while leaving the reviewed package/source graph unchanged. This closes a compiler-process extension path adjacent to, but distinct from, the already governed rustc code-generation backend and linker-plugin-LTO boundaries.

The plugin path is treated as execution provenance only. Browser Session does not learn LLVM plugin semantics, and no LLVM implementation detail becomes Browser Session domain truth.

## Residual authority

This repository-source contract does not claim control over ambient execution surfaces supplied outside the reviewed tree, including environment/direct-CLI rustflags, ancestor or `$CARGO_HOME` Cargo config, runner images, the installed rustc/LLVM distribution itself, or administrator-provided toolchain mutation. Those require CI/release supply-chain evidence, immutable toolchain identity, SBOM/provenance, and reproducibility controls rather than another leaf-source scanner.

A future approved LLVM plugin would require an explicit versioned artifact contract, immutable digest/provenance, toolchain/LLVM ABI compatibility evidence, security review, exact-head tests, SBOM/attestation, and rollback before this fail-closed rule is relaxed.

## Primary references

Rust Project. (2026). *rustc_session::options::UnstableOptions* (`llvm_plugins`). Rust nightly compiler documentation. https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/options/struct.UnstableOptions.html

Rust Project. (2026). *rustc_session options source* (`llvm_plugins`: “a list LLVM plugins to enable”). Rust compiler source documentation. https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_session/options.rs.html

Rust Project. (2026). *rustc_codegen_llvm::back::write*. Rust compiler source. The LLVM plugin list is propagated into `LLVMRustOptimize`. https://github.com/rust-lang/rust/blob/main/compiler/rustc_codegen_llvm/src/back/write.rs
