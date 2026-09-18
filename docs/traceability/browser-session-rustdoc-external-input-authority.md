# Browser Session rustdoc external-input authority

Status: Draft contract evidence on PR #317. This document does not claim hosted repository/security GREEN.

## Decision

Git-owned Cargo `rustdocflags` are subject to the same external crate/native-library input provenance gate as Git-owned `rustflags`. Both build-level and matching target-level rustdoc flags fail closed when they introduce `--extern`, `-L`, or `-l` input authority outside the reviewed Cargo production topology.

This is separate from rustdoc executable/linker selection. `build.rustdoc` and rustdoc codegen linker options were already covered; this slice closes the external-input half of the rustdoc boundary.

## Problem

Cargo documents `build.rustdocflags` and matching `target.<triple>.rustdocflags` / `target.<cfg>.rustdocflags` as extra command-line flags passed to rustdoc. `cargo rustdoc` also documents that rustdoc receives `-L` and `--extern` arguments as part of normal dependency wiring. Therefore a repository-owned rustdoc flag can widen documentation-time crate/native-library inputs even when the production Cargo manifests and the rustdoc executable remain unchanged.

The predecessor Browser Session contract classified external inputs for `rustflags` but applied only linker-selection classification to `rustdocflags`. A Git-owned `--extern review_bypass=tools/libreview_bypass.rlib` or `-Lnative=tools/review-bypass` in rustdocflags could therefore bypass the explicit input-provenance marker.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production package/source/dependency topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo compiler/rustdoc execution and input authority.
- Ordinary documentation flags such as `--document-private-items` and `--cfg docsrs` remain allowed.
- Environment `RUSTDOCFLAGS` / `CARGO_ENCODED_RUSTDOCFLAGS` and direct `cargo rustdoc -- ...` CLI injection remain CI/runtime authority surfaces and are not claimed closed here.

## RED → repair

- Predecessor exact head: `a3031442e6b3a5e24db53fe53e17724a73c1165b`.
- RED: `5928b1a614c8620203675b609b75f5489338e06d` adds `tests/test_browser_session_rustdoc_external_input_contract.py`. It exercises build-level `--extern`, target-level `-Lnative=...`, and an unrelated-rustdocflags control against the canonical Cargo compiler authority helper.
- Repair: `e8edb487b779a1c4ff22bf2ca4c62ae7e52abd8b` applies `_flags_extend_external_link_inputs(...)` to both build and target `rustdocflags`, recording `rustdocflags:external link input` without adding another topology/config scanner.

The repair is source-level contract evidence only until the exact head receives hosted executable repository/security evidence.

## Residual surfaces

Environment-selected rustdoc flags, direct `cargo rustdoc` trailing arguments, unmodeled rustdoc arguments with equivalent external-input semantics, sysroot/toolchain composition, and target-specific mechanisms outside the currently modeled Cargo config remain separate review surfaces.

## Primary references

The Rust Project Developers. (2026). *Configuration*. *The Cargo Book*. https://doc.rust-lang.org/cargo/reference/config.html

The Rust Project Developers. (2026). *cargo rustdoc*. *The Cargo Book*. https://doc.rust-lang.org/cargo/commands/cargo-rustdoc.html
