# Browser Session Rust source-indirection provenance

Status: active PR evidence only. This contract does not claim protected-main or release acceptance.

## Problem

The Browser Session trusted-adapter boundary already derives one canonical Cargo production package/source closure and rejects repository-external or dangling source objects. That is not sufficient by itself because reviewed Rust source can cause `rustc` to parse additional source bytes through language-level indirection.

Four related forms matter here:

- `include!(...)`, `include![...]`, and `include! {...}` parse another file as an expression or item. Rust macro invocation syntax permits all three delimiter forms, and the included path is relative to the source file containing the invocation.
- `#[path = "..."] mod ...;` changes the source file used for an outlined module. Rust documents the path attribute as a module-source filename override whose relative interpretation depends on the module location.
- `#[cfg_attr(..., path = "...")]` can conditionally synthesize the same `path` attribute. A direct `#[path]`-only lexical check therefore does not cover the full Rust attribute surface.
- A bare outlined module item such as `mod helper;` also causes the compiler to load another Rust file. Cargo's ordinary `src/**/*.rs` review already contains those sibling/default module files conservatively, but a custom Cargo target whose crate root lives outside `src/` can load sibling module files that are not in that closure. Rust identifiers are Unicode XID-based and raw identifiers are also valid item identifiers, so the same guard must not assume ASCII-only module names.

Primary references:

- Rust `include!` macro: https://doc.rust-lang.org/stable/std/macro.include.html
- Rust Reference, macro invocation syntax: https://doc.rust-lang.org/reference/macros.html#macro-invocation
- Rust Reference, module source filenames and `path` attribute: https://doc.rust-lang.org/reference/items/modules.html#module-source-filenames
- Rust Reference, conditional attributes with `cfg_attr`: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
- Rust Reference, identifiers and raw identifiers: https://doc.rust-lang.org/reference/identifiers.html
- Rust 2018 Edition Guide, module file layout: https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html#no-more-modrs

Without an explicit contract, a future lifecycle adapter could keep its crate, manifest, and declared custom target inside the exact Git review root while compiling additional Rust source not represented by the canonical production-source closure. That would weaken the same provenance boundary used for `DisposableContextPort`, `bind_lifecycle_port`, dependency, and source-containment review.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py::_workspace_production_sources()` remains the single writer for Cargo production package/source topology.
- This contract consumes that closure; it does not reimplement Cargo workspace, dependency, target, build-script, or source-override discovery.
- Existing `crates/originweave-core/src/root.rs` intentionally uses `#[path = "lib.rs"]`. That exact source/attribute pair is the only currently reviewed path-attribute exception.
- Default `src/` module trees are already conservatively included by the canonical `src/**/*.rs` closure. The bare-module guard is therefore limited to reviewed custom target roots outside every production package's default `src/` directory.
- The temporary custom-target guard intentionally does not attempt to reproduce the Rust identifier grammar. Once it sees an outlined module-shaped token before `;`, it fails closed; exact compiler-derived source-input provenance is the long-term replacement.
- No future BiDi adapter path or Rust source indirection is pre-authorized.

## Decision

Until compiler-derived source-input provenance is modeled, production Rust source fails closed on `include!` regardless of whether the macro invocation uses parentheses, brackets, or braces.

Any Rust attribute containing `path =` is treated as source-indirection review surface, including `cfg_attr`-generated `path`. The contract uses an exact-tree allowlist of `(source_path, normalized_attribute_body)`. The allowlist must equal the path-attribute surfaces found on the current OriginWeave production-source closure, so it cannot reserve absent future adapter paths. The current exact allowlist contains only:

`crates/originweave-core/src/root.rs` → `path = "lib.rs"`

A reviewed custom Cargo target outside a production package's default `src/` tree also fails closed if it contains a bare outlined module item. The detector deliberately treats the module-name token as opaque instead of restricting it to ASCII, so ordinary, raw, and Unicode Rust identifiers cannot select an unenumerated sibling by changing identifier spelling. This is narrower than banning modules globally: ordinary `src/` module files are already included by the canonical Cargo source closure, whereas a custom target root can otherwise cause `rustc` to read an unenumerated sibling module. Inline modules (`mod name { ... }`) do not add source bytes and are not part of this guard.

Any additional path-bearing attribute or custom-target module source must arrive in the same reviewed delta that explains and tests its source provenance. A future filesystem indirection must not silently widen the Browser Session TCB.

