# Browser Session custom-target `mod` lexical authority

Status: active PR evidence only. This document does not claim protected-main, hosted-check, or release acceptance.

## Problem

`tests/test_browser_session_rust_source_indirection_contract.py` intentionally fails closed when a reviewed Cargo production target whose crate root is outside the package's default `src/` tree contains a Rust `mod` token. That guard prevents a custom target such as `runtime/lifecycle_adapter.rs` from loading an outlined sibling module that is not already covered by the canonical `src/**/*.rs` production-source closure.

The predecessor implementation used `CUSTOM_TARGET_MOD_TOKEN.search(text)` over raw source bytes. The policy was conservative, but the implementation also treated `mod ...` text inside non-doc comments and ordinary/raw string literals as executable module authority. That is a lexical false positive: Rust non-doc comments are interpreted as whitespace, and string/raw-string literals are tokens whose contents are data rather than item grammar.

The first lexical repair exposed a second, independent language-boundary defect. Python regular-expression `\w` is not Rust's identifier grammar. Rust identifiers use Unicode `XID_Start`/`XID_Continue` from Unicode 17.0, so U+0301 COMBINING ACUTE ACCENT can continue an identifier even though Python's `\w` boundary does not treat it as a word character. Consequently `mod\u0301` is one identifier token, not the strict `mod` keyword, and a Python-`\w` keyword boundary can reject source that has no lexical `mod` token.

The same audit found that Rust's lexical whitespace is the stable Unicode `Pattern_White_Space` set, not Python `str.isspace()`. In particular U+200E LEFT-TO-RIGHT MARK and U+200F RIGHT-TO-LEFT MARK are legal Rust whitespace. Failing to skip them between `include`, `!`, and the macro delimiter creates a false negative in the existing source-provenance stop.

This matters commercially because a fail-closed provenance guard still has to distinguish executable authority from inert source text without missing legal Rust token separation. False positives create avoidable adoption friction; false negatives permit compile-time source bytes to enter outside the reviewed provenance boundary.

Primary references:

- Rust Reference, identifiers (`XID_Start`/`XID_Continue`, Unicode 17.0): https://doc.rust-lang.org/reference/identifiers.html
- Rust Reference, whitespace (`Pattern_White_Space`): https://doc.rust-lang.org/reference/whitespace.html
- Rust Reference, comments: https://doc.rust-lang.org/reference/comments.html
- Rust Reference, literal expressions: https://doc.rust-lang.org/reference/expressions/literal-expr.html
- Rust Reference, modules and module source filenames: https://doc.rust-lang.org/reference/items/modules.html

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py::_workspace_production_sources()` remains the single writer for Cargo production package/source topology.
- The custom-target rule remains intentionally conservative for *real lexical* `mod` tokens outside default `src/`: it still does not attempt to distinguish inline from outlined modules or reproduce the Rust parser.
- The scanner must reuse the existing Rust trivia/raw-string/quoted-string/character-literal discipline already used by `include!`, `use`-alias, and attribute discovery rather than introduce a second lexer.
- Python's Unicode database must not silently define Rust keyword identity. The checked Rust Reference currently targets Unicode 17.0, so the boundary cannot assume Python `\w` or the local Python runtime's identifier tables are equivalent.
- Rust whitespace handling must use the language's exact stable `Pattern_White_Space` set. Generic host-language whitespace predicates are not lexical authority.
- No new source path, module tree, adapter, or dependency is authorized.
- Default `src/` module trees remain governed by the canonical production-source closure rather than this custom-target guard.

## Decision

Custom-target module detection advances through the same lexical boundaries already used by the source-indirection contract. Non-doc line/block comments, normal strings, raw strings, and character literals are skipped before the `mod` spelling is considered. A real lexical `mod` token still fails closed exactly as before; only inert comment/literal contents stop being treated as module authority.

The `mod` keyword boundary no longer relies on Python `\w`. ASCII identifier continuation is handled directly. For non-ASCII adjacency the guard is deliberately conservative: any non-ASCII scalar that is not Rust `Pattern_White_Space` prevents classification as the ASCII `mod` keyword. This covers current and future Unicode identifier-continuation additions without pretending the host Python Unicode table is Rust's versioned `XID_Continue` authority. Rust's eleven `Pattern_White_Space` code points are explicit and stable, so non-ASCII legal whitespace such as U+200E continues to separate a real `mod` keyword.

The shared trivia skipper now uses that exact Rust whitespace set as well. This closes legal U+200E/U+200F separation around `include!` and removes host-only whitespace from the lexer contract. Comment handling remains nested and unchanged.

The repair deliberately does not parse module grammar. `mod helper;`, `mod r#type;`, `mod 관찰;`, and `mod /* trivia */ helper;` in a custom target root remain provenance stops. The change only removes raw-text/identifier-boundary false positives and closes Rust-whitespace false negatives.

