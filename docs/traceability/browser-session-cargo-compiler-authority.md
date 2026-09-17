# Browser Session Cargo compiler authority traceability

Status: Draft contract evidence on PR #317. This document does not claim executable repository/security GREEN.

## Problem

OriginWeave already fails closed on production `build.rs`, Cargo build dependencies, dependency-source overrides, repository-external source paths, and unmodeled Rust source indirection. The remaining Git-owned Cargo configuration surface is executable selection around Rust compilation, documentation, linking, and test/run execution.

Cargo permits repository configuration to replace `rustc` with `build.rustc`, execute a program in front of `rustc` with `build.rustc-wrapper`, add a workspace-only wrapper with `build.rustc-workspace-wrapper`, and replace the documentation generator executable with `build.rustdoc`. Cargo target configuration also permits a matching target table to choose `linker`, the executable used for linking, and `runner`, the wrapper used for `cargo run`, `cargo test`, and `cargo bench`. In addition, `build.rustflags` and matching target `rustflags` are passed to `rustc`, while `build.rustdocflags` and matching target `rustdocflags` are passed to `rustdoc`.

Rust documents `-C` and `--codegen` as equivalent short and long codegen-option interfaces. The `linker` codegen option directly selects which linker executable rustc invokes. Rust also documents `link-arg` and `link-args` as arguments appended to the linker invocation. On Unix-like targets where a C compiler is the linker driver, Rust explicitly documents that `-Clink-arg=-fuse-ld=$value` is passed to the driver after rustc's own linker-feature arguments and therefore generally takes priority when the driver chooses the actual linker. A repository-owned Cargo flag can therefore re-select the linker behind the nominal compiler driver without using `target.<triple>.linker` or `-C linker=<path>`.

If any of these settings enters a reviewed Browser Session production workspace without a separate provenance contract, the effective build/test/documentation execution path is no longer represented by the existing exact-tree source closure.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source override discovery.
- This contract does not authorize a future wrapper, custom compiler, custom rustdoc executable, linker, runner, generated source path, or adapter implementation.
- Environment-owned `RUSTC`/`RUSTC_WRAPPER`/`RUSTDOC`/`RUSTFLAGS`/`RUSTDOCFLAGS` authority belongs to the CI/runtime owner. This repository contract covers only Git-owned `.cargo/config.toml` and `.cargo/config` files.
- Ordinary Cargo settings, rustflags, rustdocflags, and linker arguments that do not select one of the modeled executables are not rejected merely because they occur under `[build]` or `[target]`.
- Target selection, command-line `cargo rustc`/`cargo rustdoc` flags, environment/toolchain configuration, linker plugins, linker search-path manipulation, and other arbitrary linker-argument effects remain separate review surfaces.

## RED

Commit `ba9c9299d9a81d26ec68ed1c39a9dc11b08ddcb6` added a hostile repository fixture with `build.rustc-wrapper`. The predecessor source/config boundary rejected Cargo source overrides and build scripts but did not classify compiler-wrapper execution, so the commit preserves a structural RED.

Commit `a39caf95a38862f4cb4bcb68115b5e385bd6c26e` added a hostile `build.rustdoc` fixture. The predecessor compiler-authority contract accepted that setting because its execution-key set covered only `rustc` and the two rustc wrapper keys.

Commit `883ad62125af37e9afb2551803267284e21b7ea3` added target-specific hostile fixtures using `target.<triple>.linker` and `target.<cfg>.runner`. The predecessor contract inspected only `[build]` and therefore accepted both target entries.

Commit `8b9ab4c34033b9e5990caad79169351e3d771039` added hostile Cargo-owned rustflags that selected `rustc -C linker=<path>`. The predecessor rejected direct target `linker`/`runner` keys but ignored flag-derived linker selection.

Commit `b0489000709243b27a9c849d2cada8e5fadd254e` added the corresponding Cargo-owned rustdocflags path using split and compact `-C linker=<path>` forms. The predecessor helper inspected only rustflags, so both fixtures preserved a distinct structural RED for repository-selected rustdoc linker authority.

Focused review of exact `a3434ebf7d8169a6cf5276d983d4a0c978a9720a` then found that the shared flag parser recognized only `-C` spellings even though Rust also documents the long `--codegen` interface. Commit `053b6cacd0d7d36dc619165aada99c1ac485b043` preserves that review finding as structural RED across both rustc and rustdoc flag surfaces:

```toml
[build]
rustflags = ["--codegen", "linker=tools/review-bypass-linker"]
rustdocflags = ["--codegen", "linker=tools/review-bypass-linker"]

[target.'cfg(unix)']
rustflags = "--codegen=linker=tools/review-bypass-linker"
rustdocflags = "--codegen=linker=tools/review-bypass-linker"
```

The predecessor parser accepted all four long-form cases.

