# Browser Session LLD `--mllvm` authority

Status: source-semantic repair on PR #317; hosted executable evidence is still required before GREEN.

## Problem

OriginWeave already fails closed when Git-owned Cargo `rustflags` or `rustdocflags` use rustc `-C/--codegen llvm-args=...`. The same trust boundary was incomplete at the ELF linker layer: LLD defines `mllvm` as an option that forwards additional arguments directly to LLVM option processing. Repository-owned Cargo flags could therefore route opaque LLVM options through rustc linker forwarding even though the typed rustc LLVM-option path was rejected.

The relevant Browser Session boundary is provenance, not whether a particular LLVM option is currently known to be harmful. The forwarded option namespace changes with the exact LLVM/LLD build and is not a stable, reviewed OriginWeave contract.

## Constraint and owner

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler, rustdoc, linker execution, and external-input authority. Supplemental fixtures consume that classifier; they do not duplicate Cargo production-topology discovery owned by `tests/test_browser_session_trusted_adapter_boundary.py`.

This repair does not broaden into a blanket linker-option ban. Ordinary modeled linker controls remain allowed when they do not select executable code, external inputs, mutable runtime authority, or an opaque downstream option processor.

## Decision

Git-owned Cargo configuration must fail closed when rustc/rustdoc linker forwarding reaches LLD `mllvm` in either GNU-compatible spelling and in split or equals form:

- `--mllvm <arg>`
- `--mllvm=<arg>`
- `-mllvm <arg>`
- `-mllvm=<arg>`

The canonical direct-linker parser classifies those tokens through `_linker_option_forwards_llvm_options()`. Existing `-Wl,`, `--for-linker=`, `-Xlinker`, `-C link-arg=...`, `-C link-args=...`, build/target `rustflags`, build/target `rustdocflags`, and rustdoc doctest compiler forwarding continue to converge on the same authority check.

## RED → repair evidence

- Structural RED: `763bcadd7a0b7f5ef8cf7e104214ad5d8ec2b5b1`
  - build `rustflags` with `-Wl,--mllvm=...`
  - target `rustflags` with split `-Wl,-mllvm,<arg>`
  - build `rustdocflags`
  - rustdoc `--doctest-build-arg` forwarding
  - typed linker control `-Wl,-z,relro` remains allowed
- Minimal canonical repair: `af934e55c3684a108dbf8a80e1f4270f64c8cd85`
  - adds one LLD-specific classifier to the existing direct-linker authority parser
  - reuses all existing Cargo/rustdoc/linker forwarding paths
  - does not add a second topology/config scanner

The repair commit is source-semantic evidence only. It is not a substitute for PR-triggered hosted tests, repository/security checks, current-head review, or the owned 100% documentation/test/edge-case gates.

## Alternatives rejected

Allowlisting individual LLVM options was rejected. LLD forwards into LLVM's option processor, whose accepted/debug/experimental surface depends on the exact toolchain build. A local list would become a mutable shadow specification and could silently under-model future LLVM options.

Path-based approval is not applicable because `mllvm` is an option tunnel rather than a file selector. Treating only currently observed file-consuming LLVM options as dangerous would also confuse present examples with the authority granted by the forwarding mechanism itself.

## Security and commercial effect

The build provenance contract now treats direct LLD-to-LLVM option forwarding consistently with rustc `llvm-args`: reviewed Git-owned Cargo configuration cannot introduce an opaque LLVM option channel behind the typed compiler/linker authority model. This reduces the chance that an enterprise release is materially changed by toolchain-internal or experimental LLVM switches that are absent from the reviewed OriginWeave contract and evidence set.

## Residual authority

Environment or direct-CLI linker arguments, ancestor or `$CARGO_HOME` configuration, runner image/toolchain identity, exact LLD/LLVM distribution and version, and externally supplied build inputs remain CI/release supply-chain evidence surfaces. Non-LLD linkers and option grammars remain governed only where they are explicitly modeled and evidenced.

## References

LLVM Project. (2026). *LLD - The LLVM Linker* (24.0.0git documentation). https://lld.llvm.org/

LLVM Project. (2026). *lld/ELF/Options.td* [Source code]. GitHub. https://github.com/llvm/llvm-project/blob/main/lld/ELF/Options.td

LLVM Project. (2026). *Clang command line argument reference*. https://clang.llvm.org/docs/ClangCommandLineReference.html

Retrieved September 19, 2026. The primary LLD option table defines `mllvm` as forwarding additional arguments to LLVM option processing; publication/docs freshness does not replace OriginWeave runtime qualification of the exact linker/toolchain used for a release.
