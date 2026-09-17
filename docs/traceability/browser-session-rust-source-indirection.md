# Browser Session Rust source-indirection provenance

Status: active PR evidence only. This contract does not claim protected-main or release acceptance.

## Problem

The Browser Session trusted-adapter boundary already derives one canonical Cargo production package/source closure and rejects repository-external or dangling source objects. That is not sufficient by itself because reviewed Rust source can cause `rustc` to parse additional source bytes through language-level indirection.

Four related forms matter here:

- `include!(...)`, `include![...]`, and `include! {...}` parse another file as an expression or item. Rust macro invocation syntax permits all three delimiter forms, and the included path is relative to the source file containing the invocation.
- `#[path = "..."] mod ...;` changes the source file used for an outlined module. Rust documents the path attribute as a module-source filename override whose relative interpretation depends on the module location.
- `#[cfg_attr(..., path = "...")]` can conditionally synthesize the same `path` attribute. A direct `#[path]`-only lexical check therefore does not cover the full Rust attribute surface.
- A `mod` item can cause the compiler to load another Rust file. Cargo's ordinary `src/**/*.rs` review already contains default module trees conservatively, but a custom Cargo target whose crate root lives outside `src/` can load sibling module files that are not in that closure. Rust permits comments and other trivia between grammar tokens, Unicode XID identifiers, and raw identifiers, so a security contract must not encode a narrower hand-written identifier/module grammar and call it complete.

Rust comments are also lexical trivia rather than `\s`-only whitespace. A valid attribute can therefore spell the source-loading meta item as `#[path /* reviewed trivia */ = "nested.rs"]`, including nested block comments or line-comment trivia. A regular expression that searches only for `path\s*=` silently narrows Rust's lexical grammar. Likewise, a naive `[^\]]*` attribute capture can be truncated by `]` inside a comment or string even though that byte is not the attribute's closing delimiter.

Primary references:

- Rust `include!` macro: https://doc.rust-lang.org/stable/std/macro.include.html
- Rust Reference, macro invocation syntax: https://doc.rust-lang.org/reference/macros.html#macro-invocation
- Rust Reference, module source filenames and `path` attribute: https://doc.rust-lang.org/reference/items/modules.html#module-source-filenames
- Rust Reference, conditional attributes with `cfg_attr`: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
- Rust Reference, identifiers and raw identifiers: https://doc.rust-lang.org/reference/identifiers.html
- Rust Reference, comments: https://doc.rust-lang.org/reference/comments.html
- Rust Reference, attributes: https://doc.rust-lang.org/reference/attributes.html
- Rust 2018 Edition Guide, module file layout: https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html#no-more-modrs

Without an explicit contract, a future lifecycle adapter could keep its crate, manifest, and declared custom target inside the exact Git review root while compiling additional Rust source not represented by the canonical production-source closure. That would weaken the same provenance boundary used for `DisposableContextPort`, `bind_lifecycle_port`, dependency, and source-containment review.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py::_workspace_production_sources()` remains the single writer for Cargo production package/source topology.
- This contract consumes that closure; it does not reimplement Cargo workspace, dependency, target, build-script, or source-override discovery.
- Existing `crates/originweave-core/src/root.rs` intentionally uses `#[path = "lib.rs"]`. That exact source/attribute pair is the only currently reviewed path-attribute exception.
- Default `src/` module trees are already conservatively included by the canonical `src/**/*.rs` closure. The custom-target module guard is therefore limited to reviewed production sources outside every production package's default `src/` directory.
- The temporary custom-target guard intentionally over-approximates rather than reproducing Rust's module grammar. Any lexical `mod` token in a custom target root outside default `src/` fails closed. That may reject a harmless inline module or textual occurrence, but it removes identifier/trivia/visibility grammar gaps until compiler-derived source-input provenance replaces the heuristic.
- Rust attribute review must preserve lexical token boundaries. Non-doc line/block comments, including nested block comments, are treated as trivia; comment/string contents do not terminate the surrounding attribute token tree.
- No future BiDi adapter path or Rust source indirection is pre-authorized.

## Decision

Until compiler-derived source-input provenance is modeled, production Rust source fails closed on `include!` regardless of whether the macro invocation uses parentheses, brackets, or braces.

Any Rust attribute containing a `path` meta item followed by valid Rust trivia and `=` is treated as source-indirection review surface, including `cfg_attr`-generated `path`. Attribute bodies are extracted as balanced bracket token trees while skipping Rust whitespace, line comments, nested block comments, normal/raw string literals, and simple character literals for delimiter purposes. This scanner is deliberately limited to locating reviewed path-bearing attribute surfaces; it does not claim to parse Rust modules, name resolution, or macro expansion. The contract uses an exact-tree allowlist of `(source_path, normalized_attribute_body)`. The allowlist must equal the path-attribute surfaces found on the current OriginWeave production-source closure, so it cannot reserve absent future adapter paths. The current exact allowlist contains only:

`crates/originweave-core/src/root.rs` → `path = "lib.rs"`

For a reviewed custom Cargo target outside a production package's default `src/` tree, the temporary contract no longer tries to prove that a particular `mod` spelling is outlined rather than inline. Any lexical `mod` token is a provenance stop. This deliberately conservative rule closes changes in identifier spelling and Rust trivia such as `mod r#type;`, `mod 관찰;`, or `mod /* comment */ helper;` without taking ownership of Cargo topology or pretending a regex implements the Rust parser. Ordinary `src/` module trees remain outside this guard because their sibling files are already enumerated by the canonical source closure.

