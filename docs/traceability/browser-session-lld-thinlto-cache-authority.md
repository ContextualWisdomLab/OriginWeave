# Browser Session LLD ThinLTO cache authority

## Problem

Repository-owned Rust/rustdoc linker forwarding could select an LLD ThinLTO cache with `--thinlto-cache-dir=<path>` without entering the Browser Session compiler/linker authority boundary. The joined option keeps the directory path inside one linker token, so the existing positional-native-input fallback did not see it.

This is an input-provenance problem, not merely a performance/cache setting. LLVM LLD configures its ThinLTO `FileCache` from `thinLTOCacheDir`. On a cache hit, LLVM's local cache opens `llvmcache-<key>` for read, maps the bytes into a `MemoryBuffer`, and passes that buffer to the linker callback. LLD then treats a non-null cached buffer as a native relocatable file and links its bytes. A mutable or externally restored cache can therefore contribute native object bytes to the final Browser Session artifact.

## Primary evidence

Evidence was checked against `llvm/llvm-project` source at revision `a312cb0c81cf1e4cf1a3bc469b15bd5216c59469`:

- `lld/ELF/Options.td` defines `thinlto-cache-dir=` as a joined LLD option whose value is the path to the ThinLTO cached-object directory.
- `lld/ELF/LTO.cpp` passes `ctx.arg.thinLTOCacheDir` to `localCache(...)`, then documents that `files[i]` may contain a native relocatable `MemoryBuffer` supplied by that cache and links the resulting `objBuf`.
- `llvm/lib/Support/Caching.cpp` implements a cache hit by opening `llvmcache-<key>` for read and handing the resulting `MemoryBuffer` to `AddBuffer`.

The cache key does not turn the cache directory into reviewed source authority: the cache hit path trusts the bytes found at the computed entry path and feeds them into the link. Repository configuration must therefore not be able to select a mutable external cache as an implicit native-object source without an explicit provenance contract.

## Ownership

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected rustc/rustdoc/toolchain/linker execution and input authority. `tests/test_browser_session_trusted_adapter_boundary.py` remains the owner of production Cargo package/source topology. No second Cargo/linker scanner was introduced.

The boundary is deliberately narrower than general CI cache policy. Organization-level cache storage, restoration, retention, attestation, and runner isolation remain CI/release supply-chain concerns. This slice only prevents Git-owned linker forwarding from selecting a mutable ThinLTO cache directory as an unreviewed native-object input source.

## RED → repair

Structural RED `a1829971952c95ab8d8899e24813e74443939873` adds `tests/test_browser_session_lld_thinlto_cache_authority_contract.py`. It requires joined `--thinlto-cache-dir=...` to fail closed through normal Rust linker forwarding and rustdoc doctest compiler forwarding, while keeping typed `-Wl,-z,relro` as an allowed control.

Minimal repair `38081627f21a1adadf22e1269b1d326c0012a5a4` adds `_linker_option_selects_thinlto_cache()` to the existing canonical direct-linker classifier and consumes it from `_direct_linker_arguments_extend_authority()`. The predecessor-to-repair compare changes only that authority file by `+6/-0`; the RED contract remains separate.

## Alternatives rejected

Treating ThinLTO cache selection as performance-only was rejected because upstream `localCache` reads existing cached object bytes and LLD explicitly consumes those buffers as native relocatable inputs.

Allowing repository-relative cache directories by pathname was rejected. Path location does not establish byte identity, producer identity, restoration provenance, symlink containment, cache poisoning resistance, toolchain compatibility, or reproducibility.

Reimplementing cache discovery in a separate test/helper was rejected because it would split compiler/linker input authority across multiple writers.

Disabling ThinLTO itself was not selected. The defect is repository-selected mutable cache authority, not ThinLTO compilation as such.

## Future exception evidence

If a buyer-specific workflow later requires ThinLTO caching, the exception must be owned by the CI/release supply-chain boundary and prove at least:

- exact cache producer/toolchain identity and cache-key inputs;
- immutable or integrity-verified cached object bytes before link consumption;
- repository/job/architecture isolation and purpose-bound access;
- symlink/path containment and restore-source provenance;
- SBOM/provenance attachment tying consumed cached objects to the released artifact;
- reproducibility against a cache-cold rebuild or an equivalent independent build;
- invalidation and rollback behavior when compiler, linker, target, flags, or source inputs change.

A cache hit, command acknowledgement, or successful link is not evidence that those properties hold.

## Residual acceptance

This source repair does not establish hosted GREEN, whole-PR review closure, protected-branch mergeability, or release readiness. Exact-head repository/security checks, current-head review, and the existing parent/central CodeQL prerequisite chain remain required before #317 can advance.
