# Browser Session default linker script authority traceability

Status: Draft contract evidence on PR #317. This document does not claim hosted repository/security GREEN.

## Problem

The Browser Session Cargo compiler-authority contract already fails closed on explicit GNU/LLD linker scripts selected through `-T` / `--script`. A distinct spelling remained unmodeled: GNU `ld` also accepts `-dT scriptfile` / `--default-script=scriptfile`, and LLD documents the same default-script interface. The default script is still an external file that controls linker behavior; GNU differs only in delaying its processing until the rest of the command line has been processed.

At the predecessor exact head, `_linker_option_selects_script()` recognized `-T`, attached `-T...`, `--script`, and `--script=...`, but did not recognize `-dT` or `--default-script`. Repository-owned `rustflags` or `rustdocflags` could therefore forward an equals-form default script through `-Wl,`, `--for-linker=`, or `-Xlinker` while the reviewed Cargo package/source closure remained unchanged.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler, rustdoc, linker execution, and external input authority.
- The repair must reuse the existing direct-linker parser for build/target/profile `rustflags`, `rustdocflags`, and rustdoc doctest compiler forwarding. A second Cargo or linker scanner is not introduced.
- Unrelated linker hardening remains permitted. `-Wl,-z,relro` is retained as an allowed control because it does not select a script or other external input.
- Environment/direct-CLI linker flags, the ambient linker binary/version, and externally supplied toolchain contents remain CI/release supply-chain evidence surfaces.

## RED

Commit `6bd948fd5cfbf71ecebca4e0001fc7cfcffb4da5` adds `tests/test_browser_session_linker_default_script_authority_contract.py`. Hostile fixtures cover:

- build `rustflags` forwarding `-Wl,--default-script=...`;
- target `rustflags` forwarding `--for-linker=--default-script=...`;
- build `rustdocflags` forwarding the same selector through `-Xlinker`;
- rustdoc `--doctest-build-arg` forwarding a default script;
- an allowed `-Wl,-z,relro` control.

The predecessor classifier did not match the equals-form `--default-script=...`, so these fixtures preserve an actual source-semantic provenance gap rather than asserting a broad deny rule.

## Decision and repair

Commit `63f12526044b10ffa049c3fbf9701bc31760d44f` extends the existing linker-script selector with the documented GNU/LLD spellings `-dT` and `--default-script`. Exact `-dT` / `--default-script`, attached `-dT...`, and equals `--default-script=...` are classified alongside the already reviewed `-T` / `--script` forms. The existing `-Wl,`, `--for-linker=`, `-Xlinker`, build, target, profile, rustdoc, and doctest-forwarding paths therefore inherit the repair without another topology or policy owner.

The fix intentionally does not path-allowlist linker scripts. A reviewed pathname does not prove immutable file contents, symlink containment, the complete input set introduced by script commands, linker/toolchain identity, or reproducible output. A future exception requires an immutable script digest and artifact identity, containment proof, transitive input provenance, linker/toolchain compatibility, SBOM/attestation evidence, reproducibility, and rollback qualification.

## Security and release consequence

A repository-owned default linker script can influence output layout and symbol/input resolution just as an explicit linker script can. This repair closes the modeled Git-owned Cargo path without claiming anything about ambient linker defaults, direct CLI invocation, runner images, or release-time toolchain integrity.

Hosted exact-head execution, whole-PR independent review closure, owned production rustdoc/test/edge coverage, and immutable release acceptance remain separate gates.

## Primary references

Free Software Foundation. (2025). *The GNU linker* (GNU Binutils 2.45), `-dT scriptfile` / `--default-script=scriptfile`. https://sourceware.org/binutils/docs-2.45/ld.pdf

LLVM Project. (2026). *Linker Script implementation notes and policy*. LLD documentation. Retrieved September 18, 2026, from https://lld.llvm.org/ELF/linker_script.html