## RED → repair evidence

- Predecessor exact `84fb37d31fbb5f1145b78a700ce77771319a032b` used raw `CUSTOM_TARGET_MOD_TOKEN.search(text)` for custom-target roots.
- Structural RED `bd457be344c729d550e301a403e77bb5959e5b28` adds line-comment, ordinary-string, and raw-string controls. On the predecessor implementation these fixtures are rejected even though the `mod` spelling is lexical data.
- Minimal repair `6f8b0706acaf5837e3581f1c77d43c318a54462c` adds `_has_custom_target_mod_token()` and changes the custom-target guard to consume it. The helper reuses `_skip_rust_trivia()`, `_raw_string_end()`, `_quoted_string_end()`, and `_simple_char_literal_end()`; Cargo topology ownership is unchanged.
- Edge coverage `a4ad6d45d101b79460fbc06ca9ede9b1c3b0b140` adds nested-block-comment lexical data and proves scanning resumes after a character literal to catch a later real `mod` token.
- Focused CodeRabbit review of `c36f8630cfdc9887a3fda8716c3cccbc0ae370b6` found a valid remaining identifier-boundary false positive: Python `\w` does not model Rust Unicode 17.0 `XID_Continue`, so `mod\u0301` was incorrectly classified as the strict keyword.
- Review-driven RED `a214c87d375e75a9c10edc3c6de99b4847c43327` preserves `mod\u0301` as identifier data. Repair `ec32bd9f481280b522ce7554d392021b40c7fea1` replaces the Python-regex keyword boundary with a version-independent conservative Rust boundary backed by the exact stable `Pattern_White_Space` set.
- Edge coverage `df940ba25b2c7731d6fe631290f000ce1d57bcd0` adds a combining-mark-before-`mod` control and proves U+200E Rust whitespace still separates a real `mod` keyword.
- Root-cause audit then exposed a real false negative in the shared trivia owner: Python `str.isspace()` does not recognize U+200E/U+200F even though Rust does. Structural RED `ce33ae1aa020b1b9903aa02c5952a90bfd1581e5` adds hostile `include\u200e!` and `include!\u200f(` forms. Repair `e199aac4a64e636b3a7412f3d9347644e96e636b` makes `_skip_rust_trivia()` consume the exact Rust `Pattern_White_Space` set.

## Risk and follow-up

This remains a temporary lexical security boundary. It is not compiler-derived source-input provenance and can intentionally reject legitimate inline modules in custom target roots. Before OriginWeave needs such custom-target module trees, replace the heuristic with compiler-derived or equivalently exact source-input evidence that identifies the actual bytes compiled for supported target configurations without widening Cargo ownership or relying on source-text approximations.

The root-cause audit also shows that other source-indirection token recognizers still use Python-regex `\w` boundaries (`include`, `use`, `as`, and `path`). Their concrete security/false-positive behavior must be verified with Rust `XID_Continue` hostile/control fixtures before claiming the source-indirection lexer is fully Unicode-current. Do not widen those owners by assumption; preserve a structural RED before any shared boundary change.

The existing `docs/traceability/browser-session-rust-source-indirection.md` remains the broader source-indirection record. This document narrows the current custom-target lexical correction and shared Rust-whitespace root repair and should be folded into that canonical record when the current stacked Browser Session lineage is reconciled.
