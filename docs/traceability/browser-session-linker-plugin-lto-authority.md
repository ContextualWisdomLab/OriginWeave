# Browser Session rustc linker-plugin-LTO authority traceability

Status: Draft contract evidence on PR #317. This document records a source-semantic RED→repair chain and does not claim hosted repository/security GREEN.

## Problem

The Browser Session Cargo compiler-authority contract already fails closed on linker plugins selected through linker arguments such as GNU `-plugin` / `--plugin`. That does not cover rustc's own `linker-plugin-lto` codegen option.

Rust documents `-C linker-plugin-lto` as the control that defers LTO to the native link step. Its documented values are the usual boolean forms or a **path to the linker plugin**. The rustc book gives an explicit example using `-Clinker-plugin-lto="/path/to/LLVMgold.so"`. A Git-owned Cargo `rustflags` or `rustdocflags` entry can therefore name a plugin artifact without using the already-modeled linker-argument plugin surface.

Primary references:

- Rust project. (2026). *Codegen options: linker-plugin-lto*. https://doc.rust-lang.org/rustc/codegen-options/#linker-plugin-lto
- Rust project. (2026). *Linker-plugin-based LTO*. https://doc.rust-lang.org/rustc/linker-plugin-lto.html

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the owner for Git-owned Cargo-selected Rust/linker execution and external-input authority.
- Boolean `linker-plugin-lto` enable/disable values do not themselves name a repository-selected plugin artifact and remain permitted by this slice.
- An explicit plugin path is not accepted merely because it is repository-relative. Allowing one requires immutable artifact identity, containment, toolchain compatibility, SBOM/provenance, and exact-head executable evidence in the same reviewed delta.
- Environment `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS`, direct CLI flags, runner/toolchain composition, and plugins selected outside Git-owned Cargo configuration remain CI/runtime or future modeled surfaces.

## RED

Commit `f25634aaee2486ae043c410f1589d3d60f511485` adds `tests/test_browser_session_linker_plugin_lto_contract.py`. The hostile fixture uses:

```toml
[build]
rustflags = ["-C", "linker-plugin-lto=tools/review-bypass-llvmgold.so"]
```

The predecessor exact `569bfc807df8e4f3f452f04e5dfb865d09086fac` parsed the `-C` option but did not classify `linker-plugin-lto=<path>`, so the canonical Cargo compiler-authority helper did not fail closed. The same fixture keeps `linker-plugin-lto=no` as a control so the repair is not a blanket LTO ban.

## Decision and repair

Commit `c368afacacf261a84126150d9a06e1271a479246` extends the existing codegen-option classifier instead of adding another Cargo scanner. `_codegen_option_selects_linker_plugin()` distinguishes the documented boolean values (`y`, `yes`, `on`, `true`, `n`, `no`, `off`, `false`) from an explicit value that names a plugin artifact. Path-valued or otherwise unrecognized `linker-plugin-lto=<value>` settings fail closed through the existing `rustflags:codegen linker` / `rustdocflags:codegen linker` authority path.

The repair is deliberately conservative. Bare `linker-plugin-lto` remains allowed because it enables linker-plugin LTO without naming a repository-selected plugin path. Unknown explicit values fail closed rather than being guessed safe.

## Security effect and residual risk

This closes the Git-owned Cargo path by which a reviewed Rust source tree could name an explicit LLVM linker-plugin artifact through rustc codegen configuration while avoiding the existing linker `-plugin` checks.

It does not prove the selected system linker/LTO toolchain trustworthy, validate ambient environment flags, or authorize an explicit plugin artifact. Any future explicit plugin use must arrive with artifact provenance and executable compatibility evidence rather than widening this contract by path alone.
