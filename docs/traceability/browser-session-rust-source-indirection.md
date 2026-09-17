# Browser Session Rust source-indirection provenance

Status: active PR evidence only. This contract does not claim protected-main or release acceptance.

## Problem

The Browser Session trusted-adapter boundary already derives one canonical Cargo production package/source closure and rejects repository-external or dangling source objects. That is not sufficient by itself because reviewed Rust source can cause `rustc` to parse additional source bytes through language-level indirection.

Two surfaces matter here:

- `include!(...)` parses another file as an expression or item. Rust documents the included path as relative to the source file containing the invocation.
- `#[path = "..."] mod ...;` changes the source file used for an outlined module. Rust documents the path attribute as a module-source filename override whose relative interpretation depends on the module location.

Primary references:

- Rust `include!` macro: https://doc.rust-lang.org/stable/std/macro.include.html
- Rust Reference, module source filenames and `path` attribute: https://doc.rust-lang.org/reference/items/modules.html#the-path-attribute

Without an explicit contract, a future lifecycle adapter could keep its crate, manifest, and ordinary `src/**/*.rs` entry point inside the exact Git review root while compiling additional Rust source not represented by the canonical production-source closure. That would weaken the same provenance boundary used for `DisposableContextPort`, `bind_lifecycle_port`, dependency, and source-containment review.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py::_workspace_production_sources()` remains the single writer for Cargo production package/source topology.
- This contract consumes that closure; it does not reimplement Cargo workspace, dependency, target, build-script, or source-override discovery.
- Existing `crates/originweave-core/src/root.rs` intentionally uses `#[path = "lib.rs"]`. That exact pair is the only currently reviewed path-attribute exception.
- No future BiDi adapter path or Rust source indirection is pre-authorized.

## Decision

Until compiler-derived source-input provenance is modeled, production Rust source fails closed on `include!`.

`#[path = "..."]` is exact-tree allowlisted by `(source_path, declared_path)`. The allowlist must equal the path-attribute surfaces found on the current OriginWeave production-source closure, so it cannot reserve absent future adapter paths. The current exact allowlist contains only:

`crates/originweave-core/src/root.rs` → `lib.rs`

Any additional path attribute must arrive in the same reviewed delta that explains and tests its source provenance. This is intentionally stricter than accepting apparently in-tree relative strings because Rust module-path resolution has context-sensitive semantics and a future filesystem indirection must not silently widen the Browser Session TCB.

## RED → repair evidence

- `d8741e7a88b45c5c926963801493afc4ec14462a` added hostile `include!` and parent-traversal `#[path]` fixtures while the contract body was deliberately inert, exposing the missing fail-closed behavior.
- `f42b1012306290311cd240671376096ca107bae2` added the first source-indirection guard consuming the canonical production-source closure.
- `5068055be9acc3c8dc5ebd167a2b26099cbdacd4` tightened `#[path]` handling from path-shape heuristics to an exact current-tree allowlist, preserving the existing reviewed `originweave-core/src/root.rs -> lib.rs` exception while rejecting any new path attribute until reviewed.

This evidence is structural/static on a Draft branch. It is not executable exact-head GREEN and does not replace the required parent-lineage, repository/security, or real-Chromium acceptance gates.

## Follow-up

If OriginWeave later needs `include!` or additional `#[path]` surfaces in production code, replace the temporary fail-closed policy with compiler-derived or equivalently exact source-input provenance. The replacement must cover the actual bytes compiled by Rust, preserve repository containment and immutable provenance, and arrive with hostile fixtures before any allowlist widening.
