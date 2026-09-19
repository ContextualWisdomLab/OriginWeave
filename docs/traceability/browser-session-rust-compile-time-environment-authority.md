# Browser Session Rust compile-time environment authority

## Status

Implemented on PR #317 as a focused repository contract. This document records source-semantic evidence only; it is not hosted CI, protected-main integration, or release evidence.

## Problem

Rust 1.98.1 documents `env!` as reading an environment variable at compile time and expanding to its string value, and `option_env!` as the optional form that expands to `Option<&'static str>`. These values can therefore enter Browser Session artifacts without appearing in the reviewed Rust source, dependency graph, or linker-input set.

Cargo can also set compilation environment values through build-script `cargo::rustc-env=VAR=VALUE`. The Cargo Book explicitly describes retrieving such values with `env!` in the compiled crate. Repository `[env]` configuration is already governed by `tests/test_browser_session_cargo_environment_authority_contract.py`, but that contract does not make every ambient runner variable or build-script-produced value part of reviewed artifact provenance.

Before this generation, the Rust source-indirection owner governed `include!`, module/path indirection, and the shared Rust lexical helpers; the embedded-file supplement governed `include_bytes!` and `include_str!`. Neither classified direct source-level `env!` or `option_env!`. The initial compile-time-environment repair then closed direct and namespaced spellings but still allowed the same built-in macros to be imported under a callable alias such as `use std::env as read_build_env; read_build_env!(...)`. Rust resolves that alias as the macro, so the artifact can still depend on ambient build state while the invocation no longer contains the literal token `env` or `option_env`.

## Decision

Keep `tests/test_browser_session_trusted_adapter_boundary.py` as the single writer for production Cargo package/source topology and `tests/test_browser_session_rust_source_indirection_contract.py` as the shared Rust lexical/source-indirection owner.

`tests/test_browser_session_rust_compile_time_environment_authority_contract.py` remains a focused supplemental contract that:

- first consumes the existing source-indirection assertion;
- consumes the canonical production-source closure instead of rediscovering Cargo topology;
- reuses the shared trivia, raw-string, quoted-string, character-literal, `use`/`as`, use-statement, and Rust identifier-boundary helpers;
- fails closed on lexical `env!` and `option_env!`, including namespaced spellings;
- fails closed when a Rust `use` tree gives either macro a callable direct, grouped, or raw-identifier alias;
- treats exact `as _` as a discard import, while `_` followed by a Rust identifier-continuation scalar remains a callable identifier rather than a discard alias;
- ignores mentions inside comments and string/character/raw-string literals;
- conservatively rejects locally shadowed macros with the same names until macro-expansion provenance is modeled.

The policy does not treat runtime `std::env::var` as the same build-input class. Runtime environment access is a separate product/runtime authority concern and must be governed by the runtime boundary that owns it.

## RED → repair evidence

Structural RED: `87eb778a1b5526811e43b851fe839755ee224ca2`.

The RED adds realistic workspaces whose production Rust source calls `env!` and `option_env!` and asks the pre-existing source-indirection assertion to reject them. The predecessor `c52ad3a5fdcc7e482e8a6b872e1e3d44fed6477a` has no compile-time environment classifier, so those assertions expose the missing provenance boundary while comment/string controls remain accepted.

Minimal repair: `a6eec1a700aff4cd5ad807629e6f55e44abde2aa`.

The repair stays inside the focused contract. It adds one compile-time-environment macro classifier, delegates source discovery and lexical handling to existing owners, adds a current-production postcondition, covers direct/optional/namespaced forms, and preserves comment/string controls. No Cargo topology, runtime environment policy, browser behavior, linker authority, or cross-repository owner is duplicated.

Focused review of traceability exact `75eb414f69a7be2dcc851aca58d47e156a41133c` found a valid fixture-coverage gap rather than a classifier defect: the supplemental contract depended on shared raw-string and character-literal handling without directly exercising those boundaries, and it covered namespaced `env!` but not namespaced `option_env!`. Review-driven coverage repair `f9934fe67c6cf7bd5c0ab946be6b881503a14c65` changes only the focused contract (`+19/-1`). It adds a raw-string false-positive control, proves that scanning resumes after a character literal and still rejects a following real macro, and rejects `core::option_env!`.

Focused re-review of exact `f12499cba44f95733cd4d8bb006548aff8804858e` found no defect in the direct/namespaced compile-time-environment slice. That verdict predates the callable-alias generation and is not treated as current-head review evidence.

A later CodeRabbit security review found a valid remaining bypass: Rust permits imports such as `use std::env as read_build_env;`, after which `read_build_env!(...)` executes the same compile-time environment macro without exposing the literal macro name at the call site. Structural RED `845d49bc608dbbb39c4e5de965426a549e54b639` adds direct `env!`, grouped `option_env!`, and raw-identifier alias hostile fixtures plus an exact `as _` control. Minimal repair `4830e4215b0340ac582c8afd9648b026ae4ff5d4` adds callable use-tree alias detection inside the focused supplemental contract while reusing the canonical source closure and shared Rust lexical helpers. The repair also covers a combining-mark continuation after `_` so Unicode `XID_Continue` input cannot be mistaken for the exact discard alias.

This alias repair is source-semantic evidence only until the exact successor receives fresh review and hosted execution. No predecessor focused-review verdict is carried forward as proof for the new generation.

## Security and buyer effect

The contract prevents Git-reviewed Browser Session Rust source from silently binding artifact content to ambient build values through the two standard compile-time environment macros, whether invoked by their built-in spelling, a namespace-qualified spelling, or a callable `use` alias. This narrows release provenance: a build cannot claim source-only reproducibility while an unmodeled environment variable changes compiled bytes or embedded metadata through a trivially renamed macro.

This is necessary but not sufficient for reproducible release evidence. Runner environment, build-script output, proc-macro or declarative-macro expansion that synthesizes equivalent calls, generated source, direct compiler invocation, and externally injected Cargo environment remain CI/release supply-chain evidence surfaces unless separately attested.

## Acceptance and rollback

Acceptance for this generation requires all of the following on the reconciled exact head:

- the focused contract passes with the existing Rust source-indirection, embedded-file, Cargo-environment, and trusted-adapter contracts;
- repository/security workflows run on the exact head and pass without gate weakening;
- current-head review confirms the supplemental contract consumes rather than duplicates canonical topology/lexical ownership and that callable-alias handling matches Rust identifier semantics;
- release evidence, if produced, binds the exact source tree, toolchain, environment-variable names and values that may affect compilation, producer identity for generated values, SBOM/provenance, independent reproducibility, and rollback.

If the product later needs a compile-time environment value, do not delete the fail-closed rule. Replace it in the same reviewed change with a versioned contract that binds variable name, purpose, producer, canonical value or digest, secrecy classification, target/toolchain scope, invalidation semantics, SBOM/provenance linkage, independent reproducibility, and rollback.

## References

Rust Project. (2026). *env macro (Rust 1.98.1)*. The Rust Standard Library. https://doc.rust-lang.org/core/macro.env.html

Rust Project. (2026). *option_env macro (Rust 1.98.1)*. The Rust Standard Library. https://doc.rust-lang.org/core/macro.option_env.html

Rust Project. (2026). *Identifiers*. The Rust Reference. https://doc.rust-lang.org/reference/identifiers.html

Rust Project. (2026). *Build scripts*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/build-scripts.html
