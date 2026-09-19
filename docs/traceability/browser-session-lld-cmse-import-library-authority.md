# Browser Session LLD CMSE import-library authority

Status: Source repair implemented; hosted proof pending

## Problem

LLVM LLD's ELF driver defines `--in-implib` as an ARM CMSE input selector. The option reads an existing CMSE secure-code import library from a previous program revision so LLD can preserve secure gateway entry-function addresses in a new CMSE import library or secure image.

On the reviewed Browser Session Cargo authority path, the split spelling `--in-implib FILE` is conservatively caught because `FILE` becomes an unconsumed positional linker input. The `EEq` joined spelling `--in-implib=FILE` kept the external artifact path inside the option token and previously bypassed the canonical direct-linker authority predicate.

This is an input-provenance gap, not a general ARM CMSE ban. `--out-implib=FILE` names an output destination and is not equivalent to the input selector.

## Primary evidence

LLVM upstream main at `f8f4816496f6126f371350819d48017d1c330b56` provides the current primary evidence used for this repair:

- `lld/ELF/Options.td`: `in_implib` is `EEq<"in-implib", ...>` and is documented as reading an existing CMSE secure-code import library and preserving entry-function addresses in the resulting library/image.
- `lld/ELF/Options.td`: `out_implib` is separately documented as outputting the CMSE secure-code import library to a file.
- `lld/ELF/Driver.cpp`: `OPT_in_implib` populates the CMSE input-library argument and is rejected on unsupported targets.
- LLD ARM tests exercise the CMSE input option and its validation rules.

Upstream references:

- <https://github.com/llvm/llvm-project/blob/f8f4816496f6126f371350819d48017d1c330b56/lld/ELF/Options.td>
- <https://github.com/llvm/llvm-project/blob/f8f4816496f6126f371350819d48017d1c330b56/lld/ELF/Driver.cpp>
- <https://github.com/llvm/llvm-project/blob/f8f4816496f6126f371350819d48017d1c330b56/lld/test/ELF/arm-cmse-diagnostics.s>

## Owner boundary

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo compiler/linker input authority. No parallel Cargo topology/config scanner was introduced.

The repair is consumed through the existing direct-linker classifier, so `rustflags`, `rustdocflags`, and rustdoc doctest compiler forwarding continue to share one authority decision path.

## RED → repair

- Structural RED: `411ec418c73b89a0f3923af0a16a214faafe0000` adds `tests/test_browser_session_lld_cmse_import_library_authority_contract.py`. It requires joined `--in-implib=...` to fail closed through the canonical authority helper, exercises rustdoc doctest compiler forwarding, and keeps output-only `--out-implib=...` as an allowed control.
- Minimal causal repair: `28ffd6fc4784b25e2d48f4d16b0de0acc4324c47` adds `_linker_option_selects_cmse_import_library()` and consumes it from `_direct_linker_arguments_extend_authority()`. The repair changes the canonical authority file by six added lines and no deletions; no unrelated rewrite or duplicate scanner was introduced.
- Exact compare from predecessor `65511f47d355a9c68ed669679bc406bd9230ab5f` to repair head is two commits, two files: the 36-line hostile contract plus the six-line canonical classifier repair.

No pull-request-triggered hosted workflow exists yet for repair head `28ffd6fc4784b25e2d48f4d16b0de0acc4324c47`, so this document does not claim hosted executable GREEN, repository/security GREEN, whole-PR review closure, or 100% quality-gate closure.

## Rejected alternatives

- Path allowlists are insufficient: a trusted-looking pathname does not prove the selected import library's content, producer, toolchain compatibility, symlink containment, or reproducibility.
- Blocking every CMSE option would conflate external input authority with output configuration and would reject `--out-implib` without evidence.
- A supplemental scanner would duplicate the canonical direct-linker authority owner.

## Future exception evidence

Any future decision to permit repository-selected CMSE input import libraries must prove the selected artifact digest, producer provenance, exact LLD/toolchain compatibility, repository/approved-artifact containment, SBOM/provenance inclusion, reproducibility, and rollback/invalidation behavior in the same reviewed delta.
