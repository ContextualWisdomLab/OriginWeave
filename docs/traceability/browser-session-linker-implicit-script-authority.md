# Browser Session extensionless implicit linker-script authority

## Decision

Browser Session Cargo execution/input provenance treats every unconsumed non-option linker token as an external native input, including extensionless bare filenames. The shared parser consumes only linker option operands whose arity and semantics are explicitly modeled; today that narrow allowlist contains GNU `ld`/driver `-z <keyword>`.

This is intentionally not a blanket ban on non-option words. A token such as `relro` is allowed only when it is consumed as the operand of the modeled `-z` option. The same bare token in positional position fails closed. A future exception for a repository artifact or script requires an explicit, versioned provenance contract for the referenced artifact and its transitive native inputs.

## Problem and threat

The reviewed production package/source closure and the nominal linker executable can remain unchanged while Git-owned Cargo `rustflags` append an extensionless file through `-C link-arg` or `-C link-args`.

The Rust compiler documents that `link-arg` appends one argument to the linker invocation and that `link-args` appends multiple arguments. On Unix-like targets using a C compiler as linker driver, `-Wl,$ARG` forwards an argument to the underlying linker.

GNU `ld` documents that non-option arguments are object files or archives and that an input whose format is not recognized is parsed as an implicit linker script. Such a script may contain `INPUT` or `GROUP`, which can introduce additional native inputs. GNU `ld` also states that an object argument may not appear between an option and its required operand. That command-line grammar is why positional-input classification must be arity-aware rather than suffix- or path-shape-based.

Therefore both `tools/review-bypass-input` and the bare filename `review-bypass-input` can represent transitive native-input authority. File suffixes and path separators are insufficient provenance boundaries.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo compiler/linker execution and input authority.
- The repair must not ban unrelated rustflags or modeled option operands such as the `relro` keyword in `-z relro`.
- `-Wl,`, `--for-linker=`, and repeated `-Xlinker` forwarding must reach the same direct-linker authority classifier.
- No linker or driver acknowledgement is treated as evidence that the resulting binary contains only reviewed inputs.

## RED → repair evidence

### Path-shaped extensionless inputs

- Predecessor exact head: `202ba6c92faf7a34d233690c3923680337ec0e65`.
- RED: `8f5e49f5fe63d99773d25f13a3ab50648e02ac92` adds `tests/test_browser_session_linker_implicit_script_contract.py`. It supplies an extensionless `tools/review-bypass-input` whose contents are `INPUT(tools/review-bypass-object.o)` through direct, `-Wl,`, `--for-linker=`, and `-Xlinker` forms. The predecessor suffix-only classifier does not reject those path operands.
- Repair: `cb95d5d37050bb7da70f1782da2e63a9b4583734` extends the existing positional-input classifier so a non-option token containing `/` or `\\` fails closed before suffix classification.
- Control: `b03980261159ca90788c1ccf9cc5e750d974976c` preserves `-Wl,-z,relro` and the existing option-only `--as-needed` control.

### Bare extensionless inputs

- Predecessor exact head: `19dae976f1dcfdf261ded04c43cade986ade7712`.
- RED: `ed86b335b4aa6cde223fe2e614bb26d39f789d8a` adds `tests/test_browser_session_linker_bare_implicit_script_contract.py`. It supplies a bare `review-bypass-input` through direct, `-Wl,`, `--for-linker=`, and `-Xlinker` forms while retaining direct and forwarded `-z relro` controls. The predecessor path/suffix classifier does not reject the bare positional token.
- Repair: `5f070bf0b87ae513cf06badda29914e579f850ee` replaces path/suffix heuristics with an arity-aware direct-linker token parser. Every unconsumed non-option token fails closed. `-z` consumes exactly one modeled keyword operand; repeated `-Xlinker` payloads are reconstructed into one direct-linker token sequence before classification, so `-Xlinker -z -Xlinker relro` remains permitted while `-Xlinker review-bypass-input` does not.

No second Cargo topology/config authority is introduced. Direct linker arguments and the existing GNU-style forwarding forms consume the same authority classifier.

## Residual risk and removal conditions

This slice does **not** claim universal linker-grammar closure.

- Only option arity that has an explicit harmless-operand contract is consumed. Additional GNU linker options with separate operands remain fail closed until their semantics are modeled with positive and hostile fixtures.
- Non-GNU linker/driver grammars may expose different option arities and input-control surfaces. They require authoritative target-specific semantics and hostile fixtures before being added to the shared classifier.
- Runtime environment injection (`RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`), direct `cargo rustc -- ...`, sysroot/toolchain composition, and custom target/toolchain behavior remain separate authority surfaces.
- Static argument classification proves admission policy, not the actual final link command or artifact composition. Release evidence still requires exact-head executable provenance and reproducibility evidence.

An input exception may be relaxed only when the referenced file and every transitive native input are immutable, hashed, reviewed, bound to exact build provenance, and exercised by current-head executable evidence. Until then the contract remains fail closed.

## Primary references

Rust Project. (2026). *The rustc book: Codegen options — link-arg and link-args*. https://doc.rust-lang.org/rustc/codegen-options/index.html#link-arg

GNU Project. (2026). *GNU ld: Options*. https://sourceware.org/binutils/docs/ld/Options.html

GNU Project. (2026). *GNU ld: Implicit linker scripts*. https://sourceware.org/binutils/docs/ld/Implicit-Linker-Scripts.html
