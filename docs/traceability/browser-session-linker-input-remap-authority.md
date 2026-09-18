# Browser Session linker input-remap authority

Status: Proposed until exact-head hosted repository/security checks and independent review complete.

## Problem

GNU `ld` can rewrite the link input graph after Cargo/rustc have selected the reviewed inputs. `--remap-inputs=pattern=filename` substitutes a different filename before the linker opens an input, and `--remap-inputs-file=file` loads the remapping policy from an external file. The remapping also applies to files named by `INPUT` statements in linker scripts. GNU `ld` accepts multi-letter options with either one or two leading dashes, so `-remap-inputs=...` and `-remap-inputs-file=...` are equivalent spellings.

For OriginWeave this is provenance authority, not a linker-tuning preference. A repository-owned Cargo `rustflags`/`rustdocflags` value could otherwise redirect a reviewed native/library input to an unreviewed object, archive, shared library, or `/dev/null` while leaving the Cargo dependency graph unchanged.

## Owner boundary

Browser Session keeps repository-selected compiler/rustdoc/toolchain/linker input authority in `tests/test_browser_session_cargo_compiler_authority_contract.py`. Production package/source topology remains owned by `tests/test_browser_session_trusted_adapter_boundary.py`; this repair does not duplicate that scanner or move dependency-source authority.

The canonical direct-linker parser now classifies both inline and file-backed input remapping as authority-extending, including GNU single-dash aliases. Existing `-Wl,`, `--for-linker=`, `-Xlinker`, `-C link-arg`, `-C link-args`, build/target `rustflags`, `rustdocflags`, and rustdoc doctest forwarding continue to converge on the same parser.

## RED → repair

- RED `c3416fad876bcd15477521b666f18c3c7a3a5cff` adds hostile fixtures for inline remapping, file-backed remapping, GNU single-dash aliases, build/target flags, rustdoc flags, and doctest compiler forwarding. `-Wl,-z,relro` remains an allowed control.
- Repair `333195e3001357237a3844df34729ff730da99dc` adds `_linker_option_remaps_inputs()` to the existing direct-linker authority classifier and changes no Cargo topology owner.

## Decision

Reject repository-selected input remapping by default.

A pathname allowlist is insufficient because a path does not prove the selected artifact's content identity, producer, toolchain compatibility, symlink containment, or reproducibility. A future exception must bind the remap rule and every selected replacement artifact to immutable digests, producer/source identity, exact linker/toolchain compatibility, SBOM/provenance evidence, containment, deterministic rebuild evidence, expiry/invalidation rules, and rollback.

## Residual authority

Environment/direct-CLI linker flags, compiler-driver defaults, runner filesystem contents, linker distribution/version and `PATH`, externally restored build/cache state, sysroot contents, and runtime deployment filesystem identity remain CI/release supply-chain evidence surfaces rather than repository-source exceptions.

## Evidence

GNU Binutils documents that `--remap-inputs=pattern=filename` changes input filenames before they are opened, `--remap-inputs-file=file` loads remappings from a file, `/dev/null`/`NUL` can suppress an input, and linker-script `INPUT` references are affected. It also documents that multi-letter options accept one or two leading dashes and may take `=`-joined operands.

### Reference

Free Software Foundation. (2026). *GNU linker: Command-line options*. GNU Binutils. https://sourceware.org/binutils/docs/ld/Options.html (retrieved September 19, 2026).
