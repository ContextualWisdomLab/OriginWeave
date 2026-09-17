# Browser Session production-source containment

Status: Draft evidence on PR #317; this file does not claim protected-main shipment or executable exact-head GREEN.

## Problem

OriginWeave's Browser Session trusted-adapter contract derives one Cargo production package/source closure and uses it to review `DisposableContextPort` references, Browser Session dependencies, and caller-selected lifecycle binding. The closure already rejects workspace members, local path dependencies, and explicit Cargo target paths that resolve outside the repository review root.

The remaining gap was default Rust source discovery. The canonical scanner used `manifest.parent.glob("src/**/*.rs")`, which returns a lexically in-repository path even when the Rust file itself is a symlink whose resolved target is outside the repository. That external file can therefore participate in Cargo's default production target while its bytes are not fixed by the OriginWeave exact Git head. This is a provenance and TCB-review defect even when the scanner happens to read the external bytes on one runner.

Cargo's current target reference documents `src/lib.rs`, `src/main.rs`, and `src/bin/` as default production source locations and permits manifest-relative explicit target paths. OriginWeave therefore treats the resolved filesystem object behind every source returned by the canonical Cargo topology scanner as part of the exact-head review boundary, not only the lexical path.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` is the single writer for Cargo package/source discovery **and** repository-containment validation.
- Supplemental hostile-fixture tests must delegate to that canonical function rather than wrap it with a second provenance implementation.
- Repository-external production source is rejected; it is not made trusted by a symlink placed inside the repository.
- Registry and released external dependencies remain dependency provenance concerns and are not reclassified as repository-local source.
- No future BiDi lifecycle adapter path is pre-authorized.

## RED

Commit `f157c215a49d203be2fbe146460ddf22c9081002` rewired `tests/test_browser_session_production_source_containment_contract.py` so its hostile default-source symlink fixture calls canonical `_workspace_production_sources(root)` directly and requires that function itself to fail closed.

At predecessor exact `3fd55acfaa58fde61f1ca9236db91df2b18320ad`, canonical `_workspace_production_sources(root)` only collected lexical `src/**/*.rs` paths plus explicit target paths. Repository containment lived in supplemental `_reviewed_production_sources(root)`, so the direct canonical call returned the external-target symlink instead of raising. The new test therefore exposes a real single-writer violation. No PR-triggered workflow run is emitted for the Draft head, so this remains structural RED evidence rather than runner-backed RED.

## Minimal repair

Commit `7cc1cfaec705c6980a59af84bf4459b3e358873a` moves the provenance postcondition into canonical `_workspace_production_sources(root)`:

1. gather the existing Cargo production source closure without changing workspace, dependency, or target discovery;
2. resolve every discovered source;
3. require the resolved source to remain beneath `root.resolve()`;
4. require the resolved object to be a file;
5. return the reviewed lexical paths only after those checks succeed.

The supplemental production-source containment test now contains only current-tree and hostile fixtures. It no longer defines a second `_reviewed_production_sources` policy function. Lifecycle-SPI reference review, Browser Session dependency review, lifecycle binding review, and the hostile provenance fixture therefore consume one canonical source closure and one containment decision.

This repair changes no Rust Browser Session semantics. It tightens the evidence boundary that determines which source bytes are eligible to participate in the privileged browser-integration TCB.

## Alternatives considered

### Keep containment in a supplemental wrapper

Rejected after the current review. It produced two policy writers: canonical trusted-adapter tests consumed `_workspace_production_sources` directly while only the supplemental containment test consumed `_reviewed_production_sources`. Future callers could therefore bypass the provenance guard without noticing.

### Ignore symlinks because CI reads the external target

Rejected. Reading bytes from an external filesystem object during one run is not immutable exact-head provenance and makes review/reproduction depend on runner state.

### Ban all source symlinks

Rejected as broader than necessary. A symlink whose resolved target remains inside the repository can still be covered by the exact-head review boundary; the security invariant is containment of the resolved source object.

## Evidence and follow-up

Current repair exact: `7cc1cfaec705c6980a59af84bf4459b3e358873a`.

The branch remains Draft and diverged from canonical parent #229. This file is therefore structural/source-contract evidence only. Required follow-up is exact-head independent review, then the existing parent-first lineage sequence: terminal #229 evidence, ordinary/non-force #229→#317 ancestry reconciliation with zero valid-delta loss, fresh #317 executable evidence, then #318 → #321 → #316. Real pinned-Chromium acceptance remains downstream under #299 and the canonical `.github` MV3 workflow/sandbox owner.

## Reference

The Cargo Project Developers. (2026). *Cargo targets*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/cargo-targets.html
