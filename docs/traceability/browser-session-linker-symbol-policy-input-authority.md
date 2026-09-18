# Browser Session linker symbol-policy file authority

Status: Draft

## Problem

OriginWeave's Browser Session build-provenance boundary already rejects positional native linker inputs, linker scripts, response files, plugins, custom sysroots, executable selectors, library-search inputs, and GNU `-R` / `--just-symbols` inputs. A separate class of option-shaped file inputs could still bypass that shared direct-linker classifier:

- `--version-script=<file>`
- `--dynamic-list=<file>`
- `--retain-symbols-file=<file>`
- `--export-dynamic-symbol-list=<file>`

GNU `ld` consumes each operand as file content that changes symbol visibility, dynamic symbol selection, or retention in the output artifact. Because the path is embedded in an option token, a classifier that only rejects unconsumed positional native inputs does not see it.

Rust exposes this authority through `-C link-arg` and `-C link-args`; on Unix-like targets using a compiler driver, rustc documents `-C link-arg=-Wl,$ARG` as a way to pass an argument to the actual linker. Git-owned Cargo `rustflags` and `rustdocflags` therefore make these file selectors part of reviewed compiler/linker provenance rather than an ordinary optimization surface.

## Decision

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected Cargo compiler/rustdoc/linker execution and input authority. The existing direct-linker parser now classifies the four GNU symbol-policy file selectors above through one `LINKER_SYMBOL_POLICY_FILE_OPTIONS` set and `_linker_option_selects_symbol_policy_file()` helper.

The rule is reused by build/target `rustflags`, build/target `rustdocflags`, and rustdoc `--doctest-build-arg` forwarding through the existing `-Wl,`, `--for-linker=`, `-Xlinker`, `link-arg`, and `link-args` paths. No second Cargo topology scanner or cross-service authority was introduced. `tests/test_browser_session_trusted_adapter_boundary.py` remains the production Cargo package/source-closure owner.

Inline symbol selection without an external file remains allowed. The focused control fixture uses `--export-dynamic-symbol=originweave_*` to distinguish a literal pattern from `--export-dynamic-symbol-list=<file>`.

## RED → repair evidence

- RED: `84d3e085c7c7ac5b858b4282c9d53b1c4c94b24d` adds `tests/test_browser_session_linker_symbol_policy_input_authority_contract.py`. Against predecessor `4e8fbcc98835e54c3b7d3465b752a5b0f0e69e33`, all four option-shaped hostile inputs bypassed the shared direct-linker authority predicate.
- Repair: `ba239fab05209d13fba2abdd948924a6c2668d32` extends the existing canonical classifier with the symbol-policy file option set and helper. The repair changes only the shared authority file; the focused test already exercises build `rustflags`, target `rustflags`, build `rustdocflags`, doctest forwarding, and an inline-symbol allowed control.

A source-semantic focused check on the repaired helper classifies all four hostile selectors as external authority and leaves the inline `--export-dynamic-symbol=originweave_*` control unclassified. Hosted exact-head executable evidence is still required before repository/security GREEN is claimed.

## Security and provenance effect

Repository-owned Cargo configuration can no longer select unreviewed symbol-version, dynamic-list, retained-symbol, or export-symbol-list files through the modeled GNU-compatible linker forwarding paths without tripping the Browser Session provenance contract. This prevents link output semantics from depending on an external policy file that is absent from the reviewed Cargo package/source closure.

The policy is fail closed rather than pathname-allowlist based. A path alone does not prove file digest, producer provenance, symlink containment, exact linker compatibility, reproducibility, or rollback.

## Residual authority

The following remain CI/release supply-chain evidence surfaces rather than duplicated OriginWeave leaf ownership:

- environment- or direct-CLI-injected rustc/rustdoc/linker arguments;
- linker-family-specific symbol/control-file options outside the modeled GNU-compatible grammar;
- ambient compiler/linker binaries, sysroot contents, runner image, and restored build artifacts;
- immutable release evidence tying the effective toolchain, linker, arguments, SBOM, and provenance to the shipped artifact.

Any future exception for one of these symbol-policy files must bind immutable artifact identity and digest, producer/source provenance, exact toolchain/linker compatibility, purpose, containment, reproducibility, and rollback in the same reviewed delta.

## References

Free Software Foundation. (n.d.). *GNU ld: Options*. GNU Binutils documentation. https://sourceware.org/binutils/docs/ld/Options.html

The Rust Project Developers. (n.d.). *Codegen options: `link-arg` and `link-args`*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/

Retrieved 2026-09-19.
