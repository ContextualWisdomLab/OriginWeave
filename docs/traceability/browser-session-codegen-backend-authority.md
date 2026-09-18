# Browser Session code generation backend authority

## Problem

OriginWeave's Browser Session trust boundary reviews repository-owned Cargo package/source topology and Git-owned compiler/linker/toolchain execution inputs. Rust and Cargo also expose an unstable code generation backend selection surface. A repository can select a Cargo profile `codegen-backend`, or pass rustc `-Zcodegen-backend=<path>` through Cargo `rustflags`.

This is execution provenance, not an optimization-only preference. rustc's unstable `codegen-backend` flag accepts a path to a dynamic library and loads that library as the code generation backend at runtime. Cargo's unstable `codegen-backend` feature permits profile-level backend selection, including from root `Cargo.toml` and Cargo configuration. Leaving those surfaces outside the canonical compiler-authority contract would let Git-owned configuration replace code-generation implementation without changing the reviewed production Rust source closure.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source containment.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the owner for Git-owned Cargo/rustc execution and compiler-input authority.
- Cargo profile settings are inspected only for `codegen-backend`; ordinary optimization/debug profile settings remain valid.
- This slice does not attest ambient `RUSTFLAGS`, command-line `cargo -Z codegen-backend`, the installed rustc sysroot/codegen backend artifacts, or runner/container images.

## Authoritative evidence

The Cargo Book's current unstable-features reference states that `codegen-backend` selects the backend used by rustc through a profile. It shows `[profile.dev.package.foo] codegen-backend = "cranelift"` and states that profile configuration requires either `-Z codegen-backend` or `[unstable] codegen-backend = true`. The Cargo profiles reference states that profile settings in the root workspace manifest are authoritative and may be overridden by Cargo configuration.

The Rust Unstable Book states that `-Zcodegen-backend=<path>` selects a dynamic library used as rustc's code generation backend at runtime and requires that library to expose `__rustc_codegen_backend`.

Primary references:

- Cargo Book, *Unstable Features — codegen-backend*: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#codegen-backend
- Cargo Book, *Profiles*: https://doc.rust-lang.org/nightly/cargo/reference/profiles.html
- Rust Unstable Book, *codegen-backend*: https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/codegen-backend.html

## RED → repair

RED `85954d363f78621d3f9dc1dc0ad0ef304b892fae` adds focused hostile fixtures for compact and split `-Zcodegen-backend=<path>` in build/target `rustflags`, Cargo config profile selection, and root-manifest profile selection. An ordinary `opt-level` profile setting remains an allowed control.

Repair `ff7d48bc875b3d68486e7e8265af44dbb6c4b1ed` extends the existing compiler-authority owner rather than adding a second Cargo scanner. It:

- treats active `[unstable] codegen-backend` as compiler/toolchain execution authority;
- recognizes compact and split rustc `-Zcodegen-backend` in Git-owned build/target `rustflags`;
- rejects `codegen-backend` keys in root-workspace Cargo profiles and Cargo-config profile overrides;
- leaves unrelated unstable settings and ordinary profile optimization settings alone.

## Decision and security effect

Repository-owned Browser Session build configuration may not replace rustc's code generation backend until the selected backend is explicitly versioned, integrity-bound, reproducibly obtained, and covered by the same compiler/toolchain provenance and release evidence as the Rust toolchain itself. A future approved backend must identify the exact rustc/Cargo toolchain, backend artifact or rustup component, artifact digest/signature/provenance, supported target matrix, fallback behavior, reproducibility evidence, and removal/rollback path before this fail-closed rule is relaxed.

## Residual execution/release provenance

This source contract does not prove ambient command-line or runner state. Remaining surfaces include direct `cargo -Z codegen-backend`, ambient `RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS`, ancestor or `$CARGO_HOME` configuration, rustup component installation, sysroot-provided backends, custom target/toolchain composition, and runner/container image provenance. These belong to executable CI/release evidence unless a repository-owned surface begins selecting them, in which case another focused contract is required.

No hosted repository execution, protected-head GREEN, immutable release, or browser-observed acceptance is claimed by this source-semantic repair alone.
