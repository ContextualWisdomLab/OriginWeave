# Rust toolchain freshness and reproducibility

## Decision

OriginWeave keeps Rust `1.97.1` as the exact stable compiler baseline. As of
2026-08-19 this is the current stable point release, so the generic compiler
suggestion to upgrade does not justify replacing it with a floating `stable`
channel.

Production line, region, and function coverage remains on the stable compiler.
Branch coverage uses the independently date-pinned `nightly-2026-08-18`
toolchain because upstream `cargo-llvm-cov` still identifies Rust branch
coverage as unstable and nightly-only. Every branch-coverage command must use
the same date pin, and exact-head CI must prove that `llvm-tools-preview`, the
pinned `cargo-llvm-cov` release, the workspace, and the coverage verifier remain
compatible before merge.

The root `rust-toolchain.toml` is tracked through GitHub Dependabot's
`rust-toolchain` ecosystem. Toolchain changes therefore arrive as reviewable
pull requests rather than silently changing underneath local or CI builds.
Date-pinned branch-coverage nightly updates remain explicit infrastructure
changes and must preserve the repository contract test.

## Failure interpretation

The historical OriginWeave coverage failure at PR #192 predecessor head
`ccb7d31dfe7654bab800d463c2391cc1a19c7d74` was not proof that the compiler was
too old. The compiler emitted the generic note while rejecting a non-stable
const conversion in test code. The current PR #192 head moved that conversion
out of a constant and passed the complete native CI workflow. Toolchain
freshness and source compatibility are therefore maintained as separate
controls.

## References

GitHub. (2026). *Dependabot supports updates for Rust toolchains*. GitHub
Changelog. https://github.blog/changelog/

Rust Project Developers. (2026, July 16). *Announcing Rust 1.97.1*. Rust Blog.
https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/

Taiki Endo and contributors. (2026). *cargo-llvm-cov* (Version 0.8.6)
[Computer software]. GitHub. https://github.com/taiki-e/cargo-llvm-cov