Commit `0b457b3936ef3fdd0546de06f279fc5cb2756e13` preserves a distinct linker-driver RED. It adds Cargo-owned rustflags using both split `-C link-arg=-fuse-ld=review-bypass-linker` and compact `--codegen=link-args=-fuse-ld=review-bypass-linker`. The predecessor parser only recognized the `linker=` codegen option and therefore allowed a repository flag to change the actual linker selected by a C compiler driver while the reviewed Rust source closure and nominal driver remained unchanged.

These are source-level RED fixtures; no hosted-run result is inferred from the Draft branch.

## Decision and repair

Commit `80aaa562672211582d5b2de69edc79a984559fb3` introduced a separate execution-authority contract that consumes the canonical trusted-adapter production topology contract and then inspects Git-owned Cargo configuration.

Commit `345759105a0f0d2e88142df9961ea724b1055734` extended it to `build.rustdoc`; commit `e7390cdb12c483570940411c857554540f754763` added matching target `linker` and `runner` rejection.

Commit `743a5321bb72d83541b34f12ce83628f95f581f6` closed Cargo-owned rustflags that select a linker without banning unrelated compiler flags. Commit `e157b9c5f33467425df794f42e704503de697232` reused the same narrow parser for Cargo-owned rustdocflags while retaining unrelated rustdoc flags such as `--document-private-items` and `--cfg docsrs`.

Commit `7ee4b253ff4e213e949468274d126db214a63e4a` closed the review-discovered long-form bypass in that one shared parser. It treats split `-C` / `--codegen` and compact `-C...` / `--codegen=...` as equivalent codegen-option interfaces.

Commit `d9e7d8ab4047bb25d3c6db0e195ec06240495aa9` extends that same parser rather than adding a second flag scanner. It classifies `linker=<path>` as direct linker selection and classifies `link-arg=` / `link-args=` only when their payload contains `-fuse-ld=`, the driver-level linker-selection mechanism documented by rustc. A control fixture retains ordinary `-C link-arg=-Wl,--as-needed`, avoiding a blanket ban on unrelated linker arguments.

The modeled fail-closed execution-authority surfaces are now:

- `build.rustc`
- `build.rustc-wrapper`
- `build.rustc-workspace-wrapper`
- `build.rustdoc`
- `target.<triple-or-cfg>.linker`
- `target.<triple-or-cfg>.runner`
- `build.rustflags` and matching target `rustflags` when `-C` or `--codegen` selects `linker=<path>`
- `build.rustdocflags` and matching target `rustdocflags` when `-C` or `--codegen` selects `linker=<path>`
- Cargo-owned rustc/rustdoc flag surfaces when `link-arg` or `link-args` uses `-fuse-ld=` to re-select the actual linker behind a compiler driver

A separate contract was chosen instead of expanding Cargo package/source discovery because executable selection is not package topology. A blanket rustflags/rustdocflags or linker-argument ban was rejected because non-executable-selection flags are common and do not by themselves justify widening this Browser Session provenance boundary. Allowlisting executable paths was also rejected because a path alone does not establish immutable executable identity, behavior, arguments, or provenance.

## Security effect and residual risk

The repair closes modeled Git-owned executable-selection gaps that could otherwise place an unmodeled executable between Cargo and `rustc`, replace rustdoc, choose the linker that emits repository artifacts, interpose a runner around repository test/run binaries, select a linker directly through Cargo-owned rustc/rustdoc flags, or re-select the actual linker behind a nominal C compiler driver with `-fuse-ld=` while the reviewed Rust source closure remained unchanged.

It does not prove that CI environment variables, toolchain installation, runner images, external binaries, command-line `cargo rustc`/`cargo rustdoc` flags, linker plugins, search-path overrides such as driver-specific `-B`, or arbitrary linker arguments are trustworthy. Those controls remain with their canonical CI/supply-chain owners or future focused contracts. Any future claim of complete compiler/linker provenance must account for those surfaces with realistic hostile cases rather than a catch-all Cargo-config ban.

## Acceptance and follow-up

1. Obtain independent current-head review of the `-fuse-ld=` RED→repair chain and retained executable-selection contracts.
2. After #229 exact-head required evidence becomes terminal, reconcile #317 by ordinary non-force ancestry while preserving the parent and child deltas.
3. Regenerate executable repository/security evidence on the reconciled exact head.
4. Review linker plugin and search-path manipulation separately; add a contract only when a realistic executable/provenance escape is demonstrated.
5. If any blocked executable override is ever required, replace the fail-closed rule only with an explicit design covering immutable executable identity, arguments/source-input provenance, SBOM/attestation, rollback, and buyer-visible evidence.

## References

The Cargo Project. (n.d.). *Configuration*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/config.html

The Cargo Project. (n.d.). *cargo rustdoc*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/commands/cargo-rustdoc.html

The Rust Project Developers. (n.d.). *Command-line arguments*. *The rustc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/command-line-arguments.html

The Rust Project Developers. (n.d.). *Command-line arguments*. *The rustdoc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustdoc/command-line-arguments.html

The Rust Project Developers. (n.d.). *Codegen options*. *The rustc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/codegen-options/
