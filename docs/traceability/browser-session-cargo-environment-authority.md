# Browser Session Cargo environment authority

## Problem

Repository-owned Cargo configuration is part of the reviewed Browser Session build provenance. Cargo's `[env]` table injects environment variables into processes it runs, including `rustc` invocations. Rust source can consume those values at compile time through `env!` and `option_env!`, and procedural macros execute during compilation with the compiler's resources. A Git-owned `[env]` entry can therefore change compiler-visible inputs or generated code without changing the reviewed Rust source or the existing `rustflags`/`rustdocflags` surface.

## Authority and constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. `tests/test_browser_session_cargo_compiler_authority_contract.py` owns repository-selected compiler, rustdoc, linker, and compiler-environment authority. Supplemental contracts consume that owner rather than rediscovering Cargo configuration.

Until OriginWeave has a purpose-bounded, versioned environment allowlist with exact consumer and artifact provenance, any non-empty Git-owned Cargo `[env]` table is fail-closed. An empty `[env]` table is permitted because it introduces no compiler-visible value.

This policy covers both string values and Cargo's table form, including `force = true` and `relative = true`. The latter can turn a repository-relative value into an absolute path before it is exposed to Cargo-run processes.

## RED → repair evidence

RED `46c0c90d6162a43db3857186d4ab1731e4b0f42c` adds `tests/test_browser_session_cargo_environment_authority_contract.py`. It proves that ordinary string injection, forced replacement, and config-relative path injection were previously accepted by the canonical Cargo compiler-authority owner while retaining an empty `[env]` table as a control.

Repair `67cd0f16ca5a9eeaa467ad32bbc55f60d9d1cb98` minimally extends the existing parsed-config owner. It records non-empty `[env]` keys as unmodeled compiler-environment authority and rejects them through the same Browser Session provenance error path. No second Cargo topology or config scanner is introduced.

## Security and reproducibility effect

A checked-in Cargo config can no longer change compiler-visible environment values outside the reviewed Browser Session build-input contract. This closes source-visible compile-time inputs consumed by `env!` / `option_env!` and reduces unreviewed environment available to compile-time code such as procedural macros.

This repository-source contract does not prove the ambient execution environment. Shell variables, runner image configuration, `$CARGO_HOME` or ancestor Cargo configuration, command-line `--config`, CI-injected secrets, compiler/toolchain installation state, and environment inherited from the host remain CI/release/runtime provenance surfaces. They require canonical execution-environment controls rather than inference from Git source closure.

## Primary references

- The Rust Project. (2026). *The Cargo Book: Configuration — `[env]`*. https://doc.rust-lang.org/cargo/reference/config.html#env
- The Rust Project. (2026). *Rust core macro `env!`*. https://doc.rust-lang.org/core/macro.env.html
- The Rust Project. (2026). *Rust core macro `option_env!`*. https://doc.rust-lang.org/core/macro.option_env.html
- The Rust Project. (2026). *The Rust Reference: Procedural macros*. https://doc.rust-lang.org/reference/procedural-macros.html

Cargo documents that `[env]` values are provided to build scripts and `rustc` invocations and that `force` and `relative` alter replacement and path-resolution behavior. Rust documents that `env!` and `option_env!` inspect environment variables at compile time, while procedural macros execute during compilation with the compiler's resources and build-script-like security concerns.
