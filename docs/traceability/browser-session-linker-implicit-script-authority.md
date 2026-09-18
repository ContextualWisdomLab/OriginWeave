# Browser Session extensionless implicit linker-script authority

## Decision

Browser Session Cargo execution/input provenance treats a non-option linker argument that contains a path separator as an external native input even when the file has no recognized object, archive, shared-library, bitcode, or resource suffix.

This is intentionally narrower than banning every non-option token. Bare linker option operands such as the `relro` operand in `-z relro` remain allowed. A future exception for a repository path requires an explicit, versioned provenance contract for the referenced artifact or script and its transitive native inputs.

## Problem and threat

The reviewed production package/source closure and the nominal linker executable can remain unchanged while Git-owned Cargo `rustflags` append a path such as `tools/review-bypass-input` with `-C link-arg` or `-C link-args`.

The Rust compiler documents that `link-arg` appends one argument to the linker invocation and that `link-args` appends multiple arguments. On Unix-like targets using a C compiler as linker driver, `-Wl,$ARG` forwards an argument to the underlying linker.

GNU `ld` documents a second interpretation that makes an extension allowlist insufficient: if a linker input is not recognized as an object or archive, `ld` attempts to parse it as a linker script. Such an implicit script may contain `INPUT` or `GROUP`, which can introduce additional native inputs at that point in the command line.

Therefore a repository-relative, extensionless path is not inert metadata. It can be a transitive native-input authority boundary.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo compiler/linker execution and input authority.
- The repair must not ban unrelated rustflags or ordinary option-only linker arguments.
- No linker or driver acknowledgement is treated as evidence that the resulting binary contains only reviewed inputs.

## RED → repair evidence

- Predecessor exact head: `202ba6c92faf7a34d233690c3923680337ec0e65`.
- RED: `8f5e49f5fe63d99773d25f13a3ab50648e02ac92` adds `tests/test_browser_session_linker_implicit_script_contract.py`. It supplies an extensionless `tools/review-bypass-input` whose contents are `INPUT(tools/review-bypass-object.o)` through direct, `-Wl,`, `--for-linker=`, and `-Xlinker` forms. The predecessor suffix-only classifier does not reject those path operands.
- Repair: `cb95d5d37050bb7da70f1782da2e63a9b4583734` extends the existing positional-input classifier so a non-option token containing `/` or `\\` fails closed before suffix classification. No second Cargo topology/config authority is introduced.
- Control: `b03980261159ca90788c1ccf9cc5e750d974976c` preserves a bare linker option operand (`-Wl,-z,relro`) and the existing option-only `--as-needed` control.

The repair covers repository/path-shaped extensionless positional inputs presented directly or through the already-modeled GNU-style forwarding forms. It also keeps the existing recognized native suffix and versioned `.so.*` checks.

## Residual risk and removal conditions

This slice does **not** claim universal linker-grammar closure.

- A bare extensionless filename with no `/` or `\\` can still be a positional input if the linker working directory or search semantics resolve it. Closing that path safely requires argument-arity-aware parsing or compiler-derived link-command provenance so option operands are not confused with positional inputs.
- Non-GNU linker/driver grammars may expose additional input-control surfaces. They require authoritative target-specific semantics and hostile fixtures before being added to the shared classifier.
- Runtime environment injection (`RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`), direct `cargo rustc -- ...`, sysroot/toolchain composition, and custom target/toolchain behavior remain separate authority surfaces.

A path exception may be relaxed only when the referenced file and every transitive native input are immutable, hashed, reviewed, bound to the exact build provenance, and exercised by current-head executable evidence. Until then the contract remains fail closed.

## Primary references

Rust Project. (2026). *The rustc book: Codegen options — link-arg and link-args*. https://doc.rust-lang.org/rustc/codegen-options/index.html#link-arg

GNU Project. (2026). *GNU ld: Implicit linker scripts*. https://sourceware.org/binutils/docs/ld/Implicit-Linker-Scripts.html
