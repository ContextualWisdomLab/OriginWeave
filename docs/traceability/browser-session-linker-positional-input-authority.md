# Browser Session linker positional-input authority

## Problem

The Browser Session Cargo compiler-authority contract already rejects repository-owned linker replacement, search-path reselection, response files, plugins, explicit GNU linker scripts, native-library `-L` / `-l` widening, rustc `--extern`, and other modeled execution/input-authority surfaces. It initially constrained only positional native inputs recognized by artifact suffix or repository/path shape, leaving a bare extensionless positional filename as a distinct implicit-script/native-input path.

The rustc book states that `link-arg` appends one extra argument to the linker invocation and `link-args` appends multiple arguments. On Unix-like targets rustc commonly invokes a C compiler such as `cc` or `clang` as the linker driver. GNU ld documents non-option arguments as object files or archives to be linked into the output and states that an input whose format is not recognized can be parsed as an implicit linker script. Consequently, Git-owned Cargo configuration could inject a reviewed-tree-external native input or script while leaving `Cargo.toml`, canonical production-source topology, nominal linker selection, and `-L` / `-l` settings unchanged.

This is a provenance gap: an input token alone does not establish the artifact's source revision, producer, digest, target/toolchain identity, reproducibility, or review ancestry.

## Constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. The Cargo compiler-authority contract consumes that topology and may constrain repository-owned linker input authority, but it must not implement a second Cargo workspace/package resolver.

This slice covers positional inputs supplied from Git-owned `.cargo/config.toml` / `.cargo/config` flags for the currently modeled direct and GNU-compatible forwarding grammar. It does not claim to parse every non-GNU linker grammar or every option with a separate operand.

## RED → repair

RED `e547203368da2aec62e2c94ab491eb0749d7d7b5` adds hostile cases for:

- direct `-C link-arg=tools/review-bypass-object.o`;
- target-scoped `-C link-args=tools/review-bypass-archive.a`; and
- compiler-driver forwarding through `-Wl,tools/review-bypass-object.o`, `--for-linker=tools/review-bypass-object.o`, and `-Xlinker tools/review-bypass-object.o`.

The predecessor exact `61053c9cc3ca58f5d812f3ab64660f4e662afdce` allowed these forms because its external-input classifier covered `-L`, `-l`, `--extern`, scripts/plugins/response files, and executable reselection but not positional native artifacts.

Repair `e73d221cf28aa25ae6fbd941db95d5d923c7e883` keeps canonical Cargo topology ownership unchanged and extends the shared linker-argument classifier with modeled positional-native-input detection for common object/archive artifact forms (`.o`, `.obj`, `.lo`, `.a`, `.lib`, `.rlib`, `.so` including versioned `.so.*`, `.dylib`, `.bc`, and `.res`) whether direct or passed through the already modeled linker-forwarding forms.

A subsequent hostile fixture `8f5e49f5fe63d99773d25f13a3ab50648e02ac92` proved that suffix-only classification still admitted an extensionless repository/path-shaped input such as `tools/review-bypass-input`. Repair `cb95d5d37050bb7da70f1782da2e63a9b4583734` closed that path-shaped form without duplicating Cargo topology ownership.

The remaining bare-filename gap was then preserved by RED `ed86b335b4aa6cde223fe2e614bb26d39f789d8a`: `review-bypass-input` could still enter through direct `link-arg` and the modeled `-Wl,`, `--for-linker=`, or repeated `-Xlinker` forwarding forms. Repair `5f070bf0b87ae513cf06badda29914e579f850ee` replaces path/suffix heuristics with an arity-aware direct-linker parser. Every unconsumed non-option token now fails closed as positional input authority. The parser consumes only option operands whose harmless semantics are explicitly modeled; today the narrow operand allowlist includes GNU `-z <keyword>`, preserving controls such as `-z relro`, `-Wl,-z,relro`, `--for-linker=-z,relro`, and `-Xlinker -z -Xlinker relro`.

Direct arguments and GNU-compatible forwarding forms consume the same shared classifier. This is not a blanket ban on `link-arg` / `link-args`; option-only controls such as `-Wl,--as-needed` remain allowed.

## Decision

Repository-owned Cargo flags fail closed when the currently modeled direct/GNU-compatible linker grammar introduces any unconsumed non-option positional input. A future allowlist must bind the artifact or script to exact digest, producer/source revision, target triple, linker/compiler toolchain identity, build configuration, transitive input closure, reproducibility/provenance evidence, and the consuming exact tree. File extension, path shape, or bare filename is never approval.

## Security effect

The repair prevents repository configuration from inserting prebuilt native objects, archives, extensionless files, or GNU implicit scripts while leaving the Rust source closure and nominal linker selection apparently unchanged. Direct and forwarded variants are classified by the same authority code, so `-Wl`, `--for-linker`, and `-Xlinker` do not create parallel provenance policy.

## Residual surfaces

The following remain separate review surfaces and are not pre-authorized by this decision:

- non-GNU linker/control-file grammars and currently unmodeled option-arity/input mechanisms;
- environment `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS` and direct `cargo rustc -- ...` trailing flags;
- rustup/sysroot/toolchain composition, custom target specifications, and target-specific external toolchain scripts;
- runtime-derived or toolchain-internal inputs that cannot be established from Git-owned Cargo argument syntax alone.

Path-shaped and bare extensionless positional inputs in the currently modeled direct/GNU-compatible forms are no longer residual gaps; they are fail-closed by the shared arity-aware classifier.

## Primary evidence

Rust Project. (2026). *Codegen options: `link-arg` and `link-args`*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/

The rustc documentation states that `link-arg` appends a single argument and `link-args` appends multiple arguments to the linker invocation. It also explains that Unix-like targets commonly use `cc` or `clang` as the linker driver.

Free Software Foundation. (2026). *Using ld: Command-line options*. GNU Binutils documentation. https://sourceware.org/binutils/docs/ld/Options.html

GNU ld documents non-option command-line arguments as object files or archives to be linked together and documents option/operand grammar constraints relevant to positional-input parsing.

Free Software Foundation. (2026). *Implicit linker scripts*. GNU Binutils documentation. https://sourceware.org/binutils/docs/ld/Implicit-Linker-Scripts.html

GNU ld documents that an input file whose format is not recognized can be interpreted as a linker script. This is why all unconsumed non-option tokens in the modeled grammar are provenance-bearing inputs rather than only tokens with familiar native-object suffixes.

## Verification state

The RED→repair generations are structurally present on the active #317 lineage. This dossier does not claim hosted executable GREEN or independent review of the newest exact head. Those remain required after lineage reconciliation and exact-head workflow execution before merge or release readiness.
