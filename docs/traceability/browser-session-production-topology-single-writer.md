# Browser Session production-topology single writer

- **Status:** active-PR security evidence for PR #317; not protected-main behavior
- **Owner:** OriginWeave Browser Session bounded context
- **Related contract:** `tests/test_browser_session_trusted_adapter_boundary.py`

## Problem

The Browser Session trusted-adapter gate had two different definitions of the production Cargo topology. The canonical boundary test scanned only explicit workspace members for `bind_lifecycle_port`, while the later implicit-workspace contract separately followed recursive in-repository `path` dependencies and custom `[lib].path` / `[[bin]].path` targets.

That split left a concrete composition escape: an implicit local path dependency could contain `bind_lifecycle_port` without a `DisposableContextPort` token or direct Browser Session dependency in that source file. The supplemental topology test would discover the file, but the canonical caller-selected binding gate would not inspect it. A lifecycle binding in such a package could therefore widen product composition without entering the same fail-closed check that protects explicit workspace members.

A second review found that the recursive path resolver silently returned `None` when a production `path` dependency resolved outside the repository review root. Cargo permits local path dependencies outside the repository, but this security contract cannot inspect such a package's source, manifest evolution, or lifecycle binding. Silently omitting it would treat an unreviewable production dependency as if no production edge existed.

## RED and repair

- **Structural RED `a0fb0765d7df8303df490c97dbb5945b1682d837`** adds a hostile `app -> ../plugins/browser-adapter` fixture whose implicit local package calls `bind_lifecycle_port`. The enhanced supplemental production-source closure finds the source, while the canonical `_workspace_production_sources` scanner does not.
- **Minimal causal repair `6b050ee820a29342a9a180638a38b85b33cd66a3`** moves recursive in-repository production `path` dependency traversal and custom production target discovery into the canonical trusted-adapter boundary. The same source closure now drives lifecycle-SPI references, Browser Session dependency allowlists, dormant-entry equality, and caller-selected lifecycle-binding rejection.
- **Single-writer cleanup `18b560c869c5e967a3226d5407bc7216b80f9c80`** removes the duplicate Cargo-topology implementation from the implicit-workspace contract and delegates its hostile fixtures to the canonical boundary scanner. `tests/test_browser_session_custom_target_source_contract.py` continues to consume that same helper through the implicit-workspace contract.
- **External-path RED `0b0204b30585a2a5921a7b1e193121daf23aff50`** adds a production dependency whose manifest lives outside the repository review root and requires the canonical production-package closure to reject it instead of silently dropping the edge.
- **Fail-closed repair `95c49d22e557466c063124989808d02ae363c609`** makes a declared production Cargo `path` dependency outside the repository review root an explicit contract failure. A declared local dependency whose `Cargo.toml` is missing also fails closed. Registry and Git dependencies remain outside this local-path traversal; their package/source integrity is governed by the normal locked dependency and supply-chain controls rather than being misclassified as repository-local source.

## Invariant

There is one repository-security definition of the Browser Session production package/source closure. It starts from explicit workspace packages and an optional workspace-root package, follows production in-repository `path` dependencies including workspace-inherited target-specific dependencies, and includes ordinary `src/**/*.rs` plus manifest-declared `[lib].path` and `[[bin]].path` sources.

The closure fails closed for unsupported workspace-member globs, declared production local-path dependencies that leave the repository review root, missing declared local dependency manifests, missing declared production targets, and declared target paths outside the repository review root. It must never convert an unreviewable production local dependency into absence.

Every source in that closure is subject to the same lifecycle-SPI and caller-selected binding checks. A supplemental hostile fixture may exercise the scanner, but it must not maintain a second production-topology algorithm.

## Scope

This repair changes repository security coverage only. It does not change Browser Session runtime semantics, WebDriver BiDi protocol authority, Chromium behavior, or the trust classification of privileged in-process adapters. Exact-head executable repository/security evidence remains required after the parent lineage is reconciled and the PR becomes runnable.
