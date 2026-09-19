# Browser Session LLD pass-plugin execution authority

Status: source-semantic repair; hosted executable evidence pending

## Problem

OriginWeave's Browser Session Cargo compiler authority already rejects GNU linker plugin loading through `-plugin` / `--plugin`, but LLVM LLD exposes a separate LTO pass-plugin surface. `--load-pass-plugin=<file>` selects a dynamic pass-plugin library that is carried into the LTO configuration. Before this repair, the joined spelling kept the library path inside a single option token, so the positional-native-input fallback did not see it and the existing GNU-plugin classifier did not match it.

This is executable-code authority. A repository-owned Cargo `rustflags` or `rustdocflags` value must not be able to load an unreviewed LLVM pass plugin into link-time optimization while the reviewed Cargo package/source closure remains unchanged.

## Primary evidence

Evidence is pinned to `llvm/llvm-project@3834f58744a7be80b5869c07fe576ff8f23e2315`.

- `lld/ELF/Options.td` defines `load_pass_plugins` as `EEq<"load-pass-plugin", "Load passes from plugin library">`, so the supported ELF spelling is the double-dash option with separated or `=`-joined operand.
- `lld/ELF/Driver.cpp` stores `args::getStrings(args, OPT_load_pass_plugins)` in `ctx.arg.passPlugins`.
- `lld/ELF/LTO.cpp` copies each `ctx.arg.passPlugins` filename into `LTO::Config::PassPluginFilenames`, making the selected library part of the LTO pass-plugin execution surface rather than a passive output setting.

## RED and causal repair

Structural RED: `b445b22c698a5418398d72947f6a0ca9b65eba38`.

The existing `tests/test_browser_session_linker_plugin_contract.py` adds a hostile Cargo forwarding case for `-Wl,--load-pass-plugin=tools/review-bypass-pass.so`. Existing `-Wl,--as-needed`, `-Xlinker --as-needed`, and `--for-linker=--as-needed` controls remain allowed. The RED commit changes only that focused contract (`+5/-0`).

Minimal repair: `3d0efa7b131e761174ddd7bc4b0bcc10b2d59e31`.

The canonical single writer remains `tests/test_browser_session_cargo_compiler_authority_contract.py`. `LINKER_PLUGIN_OPTIONS` now includes the separated `--load-pass-plugin` spelling, and `_linker_option_loads_plugin()` recognizes the joined `--load-pass-plugin=` spelling. RED-to-repair changes only that authority file (`+2/-2`). No supplemental Cargo/linker scanner, filename allowlist, or LTO-specific source-discovery path was added.

## Decision

Repository-owned Cargo/rustdoc linker forwarding must fail closed whenever LLD is instructed to load a pass-plugin library. The boundary treats pass-plugin selection as executable build authority, alongside linker plugin loading, tool replacement, wrapper selection, and other mechanisms that can execute code outside the reviewed build TCB.

A future buyer requirement for an LTO pass plugin must use a typed, versioned contract that proves the exact plugin artifact and its execution context. A path string or library basename is not sufficient provenance.

## Rejected alternatives

- **Rely on positional-input detection.** Rejected because the joined `--load-pass-plugin=<file>` spelling embeds the path inside the option token.
- **Allowlist plugin paths or names.** Rejected because a pathname does not prove immutable bytes, producer identity, toolchain/plugin ABI compatibility, symlink containment, or reproducibility.
- **Treat the option as ordinary linker tuning.** Rejected because LLD explicitly carries the selected filename into `PassPluginFilenames` for LTO execution.
- **Add a second pass-plugin scanner.** Rejected because the existing Cargo compiler authority contract is the canonical single writer for repository-selected compiler/rustdoc/linker execution authority.

## Acceptance and residual authority

If pass-plugin execution is ever admitted, release evidence must bind at minimum the plugin digest, producer provenance, LLVM/LLD version and plugin ABI compatibility, target/architecture, containment of the resolved artifact, SBOM/provenance, deterministic or independently reproduced output evidence, invalidation conditions, and rollback procedure.

This source-semantic repair does not establish hosted GREEN. The exact PR head still requires fresh repository/security execution, current-head independent review, and the configured coverage/rustdoc/docstring gates. Environment and direct-CLI linker arguments, ambient toolchain installation, CI-restored artifacts, and unmodeled non-LLD plugin mechanisms remain CI/release provenance surfaces rather than Browser Session domain truth.
