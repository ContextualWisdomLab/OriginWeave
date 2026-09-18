# Browser Session GNU ld default library search-path authority

## Problem

OriginWeave treats repository-owned Cargo compiler/linker configuration as reviewed Browser Session build provenance. GNU `ld` accepts `-Y path` to add `path` to its default library search path. GNU `ld` also permits a single-letter option operand to be attached directly to the option letter, so `-Ytools/shadow-libs` is a valid spelling of the same authority change.

Before this repair, the shared direct-linker classifier covered explicit `-L` / `--library-path`, `-rpath`, `--rpath`, `-rpath-link`, and `--rpath-link` inputs but did not classify compact `-Ypath`. Repository-owned Cargo `rustflags` or `rustdocflags` could therefore forward a compact `-Ypath` through `-Wl,`, `--for-linker=`, or doctest compiler arguments and change default library discovery outside the reviewed Cargo package/source closure.

## Owner boundary

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler, rustdoc, toolchain, linker execution, and external-input authority. This repair extends that existing classifier; it does not introduce another Cargo topology scanner.

The supplemental hostile fixture `tests/test_browser_session_linker_default_library_search_path_authority_contract.py` imports the canonical authority contract and verifies only the additional GNU `-Y` behavior.

## Evidence chain

- Structural RED: `b599ea47286d253e432e1235bf1d938f42e28919` adds hostile build/target `rustflags`, build `rustdocflags`, and rustdoc doctest-forwarding cases using compact `-Ypath`. The unrelated `-z relro` control remains allowed.
- Minimal causal repair: `d4672136b740575d72e2d9ea64b6b65f1cb09526` extends `_linker_option_selects_runtime_search_path()` so exact `-Y` and compact `-Ypath` are classified by the existing direct-linker authority path.
- Formatting-only follow-up: `43ce517e26c203418b872c3105a24cdac2c6e5c4` restores the pre-existing blank-line and end-of-file formatting changed incidentally by the contents update. It does not change policy semantics.

This chain is source-semantic evidence. It is not a substitute for exact-head hosted repository/security execution.

## Decision

Fail closed when Git-owned Cargo flags forward GNU `-Y` default-library search-path selection. Do not add a path allowlist at this layer.

A pathname alone does not establish the immutable identity of libraries that will later be discovered through that search root. A future reviewed exception therefore needs, in the same release evidence chain, the selected library artifact identities and digests, producer provenance, containment and symlink policy, exact linker/toolchain compatibility, SBOM/provenance binding, reproducibility evidence, and rollback behavior.

## Security effect

The Browser Session build contract no longer permits repository-owned Cargo configuration to change GNU `ld`'s default library lookup through the compact `-Ypath` grammar while leaving the reviewed Cargo package/source closure unchanged. Existing explicit runtime/link-time search-path controls continue to use the same shared classifier.

The repair deliberately does not prohibit unrelated direct-linker controls that do not select an external search root, such as `-z relro`.

## Residual authority

This repository contract does not claim control over environment or direct-CLI injection, ancestor or `$CARGO_HOME` configuration, externally selected linker/toolchain binaries, linker configuration outside the repository, sysroot contents, runner images, or externally restored build/cache artifacts. Those remain CI/release supply-chain provenance surfaces and must be proven by release evidence rather than silently copied into the Browser Session domain.

## Primary reference

Free Software Foundation. (2026). *The GNU linker*. GNU Binutils documentation. https://sourceware.org/binutils/docs/ld.pdf

The GNU linker documents `-Y path` as adding `path` to the default library search path for Solaris compatibility. Its command-line grammar also permits arguments to single-letter options either attached directly to the option letter or supplied as the immediately following argument.