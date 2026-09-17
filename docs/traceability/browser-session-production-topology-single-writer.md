# Browser Session production-topology single writer

- **Status:** active-PR security evidence for PR #317; not protected-main behavior
- **Owner:** OriginWeave Browser Session bounded context
- **Related contract:** `tests/test_browser_session_trusted_adapter_boundary.py`

## Problem

The Browser Session trusted-adapter gate had two different definitions of the production Cargo topology. The canonical boundary test scanned only explicit workspace members for `bind_lifecycle_port`, while the later implicit-workspace contract separately followed recursive in-repository `path` dependencies and custom `[lib].path` / `[[bin]].path` targets.

That split left a concrete composition escape: an implicit local path dependency could contain `bind_lifecycle_port` without a `DisposableContextPort` token or direct Browser Session dependency in that source file. The supplemental topology test would discover the file, but the canonical caller-selected binding gate would not inspect it. A lifecycle binding in such a package could therefore widen product composition without entering the same fail-closed check that protects explicit workspace members.

A second review found that the recursive path resolver silently returned `None` when a production `path` dependency resolved outside the repository review root. Cargo permits local path dependencies outside the repository, but this security contract cannot inspect such a package's source, manifest evolution, or lifecycle binding. Silently omitting it would treat an unreviewable production dependency as if no production edge existed.

A third review found that the canonical closure followed ordinary and workspace-inherited `path` dependencies but did not model Cargo dependency-source overrides. Root-manifest `[patch]` / `[replace]` and Git-owned Cargo configuration can cause Cargo's resolved production graph to differ from the direct dependency declarations inspected by the trusted-adapter scanner. Cargo configuration supports local `paths`, `[patch]`, and `[source]` replacement surfaces and is hierarchical according to the directory from which Cargo is invoked. Restricting review to only the repository-root `.cargo/config*` therefore leaves a Git-owned nested config usable whenever a repository command executes below that directory.

## RED and repair

- **Structural RED `a0fb0765d7df8303df490c97dbb5945b1682d837`** adds a hostile `app -> ../plugins/browser-adapter` fixture whose implicit local package calls `bind_lifecycle_port`. The enhanced supplemental production-source closure finds the source, while the canonical `_workspace_production_sources` scanner does not.
- **Minimal causal repair `6b050ee820a29342a9a180638a38b85b33cd66a3`** moves recursive in-repository production `path` dependency traversal and custom production target discovery into the canonical trusted-adapter boundary. The same source closure now drives lifecycle-SPI references, Browser Session dependency allowlists, dormant-entry equality, and caller-selected lifecycle-binding rejection.
- **Single-writer cleanup `18b560c869c5e967a3226d5407bc7216b80f9c80`** removes the duplicate Cargo-topology implementation from the implicit-workspace contract and delegates its hostile fixtures to the canonical boundary scanner.
- **External-path RED `0b0204b30585a2a5921a7b1e193121daf23aff50`** adds a production dependency whose manifest lives outside the repository review root.
- **Fail-closed repair `95c49d22e557466c063124989808d02ae363c609`** makes a declared production Cargo `path` dependency outside the repository review root an explicit contract failure. Missing declared local manifests fail closed as well.
- **Review-driven coverage `600547a4f7cef3a9a22744e13e2890538f21b68a`** adds the direct hostile regression for an in-repository production `path` dependency whose declared `Cargo.toml` is absent.
- **Manifest source-override RED `1c72ea693e47be05b94b330a334118446cc99f39`** adds hostile root-workspace `[patch.crates-io]` and `[replace]` fixtures.
- **Manifest source-override repair `9c3e4780fea9d111f4a362e63ef9531a3b023635`** fails closed when the workspace-root manifest contains a non-empty `[patch]` or `[replace]` table.
- **Repository-config RED `738cbea2ebfb85a966c709a88af8f10f974d4da6`** adds hostile repository-root `.cargo/config.toml` `paths`, legacy `.cargo/config` `[patch.crates-io]`, and `[source] replace-with / directory` fixtures.
- **Repository-config repair `86d24cf6afefa3bd594dad5d7062ec455d8c0572`** extends the same correction-owning package scanner to reject those source-altering repository configs.
- **Independent-review RED `4bdad2aabac71e4fb036aa0b83f0191eac981010`** adds a Git-owned nested `adapter/.cargo/config.toml` local-path override. The earlier root-only scanner misses it even though Cargo configuration is selected hierarchically from the command working directory.
- **Nested-config repair `1c2dfab389a72e9d59c1ca4cdf11d233d490d8a5`** keeps a single scanner but discovers both `.cargo/config.toml` and legacy `.cargo/config` anywhere below the repository review root, requires each resolved config file to remain inside that root, and rejects source-altering `paths`, `[patch]`, or `[source]` tables. Ordinary Cargo settings that do not alter package sources remain outside this fail-closed rule.

## Invariant

There is one repository-security definition of the Browser Session production package/source closure. It starts from explicit workspace packages and an optional workspace-root package, follows production in-repository `path` dependencies including workspace-inherited target-specific dependencies, and includes ordinary `src/**/*.rs` plus manifest-declared `[lib].path` and `[[bin]].path` sources.

The closure fails closed for unsupported workspace-member globs, declared production local-path dependencies that leave the repository review root, missing declared local dependency manifests, missing declared production targets, declared target paths outside the repository review root, Cargo build-script/build-dependency surfaces without generated-source provenance, root-manifest `[patch]` / `[replace]`, and Git-owned `.cargo/config.toml` / `.cargo/config` source-override surfaces anywhere inside the repository. It must never convert an unreviewable production dependency or source override into absence.

Every source in that closure is subject to the same lifecycle-SPI and caller-selected binding checks. A supplemental hostile fixture may exercise the scanner, but it must not maintain a second production-topology algorithm.

Cargo can also merge configuration from directories outside the Git review root, `$CARGO_HOME`, environment-derived settings, and command-line `--config`. Those are execution-environment inputs rather than repository source. They must be captured or excluded by the canonical CI/release environment before an executable build is treated as reproducible exact-head evidence; this repository contract does not pretend that untracked ambient configuration is Git-owned source.

## Primary references

Cargo Team. (2026). *Workspaces*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/workspaces.html

Cargo Team. (2026). *Dependency resolution*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/resolver.html

Cargo Team. (2026). *Configuration*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/config.html

## Scope

This repair changes repository security coverage only. It does not change Browser Session runtime semantics, WebDriver BiDi protocol authority, Chromium behavior, or the trust classification of privileged in-process adapters. Exact-head executable repository/security evidence remains required after the parent lineage is reconciled and the PR becomes runnable.
