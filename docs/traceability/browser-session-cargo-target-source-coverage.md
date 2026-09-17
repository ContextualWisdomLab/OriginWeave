# Browser Session Cargo target-source coverage

- **Status:** active-PR repository-security evidence for PR #317; not protected-main behavior
- **Owner:** OriginWeave Browser Session bounded context
- **Related authority:** `docs/traceability/browser-session-trusted-adapter-boundary.md`, `docs/THREAT_MODEL.md`, ADR 0114

## Problem

The trusted-adapter repository contract already derives its package review surface from Cargo workspace membership and recursive in-repository production `path` dependencies. Its Rust source scan, however, still treated `src/**/*.rs` as exhaustive. Cargo does not require production library and binary targets to live under `src/`: `[lib].path` and `[[bin]].path` may point at other files relative to the package manifest.

That difference matters after a production crate becomes an approved Browser Session dependency. A later change could place another `DisposableContextPort` reference in a custom production target outside `src/`; the manifest would remain on the already-reviewed dependency allowlist while the file-level lifecycle-SPI allowlist would never see the new source. The crate-level dependency gate is therefore necessary but not sufficient for exact source-surface review.

## Authoritative standard

The Cargo Book, **Cargo Targets**, states that Cargo packages consist of targets corresponding to source files, that target configuration is controlled by `[lib]`, `[[bin]]`, `[[example]]`, `[[test]]`, and `[[bench]]`, and that the `path` field specifies a target source file relative to `Cargo.toml`. Library targets default to `src/lib.rs`, while configured targets may use non-standard paths. OriginWeave treats library and binary targets as the shipped production source surface for this Browser Session composition contract; examples, integration tests, and benches do not grant shipped runtime adapter authority.

Primary source: Rust Project Developers. (2026). *Cargo Targets*. The Cargo Book. https://doc.rust-lang.org/cargo/reference/cargo-targets.html

## RED and repair

- **Structural RED `40c9fb3b452d50fdb1b688e5e7634a1b3858c5e4`** adds `tests/test_browser_session_custom_target_source_contract.py`. Its hostile package declares `[lib] path = "runtime/lifecycle_adapter.rs"` and `[[bin]] path = "command/adapter_cli.rs"`; both files reference `DisposableContextPort` and intentionally live outside `src/`. The prior `_production_sources` helper could not discover either file.
- **Minimal causal repair `b66bb5cd05999f569460c76e173bcf1fd44a2499`** extends the existing production-source closure with explicitly configured library and binary target paths from each reviewed production manifest. Declared paths are resolved relative to their package manifest, must remain inside the repository review root, and must identify an existing file. The existing `src/**/*.rs` scan remains as conservative coverage for default and auto-discovered production sources.

No Rust runtime, browser-policy, WebDriver BiDi, navigation, or lifecycle semantics changed. This is a repository-security contract repair that makes the existing TCB review policy match Cargo's actual production-target topology.

## Invariant

For every production package in the Browser Session trusted-composition closure:

1. the package manifest is reviewed if it links `originweave-browser-session`;
2. ordinary `src/**/*.rs` production sources remain inside lifecycle-SPI review;
3. every explicitly configured `[lib].path` and `[[bin]].path` is also inside lifecycle-SPI review even when it lives outside `src/`;
4. a configured production target path outside the repository review root fails closed;
5. a configured production target path naming a missing file fails closed; and
6. source and dependency allowlists continue to describe only surfaces present on the same exact tree.

A future #316 BiDi lifecycle adapter therefore cannot use a custom Cargo target path to widen the Browser Session TCB after its crate dependency has already been approved. Adapter source, Browser Session dependency, target topology, and allowlist widening must remain one reviewed exact-tree delta.

## Rejected alternatives

- **Treat `src/**/*.rs` as the production source boundary:** rejected because Cargo target `path` is authoritative and may point elsewhere.
- **Rely only on the crate dependency allowlist:** rejected because it admits a crate, not every future source file or target added inside that crate.
- **Scan every `.rs` file in the repository:** rejected because tests/examples/tooling are different authority surfaces and would collapse production composition with non-shipped code rather than model Cargo targets.
- **Move the Browser Session trust decision into the BiDi adapter:** rejected because deterministic Browser Session composition policy remains owned by Browser Session; protocol adapters consume that contract.

## Evidence state

The two commits above are structural/source-contract evidence on Draft PR #317. Draft policy does not provide executable exact-head repository/security GREEN. Parent #229 remains the executable prerequisite, and #317 still requires authorized ordinary/non-force ancestry reconciliation followed by fresh exact-head checks and review before this contract can be treated as merge evidence.
