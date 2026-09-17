# Browser Session Rust source-indirection provenance

Status: active PR evidence only. This contract does not claim protected-main or release acceptance.

## Problem

The Browser Session trusted-adapter boundary already derives one canonical Cargo production package/source closure and rejects repository-external or dangling source objects. That is not sufficient by itself because reviewed Rust source can cause `rustc` to parse additional source bytes through language-level indirection.

Three related forms matter here:

- `include!(...)`, `include![...]`, and `include! {...}` parse another file as an expression or item. Rust macro invocation syntax permits all three delimiter forms, and the included path is relative to the source file containing the invocation.
- `#[path = "..."] mod ...;` changes the source file used for an outlined module. Rust documents the path attribute as a module-source filename override whose relative interpretation depends on the module location.
- `#[cfg_attr(..., path = "...")]` can conditionally synthesize the same `path` attribute. A direct `#[path]`-only lexical check therefore does not cover the full Rust attribute surface.

Primary references:

- Rust `include!` macro: https://doc.rust-lang.org/stable/std/macro.include.html
- Rust Reference, macro invocation syntax: https://doc.rust-lang.org/reference/macros.html#macro-invocation
- Rust Reference, module source filenames and `path` attribute: https://doc.rust-lang.org/reference/items/modules.html#the-path-attribute
- Rust Reference, conditional attributes with `cfg_attr`: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute

Without an explicit contract, a future lifecycle adapter could keep its crate, manifest, and ordinary `src/**/*.rs` entry point inside the exact Git review root while compiling additional Rust source not represented by the canonical production-source closure. That would weaken the same provenance boundary used for `DisposableContextPort`, `bind_lifecycle_port`, dependency, and source-containment review.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py::_workspace_production_sources()` remains the single writer for Cargo production package/source topology.
- This contract consumes that closure; it does not reimplement Cargo workspace, dependency, target, build-script, or source-override discovery.
- Existing `crates/originweave-core/src/root.rs` intentionally uses `#[path = "lib.rs"]`. That exact source/attribute pair is the only currently reviewed path-attribute exception.
- No future BiDi adapter path or Rust source indirection is pre-authorized.

## Decision

Until compiler-derived source-input provenance is modeled, production Rust source fails closed on `include!` regardless of whether the macro invocation uses parentheses, brackets, or braces.

Any Rust attribute containing `path =` is treated as source-indirection review surface, including `cfg_attr`-generated `path`. The contract uses an exact-tree allowlist of `(source_path, normalized_attribute_body)`. The allowlist must equal the path-attribute surfaces found on the current OriginWeave production-source closure, so it cannot reserve absent future adapter paths. The current exact allowlist contains only:

`crates/originweave-core/src/root.rs` → `path = "lib.rs"`

Any additional path-bearing attribute must arrive in the same reviewed delta that explains and tests its source provenance. This is intentionally stricter than accepting apparently in-tree relative strings because Rust module-path resolution is context-sensitive and conditional attributes can change the selected source by target configuration. A future filesystem indirection must not silently widen the Browser Session TCB.

## RED → repair evidence

- `d8741e7a88b45c5c926963801493afc4ec14462a` added hostile `include!` and parent-traversal `#[path]` fixtures while the contract body was deliberately inert, exposing the missing fail-closed behavior.
- `f42b1012306290311cd240671376096ca107bae2` added the first source-indirection guard consuming the canonical production-source closure.
- `5068055be9acc3c8dc5ebd167a2b26099cbdacd4` tightened `#[path]` handling from path-shape heuristics to an exact current-tree allowlist, preserving the existing reviewed `originweave-core/src/root.rs -> lib.rs` exception while rejecting any new path attribute until reviewed.
- `a6f6d545df4d890edebe82ddc78fe4f36a0a1cfd` added a hostile `cfg_attr(..., path = ...)` fixture and generalized discovery from only direct `#[path]` spellings to any Rust attribute carrying `path =`, preventing conditional compilation from bypassing the exact-tree review surface.
- Focused CodeRabbit review of exact `72fb8b410d05a252bda7da81a428fcdbcfa31f4e` found a valid P1: the first `include!` detector matched only the parenthesized form even though Rust macro invocations also admit bracket and brace token trees. The review confirmed the other requested source-indirection invariants were structurally correct.
- `cf9a1902d9c4c184121850ea77cf3f1ca4ce29ee` added hostile `include! {...}` and `include![...]` regressions without changing the parenthesized-only detector, preserving a structural RED for both bypasses.
- `52dae82b4d4a26ae56cb81913e6d8e1daf7a6e19` repaired the detector to recognize all three valid macro delimiter forms while keeping the same fail-closed error and canonical Cargo source closure.

This evidence is structural/static on a Draft branch. It is not executable exact-head GREEN and does not replace the required parent-lineage, repository/security, or real-Chromium acceptance gates.

## Follow-up

If OriginWeave later needs `include!` or additional path-bearing attributes in production code, replace the temporary fail-closed policy with compiler-derived or equivalently exact source-input provenance. The replacement must cover the actual bytes compiled by Rust across supported target configurations, preserve repository containment and immutable provenance, and arrive with hostile fixtures before any allowlist widening.
