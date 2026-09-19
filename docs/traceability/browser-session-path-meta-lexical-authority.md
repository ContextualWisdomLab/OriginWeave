# Browser Session Rust path-meta lexical authority

## Scope

OriginWeave treats Rust module-source selection as Browser Session build provenance because a `#[path = ...]` meta item can redirect a reviewed module declaration to different source bytes. Cargo production package/source discovery remains owned by `tests/test_browser_session_trusted_adapter_boundary.py`; this slice only classifies already-discovered Rust attribute bodies and does not rediscover Cargo topology.

## Problem

The predecessor `_has_path_meta()` searched every extracted attribute body with a raw regular expression for `path` followed by Rust trivia and `=`. `_rust_attribute_bodies()` correctly excluded comments and string literals while locating attribute boundaries, but the inner classifier then rescanned the attribute body without the same lexical discipline. Consequently harmless data such as `#[doc = "path = \"review_bypass.rs\""]`, a raw doc string containing the same text, or `/* path = ... */` inside an attribute was classified as executable module-source authority.

That is a false-positive authorization result: the gate can reject reviewed source even though Rust has no `path = ...` meta item at that lexical position. Keeping such over-approximation indefinitely would create pressure to weaken the provenance gate instead of making the classifier correspond to Rust syntax.

The first lexical repair exposed the complementary false-negative: Rust raw identifiers use the `r#IDENTIFIER` spelling while the `r#` prefix is not part of the identifier itself. A raw `r#path = "..."` meta item therefore names the same `path` attribute authority, but the initial lexical classifier called the shared identifier helper with raw identifiers disabled and could miss that source-selection surface.

## Constraints

- Do not weaken actual `#[path = ...]`, raw-identifier `#[r#path = ...]`, or nested `cfg_attr(..., path = ...)` fail-closed behavior.
- Do not create a second Cargo package/source scanner.
- Reuse the existing Rust trivia, raw-string, quoted-string, character-literal, and identifier-boundary helpers so source-indirection policies share one lexical model.
- Treat comments as lexical trivia and literals as data, consistent with the Rust Reference.
- Treat a raw identifier according to Rust identifier identity rather than as inert spelling data.
- This is a source-semantic contract repair. It is not hosted exact-head GREEN, protected-main integration, or release evidence.

## Alternatives considered

1. Keep the raw regex and add allowlist entries for documentation strings. Rejected because allowlists would encode incidental text and would still miss arbitrary comment/literal spellings.
2. Parse Rust with a second external parser in this Python contract. Rejected because this policy only needs one bounded lexical distinction and a second parser would create another source-of-truth and dependency surface.
3. Reuse the existing lexical helpers inside `_has_path_meta()`. Selected because it is the smallest causal repair and preserves the existing source-indirection owner.
4. Accept only the ordinary spelling `path` and reject or ignore `r#path`. Rejected because Rust raw-identifier syntax denotes the underlying identifier without `r#`; provenance classification must not depend on that surface spelling.

## Decision and exact evidence

Structural RED **`67789f0cdb55825e1b6caa91b38643dad890759e`** adds a supplemental contract proving three data-token controls and one real nested path-meta authority case:

- ordinary doc-string text containing `path = ...` must not be classified;
- raw doc-string text containing `path = ...` must not be classified;
- non-doc comment text containing `path = ...` must not be classified;
- `cfg_attr(unix, path /* trivia */ = "unix_adapter.rs")` must remain classified.

The predecessor returns `True` for all four cases, so the first three controls are structural RED rather than documentation-only assertions.

Minimal repair **`79ad52cdd2a29da5cedbf1a134aec8779227bdb4`** changes only `_has_path_meta()` in the canonical Rust source-indirection contract. It walks the attribute body using `_skip_rust_trivia()`, `_raw_string_end()`, `_quoted_string_end()`, `_simple_char_literal_end()`, and `_rust_identifier_token_end()`. A lexical `path` token followed by Rust trivia and `=` still returns `True`; comments and string/character data are skipped before token classification.

Follow-up RED **`f8f6e665c2651198825fe806781838ffb1165493`** adds `r#path = "raw_identifier.rs"` as a source-selection authority case. The first lexical repair returns `False` for that spelling, demonstrating a false-negative bypass rather than an invented edge case.

Follow-up repair **`debd5f62b0de4edf45d786859ba6cbfb96f3dd29`** keeps the same lexical classifier and changes only the shared identifier-token call for `path` to `allow_raw=True`. This preserves ordinary `path`, comment/literal exclusion, and nested `cfg_attr` behavior while classifying the raw spelling as the same attribute authority.

No Browser Session runtime code, Cargo topology owner, WebDriver BiDi policy, Wardnet/EgressWeave/Keyverse/contextual-orchestrator contract, workflow, ruleset, or release surface is changed by these repairs.

## Standards and implementation traceability

Rust attributes are tokenized as `# [ Attr ]` / `#! [ Attr ]`; the meta-item grammar includes `SimplePath = Expression` and nested meta-item sequences. The same Reference describes ordinary non-doc comments as whitespace. Rust's identifier grammar includes raw identifiers, and the `r#` prefix is not part of the actual identifier. These rules justify recognizing lexical meta-item tokens while excluding comment/literal payload text and treating `path` / `r#path` as the same identifier authority.

The current rustc source independently confirms the compiler-side owner: `rustc_attr_parsing::attributes::path::PathParser` registers the attribute under `sym::path`, and module expansion selects the first attribute satisfying `has_name(sym::path)` before reading its string value. OriginWeave does not copy that parser; this implementation evidence only anchors the security contract to the compiler behavior it is constraining.

- Rust Project. (2026). *The Rust Reference: Attributes*. https://doc.rust-lang.org/reference/attributes.html
- Rust Project. (2026). *The Rust Reference: Comments*. https://doc.rust-lang.org/reference/comments.html
- Rust Project. (2026). *The Rust Reference: Identifiers*. https://doc.rust-lang.org/reference/identifiers.html
- Rust Project. (2026). *The Rust Reference: Paths*. https://doc.rust-lang.org/reference/paths.html
- Rust Project. (2026). `compiler/rustc_attr_parsing/src/attributes/path.rs`, revision `971903d9aee24befd88423f42826c232e27c8190`. https://github.com/rust-lang/rust/blob/971903d9aee24befd88423f42826c232e27c8190/compiler/rustc_attr_parsing/src/attributes/path.rs
- Rust Project. (2026). `compiler/rustc_expand/src/module.rs`, revision `971903d9aee24befd88423f42826c232e27c8190`. https://github.com/rust-lang/rust/blob/971903d9aee24befd88423f42826c232e27c8190/compiler/rustc_expand/src/module.rs

## Security and operability effect

The repair does not broaden which module-source selectors are permitted. It removes false-positive policy findings caused by inert attribute data and removes the raw-identifier false-negative that could make a real source selector invisible to the provenance contract. A rejected path attribute therefore corresponds to lexical Rust metadata and remains fail closed across ordinary and raw identifier spellings.

## Residual risk and follow-up

The contract is deliberately lexical rather than a complete Rust parser. New Rust attribute/macro forms that can select source bytes without a lexical `path = ...`-equivalent meta item remain a future provenance finding and must be introduced with a hostile fixture and primary-language evidence. Exact-head hosted tests and independent current-head review remain required before this generation can be treated as executable GREEN or release-ready.
