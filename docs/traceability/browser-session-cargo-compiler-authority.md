# Browser Session Cargo compiler authority traceability

Status: Draft contract evidence on PR #317. This document does not claim executable repository/security GREEN.

## Problem

OriginWeave already fails closed on production `build.rs`, Cargo build dependencies, dependency-source overrides, repository-external source paths, and unmodeled Rust source indirection. The remaining Git-owned Cargo configuration surface was executable selection around Rust compilation, documentation, linking, and test/run execution.

Cargo permits repository configuration to replace `rustc` with `build.rustc`, execute a program in front of `rustc` with `build.rustc-wrapper`, add a workspace-only wrapper with `build.rustc-workspace-wrapper`, and replace the documentation generator executable with `build.rustdoc`. Cargo target configuration also permits a matching target table to choose `linker`, the executable used for linking, and `runner`, the wrapper used for `cargo run`, `cargo test`, and `cargo bench`. In addition, `build.rustflags` and matching `target.<triple-or-cfg>.rustflags` are passed to `rustc`; rustc's `-C linker=<path>` codegen option selects which linker executable rustc invokes. If any of these settings enters a reviewed Browser Session production workspace without a separate provenance contract, the effective build/test execution path is no longer represented by the existing exact-tree source closure.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source override discovery.
- This contract does not authorize a future wrapper, custom compiler, custom rustdoc executable, linker, runner, generated source path, or adapter implementation.
- Environment-owned `RUSTC`/`RUSTC_WRAPPER`/`RUSTDOC`/`RUSTFLAGS` authority belongs to the CI/runtime owner. This repository contract covers only Git-owned `.cargo/config.toml` and `.cargo/config` files.
- Ordinary Cargo settings and rustflags that do not select one of the modeled executables are not rejected merely because they occur under `[build]` or `[target]`.
- `rustdocflags`, target selection, command-line `cargo rustc` flags, environment/toolchain configuration, and linker arguments remain separate review surfaces. This slice models Cargo-owned rustflags only when they select rustc's linker executable.

## RED

Commit `ba9c9299d9a81d26ec68ed1c39a9dc11b08ddcb6` added a hostile repository fixture with:

```toml
[build]
rustc-wrapper = "tools/review-bypass-wrapper"
```

The fixture required the Browser Session security contract to fail closed with a Cargo compiler-execution provenance error. The predecessor source/config boundary rejected Cargo `paths`, `[patch]`, `[source]`, build scripts, and build dependencies, but it did not classify `build.rustc-wrapper`. This commit therefore preserves a structural RED; no hosted execution result is inferred from the Draft branch.

Commit `a39caf95a38862f4cb4bcb68115b5e385bd6c26e` added a hostile `build.rustdoc` fixture. The predecessor compiler-authority contract accepted that setting because its execution-key set covered only `rustc` and the two rustc wrapper keys. This preserves a distinct structural RED for repository-selected rustdoc execution.

Commit `883ad62125af37e9afb2551803267284e21b7ea3` added target-specific hostile fixtures using `target.<triple>.linker` and `target.<cfg>.runner`. The predecessor contract inspected only `[build]` and therefore accepted both target entries. This is structural RED evidence for direct Git-owned target executable selection, not hosted-run evidence.

Commit `8b9ab4c34033b9e5990caad79169351e3d771039` added two flag-derived hostile fixtures:

```toml
[build]
rustflags = ["-C", "linker=tools/review-bypass-linker"]

[target.'cfg(unix)']
rustflags = "-C linker=tools/review-bypass-linker"
```

The predecessor helper rejected direct `linker`/`runner` keys but ignored `rustflags`, so both forms could select an unmodeled linker executable while every production Rust source remained inside the reviewed repository. This commit preserves the rustflags linker-authority structural RED before the repair.

## Decision and repair

Commit `80aaa562672211582d5b2de69edc79a984559fb3` introduced a separate compiler-authority contract that first consumes the canonical trusted-adapter production topology contract and then inspects Git-owned Cargo configuration for Rust compiler execution keys.

Commit `345759105a0f0d2e88142df9961ea724b1055734` extended the same bounded contract to `build.rustdoc`, because Cargo documents it as the program path used for rustdoc execution.

Commit `e7390cdb12c483570940411c857554540f754763` extended the same config-owner contract to matching `[target]` tables and fails closed on direct `linker` and `runner` selection.

Commit `743a5321bb72d83541b34f12ce83628f95f581f6` closes the flag-derived linker path without banning unrelated compiler flags. The contract now recognizes Cargo-owned rustflags in both documented representations, a space-separated string or an array of argument strings, and fails closed when the resulting rustc argument sequence contains either `-C` followed by `linker=<path>` or compact `-Clinker=<path>`. It applies the same rule to `build.rustflags` and matching target-table `rustflags`. A regression keeps unrelated flags such as `-C opt-level=2` and `--cfg` allowed.

The modeled fail-closed execution-authority surfaces are now:

- `build.rustc`
- `build.rustc-wrapper`
- `build.rustc-workspace-wrapper`
- `build.rustdoc`
- `target.<triple-or-cfg>.linker`
- `target.<triple-or-cfg>.runner`
- `build.rustflags` when they select rustc `-C linker=<path>`
- `target.<triple-or-cfg>.rustflags` when they select rustc `-C linker=<path>`

A separate contract was chosen instead of expanding Cargo package/source discovery because executable selection is not package topology. Folding it into the topology scanner would blur single-writer responsibilities. A blanket rustflags ban was rejected because non-executable-selection flags are common and do not by themselves justify widening this Browser Session provenance boundary. Allowlisting executable paths was also rejected because a path alone does not establish immutable executable identity, behavior, arguments, or provenance.

## Security effect and residual risk

The repair closes modeled Git-owned executable-selection gaps that could otherwise place an unmodeled executable between Cargo and `rustc`, replace rustdoc, choose the linker that emits repository artifacts, interpose a runner around repository test/run binaries, or select a linker indirectly through Cargo-owned rustflags while the reviewed Rust source closure remained unchanged.

It does not prove that CI environment variables, toolchain installation, runner images, external binaries, command-line `cargo rustc` flags, or arbitrary linker arguments are trustworthy; those controls remain with their canonical CI/supply-chain owners or future focused contracts. `rustdocflags` also remain outside this slice. Any future claim of complete compiler/linker provenance must account for those surfaces with realistic hostile cases rather than a catch-all Cargo-config ban.

## Acceptance and follow-up

1. Obtain independent current-head review of the rustflags RED→repair chain and the retained direct executable-selection contracts.
2. After #229 exact-head required evidence becomes terminal, reconcile #317 by ordinary non-force ancestry while preserving the parent and child deltas.
3. Regenerate executable repository/security evidence on the reconciled exact head.
4. Review `rustdocflags`, command-line `cargo rustc` authority, and linker-argument surfaces separately; add a contract only when a realistic executable/provenance escape is demonstrated.
5. If any blocked executable override is ever required, replace the fail-closed rule only with an explicit design covering immutable executable identity, arguments/source-input provenance, SBOM/attestation, rollback, and buyer-visible evidence.

## References

The Cargo Project. (n.d.). *Configuration*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/config.html

The Rust Project Developers. (n.d.). *Codegen options*. *The rustc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/codegen-options/index.html

The Cargo Project. (n.d.). *Build cache*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/build-cache.html
