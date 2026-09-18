# Browser Session rustdoc doctest compiler-argument authority

Status: Draft; source-semantic contract current on PR #317. Hosted executable evidence is still required before this generation is GREEN.

## Problem

Cargo can pass repository-owned `[build].rustdocflags` and matching `target.<tuple|cfg>.rustdocflags` directly to `rustdoc`. Nightly rustdoc's `--doctest-build-arg` then forwards one argument per occurrence to the compiler used to build documentation tests. This is a second-order compiler-input boundary: a reviewed rustdoc invocation can otherwise smuggle `--sysroot`, `--extern`, response files, code-generation backends/plugins, linker selection, linker scripts, native inputs, or other already-governed rustc/linker authority into doctest compilation.

The Browser Session provenance contract therefore has to inspect the forwarded compiler argument vector, not merely the outer rustdoc command line.

## Ownership and constraints

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source/dependency topology. `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single owner for Git-owned Cargo-selected compiler/linker execution and external-input authority. This repair reuses its existing rustc/codegen/linker classifiers and does not add a second Cargo topology scanner.

The rule does not blanket-ban `--doctest-build-arg`. Forwarded arguments that do not widen executable or external-input authority, such as a reviewed `--cfg` or `-Copt-level=2`, remain allowed.

## RED → repair

RED `eeb6944e9f77f80dfc0aa5812dcce42b02b8f7b4` adds hostile build/target fixtures for:

- forwarded `--sysroot=...`,
- split forwarded `-C` + `linker=...`, and
- forwarded `-Zcodegen-backend=...`.

It also retains `--cfg=originweave_reviewed` and `-Copt-level=2` as non-authority controls.

Repair `35733ea613225277f4baa7e100d4403453810e2f` adds `_flags_select_rustdoc_doctest_compiler_authority()` to the canonical Cargo compiler-authority contract. The helper reconstructs the ordered compiler arguments carried by split and `--doctest-build-arg=...` forms, fails closed on a missing operand, and reuses the existing codegen, linker, and external-input classifiers. Both build-level and target-level `rustdocflags` consume the same helper.

## Primary evidence

- The Cargo Book, *Configuration*, documents `build.rustdocflags` and `target.<tuple|cfg>.rustdocflags` as low-level custom flags passed to rustdoc and separately identifies environment/direct-command sources with higher precedence.
- The nightly rustdoc book, *Unstable features*, documents `--doctest-build-arg` as a way to add arguments to rustc when compiling doctests; the unstable command-line family requires nightly rustdoc with `-Z unstable-options`.

## Security effect

Git-owned Cargo configuration can no longer use rustdoc's doctest compiler forwarding as an unchecked tunnel around the Browser Session compiler/linker provenance policy. The inner compiler vector is evaluated with the same authority semantics as direct Cargo rustflags rather than with a duplicate policy.

## Residual execution provenance

This repository-source contract intentionally does not claim authority over:

- `RUSTDOCFLAGS`, `CARGO_ENCODED_RUSTDOCFLAGS`, `CARGO_BUILD_RUSTDOCFLAGS`, or target-specific environment overrides;
- direct `cargo rustdoc -- ...` or manual rustdoc invocation;
- runner image, PATH, rustup/toolchain, default rustdoc/rustc identity, or externally supplied compiler/linker artifacts;
- future rustdoc/rustc options that introduce a new execution/input grammar not yet represented by the canonical classifiers;
- immutable artifact identity, SBOM/attestation, sandbox policy, compatibility qualification, and rollback for any future approved external compiler component.

Those surfaces require CI/release environment evidence from their canonical owners. A newly introduced rustdoc/rustc forwarding primitive is a fresh provenance finding until it is mapped to the canonical classifier and covered by hostile and control fixtures.

## Acceptance

This generation is source-semantic only until the exact reconciled head has hosted repository/security execution and current-head independent review. A command acknowledgement, static inspection, predecessor workflow result, or skipped Draft workflow is not executable GREEN.
