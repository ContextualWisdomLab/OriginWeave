# Browser Session custom-target `mod` lexical authority

Status: active PR evidence only. This document does not claim protected-main, hosted-check, or release acceptance.

## Problem

`tests/test_browser_session_rust_source_indirection_contract.py` intentionally fails closed when a reviewed Cargo production target whose crate root is outside the package's default `src/` tree contains a Rust `mod` token. That guard prevents a custom target such as `runtime/lifecycle_adapter.rs` from loading an outlined sibling module that is not already covered by the canonical `src/**/*.rs` production-source closure.

The predecessor implementation used `CUSTOM_TARGET_MOD_TOKEN.search(text)` over raw source bytes. The policy was conservative, but the implementation also treated `mod ...` text inside non-doc comments and ordinary/raw string literals as executable module authority. That is a lexical false positive: Rust non-doc comments are interpreted as whitespace, and string/raw-string literals are tokens whose contents are data rather than item grammar.

This matters commercially because a fail-closed provenance guard still has to distinguish executable authority from inert source text. Rejecting harmless comments or literal data creates avoidable adoption friction and encourages pressure to weaken the security gate instead of repairing its lexical boundary.

Primary references:

- Rust Reference, comments: https://doc.rust-lang.org/reference/comments.html
- Rust Reference, literal expressions: https://doc.rust-lang.org/reference/expressions/literal-expr.html
- Rust Reference, modules and module source filenames: https://doc.rust-lang.org/reference/items/modules.html

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py::_workspace_production_sources()` remains the single writer for Cargo production package/source topology.
- The custom-target rule remains intentionally conservative for *real lexical* `mod` tokens outside default `src/`: it still does not attempt to distinguish inline from outlined modules or reproduce the Rust parser.
- The repair must reuse the existing Rust trivia/raw-string/quoted-string/character-literal scanner discipline already used by `include!`, `use`-alias, and attribute discovery rather than introduce a second lexer.
- No new source path, module tree, adapter, or dependency is authorized.
- Default `src/` module trees remain governed by the canonical production-source closure rather than this custom-target guard.

## Decision

Custom-target module detection now advances through the same lexical boundaries already used by the source-indirection contract. Non-doc line/block comments, normal strings, raw strings, and character literals are skipped before `CUSTOM_TARGET_MOD_TOKEN` is matched. A real lexical `mod` token still fails closed exactly as before; only inert comment/literal contents stop being treated as module authority.

The repair deliberately does not parse module grammar. `mod helper;`, `mod r#type;`, `mod 관찰;`, and `mod /* trivia */ helper;` in a custom target root remain provenance stops. The change only removes raw-text false positives where no lexical `mod` token exists.

## RED → repair evidence

- Predecessor exact `84fb37d31fbb5f1145b78a700ce77771319a032b` used raw `CUSTOM_TARGET_MOD_TOKEN.search(text)` for custom-target roots.
- Structural RED `bd457be344c729d550e301a403e77bb5959e5b28` adds line-comment, ordinary-string, and raw-string controls. On the predecessor implementation these fixtures are rejected even though the `mod` spelling is lexical data.
- Minimal repair `6f8b0706acaf5837e3581f1c77d43c318a54462c` adds `_has_custom_target_mod_token()` and changes the custom-target guard to consume it. The helper reuses `_skip_rust_trivia()`, `_raw_string_end()`, `_quoted_string_end()`, and `_simple_char_literal_end()`; Cargo topology ownership is unchanged.
- Compare `84fb37d3... → bd457be3...` is one test-only file, +64/-0. Compare `84fb37d3... → 6f8b0706...` is two files: the RED contract plus the canonical lexical repair, with no production Rust implementation change.

## Risk and follow-up

This remains a temporary lexical security boundary. It is not compiler-derived source-input provenance and can intentionally reject legitimate inline modules in custom target roots. Before OriginWeave needs such custom-target module trees, replace the heuristic with compiler-derived or equivalently exact source-input evidence that identifies the actual bytes compiled for supported target configurations without widening Cargo ownership or relying on source-text approximations.

The existing `docs/traceability/browser-session-rust-source-indirection.md` remains the broader source-indirection record. This document narrows only the current custom-target lexical correction and should be folded into that canonical record when the current stacked Browser Session lineage is reconciled.
