# Browser Session MRI linker script authority traceability

Status: Draft contract evidence on PR #317. This document does not claim hosted repository/security GREEN.

## Problem

GNU `ld` supports an alternate MRI-compatible linker command language through `-c MRI-commandfile` / `--mri-script=MRI-commandfile`. The command file is an external linker input and, if it is not in the current directory, GNU `ld` searches preceding `-L` directories for it. The Browser Session direct-linker classifier already failed closed on GNU/LLD general-purpose linker scripts and default linker scripts, but the equals-form `--mri-script=...` was not a recognized script selector.

A repository-owned `rustflags` or `rustdocflags` value could therefore forward `--mri-script=...` through `-Wl,`, `--for-linker=`, or `-Xlinker` while the reviewed Cargo package/source closure remained unchanged.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler, rustdoc, linker execution, and external input authority.
- The repair must reuse the shared direct-linker parser for build/target/profile `rustflags`, `rustdocflags`, and rustdoc doctest forwarding.
- `-Wl,-z,relro` remains an allowed control because it does not select an external script or command file.
- Ambient linker flags, direct CLI invocation, runner/toolchain contents, and the linker implementation/version remain CI/release provenance surfaces.

## RED

Commit `975e35448aafd9aba57935897431841a29101864` adds `tests/test_browser_session_linker_mri_script_authority_contract.py` with hostile fixtures for:

- build `rustflags` forwarding `-Wl,--mri-script=...`;
- target `rustflags` forwarding `--for-linker=--mri-script=...`;
- build `rustdocflags` forwarding the selector through `-Xlinker`;
- rustdoc `--doctest-build-arg` forwarding an MRI command file;
- an allowed `-Wl,-z,relro` control.

At the predecessor exact head, `--mri-script=...` was neither a known script selector nor a positional input because it begins with `-`. The equals form therefore preserved a concrete source-semantic bypass.

## Decision and repair

Commit `5a066aacff29534d934f85b203121be5125347c4` extends the existing linker-script classifier with the documented GNU spellings `-c` and `--mri-script`, plus `--mri-script=...`. This is a shared-owner repair: `-Wl,`, `--for-linker=`, `-Xlinker`, build/target/profile flags, rustdoc, and doctest forwarding continue to consume one direct-linker authority classifier.

The short split form `-c MRI-commandfile` was already incidentally rejected because the following filename became a positional native input. Modeling `-c` explicitly makes the policy reflect linker semantics instead of relying on that incidental parse outcome. The equals-form long option is the material bypass closed by this generation.

A pathname allowlist was rejected. An MRI command file is executable linker configuration in the provenance sense: pathname review alone does not prove immutable contents, transitive inputs, search-path resolution, symlink containment, linker/toolchain compatibility, reproducibility, SBOM/attestation, or rollback. Any future exception requires versioned immutable artifact identity and those release properties.

## Security and release consequence

This closes the modeled Git-owned Cargo path in which an MRI command file could alter linker behavior without entering the reviewed source/dependency topology. It does not claim integrity of ambient/default linker state, direct CLI flags, runner images, or externally restored toolchain artifacts.

Hosted exact-head execution, whole-PR review closure, owned production rustdoc/test/edge coverage, and immutable release acceptance remain separate gates.

## Primary reference

Free Software Foundation. (2025). *The GNU linker* (GNU Binutils 2.45), `-c MRI-commandfile` / `--mri-script=MRI-commandfile`. https://sourceware.org/binutils/docs-2.45/ld.pdf
