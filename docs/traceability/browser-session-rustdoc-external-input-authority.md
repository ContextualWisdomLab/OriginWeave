# Browser Session rustdoc external-input authority

Status: Draft contract evidence on PR #317. This document does not claim hosted repository/security GREEN.

## Decision

Git-owned Cargo `rustdocflags` are subject to the same external crate/native-library input provenance gate as Git-owned `rustflags`. Both build-level and matching target-level rustdoc flags fail closed when they introduce `--extern`, `-L` / `--library-path`, or `-l` input authority outside the reviewed Cargo production topology.

This is separate from rustdoc executable/linker selection. `build.rustdoc` and rustdoc codegen linker options were already covered; this slice closes the external-input half of the rustdoc boundary.

## Problem

Cargo documents `build.rustdocflags` and matching `target.<triple>.rustdocflags` / `target.<cfg>.rustdocflags` as extra command-line flags passed to rustdoc. `cargo rustdoc` also documents that rustdoc receives `-L` and `--extern` arguments as part of normal dependency wiring. Rustdoc itself exposes `-L PATH` and its long alias `--library-path PATH` to add dependency search paths. Therefore a repository-owned rustdoc flag can widen documentation-time crate/native-library inputs even when the production Cargo manifests and the rustdoc executable remain unchanged.

The predecessor Browser Session contract classified external inputs for `rustflags` and later applied that classifier to `rustdocflags`, but the classifier recognized only `-L` and not rustdoc's equivalent `--library-path`. A Git-owned `--library-path tools/review-bypass-deps` or `--library-path=tools/review-bypass-deps` could therefore bypass the explicit `rustdocflags:external link input` marker.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production package/source/dependency topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo compiler/rustdoc execution and input authority.
- Ordinary documentation flags such as `--document-private-items` and `--cfg docsrs` remain allowed.
- Environment `RUSTDOCFLAGS` / `CARGO_ENCODED_RUSTDOCFLAGS` and direct `cargo rustdoc -- ...` CLI injection remain CI/runtime authority surfaces and are not claimed closed here.

## RED → repair

The original rustdoc external-input slice was introduced by RED `5928b1a614c8620203675b609b75f5489338e06d` and repair `e8edb487b779a1c4ff22bf2ca4c62ae7e52abd8b`, which applied the shared external-input classifier to build and target `rustdocflags`.

Follow-up review found that the long rustdoc alias was still outside that classifier:

- RED `edbd3ee2d1cfca8782c9d22cd4fa556107637adb` adds `tests/test_browser_session_rustdoc_library_path_input_authority_contract.py` with build-level split `--library-path PATH`, target-level `--library-path=PATH`, and an unrelated-rustdocflag control. The hostile fixtures require the policy-specific `rustdocflags:external link input` marker.
- Repair `af11b318a4eb64867cc9eabedcf236b62fbac67f` extends the existing shared external-library-input classifier with `--library-path` split/equal forms. No new Cargo topology/config scanner is introduced.
- Supplemental rustdoc metadata, render-input, doctest-compiler, and doctest-execution fixtures now assert their policy-specific markers instead of accepting only the common `Cargo .*execution override` prefix.

These commits are source-level contract evidence only until the exact head receives hosted executable repository/security evidence.

## Residual surfaces

Environment-selected rustdoc flags, direct `cargo rustdoc` trailing arguments, unmodeled rustdoc arguments with equivalent external-input semantics, sysroot/toolchain composition, and target-specific mechanisms outside the currently modeled Cargo config remain separate review surfaces.

## Primary references

The Rust Project Developers. (2026). *Configuration*. *The Cargo Book*. https://doc.rust-lang.org/cargo/reference/config.html

The Rust Project Developers. (2026). *cargo rustdoc*. *The Cargo Book*. https://doc.rust-lang.org/cargo/commands/cargo-rustdoc.html

The Rust Project Developers. (2026). *Command-line arguments*. *The rustdoc book*. https://doc.rust-lang.org/rustdoc/command-line-arguments.html
