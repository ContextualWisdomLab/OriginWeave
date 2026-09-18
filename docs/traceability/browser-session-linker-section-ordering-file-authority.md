# Browser Session linker section-ordering-file authority

Status: Proposed until exact-head hosted repository/security checks and independent review complete.

## Problem

GNU `ld --section-ordering-file=script` reads an external script using `SECTIONS` syntax and augments the current linker script by mapping input sections to the start of existing output sections. The GNU linker documentation lists it as another way to specify linker scripts. Because GNU multi-letter options accept one or two leading dashes, `-section-ordering-file=script` is an equivalent spelling.

For OriginWeave this is linker-script provenance authority. A repository-owned Cargo `rustflags` or `rustdocflags` value can otherwise load an unreviewed section-ordering script after Cargo/rustc select the reviewed source/dependency closure and change which input sections are mapped and ordered in the output artifact.

## Owner boundary

Browser Session keeps repository-selected compiler/rustdoc/toolchain/linker input authority in `tests/test_browser_session_cargo_compiler_authority_contract.py`. Production package/source topology remains owned by `tests/test_browser_session_trusted_adapter_boundary.py`; this repair does not duplicate that scanner or move dependency-source authority.

`--section-ordering-file` is classified by the existing linker-script authority helper so the current `-Wl,`, `--for-linker=`, `-Xlinker`, `-C link-arg`, `-C link-args`, build/target `rustflags`, `rustdocflags`, and rustdoc doctest forwarding all converge on one policy path.

## RED → repair

- RED `40f244c914f74e177273576612342ba03245fde9` covers build/target `rustflags`, build `rustdocflags`, rustdoc doctest compiler forwarding, GNU double-dash and single-dash forms, and an allowed `-Wl,-z,relro` control.
- Repair `d33b5f7c107cbd552fd7931f9f5e0b434624b609` extends only `_linker_option_selects_script()` in the canonical direct-linker authority contract. No second scanner or package/source-topology owner is introduced.

## Decision

Reject repository-selected section-ordering scripts by default.

A pathname allowlist is insufficient because a path does not prove script content identity, producer, symlink containment, linker compatibility, or reproducibility. Any future exception must bind the script to an immutable digest, reviewed producer/source identity, exact linker/toolchain compatibility, containment, SBOM/provenance evidence, deterministic rebuild evidence, expiry/invalidation rules, and rollback.

## Residual authority

Environment/direct-CLI linker flags, compiler-driver defaults, runner filesystem contents, linker distribution/version and `PATH`, externally restored build/cache state, sysroot contents, and externally materialized linker scripts remain CI/release supply-chain evidence surfaces rather than repository-source exceptions.

## Evidence

GNU Binutils documents `--section-ordering-file=script` as an external file that uses `SECTIONS` syntax to augment the current linker script and map input sections to output sections. The same options manual documents that multi-letter options accept one or two leading dashes and may use `=`-joined operands.

### Reference

Free Software Foundation. (2026). *GNU linker: Command-line options*. GNU Binutils. https://sourceware.org/binutils/docs/ld/Options.html (retrieved September 19, 2026).
