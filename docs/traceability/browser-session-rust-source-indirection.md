# Browser Session Rust source-indirection provenance

Status: active PR evidence only. This contract does not claim protected-main or release acceptance.

## Problem

The Browser Session trusted-adapter boundary already derives one canonical Cargo production package/source closure and rejects repository-external or dangling source objects. That is not sufficient by itself because reviewed Rust source can cause `rustc` to parse additional source bytes through language-level indirection.

Four related forms matter here:

- `include!(...)`, `include![...]`, and `include! {...}` parse another file as an expression or item. Rust macro invocation syntax permits all three delimiter forms, and the included path is relative to the source file containing the invocation.
- `#[path = "..."] mod ...;` changes the source file used for an outlined module. Rust documents the path attribute as a module-source filename override whose relative interpretation depends on the module location.
- `#[cfg_attr(..., path = "...")]` can conditionally synthesize the same `path` attribute. A direct `#[path]`-only lexical check therefore does not cover the full Rust attribute surface.
- A `mod` item can cause the compiler to load another Rust file. Cargo's ordinary `src/**/*.rs` review already contains default module trees conservatively, but a custom Cargo target whose crate root lives outside `src/` can load sibling module files that are not in that closure. Rust permits comments and other trivia between grammar tokens, Unicode XID identifiers, and raw identifiers, so a security contract must not encode a narrower hand-written identifier/module grammar and call it complete.

Rust comments are lexical trivia rather than `\s`-only whitespace. This matters for both attributes and macro invocation punctuation. A valid source-loading attribute can spell its meta item as `#[path /* reviewed trivia */ = "nested.rs"]`, and a valid macro invocation can separate `include`, `!`, and its delimiter with non-doc comment trivia. A detector that requires `include\s*!\s*(` or `path\s*=` therefore recognizes a grammar narrower than Rust's. Likewise, a naive `[^\]]*` attribute capture can be truncated by `]` inside a comment or string even though that byte is not the attribute's closing delimiter.

Raw identifiers are part of the same source-provenance surface. The Rust Reference defines a raw identifier as `r#` plus an identifier/keyword and states that the `r#` prefix is not part of the actual identifier. Macro invocations resolve a `SimplePath`, whose path segment admits an `IDENTIFIER`, so `r#include!(...)` names the same underlying `include` identifier through raw spelling. A detector that deliberately excludes `include` when immediately preceded by `#` therefore leaves a valid source-loading spelling outside the fail-closed contract.

Macro renaming is also source provenance. Rust `use` declarations create local synonymous bindings, can import macro names, and permit `as` aliases. Because `include` is exported from `core`, `use core::include as embed; embed!("...")` can invoke the same source-loading macro without any later literal `include!` token. A direct-invocation-only scanner therefore has a second name-resolution bypass even after ordinary/raw spelling and comment trivia are covered.

Primary references:

- Rust `include!` macro: https://doc.rust-lang.org/stable/std/macro.include.html
- Rust `core::include` macro export/source: https://doc.rust-lang.org/core/macro.include.html
- Rust Reference, macro invocation syntax: https://doc.rust-lang.org/reference/macros.html#macro-invocation
- Rust Reference, use declarations and aliases: https://doc.rust-lang.org/reference/items/use-declarations.html
- Rust Reference, namespaces and macro imports: https://doc.rust-lang.org/reference/names/namespaces.html
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
- Rust attribute and `include!` review must preserve lexical token boundaries. Non-doc line/block comments, including nested block comments, are treated as trivia between Rust tokens; comment/string contents do not terminate the surrounding attribute token tree.
- Raw-identifier spelling must not create a second identity for a source-loading macro. Ordinary `include` and raw `r#include` are the same fail-closed provenance surface.
- Import aliases must not create a third identity for the same macro. A named `use ... include as alias` binding is fail-closed until compiler-derived macro/source provenance replaces the lexical contract; `as _` is not treated as callable alias authority because it creates no name that can be invoked later.
- No future BiDi adapter path or Rust source indirection is pre-authorized.

## Decision

Until compiler-derived source-input provenance is modeled, production Rust source fails closed on `include!` regardless of whether the macro path uses ordinary `include` or raw-identifier `r#include`, regardless of whether the invocation uses parentheses, brackets, or braces, and regardless of Rust whitespace/comment trivia between the identifier token, `!`, and the opening delimiter. The detector locates either spelling of the same underlying identifier and advances through the same nested non-doc-comment trivia skipper already used by the source-indirection contract before validating `!` and one of the three macro delimiters. It does not broaden the match to longer identifiers and does not authorize any source path.

