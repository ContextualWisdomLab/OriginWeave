# Browser Session linker `--just-symbols` / `-R` input authority

Status: Draft

## Problem

OriginWeave's Browser Session build-provenance boundary already rejects positional native inputs, linker scripts, response files, plugins, custom sysroots, executable selectors, and modeled library-search inputs. A remaining GNU-compatible linker form could still inject an external file without becoming an unconsumed positional token:

- `--just-symbols=<file>`
- compact `-R<file>`

GNU `ld` reads symbol names and absolute addresses from the `-R` / `--just-symbols` operand without relocating or including that file in the output. The same `-R` spelling is treated as an rpath when its operand is a directory. GNU `ld` also permits single-letter option operands to be joined to the option letter. Therefore both forms can change link semantics while bypassing a classifier that only rejects positional native inputs.

Rust exposes the path through `-C link-arg` and `-C link-args`; on Unix-like targets using a compiler driver, rustc documents `-C link-arg=-Wl,$ARG` as the way to pass an argument to the actual linker. Repository-owned Cargo `rustflags` and `rustdocflags` therefore form a reviewed provenance boundary rather than an ordinary optimization surface.

## Decision

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected Cargo compiler/rustdoc/linker execution and input authority. The existing direct-linker classifier now fails closed on:

- split `-R` and `--just-symbols` selectors;
- compact `-R<path>`;
- `--just-symbols=<path>`.

The rule is intentionally shared by build/target `rustflags`, build/target `rustdocflags`, and rustdoc `--doctest-build-arg` forwarding. No second Cargo topology or linker scanner was introduced. `tests/test_browser_session_trusted_adapter_boundary.py` remains the production Cargo topology/source-closure owner.

The rule does not ban ordinary linker hardening options. `-Wl,-z,relro` remains an allowed control in the focused hostile fixture.

## RED → repair evidence

- RED: `f271a2aefe58d12f988ca3c0b89c9b917a5ca550` adds `tests/test_browser_session_linker_just_symbols_input_authority_contract.py` and demonstrates that equals/compact just-symbols forms were not classified by the predecessor shared authority.
- Repair: `7b75fa31291fba28867331a72338174848e4a28c` adds `_linker_option_selects_just_symbols_or_rpath()` to the existing direct-linker authority parser and routes it through the existing `rustflags:codegen linker`, `rustdocflags:codegen linker`, and doctest compiler-authority call sites.

The focused fixture covers build `rustflags`, target `rustflags`, build `rustdocflags`, rustdoc doctest forwarding, and an ordinary `-z relro` control.

## Security and provenance effect

A Git-owned Cargo configuration can no longer select an unreviewed symbol-address file with the modeled GNU-compatible `--just-symbols=<file>` or compact `-R<path>` forms without tripping the Browser Session provenance contract. If `-R` names a directory, the same fail-closed decision prevents repository configuration from silently injecting an rpath through this ambiguous spelling.

This is a source-semantic contract. It does not by itself establish hosted executable GREEN, compiler-driver parity for every non-GNU linker, or release provenance.

## Residual authority

The following remain outside this leaf repository-source contract and require CI/release or linker-family evidence rather than duplicated OriginWeave ownership:

- environment- or direct-CLI-injected rustc/rustdoc/linker arguments;
- linker-family-specific symbol/control-file mechanisms that are not GNU-compatible grammar already modeled by the shared parser;
- ambient compiler/linker binaries, sysroot contents, runner image, and restored build artifacts;
- immutable release evidence tying the effective toolchain, linker, arguments, SBOM, and provenance to the shipped artifact.

Any future exception for a symbol-address input must prove immutable artifact identity and digest, producer provenance, exact toolchain/linker compatibility, purpose, containment, reproducibility, and rollback in the same reviewed delta. A pathname allowlist is insufficient.

## References

Free Software Foundation. (n.d.). *GNU ld: Command-line options*. GNU Binutils documentation. https://sourceware.org/binutils/docs/ld/Options.html

The Rust Project Developers. (n.d.). *Codegen options: `link-arg` and `link-args`*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/

Retrieved 2026-09-19.
