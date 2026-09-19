# Browser Session Rust `include!` lexical authority

Status: Draft repair evidence on the #317 Browser Session source-provenance lane. Hosted exact-head repository/security acceptance, whole-PR review, and protected-main shipment remain separate gates.

## Problem

OriginWeave deliberately fails closed when reviewed production Rust source uses `include!`, because the macro parses another file into the surrounding crate at compile time and therefore extends the executable source-input closure. The existing detector searched the raw source text with `INCLUDE_TOKEN.finditer(...)` and likewise searched raw text for `use ... include as ...` aliases. That classified `include!(...)` or `use core::include as ...` appearing only inside comments or string literals as executable authority.

That behavior is conservative but incorrect: Rust non-doc comments are lexically whitespace, and string/character/raw-string contents are literal tokens rather than macro invocations. A provenance guard that cannot distinguish lexical data from executable syntax creates false-positive security failures and makes reviewed documentation/log strings an accidental build-authority gate.

## Evidence and repair

- Structural RED `6bf90e950ebbe09f28f56d4e6665433265cb238d` adds independent controls requiring line-comment and ordinary-string `include!(...)` text to remain lexical data while a real `include!(...)` invocation still fails closed.
- Minimal causal repair `6ad8f195e1bc9c649024b1e29244f00313d9a3e9` keeps ownership in `tests/test_browser_session_rust_source_indirection_contract.py`. `_has_include_macro()` and `_has_aliased_include_import()` now reuse the existing Rust trivia/raw-string/quoted-string/character-literal helpers before recognizing `include` or `use` tokens. No Cargo topology, production crate, or browser-domain authority is duplicated.
- Edge-case successor `2f3311b47509e86421917099c678919f39c43d01` adds raw-string false-positive coverage, commented `use core::include as ...` coverage, and character-literal scan resumption before a real hostile `include!`.

The repair is intentionally lexical rather than pathname-based. Allowlisting a path would not fix the category error: comment/literal text must never become authority regardless of its spelling, while a real `include!` remains provenance-relevant even for an in-repository path until the compiler-derived source-input contract explicitly models it.

## Invariants

1. A lexical `include!` macro invocation in a reviewed production source fails closed until an explicit source-provenance contract admits the included file.
2. A lexical `use ... include as <callable>` declaration fails closed under the same authority boundary.
3. Line/block comments and ordinary/raw string or character literal contents do not create source-input authority merely because their text resembles `include!` or an alias declaration.
4. Scanning resumes after a literal token and still rejects a following real `include!` invocation.
5. `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology; this repair changes only lexical classification inside the existing Rust source-indirection owner.

## Rejected alternatives

- **Keep raw-text matching because it is safer.** Rejected. False positives make comments and documentation semantically equivalent to compiler inputs, which is not a defensible security boundary and creates pressure to add ad hoc suppressions.
- **Add pathname exceptions.** Rejected. The defect is lexical classification, not path selection; exceptions would weaken the real `include!` boundary without fixing comment/literal handling.
- **Create another repository-wide Rust scanner.** Rejected. That would violate single-writer ownership and duplicate the existing shared lexer/topology contracts.

## Acceptance

This slice is acceptable only when the RED controls are demonstrably failing on the predecessor and GREEN on the repair generation, the current production-source contract remains fail closed for real `include!`, current-head static review finds no ownership or lexer regression, and hosted repository/security checks are independently satisfied after the central workflow prerequisite chain permits them. Command acknowledgement or a static review response is not hosted GREEN.

## Authoritative references

Rust Project. (2026). *The Rust Reference: Comments*. https://doc.rust-lang.org/reference/comments.html (retrieved September 19, 2026). Non-doc comments are tokenized as whitespace.

Rust Project. (2026). *The Rust Reference: Macros — macro invocation*. https://doc.rust-lang.org/reference/macros.html (retrieved September 19, 2026). A macro invocation has the token form `SimplePath ! DelimTokenTree`.

Rust Project. (2026). *Macro `include`*. https://doc.rust-lang.org/nightly/core/macro.include.html (retrieved September 19, 2026). `include!` parses another file into the surrounding context at compile time and resolves the path relative to the current file.