The same contract also fails closed when a Rust `use` tree gives `include` a callable alias. It scans `use` declarations through their semicolon while ignoring Rust comment trivia, then rejects a named `include as ...` binding. This covers direct and grouped use trees without trying to implement general macro name resolution. Ordinary direct imports remain covered by the later literal `include!` invocation; an underscore import is not a callable alias and is not rejected by this alias-specific rule. This remains a temporary lexical security boundary, not a claim of compiler-equivalent name resolution.

Any Rust attribute containing a `path` meta item followed by valid Rust trivia and `=` is treated as source-indirection review surface, including `cfg_attr`-generated `path`. Attribute bodies are extracted as balanced bracket token trees while skipping Rust whitespace, line comments, nested block comments, normal/raw string literals, and simple character literals for delimiter purposes. This scanner is deliberately limited to locating reviewed path-bearing attribute surfaces; it does not claim to parse Rust modules, name resolution, or macro expansion. The contract uses an exact-tree allowlist of `(source_path, normalized_attribute_body)`. The allowlist must equal the path-attribute surfaces found on the current OriginWeave production-source closure, so it cannot reserve absent future adapter paths. The current exact allowlist contains only:

`crates/originweave-core/src/root.rs` → `path = "lib.rs"`

For a reviewed custom Cargo target outside a production package's default `src/` tree, the temporary contract no longer tries to prove that a particular `mod` spelling is outlined rather than inline. Any lexical `mod` token is a provenance stop. This deliberately conservative rule closes changes in identifier spelling and Rust trivia such as `mod r#type;`, `mod 관찰;`, or `mod /* comment */ helper;` without taking ownership of Cargo topology or pretending a regex implements the Rust parser. Ordinary `src/` module trees remain outside this guard because their sibling files are already enumerated by the canonical source closure.

Any additional path-bearing attribute, `include!` form outside the current fail-closed policy, aliased source-loading macro, or custom-target module source must arrive in the same reviewed delta that explains and tests its source provenance. A future filesystem or macro-name indirection must not silently widen the Browser Session TCB.

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
- `23b7b241c3e9389a43a359a14c4dfa74036d8829` preserved a new structural RED for macro-invocation trivia: block comments between `include` and `!`, block comments between `!` and the opening delimiter, and line-comment trivia between `include` and `!` all bypass the predecessor `include\s*!\s*[([{]` detector even though Rust treats non-doc comments as whitespace between grammar tokens.
- `bf132fb7b2c2d1a2fdae312962e2f0e0380fc1a5` repaired that gap by replacing the whitespace-only macro regex with token-plus-trivia recognition that reuses the existing nested Rust comment skipper before `!` and before the opening delimiter. The Cargo source closure and fail-closed policy are unchanged.
- `783bbc614d08c3a9299849524e3dbda8545ab29f` preserved a new structural RED for raw-identifier macro spelling. The predecessor detector intentionally rejected `include` when immediately preceded by `#`, so `r#include!("../generated_adapter.rs")` was outside the fail-closed include provenance stop even though Rust raw identifiers retain the same underlying identifier.
- `5c3b86091199f436374111d6a4056d47c13c1448` repaired the detector minimally by admitting an optional `r#` prefix as part of the include identifier token. Existing comment-trivia and three-delimiter handling remains unchanged; longer identifiers remain excluded and no path is allowlisted.
- `2c13993f24b07a7048d7879e594e72a744eeb95f` preserved a new structural RED for macro-name aliasing: a production source imports `core::include as embed` and invokes `embed!("../generated_adapter.rs")`. The predecessor direct-invocation detector sees the `include` token only inside the `use` tree, where no `!` follows it, and therefore does not stop the aliased source load.
- `4b8dd0832c6965c7e887b7538caa2f4ddc1d917c` is the minimal causal repair. The source-indirection contract now recognizes named `include as alias` bindings inside Rust `use` declarations, including grouped use trees and comment trivia, while preserving the existing direct/raw invocation detector and treating `as _` as non-callable. Cargo topology and source containment remain owned by the existing canonical closure.

This evidence is structural/static on a Draft branch. It is not executable exact-head GREEN and does not replace the required parent-lineage, repository/security, or real-Chromium acceptance gates.

## Follow-up

Replace the temporary custom-target lexical stop and hand-maintained attribute/macro-source review with compiler-derived or equivalently exact source-input provenance before OriginWeave needs legitimate custom-target module trees or broader source-generating attributes/macros. The replacement must cover the actual bytes compiled by Rust across supported target configurations, preserve repository containment and immutable provenance, distinguish inline modules from source-loading outlined modules without ad-hoc grammar drift, cover macro name resolution/re-exports without lexical approximation, and arrive with hostile fixtures before any allowlist widening.