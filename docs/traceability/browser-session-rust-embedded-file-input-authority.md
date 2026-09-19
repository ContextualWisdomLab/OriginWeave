# Browser Session Rust embedded-file input authority

## Status

Implemented on PR #317 as a focused repository contract. This document records source-semantic evidence only; it is not hosted CI, protected-main integration, or release evidence.

## Problem

`include_bytes!` and `include_str!` are compile-time file inputs. Rust 1.98.1 documents that both macros locate a file relative to the current source file at compile time; `include_bytes!` places the file bytes in a `&'static [u8; N]`, while `include_str!` places UTF-8 file contents in a `&'static str`.

Rust also resolves bang-style macros in the macro namespace, and `use` declarations can create aliases for imported macro names. The standard library re-exports `include_bytes` and `include_str` from `core`. Consequently, checking only the literal invocation spellings `include_bytes!(...)` and `include_str!(...)` is insufficient: `use core::include_bytes as read_blob; read_blob!(...)` selects the same compile-time file bytes while hiding the built-in macro name at the call site.

Before the first generation, `tests/test_browser_session_rust_source_indirection_contract.py` governed `include!`, module/path indirection, and the shared Rust lexical helpers, but it did not classify `include_bytes!` or `include_str!`. Before the alias repair, the supplemental embedded-file contract classified direct and namespaced calls but not callable `use ... as ...` aliases. A reviewed production `.rs` file could therefore select additional file bytes that were not represented in the canonical production-source closure.

For Browser Session, that is a provenance gap: the final artifact may contain compile-time-selected bytes even though the selected file itself is outside the source set inspected by the trusted-adapter/source-indirection contracts.

## Decision

Keep `tests/test_browser_session_trusted_adapter_boundary.py` as the single writer for production Cargo package/source topology and keep `tests/test_browser_session_rust_source_indirection_contract.py` as the shared Rust lexical/source-indirection owner.

The focused supplemental contract, `tests/test_browser_session_rust_embedded_file_input_authority_contract.py`:

- first consumes the existing source-indirection contract;
- consumes the canonical production source closure instead of rediscovering Cargo topology;
- reuses the shared trivia, raw-string, quoted-string, character-literal, `use`, `as`, and use-statement helpers;
- fails closed when lexical production source invokes `include_bytes!` or `include_str!`, including namespaced spellings such as `core::include_bytes!`;
- fails closed when a `use` tree gives either macro a callable alias, including grouped imports and aliases beginning with `_`;
- treats `use ... as _` as an unnameable import rather than callable alias authority, consistent with the Rust Reference;
- does not classify mentions inside Rust comments or string/character/raw-string literals as file-input authority.

The policy is intentionally conservative about macro resolution. A locally shadowed macro named `include_bytes!` or `include_str!`, or a callable alias of those imported names, is still rejected until macro-expansion provenance is modeled. This avoids allowing name shadowing or aliasing to become a bypass around the compile-time file-input boundary.

## RED → repair evidence

Initial structural RED: `5cac6feeadb008e200c8590707f7f18d19f9c6c5`.

The initial RED adds realistic workspaces with existing `unreviewed.bin` and `unreviewed.txt` files and requires the pre-existing source-indirection assertion to reject `include_bytes!(...)` and `include_str!(...)`. Initial repair `c163991ddc84fd519194cdae54643b252ec316d2` adds the direct/namespaced embedded-file classifier while delegating package/source discovery to the existing owners.

Alias-bypass RED: `369b807db9988844626dcf2ea1f38eb67379e5e6`.

That RED adds callable aliases for both built-ins: `use core::include_bytes as read_blob` and `use std::include_str as read_text`. The predecessor classifier sees the built-in name inside the `use` item without a following `!`, then sees only the alias at invocation, so both hostile workspaces pass when they must fail closed.

Minimal alias repair: `ae1cf874a39ffd4b00a67216d6a2caa62e615721`.

The repair stays inside the focused embedded-file contract, reuses the shared Rust lexical/use helpers, and classifies callable aliases without creating another Cargo topology or general Rust lexer owner.

Alias-edge RED: `423c4088409215e30199943b7167150b3082fa3d`.

Reviewing the first repair exposed an identifier-boundary bug: treating any alias beginning with `_` as the special underscore import would allow a callable alias such as `_read_blob`. Repair `b978c3c79ad0d0938bdb9d14c712693b9ed87417` distinguishes the exact unnameable `_` binding from ordinary identifiers beginning with `_`. Coverage successor `3f40f6662cf68c301579d3a70b3e76d0325eb705` adds grouped-use, exact-underscore, and comment controls without changing ownership.

Focused review of exact `2c108fc14e9a2eaee79ea16219248c42aa1fd815` found one valid fixture-coverage gap rather than a classifier defect: the alias scanner consumes shared raw-string and simple-character-literal helpers, but the focused alias contract did not directly regress those lexical paths. Review-driven test-only repair `07841fbb4d84541758c796011dbcc402c6d40471` adds two separate fixtures: raw-string alias text must remain lexical data, and scanning must resume after a character literal so a following real aliased embedded-file input still fails closed. The classifier and ownership boundaries are unchanged by that repair.

No Cargo topology, runtime browser behavior, linker authority, or cross-repository owner is duplicated by this generation.

## Security and buyer effect

The contract prevents Git-reviewed Browser Session Rust source from silently importing unmodeled compile-time file bytes through the standard embedded-file macros even when their invocation names are changed by `use` aliasing. This narrows artifact provenance to inputs that have an explicit reviewed contract rather than relying on the source file alone as evidence of what entered the binary.

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

Rust Project. (2026). *Use declarations*. The Rust Reference. https://doc.rust-lang.org/reference/items/use-declarations.html

Rust Project. (2026). *Namespaces*. The Rust Reference. https://doc.rust-lang.org/reference/names/namespaces.html

Rust Project. (2026). *Macros*. The Rust Reference. https://doc.rust-lang.org/reference/macros.html