## RED → repair evidence

- `d8741e7a88b45c5c926963801493afc4ec14462a` added hostile `include!` and parent-traversal `#[path]` fixtures while the contract body was deliberately inert, exposing the missing fail-closed behavior.
- `f42b1012306290311cd240671376096ca107bae2` added the first source-indirection guard consuming the canonical production-source closure.
- `5068055be9acc3c8dc5ebd167a2b26099cbdacd4` tightened `#[path]` handling from path-shape heuristics to an exact current-tree allowlist, preserving the existing reviewed `originweave-core/src/root.rs -> lib.rs` exception while rejecting any new path attribute until reviewed.
- `a6f6d545df4d890edebe82ddc78fe4f36a0a1cfd` added a hostile `cfg_attr(..., path = ...)` fixture and generalized discovery from only direct `#[path]` spellings to any Rust attribute carrying `path =`, preventing conditional compilation from bypassing the exact-tree review surface.
- Focused CodeRabbit review of exact `72fb8b410d05a252bda7da81a428fcdbcfa31f4e` found a valid P1: the first `include!` detector matched only the parenthesized form even though Rust macro invocations also admit bracket and brace token trees. The review confirmed the other requested source-indirection invariants were structurally correct.
- `cf9a1902d9c4c184121850ea77cf3f1ca4ce29ee` added hostile `include! {...}` and `include![...]` regressions without changing the parenthesized-only detector, preserving a structural RED for both bypasses.
- `52dae82b4d4a26ae56cb81913e6d8e1daf7a6e19` repaired the detector to recognize all three valid macro delimiter forms while keeping the same fail-closed error and canonical Cargo source closure.
- `d823d04a105fa8234077039d96cc168cf942bc8d` added a hostile custom `[lib].path = "runtime/lifecycle_adapter.rs"` whose crate root declares `mod helper;` and whose sibling `runtime/helper.rs` is outside the canonical `src/**/*.rs` closure. The pre-repair contract did not reject that source expansion, preserving the structural RED.
- `eb2ea168fd951bbc817f24cd08fb2b0b5805a775` repaired the gap without widening Cargo topology ownership: the indirection contract derives production package `src/` roots from the canonical manifest closure, permits bare outlined modules only where the canonical `src/**/*.rs` closure already covers their files, and fails closed on bare outlined modules from custom target roots outside `src/`. A positive fixture keeps ordinary `src/lib.rs -> mod nested;` valid.
- Focused CodeRabbit review of exact `f28e96f5dca746adab2df3fb909bd205d13947ed` found a valid P1 in that new guard: the regex accepted only ordinary identifiers and missed valid Rust raw identifiers such as `mod r#type;`, allowing the same custom-target sibling-source bypass under a different legal spelling.
- `8d396db28df3e9757a1f3ee96eb65bca15a16e4f` added a hostile raw-identifier module fixture while preserving the ordinary-identifier-only detector, keeping that reviewer finding as a structural RED.
- `aabd724d0d41a526ca41ade4e47349b94c0151f5` repaired the reported raw-identifier case while retaining the same custom-target-only scope and positive default-`src/` fixture.
- Focused CodeRabbit review of exact `8ba03a5cf3022dbe21c9f4e1b0443e7481941661` then found the same grammar-assumption class was still incomplete: the detector remained ASCII-only even though Rust permits Unicode identifiers.
- `aa90b3ef465785ab0250097dedbc917dc2ce9cc6` added a hostile Unicode module fixture (`mod 관찰;`) while leaving the ASCII-only detector unchanged, preserving that reviewer finding as a structural RED.
- `e9dfcbefcc7d2ed022564a76edd3715f1f30d071` removed the ASCII identifier assumption. The custom-target guard now treats the module-name token opaquely and fails closed before `;`, covering ordinary, raw, and Unicode identifier spellings without reimplementing Rust XID tables or widening Cargo topology ownership.

This evidence is structural/static on a Draft branch. It is not executable exact-head GREEN and does not replace the required parent-lineage, repository/security, or real-Chromium acceptance gates.

## Follow-up

If OriginWeave later needs `include!`, additional path-bearing attributes, or custom-target outlined module trees in production code, replace the temporary fail-closed policy with compiler-derived or equivalently exact source-input provenance. The replacement must cover the actual bytes compiled by Rust across supported target configurations, preserve repository containment and immutable provenance, and arrive with hostile fixtures before any allowlist widening.