Any additional path-bearing attribute or custom-target module source must arrive in the same reviewed delta that explains and tests its source provenance. A future filesystem indirection must not silently widen the Browser Session TCB.

## RED → repair evidence

- `d8741e7a88b45c5c926963801493afc4ec14462a` added hostile `include!` and parent-traversal `#[path]` fixtures while the contract body was deliberately inert, exposing the missing fail-closed behavior.
- `f42b1012306290311cd240671376096ca107bae2` added the first source-indirection guard consuming the canonical production-source closure.
- `5068055be9acc3c8dc5ebd167a2b26099cbdacd4` tightened `#[path]` handling from path-shape heuristics to an exact current-tree allowlist, preserving the existing reviewed `originweave-core/src/root.rs -> lib.rs` exception while rejecting any new path attribute until reviewed.
- `a6f6d545df4d890edebe82ddc78fe4f36a0a1cfd` added a hostile `cfg_attr(..., path = ...)` fixture and generalized discovery from only direct `#[path]` spellings to any Rust attribute carrying `path =`, preventing conditional compilation from bypassing the exact-tree review surface.
- Focused CodeRabbit review of exact `72fb8b410d05a252bda7da81a428fcdbcfa31f4e` found a valid P1: the first `include!` detector matched only the parenthesized form even though Rust macro invocations also admit bracket and brace token trees.
- `cf9a1902d9c4c184121850ea77cf3f1ca4ce29ee` added hostile `include! {...}` and `include![...]` regressions without changing the parenthesized-only detector, preserving a structural RED for both bypasses.
- `52dae82b4d4a26ae56cb81913e6d8e1daf7a6e19` repaired the detector to recognize all three valid macro delimiter forms while keeping the same fail-closed error and canonical Cargo source closure.
- `d823d04a105fa8234077039d96cc168cf942bc8d` added a hostile custom `[lib].path = "runtime/lifecycle_adapter.rs"` whose crate root declares `mod helper;` and whose sibling `runtime/helper.rs` is outside the canonical `src/**/*.rs` closure. The pre-repair contract did not reject that source expansion, preserving the structural RED.
- `eb2ea168fd951bbc817f24cd08fb2b0b5805a775` added the first custom-target module guard while preserving the positive ordinary `src/lib.rs -> mod nested;` fixture.
- Focused CodeRabbit review of exact `f28e96f5dca746adab2df3fb909bd205d13947ed` found a valid P1: the first guard missed raw identifiers such as `mod r#type;`.
- `8d396db28df3e9757a1f3ee96eb65bca15a16e4f` preserved that raw-identifier finding as a hostile structural RED, and `aabd724d0d41a526ca41ade4e47349b94c0151f5` repaired the reported raw spelling.
- Focused CodeRabbit review of exact `8ba03a5cf3022dbe21c9f4e1b0443e7481941661` found the same hand-written grammar was still ASCII-only even though Rust identifiers can be Unicode.
- `aa90b3ef465785ab0250097dedbc917dc2ce9cc6` preserved a hostile Unicode module RED (`mod 관찰;`), and `e9dfcbefcc7d2ed022564a76edd3715f1f30d071` removed the ASCII identifier assumption.
- Focused CodeRabbit review of exact `fc0275ca9099955819777b72e73b5c25891e826a` found one remaining P1 in the same grammar-emulation approach: valid Rust trivia can occur between `mod` and its module-name token, so a detector that requires direct horizontal whitespace before an identifier remains bypassable.
- `130e9512d8a96db782de0a7b98e490b6db17a3c6` preserves that finding as a structural RED with `mod /* reviewed trivia */ helper;` while leaving the prior detector unchanged.
- `bb83cf9571c2091bbb088176a9881de5fb46739b` removes the fragile hand-written module-name grammar. For custom target roots outside default `src/`, the temporary gate now fails closed on any lexical `mod` token. This is intentionally conservative and temporary; it closes trivia/raw/Unicode spelling classes without taking ownership of the Rust parser or Cargo topology.
- `dce0c96fb8a05cce5605ecf42df858c16276099d` preserved a new structural RED showing that Rust block-comment trivia between `path` and `=` bypasses the prior `path\s*=` detector.
- `781f3b1db14bf7591091cb7be5bb552e6e5497b1` broadened that RED to nested block comments, line-comment trivia, and a closing bracket inside a comment, demonstrating that both the path-meta regex and the `[^\]]*` attribute-body regex were narrower than Rust lexical rules.
- `04e08a48fb7572860b0de564bdbf615ad2da5186` repaired the attribute review surface without changing Cargo topology: balanced attribute token trees now ignore comment/string delimiters correctly, and `path` followed by Rust trivia and `=` enters the exact-tree allowlist review surface.

This evidence is structural/static on a Draft branch. It is not executable exact-head GREEN and does not replace the required parent-lineage, repository/security, or real-Chromium acceptance gates.

## Follow-up

Replace the temporary custom-target lexical stop and hand-maintained attribute-source review with compiler-derived or equivalently exact source-input provenance before OriginWeave needs legitimate custom-target module trees or broader source-generating attributes/macros. The replacement must cover the actual bytes compiled by Rust across supported target configurations, preserve repository containment and immutable provenance, distinguish inline modules from source-loading outlined modules without ad-hoc grammar drift, and arrive with hostile fixtures before any allowlist widening.
