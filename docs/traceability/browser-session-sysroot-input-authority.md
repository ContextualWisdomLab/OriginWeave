# Browser Session sysroot input authority traceability

Status: Draft contract evidence on PR #317. This document does not claim hosted repository/security GREEN.

## Problem

The Browser Session Cargo authority contract already fails closed on repository-owned compiler/linker executable replacement and modeled external crate/native-library inputs. A separate Rust compiler input surface remained: Git-owned Cargo `rustflags` or `rustdocflags` could pass `--sysroot <path>` or `--sysroot=<path>` and select a different Rust sysroot while the reviewed Cargo package/source closure remained unchanged.

`rustc --sysroot` overrides the system root used to find crates distributed with Rust. `rustdoc --sysroot` likewise changes the sysroot used while compiling documentation. A repository-selected sysroot is therefore compiler/documentation input authority, not ordinary diagnostic or optimization configuration.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. This supplemental contract consumes that authority and does not duplicate Cargo topology discovery.
- A repository-selected sysroot remains fail-closed until a separate reviewed contract establishes immutable toolchain/sysroot identity, artifact provenance, SBOM/reproducibility expectations, and release/runtime qualification.
- This contract covers Git-owned `.cargo/config.toml` and `.cargo/config` `rustflags`/`rustdocflags`. Environment `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `RUSTDOCFLAGS`, `CARGO_ENCODED_RUSTDOCFLAGS`, direct compiler/documentation CLI arguments, runner images, rustup/toolchain installation, and ambient sysroot composition remain CI/runtime-owner surfaces.
- Unrelated Rust flags are not rejected merely because they occur in `rustflags` or `rustdocflags`.

## RED

Commit `3943f268395180d199992709393190510ae48b2d` adds `tests/test_browser_session_sysroot_input_contract.py`. The hostile fixtures cover:

- build-level split `rustflags = ["--sysroot", "tools/review-bypass-sysroot"]`;
- build-level equals-form `rustflags = ["--sysroot=tools/review-bypass-sysroot"]`;
- target-scoped split `rustflags`;
- build-level equals-form `rustdocflags`;
- target-scoped split `rustdocflags`.

The predecessor classifier handled `--extern`, `-L`, and `-l` but did not classify `--sysroot`, so these fixtures preserve the missing compiler/documentation-input authority as a source-semantic RED. The control fixture keeps unrelated `--remap-path-prefix` configuration allowed.

## Decision and repair

Commit `f20401f0368ac9ab5f9756fc285972791526887c` extends the existing `_rustc_argument_extends_external_inputs()` classifier rather than adding a new Cargo scanner. Split and equals `--sysroot` spellings now classify as external compiler/documentation input authority. Because build/target `rustflags` and `rustdocflags` already consume `_flags_extend_external_link_inputs()`, all four Git-owned Cargo configuration paths fail closed through the same reviewed authority boundary.

The repair intentionally does not inspect or allowlist sysroot contents. A path allowlist would not establish that the standard-library crates, compiler-private crates, metadata, native objects, or supporting toolchain artifacts are the immutable reviewed artifacts expected by a release.

## Security effect and residual risk

The repair closes the modeled Git-owned Cargo path that could replace rustc/rustdoc sysroot inputs without changing the reviewed Browser Session Cargo package/source closure. It does not prove environment- or direct-CLI-selected sysroots, runner-image contents, rustup/toolchain installation state, or the integrity/reproducibility of the default sysroot. Those remain CI/release/toolchain provenance concerns and must not be inferred from this source contract.

## Primary references

The Rust Project. (n.d.). *Command-line arguments: `--sysroot`: override the system root*. The rustc book. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/command-line-arguments.html#--sysroot-override-the-system-root

The Rust Project. (n.d.). *Command-line arguments: `--sysroot`: override the system root*. The rustdoc book. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustdoc/command-line-arguments.html#--sysroot-override-the-system-root
