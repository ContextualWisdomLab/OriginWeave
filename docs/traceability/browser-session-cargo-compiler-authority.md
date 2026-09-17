# Browser Session Cargo compiler authority traceability

Status: Draft contract evidence on PR #317. This document does not claim executable repository/security GREEN.

## Problem

OriginWeave already fails closed on production `build.rs`, Cargo build dependencies, dependency-source overrides, repository-external source paths, and unmodeled Rust source indirection. The remaining Git-owned Cargo configuration surface was Rust tool execution itself.

Cargo permits repository configuration to replace `rustc` with `build.rustc`, execute a program in front of `rustc` with `build.rustc-wrapper`, add a workspace-only wrapper with `build.rustc-workspace-wrapper`, and replace the documentation generator executable with `build.rustdoc`. Cargo specifies that the wrappers receive the real compiler path and compiler arguments, while `build.rustdoc` is the executable Cargo invokes for rustdoc. If any of these settings enters a reviewed Browser Session production workspace without a separate provenance contract, the effective Rust tool invocation is no longer represented by the existing exact-tree source closure. A wrapper can inspect or transform compiler arguments, and a replacement rustdoc executable can execute repository-selected code during the documentation gate even when every discovered `.rs` path remains inside the repository.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source override discovery.
- This contract does not authorize a future wrapper, custom compiler, custom rustdoc executable, generated source path, or adapter implementation.
- Environment-owned `RUSTC`/`RUSTC_WRAPPER`/`RUSTDOC` authority belongs to the CI/runtime owner. This repository contract covers only Git-owned `.cargo/config.toml` and `.cargo/config` files.
- Ordinary Cargo settings that do not replace or wrap Rust tool execution are not rejected by this contract merely because they occur under `[build]`.

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

## Decision and repair

Commit `80aaa562672211582d5b2de69edc79a984559fb3` introduced a separate compiler-authority contract that first consumes the canonical trusted-adapter production topology contract and then inspects Git-owned Cargo configuration for Rust compiler execution keys.

Commit `345759105a0f0d2e88142df9961ea724b1055734` extended the same bounded contract to `build.rustdoc`, because Cargo documents it as the program path used for rustdoc execution. The fail-closed key set is now:

- `build.rustc`
- `build.rustc-wrapper`
- `build.rustc-workspace-wrapper`
- `build.rustdoc`

Any configured key fails closed until a reviewed provenance/attestation design exists. Regression coverage includes all four settings, nested extensionless `.cargo/config`, the current repository tree, and an unrelated `[build]` configuration that remains allowed.

A separate contract was chosen instead of expanding Cargo package/source discovery because Rust tool invocation authority is not package topology. Folding it into the topology scanner would blur single-writer responsibilities. Allowlisting executable paths was rejected because it would pre-authorize build/documentation execution authority without immutable artifact identity, behavior, or provenance.

## Security effect and residual risk

The repair closes Git-owned Rust tool-execution gaps that could otherwise place an unmodeled executable between Cargo and `rustc`, or replace the rustdoc executable used by repository documentation gates, while the reviewed Rust source closure remained unchanged. It does not prove that CI environment variables, toolchain installation, runner images, or external compiler/documentation binaries are trustworthy; those controls remain with their canonical CI/supply-chain owners.

Cargo compiler flags, rustdoc flags, target linkers/runners, and target selection are not treated as equivalent to direct Rust tool executable replacement in this slice. They remain separate review surfaces. If a concrete flag/target setting can introduce unreviewed executable behavior, source bytes, or bypass a Browser Session invariant, it requires its own hostile case and causal contract rather than a catch-all Cargo-config ban.

## Acceptance and follow-up

1. Obtain independent current-head review of both RED→repair chains.
2. After #229 exact-head required evidence becomes terminal, reconcile #317 by ordinary non-force ancestry while preserving the parent and child deltas.
3. Regenerate executable repository/security evidence on the reconciled exact head.
4. If a Rust compiler wrapper or replacement rustdoc executable is ever required, replace this fail-closed rule only with an explicit design covering immutable executable identity, arguments/source-input provenance, SBOM/attestation, rollback, and buyer-visible evidence.

## References

The Cargo Project. (n.d.). *Configuration*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/config.html

The Cargo Project. (n.d.). *Build cache*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/build-cache.html
