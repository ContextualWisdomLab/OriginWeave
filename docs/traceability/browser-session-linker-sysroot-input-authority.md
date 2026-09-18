# Browser Session linker sysroot input authority traceability

Status: Draft contract evidence on PR #317. This document does not claim hosted repository/security GREEN.

## Problem

The Browser Session Cargo compiler-authority contract already failed closed when Git-owned `rustflags` or `rustdocflags` selected a Rust compiler/rustdoc sysroot with `--sysroot`. A distinct linker surface remained. Rust codegen flags can forward direct linker arguments through `-Wl,`, `--for-linker=`, or `-Xlinker`; the shared direct-linker classifier did not classify GNU/LLD `--sysroot=<directory>` as external linker input authority.

GNU `ld` documents `--sysroot=directory` as replacing the linker's configured sysroot location. LLD documents its ELF linker as a GNU-linker-compatible replacement accepting GNU command-line arguments. A repository-owned forwarded linker sysroot can therefore change where the linker resolves system libraries and related link inputs without changing the reviewed Cargo package/source closure.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo-selected compiler, rustdoc, and linker execution/input authority.
- The fix must cover build/target `rustflags`, build/target `rustdocflags`, and rustdoc doctest compiler forwarding through the existing parser; no second Cargo topology or linker-authority scanner is introduced.
- Unrelated linker hardening remains permitted. In particular, `-Wl,-z,relro` does not select an external input and remains an allowed control.
- Environment-selected flags, direct rustc/rustdoc/linker CLI invocation, runner/toolchain contents, and the identity of default system sysroots remain CI/release supply-chain evidence surfaces.

## RED

Commit `1af97ad213eb314c408dd5b0b20b87b044822e72` adds `tests/test_browser_session_linker_sysroot_input_authority_contract.py` with hostile fixtures for:

- build `rustflags` forwarding `-Wl,--sysroot=...`;
- target `rustflags` forwarding `--for-linker=--sysroot=...`;
- build `rustdocflags` forwarding the equals form through `-Xlinker`;
- rustdoc `--doctest-build-arg` forwarding a linker sysroot;
- an allowed `-Wl,-z,relro` control.

At the parent exact head, `_direct_linker_arguments_extend_authority()` classified plugins, executable error handlers, DTLTO executable selectors, linker scripts, response files, native library selectors, and positional native inputs, but `--sysroot=<directory>` matched none of those branches. The new hostile cases therefore preserve a concrete source-semantic gap rather than broadening policy by assertion.

## Decision and repair

Commit `372f319a4c68669a29c10e56e7e726d469e6c318` extends the existing `_linker_argument_extends_external_inputs()` classifier so split or equals `--sysroot` is treated as external linker input authority. The repair is two lines in the canonical owner. Existing `-Wl,`, `--for-linker=`, `-Xlinker`, build, target, rustdoc, profile, and doctest-forwarding paths consume that same classifier.

The earlier rustc/rustdoc `--sysroot` check remains valid and intentionally redundant at its more specific compiler boundary. The added linker-level classification covers the same spelling only after it has been forwarded into linker authority.

A path allowlist was rejected. A path string does not prove immutable sysroot contents, system-library identity, symlink containment, producer toolchain, SBOM/provenance, or reproducibility. Any future approved linker sysroot requires versioned immutable artifact identity and release evidence rather than pathname trust.

## Security and release consequence

This closes the modeled Git-owned Cargo path in which reviewed Rust source/dependency topology remained unchanged while the linker resolved inputs under a repository-selected sysroot. It does not prove the integrity of the ambient/default linker sysroot, runner image, direct CLI flags, environment variables, or externally restored toolchain state. Those remain release/toolchain provenance requirements and must not be inferred from this source contract.

Hosted exact-head execution, whole-PR review closure, owned production rustdoc/test/edge coverage, and release acceptance remain separate gates.

## Primary references

Free Software Foundation. (2025). *The GNU linker* (GNU Binutils 2.45), `--sysroot=directory`. https://sourceware.org/binutils/docs-2.45/ld.pdf

LLVM Project. (2026). *LLD - The LLVM linker*. Retrieved September 18, 2026, from https://lld.llvm.org/
