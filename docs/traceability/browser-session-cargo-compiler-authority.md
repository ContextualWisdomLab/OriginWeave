# Browser Session Cargo compiler authority traceability

Status: Draft contract evidence on PR #317. This document does not claim executable repository/security GREEN.

## Problem

OriginWeave already fails closed on production `build.rs`, Cargo build dependencies, dependency-source overrides, repository-external source paths, and unmodeled Rust source indirection. The remaining Git-owned Cargo configuration surface was compiler execution itself.

Cargo permits repository configuration to replace `rustc` with `build.rustc`, execute a program in front of `rustc` with `build.rustc-wrapper`, or add a workspace-only wrapper with `build.rustc-workspace-wrapper`. Cargo specifies that the wrapper receives the real compiler path and the compiler arguments. If any of these settings enters a reviewed Browser Session production workspace without a separate provenance contract, the effective compiler invocation is no longer represented by the existing exact-tree source closure. A wrapper can inspect or transform arguments and therefore sits inside the build trusted computing base even when every discovered `.rs` path remains inside the repository.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source override discovery.
- This contract does not authorize a future wrapper, custom compiler, generated source path, or adapter implementation.
- Environment-owned `RUSTC`/`RUSTC_WRAPPER` authority belongs to the CI/runtime owner. This repository contract covers only Git-owned `.cargo/config.toml` and `.cargo/config` files.
- Ordinary Cargo settings that do not replace or wrap compiler execution are not rejected by this contract merely because they occur under `[build]`.

## RED

Commit `ba9c9299d9a81d26ec68ed1c39a9dc11b08ddcb6` added a hostile repository fixture with:

```toml
[build]
rustc-wrapper = "tools/review-bypass-wrapper"
```

The fixture required the Browser Session security contract to fail closed with a Cargo compiler-execution provenance error. The predecessor source/config boundary rejected Cargo `paths`, `[patch]`, `[source]`, build scripts, and build dependencies, but it did not classify `build.rustc-wrapper`. This commit therefore preserves a structural RED; no hosted execution result is inferred from the Draft branch.

## Decision and repair

Commit `80aaa562672211582d5b2de69edc79a984559fb3` introduced a separate compiler-authority contract that first consumes the canonical trusted-adapter production topology contract and then inspects Git-owned Cargo configuration for exactly these compiler execution keys:

- `build.rustc`
- `build.rustc-wrapper`
- `build.rustc-workspace-wrapper`

Any configured key fails closed until a reviewed provenance/attestation design exists. Regression coverage includes all three settings, nested extensionless `.cargo/config`, the current repository tree, and an unrelated `[build]` configuration that remains allowed.

A separate contract was chosen instead of expanding Cargo package/source discovery because compiler invocation authority is not package topology. Folding it into the topology scanner would blur single-writer responsibilities. Allowlisting wrapper paths was rejected because it would pre-authorize executable build authority without immutable artifact identity, behavior, or provenance.

## Security effect and residual risk

The repair closes a Git-owned compiler-execution gap that could otherwise place an unmodeled executable between Cargo and `rustc` while the reviewed Rust source closure remained unchanged. It does not prove that CI environment variables, toolchain installation, runner images, or external compiler binaries are trustworthy; those controls remain with their canonical CI/supply-chain owners.

Cargo compiler flags and target selection are not treated as equivalent to replacing the compiler executable in this slice. If a concrete flag/target configuration can introduce unreviewed executable source bytes or bypass a Browser Session invariant, it requires its own hostile case and causal contract rather than a catch-all Cargo-config ban.

## Acceptance and follow-up

1. Obtain independent current-head review of the RED→repair chain.
2. After #229 exact-head required evidence becomes terminal, reconcile #317 by ordinary non-force ancestry while preserving the parent and child deltas.
3. Regenerate executable repository/security evidence on the reconciled exact head.
4. If a Rust compiler wrapper is ever required, replace this fail-closed rule only with an explicit design covering immutable executable identity, arguments/source-input provenance, SBOM/attestation, rollback, and buyer-visible evidence.

## References

The Cargo Project. (n.d.). *Configuration*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/config.html

The Cargo Project. (n.d.). *Build cache*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/build-cache.html
