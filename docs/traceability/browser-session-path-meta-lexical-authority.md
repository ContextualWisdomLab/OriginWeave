# Browser Session Rust path-meta lexical authority

## Scope

OriginWeave treats Rust module-source selection as Browser Session build provenance because a `#[path = ...]` meta item can redirect a reviewed module declaration to different source bytes. Cargo production package/source discovery remains owned by `tests/test_browser_session_trusted_adapter_boundary.py`; this slice only classifies already-discovered Rust attribute bodies and does not rediscover Cargo topology.

## Problem

The predecessor `_has_path_meta()` searched every extracted attribute body with a raw regular expression for `path` followed by Rust trivia and `=`. `_rust_attribute_bodies()` correctly excluded comments and string literals while locating attribute boundaries, but the inner classifier then rescanned the attribute body without the same lexical discipline. Consequently harmless data such as `#[doc = "path = \"review_bypass.rs\""]`, a raw doc string containing the same text, or `/* path = ... */` inside an attribute was classified as executable module-source authority.

That is a false-positive authorization result: the gate can reject reviewed source even though Rust has no `path = ...` meta item at that lexical position. Keeping such over-approximation indefinitely would create pressure to weaken the provenance gate instead of making the classifier correspond to Rust syntax.

## Constraints

- Do not weaken actual `#[path = ...]` or nested `cfg_attr(..., path = ...)` fail-closed behavior.
- Do not create a second Cargo package/source scanner.
- Reuse the existing Rust trivia, raw-string, quoted-string, character-literal, and identifier-boundary helpers so source-indirection policies share one lexical model.
- Treat comments as lexical trivia and literals as data, consistent with the Rust Reference.
- This is a source-semantic contract repair. It is not hosted exact-head GREEN, protected-main integration, or release evidence.

## Alternatives considered

1. Keep the raw regex and add allowlist entries for documentation strings. Rejected because allowlists would encode incidental text and would still miss arbitrary comment/literal spellings.
2. Parse Rust with a second external parser in this Python contract. Rejected because this policy only needs one bounded lexical distinction and a second parser would create another source-of-truth and dependency surface.
3. Reuse the existing lexical helpers inside `_has_path_meta()`. Selected because it is the smallest causal repair and preserves the existing source-indirection owner.

## Decision and exact evidence

Structural RED **`67789f0cdb55825e1b6caa91b38643dad890759e`** adds a supplemental contract proving three data-token controls and one real nested path-meta authority case:

- ordinary doc-string text containing `path = ...` must not be classified;
- raw doc-string text containing `path = ...` must not be classified;
- non-doc comment text containing `path = ...` must not be classified;
- `cfg_attr(unix, path /* trivia */ = "unix_adapter.rs")` must remain classified.

The predecessor returns `True` for all four cases, so the first three controls are structural RED rather than documentation-only assertions.

Minimal repair **`79ad52cdd2a29da5cedbf1a134aec8779227bdb4`** changes only `_has_path_meta()` in the canonical Rust source-indirection contract. It walks the attribute body using `_skip_rust_trivia()`, `_raw_string_end()`, `_quoted_string_end()`, `_simple_char_literal_end()`, and `_rust_identifier_token_end()`. A lexical `path` token followed by Rust trivia and `=` still returns `True`; comments and string/character data are skipped before token classification.

No Browser Session runtime code, Cargo topology owner, WebDriver BiDi policy, Wardnet/EgressWeave/Keyverse/contextual-orchestrator contract, workflow, ruleset, or release surface is changed by this repair.

## Standards traceability

Rust attributes are tokenized as `# [ Attr ]` / `#! [ Attr ]`; the meta-item grammar includes `SimplePath = Expression` and nested meta-item sequences. The same Reference describes ordinary non-doc comments as whitespace. These rules justify recognizing lexical meta-item tokens while excluding comment and literal payload text from source-selection authority.

- Rust Project. (2026). *The Rust Reference: Attributes*. https://doc.rust-lang.org/reference/attributes.html
- Rust Project. (2026). *The Rust Reference: Comments*. https://doc.rust-lang.org/reference/comments.html
- Rust Project. (2026). *The Rust Reference: Paths*. https://doc.rust-lang.org/reference/paths.html

## Security and operability effect

The repair does not broaden which actual module-source selectors are allowed. It removes false-positive policy findings that arose from inert attribute data while retaining fail-closed treatment for real `path = ...` meta items. This keeps evidence classification explainable: a rejected path attribute now corresponds to lexical Rust metadata rather than a substring that happened to occur inside data.

## Residual risk and follow-up

The contract is deliberately lexical rather than a complete Rust parser. New Rust attribute/macro forms that can select source bytes without a lexical `path = ...` meta item remain a future provenance finding and must be introduced with a hostile fixture and primary-language evidence. Exact-head hosted tests and independent current-head review remain required before this generation can be treated as executable GREEN or release-ready.
