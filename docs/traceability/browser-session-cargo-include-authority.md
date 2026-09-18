# Browser Session Cargo include authority

## Problem

Cargo configuration can load additional TOML configuration through the top-level `include` key. Cargo resolves include paths relative to the including configuration file, accepts path strings and inline tables, and recursively processes includes before merging the including file on top. A checked-in `.cargo/config.toml` can therefore delegate compiler, rustdoc, linker, environment, target, or build-script-override authority to another repository file that is not itself named `.cargo/config.toml` or `.cargo/config`.

The Browser Session Cargo compiler-authority owner discovered Git-owned `.cargo/config*` files and classified their parsed keys, but it did not model `include`. An included repository TOML file could consequently carry `[env]`, compiler/linker selectors, rustflags/rustdocflags, or `target.<triple>.<links>` metadata outside the reviewed config closure.

## Authority and constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production package/source topology. `tests/test_browser_session_cargo_compiler_authority_contract.py` owns Git-selected Cargo compiler/input configuration authority. The focused include contract imports that owner and does not add another workspace or Cargo-config discovery implementation.

Until OriginWeave has a recursive, containment-checked, cycle-safe, versioned include graph whose effective merged values are reviewed under the same authority rules, any repository-owned Cargo top-level `include` is fail-closed. This is intentionally narrower and safer than partially following includes while missing precedence, recursion, optional entries, or path semantics.

## RED → repair evidence

RED `c8829bfbe5af0bf44f5531189576ff2f5fce7ab1` adds `tests/test_browser_session_cargo_include_authority_contract.py`. It supplies a real repository-relative included TOML file containing an unreviewed `[env]` value and exercises Cargo's path-string, inline-table, and optional-inline-table include forms. A config with no include remains the control.

Repair `0472a50840876fc80cf431d2d66085566f06b335` minimally extends the existing parsed-config owner. Presence of the top-level `include` key is recorded as unmodeled Cargo execution/input authority and rejected through the same Browser Session provenance error path. No recursive include parser or second config scanner is introduced.

## Residual execution provenance

This source contract does not observe command-line `cargo --config`, configuration inherited from ancestor directories or `$CARGO_HOME`, environment-variable overrides, runner images, toolchain installation state, or files outside the Git-owned repository closure. Those remain CI/release/runtime provenance surfaces. If repository Cargo includes are later required, acceptance must prove the complete recursive include graph, path containment, optional-file semantics, merge precedence, cycle behavior, and the effective compiler/input authority after merging.

## Primary reference

- The Rust Project. (2026). *The Cargo Book: Configuration — Including extra configuration files*. https://doc.rust-lang.org/cargo/reference/config.html#include

Cargo documents that top-level `include` loads additional `.toml` files, supports path strings and inline tables with `optional`, recursively processes nested includes, and merges the including file after its included files. That behavior makes the include graph part of exact build provenance rather than a formatting convenience.
