# Contributing to OriginWeave

## Before opening a change

Read `AGENTS.md`, `ARCHITECTURE.md`, `SECURITY.md`, and the relevant ADRs. Open or reference an issue for changes that alter a public contract, protocol, security boundary, persistent schema, or release behavior.

## Local verification

```bash
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
python3 -m compileall -q scripts tests
python3 -m unittest discover -s tests -p 'test_*.py'
```

CI performs exact Rust branch coverage on a pinned nightly compiler. Do not remove or bypass the coverage gate.

## Pull requests

A pull request should contain one coherent product or platform slice. Its description must identify the gap, explain the security and compatibility effects, list current verification evidence, link standards or primary research, and state residual risk.

Reviews are not ceremonial. Resolve every actionable thread in code or explain why it is not applicable. Re-run exact-head checks after changes. Do not merge your own change by weakening branch protection or required reviews.

## Commit and naming rules

Use concise imperative commit messages. New Rust crates use the `originweave-*` package prefix and `originweave_*` module convention. Persistent database objects use two-or-more-word `snake_case` names. Avoid stale internal product names.

## Dependencies

Prefer the standard library and existing workspace dependencies. A new dependency requires a documented need, maintenance and license assessment, pinned/locked resolution, and supply-chain checks. GitHub Actions must be pinned by immutable commit SHA.

## Documentation

Update user-facing documentation and `CHANGELOG.md` with behavior changes. Add or amend an ADR when changing a binding architectural decision. Add APA 7 references to `docs/doctoring.md` when research or standards justify a design.
