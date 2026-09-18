# Browser Session direct LLVM argument authority

Status: Draft evidence for PR #317. This document describes a repository-source provenance boundary; it is not hosted execution or browser-observed acceptance evidence.

## Problem

`rustc -C llvm-args="..."` passes arguments directly to LLVM. The rustc book explicitly states that this surface talks directly to LLVM and is not covered by rustc's normal CLI stability guarantees. Current rustc also combines target-spec LLVM arguments with user `-Cllvm-args`, places the user-provided values after target-spec values, and forwards the resulting argument vector to `LLVMRustSetLLVMOptions`; the source notes LLVM `cl::opt` last-wins behavior.

That means a Git-owned Cargo configuration can bypass the reviewed, typed rustc option surface without changing the Cargo package graph, compiler binary, codegen backend, or linker. Treating individual LLVM flags as ordinary tuning would require OriginWeave to mirror a compiler-version-specific LLVM command-line grammar and continuously distinguish harmless tuning from options that alter execution, consume files, or change target/code-generation behavior. That is not a stable Browser Session contract.

## Decision

Repository-owned `llvm-args` is fail closed wherever the existing Cargo compiler-authority contract already accepts Rust flags:

- `[build].rustflags` and `[target.*].rustflags`;
- profile `rustflags` in the root manifest or Cargo config;
- `[build].rustdocflags` and `[target.*].rustdocflags`;
- rustdoc `--doctest-build-arg` forwarding into rustc.

The shared `_flags_select_codegen_backend()` classifier recognizes split `-C llvm-args=...`, compact `-Cllvm-args=...`, split `--codegen llvm-args=...`, and `--codegen=llvm-args=...`. Direct rustdoc flag paths now invoke that same classifier rather than maintaining a rustdoc-specific LLVM list.

Typed, modeled codegen options such as `-C opt-level=2` remain allowed. This is deliberately not a blanket ban on code-generation tuning.

## Alternatives considered

An LLVM-argument allowlist was rejected for now. LLVM options are toolchain-version-specific, include hidden/internal options, and are explicitly outside rustc's normal CLI stability contract. A path- or prefix-based heuristic would therefore create a false sense of provenance coverage.

Blocking only a known dynamic-plugin spelling was also rejected. The security boundary is the direct LLVM option tunnel itself, not one currently observed spelling. If a buyer requires a specific LLVM option later, the correct path is an explicit versioned contract tied to the exact rustc/LLVM toolchain and immutable build evidence.

## RED -> repair

- RED: `ef3455a922d9467e8dcd2253dbb820b9dcde7303` adds build, target, profile, direct-rustdoc, and doctest-forwarding hostile cases plus a typed-codegen control.
- Repair: `883f63a4a7706af6d70d93ac6e5fcfb632a7f7b1` extends the existing shared code-generation classifier and wires direct rustdoc flags through that same owner.

The repair does not create another Cargo topology scanner. `tests/test_browser_session_trusted_adapter_boundary.py` remains the production Cargo package/source-topology owner, while `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the repository-selected compiler/rustdoc/toolchain execution and input-authority owner.

## Residual boundary

This source contract does not prove the absence of ambient LLVM arguments supplied outside reviewed Git content. Environment/direct CLI arguments, ancestor or `$CARGO_HOME` configuration, runner images, rustup/toolchain contents, and externally materialized compiler artifacts remain CI/release supply-chain evidence surfaces. A future approved direct LLVM option requires an exact toolchain identity, documented option semantics for that compiler build, reproducibility evidence, SBOM/provenance linkage where applicable, and rollback criteria.

## References

Rust Project. (2026). *Codegen options: llvm-args*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/#llvm-args

Rust Project. (2026). *rustc_codegen_llvm::llvm_util source*. Nightly rustc documentation. https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_codegen_llvm/llvm_util.rs.html

LLVM Project. (2026). *Using the new pass manager*. https://llvm.org/docs/NewPassManager.html
