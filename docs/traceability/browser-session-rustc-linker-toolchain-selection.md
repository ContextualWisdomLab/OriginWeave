# Browser Session rustc linker toolchain selection traceability

Status: Draft contract evidence on PR #317. This document does not claim executable repository/security GREEN.

## Problem

The existing Cargo compiler-authority contract already fails closed when Git-owned Cargo configuration selects `build.rustc`, rustc wrappers, `build.rustdoc`, target `linker`/`runner`, `-C linker=...`, and modeled driver/linker execution extensions. Rustc still exposes codegen options that can change which linker or auxiliary executable is invoked without spelling `linker=`.

The rustc codegen reference states that `link-self-contained` controls whether linking uses Rust-shipped libraries and objects and also controls which binary is used for the linker. The same reference documents `linker-features`; on `x86_64-unknown-linux-gnu`, the `lld` feature controls whether rustc tries to use an LLD linker and may select either the system linker or the self-contained `rust-lld` path. `linker-flavor` determines which linker flavor rustc uses and, when no explicit `-C linker` is supplied, determines the linker to use. The `dlltool` option accepts a path to the dlltool executable rustc invokes for `windows-gnu` raw-dylib import-library generation. These are execution-authority choices, not ordinary optimization flags.

A repository-owned `.cargo/config.toml` or `.cargo/config` can supply these options through `[build].rustflags`, matching `[target].rustflags`, and equivalent rustdoc flag surfaces. Therefore a reviewed source tree can keep the nominal Cargo target and omit `target.<...>.linker` while still changing the effective native tool selected by rustc.

Primary reference: Rust Project, *The rustc book: Codegen options*, `link-self-contained`, `linker-features`, `linker-flavor`, `dlltool`, and `linker` sections: https://doc.rust-lang.org/rustc/codegen-options/

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source override discovery.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the shared parser for Git-owned Cargo compiler/linker execution authority. This slice does not introduce a second Cargo flag scanner.
- No self-contained linker, system LLD, `rust-lld`, alternate linker flavor, custom dlltool, custom linker feature policy, or future adapter implementation is pre-authorized by this repair.
- Environment/toolchain-owned native-tool selection remains a CI/supply-chain concern. This contract covers Git-owned Cargo configuration only.
- Ordinary codegen options that do not select or alter the modeled compiler/linker/native-tool execution boundary remain allowed.

## RED

Commit `292f62f78b5f10548e7a9745f657fc732d4e8e77` adds `tests/test_browser_session_linker_toolchain_selection_contract.py` with hostile Cargo configurations for `[build].rustflags = ["-C", "link-self-contained=+linker"]` and target-scoped `rustflags = ["--codegen=linker-features=+lld"]`. The predecessor shared parser recognized direct `linker=` and flag-derived driver/linker extensions but ignored both rustc-managed linker-selection options.

Commit `750650e87c2e3c01655b14a11cc6da3e1fd33b13` extends that same hostile contract after the first repair and preserves two additional source-semantic REDs: `-C linker-flavor=ld.lld` and `--codegen=dlltool=tools/review-bypass-dlltool`. Both can alter a native executable selected by rustc without using the already-modeled direct `linker=` setting. The `debuginfo=1` control remains allowed.

## Decision and repair

Commit `7523b391b1e296a88115d9411619ac22ad2d0a42` extends the existing `_flags_select_linker` parser only. `link-self-contained` and `linker-features`, in exact or `=<value>` form after `-C` / `--codegen`, are classified as execution-authority changes and fail closed through the existing Cargo compiler-authority error path.

Commit `e9a3e85110153a667bf3dd6025bf57d08975ead6` extends the same shared parser to `linker-flavor` and `dlltool`. It does not infer a safe flavor or allowlist a repository-selected dlltool path; either surface requires a separate explicit provenance contract before it can enter the reviewed production build boundary.

The repair intentionally does not parse target-specific linker heuristics or infer which LLD/dlltool binary would be chosen. A future exception requires an explicit, versioned toolchain provenance contract that identifies the exact native-tool artifact, selection semantics, integrity evidence, and rollback behavior on the same reviewed tree.

## Security effect

Git-owned Cargo configuration can no longer switch from the reviewed nominal linker path to rustc-managed self-contained/system LLD selection, another inferred linker flavor, or a repository-selected dlltool through these codegen options without an explicit Browser Session provenance contract. The boundary remains deterministic and fail closed while unrelated codegen options stay available.

## Residual surfaces

This repair does not claim full linker-input provenance. Positional native inputs and implicit linker scripts, library/search-path selection, non-GNU control-file mechanisms, command-line `cargo rustc` flags, environment variables, rustup/toolchain composition, and target-specific external toolchain scripts remain separate review surfaces.
