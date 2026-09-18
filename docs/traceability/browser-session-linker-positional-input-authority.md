# Browser Session linker positional-input authority

## Problem

The Browser Session Cargo compiler-authority contract already rejects repository-owned linker replacement, search-path reselection, response files, plugins, explicit GNU linker scripts, native-library `-L` / `-l` widening, rustc `--extern`, and other modeled execution/input-authority surfaces. It did not constrain positional native object/archive arguments supplied through rustc `-C link-arg` / `link-args`.

The rustc book states that `link-arg` appends one extra argument to the linker invocation and `link-args` appends multiple arguments. On Unix-like targets rustc commonly invokes a C compiler such as `cc` or `clang` as the linker driver. GNU ld documents non-option arguments as object files or archives to be linked into the output. Consequently, Git-owned Cargo configuration could inject a reviewed-tree-external object/archive into the final Browser Session binary without changing `Cargo.toml`, canonical production-source topology, nominal linker selection, or `-L` / `-l` settings.

This is a provenance gap: an object/archive pathname alone does not establish the artifact's source revision, producer, digest, target/toolchain identity, reproducibility, or review ancestry.

## Constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. The Cargo compiler-authority contract consumes that topology and may constrain repository-owned linker input authority, but it must not implement a second Cargo workspace/package resolver.

This slice covers modeled positional native artifacts supplied from Git-owned `.cargo/config.toml` / `.cargo/config` rustflags. It does not claim to parse every linker grammar. In particular, extensionless/non-GNU control files and implicit linker-script inputs remain distinct residual surfaces until they have an authoritative parser/provenance contract.

## RED → repair

RED `e547203368da2aec62e2c94ab491eb0749d7d7b5` adds hostile cases for:

- direct `-C link-arg=tools/review-bypass-object.o`;
- target-scoped `-C link-args=tools/review-bypass-archive.a`; and
- compiler-driver forwarding through `-Wl,tools/review-bypass-object.o`, `--for-linker=tools/review-bypass-object.o`, and `-Xlinker tools/review-bypass-object.o`.

The predecessor exact `61053c9cc3ca58f5d812f3ab64660f4e662afdce` allowed these forms because its external-input classifier covered `-L`, `-l`, `--extern`, scripts/plugins/response files, and executable reselection but not positional native artifacts.

Repair `e73d221cf28aa25ae6fbd941db95d5d923c7e883` keeps canonical Cargo topology ownership unchanged and extends the shared linker-argument classifier with a modeled positional-native-input predicate. It rejects common object/archive artifact forms (`.o`, `.obj`, `.lo`, `.a`, `.lib`, `.rlib`, `.so` including versioned `.so.*`, `.dylib`, `.bc`, and `.res`) whether direct or passed through the already modeled linker-forwarding forms. Existing option-only controls such as `-Wl,--as-needed` remain allowed; the repair is not a blanket ban on `link-arg` / `link-args`.

## Decision

Repository-owned Cargo flags must fail closed when they add modeled positional native object/archive inputs outside the canonical reviewed source/dependency closure. A future allowlist must bind an artifact to exact digest, producer/source revision, target triple, linker/compiler toolchain identity, build configuration, reproducibility/provenance evidence, and the consuming exact tree. File extension or pathname alone is never approval.

## Security effect

The repair prevents a repository configuration change from inserting a prebuilt native object/archive into the Browser Session binary while leaving the Rust source closure and nominal linker selection apparently unchanged. Direct and forwarded variants are classified by the same shared authority code, so `-Wl`, `--for-linker`, and `-Xlinker` do not create parallel provenance policy.

## Residual surfaces

The following remain separate review surfaces and are not pre-authorized by this decision:

- extensionless or otherwise unmodeled positional inputs, including GNU ld implicit-script interpretation;
- non-GNU linker/control-file input mechanisms;
- environment `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS` and direct `cargo rustc -- ...` trailing flags;
- rustup/sysroot/toolchain composition, custom target specifications, and target-specific external toolchain scripts.

## Primary evidence

Rust Project. (2026). *Codegen options: `link-arg` and `link-args`*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/

The rustc documentation states that `link-arg` appends a single argument and `link-args` appends multiple arguments to the linker invocation. It also explains that Unix-like targets commonly use `cc` or `clang` as the linker driver.

Free Software Foundation. (2026). *Using ld: Command-line options*. GNU Binutils documentation. https://sourceware.org/binutils/docs/ld/Options.html

GNU ld documents non-option command-line arguments as object files or archives to be linked together. It also documents that unrecognized input-file formats may be interpreted as linker scripts, which is why implicit script/control-file handling remains a separately tracked residual surface rather than being silently declared closed by this artifact-suffix classifier.

## Verification state

The RED and minimal repair are structurally present on the active #317 lineage. This dossier does not claim hosted executable GREEN or independent current-head review. Those remain required after lineage reconciliation and exact-head workflow execution before merge or release readiness.