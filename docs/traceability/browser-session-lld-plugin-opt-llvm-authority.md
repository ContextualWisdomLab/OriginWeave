# Browser Session LLD `plugin-opt=-` LLVM option authority

Status: source-semantic repair; hosted executable evidence pending

## Problem

OriginWeave's Browser Session Cargo compiler authority already fails closed for LLD `-mllvm` / `--mllvm`, but LLVM LLD exposes a second compatibility spelling that reaches the same LLVM option parser. A Git-owned Cargo `rustflags` or `rustdocflags` value can forward `-plugin-opt=-<llvm-option>` or `--plugin-opt=-<llvm-option>` through the compiler driver. Before this repair, those joined tokens were not classified as LLVM-option authority and could pass the reviewed direct-linker boundary.

This is an execution/provenance boundary, not a claim that every LLVM option is independently dangerous. The problem is that a reviewed Cargo configuration could open the opaque LLVM option namespace without the same explicit review applied to `-mllvm`.

## Primary evidence

Evidence is pinned to `llvm/llvm-project@3ff9abe8930acc7b4c4e2387c1357ca6f2d15c00`.

- `lld/ELF/Options.td` defines `plugin_opt_eq_minus` as `J<"plugin-opt=-">` and describes it as `Specify an LLVM option for compatibility with LLVMgold.so`.
- LLD's `J` grammar accepts both one-dash and two-dash multi-letter spellings.
- `lld/ELF/Driver.cpp` iterates `OPT_plugin_opt_eq_minus` and calls `parseClangOption(ctx, std::string("-") + arg->getValue(), arg->getSpelling())`.
- The adjacent `-mllvm` path also calls `parseClangOption`, so the two surfaces share the same underlying LLVM option-processing authority even though their command-line spellings differ.

The generic `plugin-opt=<value>` compatibility path is not blanket-blocked by this decision. LLD explicitly ignores a GCC `lto-wrapper` path and errors on other unsupported generic values. The repair therefore targets only the documented `plugin-opt=-` LLVM-option tunnel rather than treating every `plugin-opt` spelling as equivalent.

## RED and causal repair

Structural RED: `a36e22d71b73877c243fa6ecdb34806b00d1f1f5`.

The existing `tests/test_browser_session_lld_mllvm_authority_contract.py` now covers:

- build-level `rustflags` with `--plugin-opt=-...`;
- target-level `rustflags` with the one-dash `-plugin-opt=-...` spelling;
- build-level `rustdocflags`;
- rustdoc doctest compiler forwarding;
- an ordinary typed linker control (`-Wl,-z,relro`) as the allowed control.

Minimal repair: `64627f969f9c04c52ebed4efd3f7ca05b5cb42ea`.

The canonical single writer remains `tests/test_browser_session_cargo_compiler_authority_contract.py`. `_linker_option_forwards_llvm_options()` now recognizes only the additional `--plugin-opt=-` and `-plugin-opt=-` prefixes. The RED-to-repair compare changes that owner file by one replacement line (`+1/-1`); no second Cargo/linker scanner, pathname allowlist, provider-specific policy, or new topology discovery was introduced.

## Decision

Repository-owned Cargo configuration must fail closed when Rust/rustdoc linker forwarding opens LLD's opaque LLVM option-processing namespace through either `mllvm` or `plugin-opt=-`.

The deterministic Browser Session policy boundary is not delegated to LLVM option behavior. If a future buyer requirement needs a specific LLVM option, it must be modeled as a typed, reviewed contract with its security, reproducibility, toolchain-version, target/architecture, and rollback consequences stated explicitly. Reopening the generic opaque tunnel is not an acceptable shortcut.

## Rejected alternatives

- **Block every `plugin-opt=` spelling.** Rejected because current LLD has typed compatibility spellings and a generic GCC `lto-wrapper` compatibility path with different semantics. A blanket ban would conflate unrelated grammar with LLVM-option authority.
- **Allowlist LLVM option strings.** Rejected because an option name alone does not prove semantics across LLVM revisions, targets, code-generation pipelines, or security/reproducibility consequences.
- **Add a supplemental scanner.** Rejected because the existing Cargo compiler authority contract is the single writer for repository-selected compiler/rustdoc/linker execution and input authority.

## Residual authority and release evidence

This repair covers Git-owned Cargo `rustflags` / `rustdocflags` paths already consumed by the canonical contract, including doctest compiler forwarding. Environment `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `RUSTDOCFLAGS`, `CARGO_ENCODED_RUSTDOCFLAGS`, direct Cargo/rustc/rustdoc CLI arguments, ambient toolchain configuration, and unmodeled non-LLD linker grammars remain CI/release execution-provenance surfaces.

The current PR head still requires fresh hosted repository/security execution and whole-current-head review after this source/doc generation. Source-semantic repair and static primary-source traceability do not substitute for protected-head GREEN, reproducibility evidence, SBOM/provenance, or release acceptance.
