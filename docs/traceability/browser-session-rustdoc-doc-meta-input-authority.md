# Browser Session rustdoc documentation-metadata input authority

Status: source-semantic repair evidence; not hosted executable GREEN.

## Problem

Nightly rustdoc exposes `--write-doc-meta-dir` and `--read-doc-meta-dir` behind `-Z unstable-options`. The write option emits a crate's shared documentation metadata to a directory. The read option is different: rustdoc enters finalize mode without crate source and merges cross-crate state from one or more supplied metadata directories into the documentation output.

Repository-owned Cargo `rustdocflags` could therefore select an external `--read-doc-meta-dir` and change generated cross-crate documentation/search state without changing the reviewed Rust source, Cargo package topology, or compiler/linker inputs. Treating the source tree as the complete documentation provenance would be false.

## Boundary and alternatives

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected rustc/rustdoc execution and external input authority. No second Cargo scanner or metadata parser is introduced.

Path allowlisting was rejected because a directory spelling does not prove immutable contents, ownership, symlink containment, release identity, or compatibility with the rustdoc/toolchain that generated the metadata. Blocking both read and write metadata directories was also rejected: `--write-doc-meta-dir` selects an output destination rather than an external documentation input.

## RED → repair

RED `b9103bd862a295954f7fc809e0d732fa982713f4` adds `tests/test_browser_session_rustdoc_doc_meta_input_authority_contract.py` with:

- build-level split `--read-doc-meta-dir PATH`,
- target-level equals `--read-doc-meta-dir=PATH`, and
- `--write-doc-meta-dir PATH` as an allowed output-only control.

The prior authority classifier accepted both hostile read cases.

Repair `47ff4370afdda5487224c437f6883d8947500c3f` broadens the existing rustdoc documentation-input classifier rather than creating another topology scan. `RUSTDOC_DOCUMENTATION_INPUT_OPTIONS` now includes `--read-doc-meta-dir` alongside the existing rendered-file inputs. Build- and target-level `rustdocflags` use the same `_flags_select_rustdoc_documentation_input()` call site and report `rustdocflags:documentation input`.

`--write-doc-meta-dir` remains allowed because it is an output destination. That distinction is a contract invariant, not a naming convenience.

## Primary reference

Rust Project. (2026). *The rustdoc book: Unstable features*. https://doc.rust-lang.org/nightly/rustdoc/unstable-features.html

The current rustdoc book states that `--read-doc-meta-dir` runs rustdoc in finalize mode, accepts multiple metadata directories, and is used to merge cross-crate state; no crate source is supplied in that mode. It separately states that `--write-doc-meta-dir` writes shared metadata to a directory.

## Buyer/security effect

A repository review can no longer silently acquire cross-crate documentation metadata through Git-owned build/target `rustdocflags` while claiming that generated documentation is derived only from the reviewed source/dependency closure. An eventual approved metadata import requires immutable directory/artifact identity, producer toolchain identity, crate/source provenance, content digests, compatibility evidence, SBOM/provenance linkage, and rollback/expiry rules.

This source contract does not prove a generated-docs publication, GitHub Pages deployment, browser rendering, accessibility, CSP, or immutable release. Environment/direct-CLI `RUSTDOCFLAGS`, `CARGO_ENCODED_RUSTDOCFLAGS`, ancestor/`$CARGO_HOME` config, external metadata producers, and runner/toolchain state remain CI/release provenance surfaces.
