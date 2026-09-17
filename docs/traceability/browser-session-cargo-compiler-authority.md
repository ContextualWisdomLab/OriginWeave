# Browser Session Cargo compiler authority traceability

Status: Draft contract evidence on PR #317. This document does not claim executable repository/security GREEN.

## Problem

OriginWeave already fails closed on production `build.rs`, Cargo build dependencies, dependency-source overrides, repository-external source paths, and unmodeled Rust source indirection. The remaining Git-owned Cargo configuration surface was direct executable selection around Rust compilation, documentation, linking, and test/run execution.

Cargo permits repository configuration to replace `rustc` with `build.rustc`, execute a program in front of `rustc` with `build.rustc-wrapper`, add a workspace-only wrapper with `build.rustc-workspace-wrapper`, and replace the documentation generator executable with `build.rustdoc`. Cargo target configuration also permits a matching target table to choose `linker`, the executable used for linking, and `runner`, the wrapper used for `cargo run`, `cargo test`, and `cargo bench`. If any of these settings enters a reviewed Browser Session production workspace without a separate provenance contract, the effective build/test execution path is no longer represented by the existing exact-tree source closure. A wrapper or runner can execute repository-selected behavior even when every discovered `.rs` path remains inside the repository, while a custom linker participates directly in producing the executable artifact.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source override discovery.
- This contract does not authorize a future wrapper, custom compiler, custom rustdoc executable, linker, runner, generated source path, or adapter implementation.
- Environment-owned `RUSTC`/`RUSTC_WRAPPER`/`RUSTDOC` authority belongs to the CI/runtime owner. This repository contract covers only Git-owned `.cargo/config.toml` and `.cargo/config` files.
- Ordinary Cargo settings that do not directly select one of the modeled executables are not rejected merely because they occur under `[build]` or `[target]`.
- `rustflags`, `rustdocflags`, target selection, and environment/toolchain configuration remain separate review surfaces. In particular, rustc flags can influence linker behavior; this slice does not claim total linker-policy closure through every flag spelling.

## RED

Commit `ba9c9299d9a81d26ec68ed1c39a9dc11b08ddcb6` added a hostile repository fixture with:

```toml
[build]
rustc-wrapper = "tools/review-bypass-wrapper"
```

The fixture required the Browser Session security contract to fail closed with a Cargo compiler-execution provenance error. The predecessor source/config boundary rejected Cargo `paths`, `[patch]`, `[source]`, build scripts, and build dependencies, but it did not classify `build.rustc-wrapper`. This commit therefore preserves a structural RED; no hosted execution result is inferred from the Draft branch.

Commit `a39caf95a38862f4cb4bcb68115b5e385bd6c26e` added a second hostile fixture using:

```toml
[build]
rustdoc = "tools/review-bypass-rustdoc"
```

The predecessor compiler-authority contract accepted that setting because its execution-key set covered only `rustc` and the two rustc wrapper keys. The new fixture therefore preserves a distinct structural RED: the repository could select an arbitrary rustdoc executable for the documentation gate while the reviewed Rust source and Cargo package topology remained unchanged.

Commit `883ad62125af37e9afb2551803267284e21b7ea3` added target-specific hostile fixtures using both direct target executable surfaces:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "tools/review-bypass-linker"

[target.'cfg(unix)']
runner = "tools/review-bypass-runner"
```

The predecessor contract inspected only `[build]` and therefore accepted both target entries. This is structural RED evidence for direct Git-owned target executable selection, not hosted-run evidence.

## Decision and repair

Commit `80aaa562672211582d5b2de69edc79a984559fb3` introduced a separate compiler-authority contract that first consumes the canonical trusted-adapter production topology contract and then inspects Git-owned Cargo configuration for Rust compiler execution keys.

Commit `345759105a0f0d2e88142df9961ea724b1055734` extended the same bounded contract to `build.rustdoc`, because Cargo documents it as the program path used for rustdoc execution.

Commit `e7390cdb12c483570940411c857554540f754763` extended the same config-owner contract to matching `[target]` tables and fails closed on direct `linker` and `runner` selection. Cargo documents `linker` as the linker path for that target and `runner` as the wrapper for `cargo run`, `cargo test`, and `cargo bench`. The modeled fail-closed executable keys are now:

- `build.rustc`
- `build.rustc-wrapper`
- `build.rustc-workspace-wrapper`
- `build.rustdoc`
- `target.<triple-or-cfg>.linker`
- `target.<triple-or-cfg>.runner`

Any configured key fails closed until a reviewed provenance/attestation design exists. Regression coverage includes all four build-level Rust-tool settings, target-triple linker selection, `cfg(...)` runner selection, nested extensionless `.cargo/config`, the current repository tree, and unrelated `[build]` configuration that remains allowed.

A separate contract was chosen instead of expanding Cargo package/source discovery because executable selection is not package topology. Folding it into the topology scanner would blur single-writer responsibilities. Allowlisting executable paths was rejected because a path alone does not establish immutable executable identity, behavior, arguments, or provenance.

## Security effect and residual risk

The repair closes modeled Git-owned executable-selection gaps that could otherwise place an unmodeled executable between Cargo and `rustc`, replace rustdoc, choose the linker that emits repository artifacts, or interpose a runner around repository test/run binaries while the reviewed Rust source closure remained unchanged. It does not prove that CI environment variables, toolchain installation, runner images, external binaries, or compiler flags are trustworthy; those controls remain with their canonical CI/supply-chain owners or future focused contracts.

`rustflags` and `rustdocflags` remain intentionally outside this direct-key slice. Rustc documents `-C linker=<path>` as another way to choose the linker executable, so the current contract must not be described as complete linker provenance until flag-derived executable selection is modeled with its own hostile cases. Target selection and Cargo's broader flag surfaces likewise require causal review rather than a catch-all Cargo-config ban.

## Acceptance and follow-up

1. Obtain independent current-head review of the direct executable-selection RED→repair chains.
2. Add a separate hostile flag-provenance slice before claiming complete linker execution authority; at minimum cover Cargo-owned rustflags forms that can select `rustc -C linker` without broad false-positive rejection of unrelated flags.
3. After #229 exact-head required evidence becomes terminal, reconcile #317 by ordinary non-force ancestry while preserving the parent and child deltas.
4. Regenerate executable repository/security evidence on the reconciled exact head.
5. If any blocked executable override is ever required, replace the fail-closed rule only with an explicit design covering immutable executable identity, arguments/source-input provenance, SBOM/attestation, rollback, and buyer-visible evidence.

## References

The Cargo Project. (n.d.). *Configuration*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/config.html

The Rust Project Developers. (n.d.). *Codegen options*. *The rustc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/codegen-options/index.html

The Cargo Project. (n.d.). *Build cache*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/build-cache.html
