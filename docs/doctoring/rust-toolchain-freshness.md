# Rust toolchain freshness and reproducibility

## Decision

OriginWeave proposes Rust `1.98.1` as the exact stable compiler baseline. As of
2026-09-09, Rust `1.98.1` is the current stable point release. The project keeps
an exact patch-version pin rather than a floating `stable` channel so local and
CI builds remain reproducible and toolchain changes stay reviewable.

Rust `1.98.1` also fixes an upstream vtable-generation miscompilation that could
produce a null function pointer and undefined behavior. That compiler fix is a
material reason to advance the supported stable baseline once the complete
repository, documentation, coverage, and workflow contracts have been verified
on the same exact head.

Production line, region, and function coverage remains on the stable compiler.
Branch coverage uses the independently date-pinned `nightly-2026-08-18`
toolchain because upstream `cargo-llvm-cov` identifies Rust branch coverage as
unstable and nightly-only. The stable baseline update does not silently move
that nightly. Every branch-coverage command must use the same reviewed nightly
pin, and exact-head CI must prove that `llvm-tools-preview`, the pinned
`cargo-llvm-cov` release, the workspace, and the coverage verifier remain
compatible before merge.

The root `rust-toolchain.toml` is tracked through GitHub Dependabot's
`rust-toolchain` ecosystem. Toolchain changes therefore arrive as reviewable
pull requests rather than silently changing underneath local or CI builds.
Repository contracts intentionally duplicate the supported stable version so a
manifest-only bump fails closed instead of changing compiler authority by
accident. Date-pinned branch-coverage nightly updates remain explicit
infrastructure changes and must preserve the repository contract test.

## Failure interpretation

Dependabot PR #301 predecessor head
`ddff2b88a8557374237f916cfc3beaabf5629760` changed only
`rust-toolchain.toml` to Rust `1.98.1`. Native CI run `34320596205` failed at the
Python repository contracts because the supported baseline still required
`1.97.1`; formatting, workspace tests, strict Clippy, and rustdoc therefore did
not execute in that Rust-contract job. Production coverage succeeding on the
same predecessor does not replace that failed gate. The failure is expected
change-control evidence: a compiler-baseline update must move the manifest,
contracts, governing documentation, and workflow provenance coherently rather
than weakening or deleting the pinning regression.

The historical OriginWeave coverage failure at PR #192 predecessor head
`ccb7d31dfe7654bab800d463c2391cc1a19c7d74` was not proof that the compiler was
too old. The compiler emitted the generic note while rejecting a non-stable
const conversion in test code. The later PR #192 head moved that conversion out
of a constant and passed the complete native CI workflow. Toolchain freshness
and source compatibility remain separate controls.

## References

GitHub. (2025, August 19). *Dependabot now supports Rust toolchain updates*.
GitHub Changelog.
https://github.blog/changelog/2025-08-19-dependabot-now-supports-rust-toolchain-updates/

Rust Project Developers. (2026, September 3). *Announcing Rust 1.98.1*. Rust
Blog. https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/

Rust Project Developers. (2026, September 3). *Version 1.98.1 (2026-09-03)*.
Rust release notes.
https://doc.rust-lang.org/stable/releases.html#version-1981-2026-09-03

Taiki Endo and contributors. (2026). *cargo-llvm-cov* (Version 0.8.6)
[Computer software]. GitHub. https://github.com/taiki-e/cargo-llvm-cov
