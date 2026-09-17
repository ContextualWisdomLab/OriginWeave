# Browser Session Cargo compiler authority traceability

Status: Draft contract evidence on PR #317. This document does not claim executable repository/security GREEN.

## Problem

OriginWeave already fails closed on production `build.rs`, Cargo build dependencies, dependency-source overrides, repository-external source paths, and unmodeled Rust source indirection. The remaining Git-owned Cargo configuration surface was executable selection around Rust compilation, documentation, linking, and test/run execution.

Cargo permits repository configuration to replace `rustc` with `build.rustc`, execute a program in front of `rustc` with `build.rustc-wrapper`, add a workspace-only wrapper with `build.rustc-workspace-wrapper`, and replace the documentation generator executable with `build.rustdoc`. Cargo target configuration also permits a matching target table to choose `linker`, the executable used for linking, and `runner`, the wrapper used for `cargo run`, `cargo test`, and `cargo bench`. In addition, `build.rustflags` and matching target `rustflags` are passed to `rustc`, while `build.rustdocflags` and matching target `rustdocflags` are passed to `rustdoc`. Both rustc and rustdoc accept `-C` codegen options; rustc documents `-C linker=<path>` as selecting which linker executable it invokes, and rustdoc documents that its `-C` arguments are the same codegen arguments passed through to rustc when documentation or documentation tests compile Rust code. If any of these settings enters a reviewed Browser Session production workspace without a separate provenance contract, the effective build/test/documentation execution path is no longer represented by the existing exact-tree source closure.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source override discovery.
- This contract does not authorize a future wrapper, custom compiler, custom rustdoc executable, linker, runner, generated source path, or adapter implementation.
- Environment-owned `RUSTC`/`RUSTC_WRAPPER`/`RUSTDOC`/`RUSTFLAGS`/`RUSTDOCFLAGS` authority belongs to the CI/runtime owner. This repository contract covers only Git-owned `.cargo/config.toml` and `.cargo/config` files.
- Ordinary Cargo settings, rustflags, and rustdocflags that do not select one of the modeled executables are not rejected merely because they occur under `[build]` or `[target]`.
- Target selection, command-line `cargo rustc`/`cargo rustdoc` flags, environment/toolchain configuration, and arbitrary linker arguments remain separate review surfaces.

## RED

Commit `ba9c9299d9a81d26ec68ed1c39a9dc11b08ddcb6` added a hostile repository fixture with `build.rustc-wrapper`. The predecessor source/config boundary rejected Cargo source overrides and build scripts but did not classify compiler-wrapper execution, so the commit preserves a structural RED.

Commit `a39caf95a38862f4cb4bcb68115b5e385bd6c26e` added a hostile `build.rustdoc` fixture. The predecessor compiler-authority contract accepted that setting because its execution-key set covered only `rustc` and the two rustc wrapper keys.

Commit `883ad62125af37e9afb2551803267284e21b7ea3` added target-specific hostile fixtures using `target.<triple>.linker` and `target.<cfg>.runner`. The predecessor contract inspected only `[build]` and therefore accepted both target entries.

Commit `8b9ab4c34033b9e5990caad79169351e3d771039` added hostile Cargo-owned rustflags that selected `rustc -C linker=<path>`. The predecessor rejected direct target `linker`/`runner` keys but ignored flag-derived linker selection.

Commit `b0489000709243b27a9c849d2cada8e5fadd254e` adds the corresponding rustdoc path:

```toml
[build]
rustdocflags = ["-C", "linker=tools/review-bypass-linker"]

[target.'cfg(unix)']
rustdocflags = "-Clinker=tools/review-bypass-linker"
```

The predecessor helper inspected only `rustflags`. Cargo documents both forms as flags passed to rustdoc, and rustdoc documents `-C` as rustc codegen options used when it compiles documentation or documentation tests. The fixtures therefore preserve a distinct structural RED for repository-owned rustdoc flag-derived linker authority; no hosted-run result is inferred from this Draft branch.

## Decision and repair

Commit `80aaa562672211582d5b2de69edc79a984559fb3` introduced a separate execution-authority contract that consumes the canonical trusted-adapter production topology contract and then inspects Git-owned Cargo configuration.

Commit `345759105a0f0d2e88142df9961ea724b1055734` extended it to `build.rustdoc`; commit `e7390cdb12c483570940411c857554540f754763` added matching target `linker` and `runner` rejection.

Commit `743a5321bb72d83541b34f12ce83628f95f581f6` closes Cargo-owned rustflags that select a linker without banning unrelated compiler flags. It recognizes both documented Cargo representations, a space-separated string or an array of argument strings, and both split `-C`, `linker=<path>` and compact `-Clinker=<path>` spellings.

Commit `e157b9c5f33467425df794f42e704503de697232` reuses the same narrow flag parser for Cargo-owned `rustdocflags`. It now applies the linker-selection rule to `build.rustdocflags` and matching target-table `rustdocflags` while retaining unrelated rustdoc flags such as `--document-private-items` and `--cfg docsrs`.

The modeled fail-closed execution-authority surfaces are now:

- `build.rustc`
- `build.rustc-wrapper`
- `build.rustc-workspace-wrapper`
- `build.rustdoc`
- `target.<triple-or-cfg>.linker`
- `target.<triple-or-cfg>.runner`
- `build.rustflags` and matching target `rustflags` when they select `-C linker=<path>`
- `build.rustdocflags` and matching target `rustdocflags` when they select `-C linker=<path>`

A separate contract was chosen instead of expanding Cargo package/source discovery because executable selection is not package topology. A blanket rustflags/rustdocflags ban was rejected because non-executable-selection flags are common and do not by themselves justify widening this Browser Session provenance boundary. Allowlisting executable paths was also rejected because a path alone does not establish immutable executable identity, behavior, arguments, or provenance.

## Security effect and residual risk

The repair closes modeled Git-owned executable-selection gaps that could otherwise place an unmodeled executable between Cargo and `rustc`, replace rustdoc, choose the linker that emits repository artifacts, interpose a runner around repository test/run binaries, or select a linker indirectly through Cargo-owned rustc/rustdoc flags while the reviewed Rust source closure remained unchanged.

It does not prove that CI environment variables, toolchain installation, runner images, external binaries, command-line `cargo rustc`/`cargo rustdoc` flags, or arbitrary linker arguments are trustworthy; those controls remain with their canonical CI/supply-chain owners or future focused contracts. Any future claim of complete compiler/linker provenance must account for those surfaces with realistic hostile cases rather than a catch-all Cargo-config ban.

## Acceptance and follow-up

1. Obtain independent current-head review of the rustdocflags RED→repair chain and the retained direct executable-selection contracts.
2. After #229 exact-head required evidence becomes terminal, reconcile #317 by ordinary non-force ancestry while preserving the parent and child deltas.
3. Regenerate executable repository/security evidence on the reconciled exact head.
4. Review command-line Cargo flags and linker-argument surfaces separately; add a contract only when a realistic executable/provenance escape is demonstrated.
5. If any blocked executable override is ever required, replace the fail-closed rule only with an explicit design covering immutable executable identity, arguments/source-input provenance, SBOM/attestation, rollback, and buyer-visible evidence.

## References

The Cargo Project. (n.d.). *Configuration*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/config.html

The Cargo Project. (n.d.). *cargo rustdoc*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/commands/cargo-rustdoc.html

The Rust Project Developers. (n.d.). *Command-line arguments*. *The rustdoc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustdoc/command-line-arguments.html

The Rust Project Developers. (n.d.). *Codegen options*. *The rustc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/codegen-options/index.html
