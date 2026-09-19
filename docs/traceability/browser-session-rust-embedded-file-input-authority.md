# Browser Session Rust embedded-file input authority

## Status

Implemented on PR #317 as a focused repository contract. This document records source-semantic evidence only; it is not hosted CI, protected-main integration, or release evidence.

## Problem

`include_bytes!` and `include_str!` are compile-time file inputs. Rust 1.98.1 documents that both macros locate a file relative to the current source file at compile time; `include_bytes!` places the file bytes in a `&'static [u8; N]`, while `include_str!` places UTF-8 file contents in a `&'static str`.

Before this generation, `tests/test_browser_session_rust_source_indirection_contract.py` governed `include!`, module/path indirection, and the shared Rust lexical helpers, but it did not classify `include_bytes!` or `include_str!`. A reviewed production `.rs` file could therefore select additional file bytes that were not represented in the canonical production-source closure.

For Browser Session, that is a provenance gap: the final artifact may contain compile-time-selected bytes even though the selected file itself is outside the source set inspected by the trusted-adapter/source-indirection contracts.

## Decision

Keep `tests/test_browser_session_trusted_adapter_boundary.py` as the single writer for production Cargo package/source topology and keep `tests/test_browser_session_rust_source_indirection_contract.py` as the shared Rust lexical/source-indirection owner.

Add a focused supplemental contract, `tests/test_browser_session_rust_embedded_file_input_authority_contract.py`, that:

- first consumes the existing source-indirection contract;
- consumes the canonical production source closure instead of rediscovering Cargo topology;
- reuses the shared trivia, raw-string, quoted-string, and character-literal lexer helpers;
- fails closed when lexical production source invokes `include_bytes!` or `include_str!`, including namespaced spellings such as `core::include_bytes!`;
- does not classify mentions inside Rust comments or string/character/raw-string literals as file-input authority.

The policy is intentionally conservative about macro resolution. A locally shadowed macro named `include_bytes!` or `include_str!` is still rejected until macro-expansion provenance is modeled. This avoids allowing name shadowing to become a bypass around the compile-time file-input boundary.

## RED → repair evidence

Structural RED: `5cac6feeadb008e200c8590707f7f18d19f9c6c5`.

The RED adds realistic workspaces with existing `unreviewed.bin` and `unreviewed.txt` files and requires the pre-existing source-indirection assertion to reject `include_bytes!(...)` and `include_str!(...)`. The predecessor has no such classifier, so those requirements expose the gap rather than manufacturing an unrelated failure.

Minimal repair: `c163991ddc84fd519194cdae54643b252ec316d2`.

The repair changes only the new focused contract. It introduces the embedded-file macro classifier, delegates all package/source discovery to the existing owners, adds a current-production postcondition, covers direct and namespaced selectors, and adds comment/string controls. No Cargo topology, runtime browser behavior, linker authority, or cross-repository owner is duplicated.

## Security and buyer effect

The contract prevents Git-reviewed Browser Session Rust source from silently importing unmodeled compile-time file bytes through these two standard macros. This narrows artifact provenance to inputs that have an explicit reviewed contract rather than relying on the source file alone as evidence of what entered the binary.

This is necessary but not sufficient for release provenance. Ambient filesystem contents, environment-driven paths, proc-macro or declarative-macro expansion that synthesizes equivalent file inputs, generated source, and direct compiler invocation remain CI/release supply-chain surfaces unless separately attested.

## Acceptance and rollback

Acceptance for this generation requires all of the following on the reconciled exact head:

- the focused contract passes together with the existing Rust source-indirection and trusted-adapter contracts;
- repository/security workflows run on the exact head and pass without gate weakening;
- current-head review confirms the supplemental contract consumes rather than duplicates canonical topology/lexical ownership;
- release evidence, if a release is produced, records the exact source tree, toolchain, filesystem/input provenance, SBOM/provenance, and reproducible-build result.

If the product later needs compile-time embedded files, do not delete the fail-closed rule. Replace it in the same reviewed change with a versioned manifest or equivalent contract that binds source selector, canonical path, content digest, producer/source identity, containment/symlink rules, target/toolchain compatibility, SBOM/provenance, independent reproducibility, invalidation, and rollback.

## References

Rust Project. (2026). *include_bytes macro (Rust 1.98.1)*. The Rust Standard Library. https://doc.rust-lang.org/stable/std/macro.include_bytes.html

Rust Project. (2026). *include_str macro (Rust 1.98.1)*. The Rust Standard Library. https://doc.rust-lang.org/stable/std/macro.include_str.html
