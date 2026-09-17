# Browser Session GCC linker-driver wrapper authority

Status: Draft source-level traceability for PR #317. This document does not claim hosted exact-head repository/security GREEN.

## Problem

The Browser Session Cargo execution-authority contract already fails closed on direct linker selection, driver program-search replacement, opaque driver response files, modeled linker plugin loading, and GCC specs files. GCC has a separate driver-level execution extension: `-wrapper` runs every GCC subcommand under a caller-selected wrapper program.

Rust documents `-C link-arg` and `-C link-args` as arguments appended to the linker invocation. On the GNU-compatible Unix path used by the current contract, that invocation may be the GCC driver. A Git-owned Cargo configuration can therefore pass `-wrapper tools/review-wrapper,--args` through rustc/rustdoc linker flags while keeping the nominal compiler driver, reviewed Rust source closure, and visible Cargo `linker` setting unchanged. The wrapper becomes part of the build/link execution TCB.

GCC's current official documentation states that `-wrapper` invokes all subcommands under the named wrapper program. A Debian GCC 14.2.0 `gcc -###` reproduction with `-wrapper /bin/echo,--` showed the link subprocess rendered as `/bin/echo -- .../collect2 ...`, confirming that the option reaches the link-stage subcommand path. This runtime realism probe is supplemental evidence only; it is not OriginWeave exact-head CI.

## RED

Commit `2ae24ca196e54c59068ce0ff243bbbef2dcc0d83` adds hostile repository fixtures for both Cargo encodings used by the existing shared parser:

- repeated `-C link-arg=-wrapper` plus `-C link-arg=tools/review-wrapper,--args`;
- one `--codegen=link-args=-wrapper tools/review-wrapper,--args` value.

The predecessor classifier accepted exact `-wrapper` because it only recognized `@file`, GCC `-specs`, `-fuse-ld=`, and driver `-B` as driver execution-authority surfaces. The fixtures consume the canonical Browser Session Cargo compiler-authority contract rather than duplicating Cargo topology discovery.

## Decision and repair

Commit `9aaa60019e2b8e150bd6e2f6d5f475442786825e` changes only the existing `_linker_driver_argument_selects_executable` classifier. Exact GCC `-wrapper` now fails closed before the subsequent wrapper-program argument can extend link execution authority. Existing handling for response files, specs, linker selection, program-search prefixes, linker plugins, and ordinary non-execution linker arguments is unchanged.

The contract intentionally rejects the option token itself even when malformed or missing its operand. Allowing a repository-owned `-wrapper` requires a separate immutable wrapper identity/provenance contract on the same reviewed tree; documenting a path string is not sufficient.

## Boundary and residual risk

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the shared Git-owned Cargo execution-authority classifier.
- This repair does not authorize a wrapper executable or expand the future BiDi adapter allowlist.
- Environment-owned tool variables and image/toolchain selection remain CI/supply-chain owner concerns.
- Non-GNU driver mechanisms, linker scripts, other arbitrary linker arguments, and external toolchain provenance remain separate review surfaces.

## Primary references

Free Software Foundation. (2026). *Using the GNU Compiler Collection (GCC): Overall options*. https://gcc.gnu.org/onlinedocs/gcc/Overall-Options.html

Rust Project Developers. (2026). *The rustc book: Codegen options*. https://doc.rust-lang.org/rustc/codegen-options/index.html
