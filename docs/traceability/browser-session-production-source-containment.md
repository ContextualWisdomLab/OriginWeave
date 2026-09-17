# Browser Session production-source containment

Status: Draft evidence on PR #317; this file does not claim protected-main shipment or executable exact-head GREEN.

## Problem

OriginWeave's Browser Session trusted-adapter contract derives one Cargo production package/source closure and uses it to review `DisposableContextPort` references, Browser Session dependencies, and caller-selected lifecycle binding. The closure already rejects workspace members, local path dependencies, and explicit Cargo target paths that resolve outside the repository review root.

The remaining gap was default Rust source discovery. The canonical scanner used `manifest.parent.glob("src/**/*.rs")`, which returns a lexically in-repository path even when the Rust file itself is a symlink whose resolved target is outside the repository. That external file can therefore participate in Cargo's default production target while its bytes are not fixed by the OriginWeave exact Git head. This is a provenance and TCB-review defect even when the scanner happens to read the external bytes on one runner.

Cargo's current target reference documents `src/lib.rs`, `src/main.rs`, and `src/bin/` as default production source locations and permits manifest-relative explicit target paths. OriginWeave therefore treats the resolved filesystem object behind every source returned by the canonical Cargo topology scanner as part of the exact-head review boundary, not only the lexical path.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for Cargo package/source discovery.
- The containment guard must consume that canonical closure rather than reimplement workspace membership, dependency recursion, or target discovery.
- Repository-external production source is rejected; it is not made trusted by a symlink placed inside the repository.
- Registry and released external dependencies remain dependency provenance concerns and are not reclassified as repository-local source.
- No future BiDi lifecycle adapter path is pre-authorized.

## RED

Commit `e05381c00b76a96f2b3932941597941eac90a4f5` added a hostile fixture in `tests/test_browser_session_production_source_containment_contract.py`:

1. create an in-repository Cargo workspace member;
2. make its default `src/lib.rs` a symlink to `../external-lifecycle-adapter.rs` outside the repository root;
3. call the canonical `_workspace_production_sources(root)` scanner;
4. require fail-closed repository containment.

The current canonical scanner returns the lexical `adapter/src/lib.rs` path instead of rejecting the external resolved source. No PR-triggered workflow run was emitted for that Draft exact head, so this is structural RED evidence rather than runner-backed RED.

## Minimal repair

Commit `e37adeeeb9ace7e42707f14286841fda5c2ab239` adds a provenance postcondition over the canonical source closure. `_reviewed_production_sources(root)` resolves every discovered source, requires the resolved object to remain below `root.resolve()`, requires it to be a file, and otherwise raises `AssertionError`. It does not duplicate Cargo topology discovery.

Two contracts now apply:

- the current exact OriginWeave production-source closure must satisfy the provenance postcondition;
- the hostile default-source symlink must still be discovered by the canonical scanner and then be rejected by the provenance guard.

This repair changes no Rust Browser Session semantics. It tightens the evidence boundary that determines which source bytes are eligible to participate in the privileged browser-integration TCB.

## Alternatives considered

### Resolve every path inside the canonical topology scanner

This would make containment inseparable from discovery but requires modifying the existing single-writer scanner and all downstream path-identity expectations. It remains a valid future consolidation if the contract is moved into reusable repository tooling.

### Ignore symlinks because CI reads the external target

Rejected. Reading bytes from an external filesystem object during one run is not immutable exact-head provenance and makes review/reproduction depend on runner state.

### Ban all source symlinks

Rejected as broader than necessary. A symlink whose resolved target remains inside the repository can still be covered by the exact-head review boundary; the security invariant is containment of the resolved source object.

## Evidence and follow-up

Current repair exact: `e37adeeeb9ace7e42707f14286841fda5c2ab239`.

The branch remains Draft and diverged from canonical parent #229. This file is therefore structural/source-contract evidence only. Required follow-up is exact-head independent review, then the existing parent-first lineage sequence: terminal #229 evidence, ordinary/non-force #229→#317 ancestry reconciliation with zero valid-delta loss, fresh #317 executable evidence, then #318 → #321 → #316. Real pinned-Chromium acceptance remains downstream under #299 and the canonical `.github` MV3 workflow/sandbox owner.

## Reference

The Cargo Project Developers. (2026). *Cargo targets*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/cargo-targets.html
