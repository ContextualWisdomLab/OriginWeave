# Browser Session production-topology single writer

- **Status:** active-PR security evidence for PR #317; not protected-main behavior
- **Owner:** OriginWeave Browser Session bounded context
- **Related contract:** `tests/test_browser_session_trusted_adapter_boundary.py`

## Problem

The Browser Session trusted-adapter gate had two different definitions of the production Cargo topology. The canonical boundary test scanned only explicit workspace members for `bind_lifecycle_port`, while the later implicit-workspace contract separately followed recursive in-repository `path` dependencies and custom `[lib].path` / `[[bin]].path` targets.

That split left a concrete composition escape: an implicit local path dependency could contain `bind_lifecycle_port` without a `DisposableContextPort` token or direct Browser Session dependency in that source file. The supplemental topology test would discover the file, but the canonical caller-selected binding gate would not inspect it. A lifecycle binding in such a package could therefore widen product composition without entering the same fail-closed check that protects explicit workspace members.

A second review found that the recursive path resolver silently returned `None` when a production `path` dependency resolved outside the repository review root. Cargo permits local path dependencies outside the repository, but this security contract cannot inspect such a package's source, manifest evolution, or lifecycle binding. Silently omitting it would treat an unreviewable production dependency as if no production edge existed.

A third review found that the canonical closure followed ordinary and workspace-inherited `path` dependencies but did not model Cargo dependency-source overrides. Cargo applies root-manifest `[patch]` entries as an overlay to dependency resolution and still supports deprecated `[replace]`; either mechanism can cause the resolved production graph to differ from the direct dependency declarations inspected by the trusted-adapter scanner. Until override provenance is modeled, accepting those tables would let a production package enter through a resolution surface that the exact-head package/source closure does not own.

## RED and repair

- **Structural RED `a0fb0765d7df8303df490c97dbb5945b1682d837`** adds a hostile `app -> ../plugins/browser-adapter` fixture whose implicit local package calls `bind_lifecycle_port`. The enhanced supplemental production-source closure finds the source, while the canonical `_workspace_production_sources` scanner does not.
- **Minimal causal repair `6b050ee820a29342a9a180638a38b85b33cd66a3`** moves recursive in-repository production `path` dependency traversal and custom production target discovery into the canonical trusted-adapter boundary. The same source closure now drives lifecycle-SPI references, Browser Session dependency allowlists, dormant-entry equality, and caller-selected lifecycle-binding rejection.
- **Single-writer cleanup `18b560c869c5e967a3226d5407bc7216b80f9c80`** removes the duplicate Cargo-topology implementation from the implicit-workspace contract and delegates its hostile fixtures to the canonical boundary scanner. `tests/test_browser_session_custom_target_source_contract.py` continues to consume that same helper through the implicit-workspace contract.
- **External-path RED `0b0204b30585a2a5921a7b1e193121daf23aff50`** adds a production dependency whose manifest lives outside the repository review root and requires the canonical production-package closure to reject it instead of silently dropping the edge.
- **Fail-closed repair `95c49d22e557466c063124989808d02ae363c609`** makes a declared production Cargo `path` dependency outside the repository review root an explicit contract failure. A declared local dependency whose `Cargo.toml` is missing also fails closed. Registry and Git dependencies remain outside this local-path traversal; their package/source integrity is governed by the normal locked dependency and supply-chain controls rather than being misclassified as repository-local source.
- **Review-driven coverage `600547a4f7cef3a9a22744e13e2890538f21b68a`** adds the direct hostile regression for the second branch of that fail-closed behavior: an in-repository production `path` dependency whose declared `Cargo.toml` is absent must raise instead of disappearing from the package closure. This was requested by the current-head independent review and does not change production behavior.
- **Source-override RED `1c72ea693e47be05b94b330a334118446cc99f39`** adds hostile root-workspace `[patch.crates-io]` and `[replace]` fixtures and requires the production-package closure to reject those unmodeled resolution overlays.
- **Source-override repair `9c3e4780fea9d111f4a362e63ef9531a3b023635`** keeps `tests/test_browser_session_trusted_adapter_boundary.py` as the correction-owning scanner and fails closed when the workspace-root manifest contains a non-empty `[patch]` or `[replace]` table. The repair does not attempt to infer the resolved graph from an override; any future override requires a reviewed provenance contract in the same exact-tree delta.

## Invariant

There is one repository-security definition of the Browser Session production package/source closure. It starts from explicit workspace packages and an optional workspace-root package, follows production in-repository `path` dependencies including workspace-inherited target-specific dependencies, and includes ordinary `src/**/*.rs` plus manifest-declared `[lib].path` and `[[bin]].path` sources.

The closure fails closed for unsupported workspace-member globs, declared production local-path dependencies that leave the repository review root, missing declared local dependency manifests, missing declared production targets, declared target paths outside the repository review root, Cargo build-script/build-dependency surfaces without generated-source provenance, and root-manifest dependency-source overrides that would make the resolved graph differ from the reviewed direct-dependency closure. It must never convert an unreviewable production local dependency or source override into absence.

Every source in that closure is subject to the same lifecycle-SPI and caller-selected binding checks. A supplemental hostile fixture may exercise the scanner, but it must not maintain a second production-topology algorithm.

Repository-local Cargo configuration is an adjacent execution-environment surface, not silently equivalent to this manifest closure. Cargo supports configuration-level dependency patches and local-path overrides; if OriginWeave adds repository `.cargo/config*` override behavior, that surface must receive its own reviewed fail-closed/provenance contract before it can be treated as exact-head production evidence.

## Primary references

Cargo Team. (2026). *Workspaces*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/workspaces.html

Cargo Team. (2026). *Dependency resolution*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/resolver.html

Cargo Team. (2026). *Configuration*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/config.html

## Scope

This repair changes repository security coverage only. It does not change Browser Session runtime semantics, WebDriver BiDi protocol authority, Chromium behavior, or the trust classification of privileged in-process adapters. Exact-head executable repository/security evidence remains required after the parent lineage is reconciled and the PR becomes runnable.
