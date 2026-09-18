# Browser Session LLD CMSE import-library authority

Status: Open repair finding

## Problem

LLVM LLD's ELF driver defines `--in-implib` as an ARM CMSE input selector. The option reads an existing CMSE secure-code import library from a previous program revision so LLD can preserve secure gateway entry-function addresses in a new CMSE import library or secure image.

On the reviewed Browser Session Cargo authority path, the split spelling `--in-implib FILE` is conservatively caught because `FILE` becomes an unconsumed positional linker input. The `EEq` joined spelling `--in-implib=FILE` keeps the external artifact path inside the option token. The current direct-linker classifier does not model that option, so the joined spelling is not yet rejected by the canonical authority predicate.

This is an input-provenance gap, not a general ARM CMSE ban. `--out-implib=FILE` names an output destination and is not equivalent to the input selector.

## Primary evidence

- LLVM LLD `lld/ELF/Options.td`: `in_implib` is `EEq<"in-implib", ...>` and is documented as reading an existing CMSE secure-code import library and preserving entry-function addresses in the resulting library/image.
- LLVM LLD `lld/ELF/Arch/ARM.cpp`: the CMSE import library is an ELF object with a symbol table; `--in-implib` selects an input import library from a previous revision of the program.
- LLVM LLD ARM tests exercise `--in-implib=lib.o`, reject multiple input import libraries, reject use without `--cmse-implib`, and reject the option on non-ARM targets.

Upstream references:

- <https://github.com/llvm/llvm-project/blob/main/lld/ELF/Options.td>
- <https://github.com/llvm/llvm-project/blob/main/lld/ELF/Arch/ARM.cpp>
- <https://github.com/llvm/llvm-project/blob/main/lld/test/ELF/arm-cmse-diagnostics.s>

## Owner boundary

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo compiler/linker input authority. No parallel Cargo topology/config scanner is introduced here.

The repair belongs in the existing direct-linker classifier and must be consumed through the existing `rustflags`, `rustdocflags`, and doctest compiler-forwarding paths.

## Required repair

1. Add a hostile contract for the joined `--in-implib=...` spelling and retain `--out-implib=...` as an output-only control.
2. Add one bounded predicate for split/joined `--in-implib` and consume it from `_direct_linker_arguments_extend_authority()`.
3. Prove the exact repaired head with the focused contract before treating this document as closed.

The earlier attempted RED fixture was removed rather than leaving the branch knowingly red before the canonical single-writer repair could be applied atomically.

## Rejected alternatives

- Path allowlists are insufficient: a trusted-looking pathname does not prove the selected import library's content, producer, toolchain compatibility, symlink containment, or reproducibility.
- Blocking every CMSE option would conflate external input authority with output configuration and would reject `--out-implib` without evidence.
- A supplemental scanner would duplicate the canonical direct-linker authority owner.

## Future exception evidence

Any future decision to permit repository-selected CMSE input import libraries must prove the selected artifact digest, producer provenance, exact LLD/toolchain compatibility, repository/approved-artifact containment, SBOM/provenance inclusion, reproducibility, and rollback/invalidation behavior in the same reviewed delta.
