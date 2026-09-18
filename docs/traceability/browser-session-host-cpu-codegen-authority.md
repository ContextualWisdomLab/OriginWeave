# Browser Session host-CPU codegen authority traceability

Status: Draft contract evidence on PR #317. This document does not claim hosted executable, repository/security, coverage, release, or runtime GREEN.

## Problem

Rust documents `-C target-cpu=<cpu>` as the compiler control that selects the processor for generated code. The special value `native` means the processor of the host machine. Rust also documents unstable `-Z tune-cpu=<cpu>` as using the same CPU value set for instruction scheduling; `native` therefore makes the result depend on the machine that happened to run the compiler.

A repository-owned Cargo configuration can place those options in build/target `rustflags`, profile `rustflags`, build/target `rustdocflags`, or rustdoc `--doctest-build-arg`. In that form the reviewed Git tree no longer determines the code-generation CPU by itself. Two otherwise identical builds may select different instruction sets or scheduling according to runner hardware, which is incompatible with the release contract's reproducibility and exact-provenance requirements.

The risk is narrower than `target-cpu` in general. An explicit CPU such as `x86-64-v3` is repository-visible and deterministic at this boundary; it still requires normal target/runtime compatibility evidence, but it is not an ambient host selector. This contract therefore fails closed only on `target-cpu=native` and `tune-cpu=native` in Git-owned Cargo flag surfaces.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology.
- `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo compiler/rustdoc/toolchain execution and input authority.
- This contract does not ban explicit fixed `target-cpu` values or ordinary codegen options solely because they tune code generation.
- Ambient `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `RUSTDOCFLAGS`, direct `cargo rustc`/`cargo rustdoc` flags, runner CPU selection, virtualization, and externally selected build images remain CI/release supply-chain evidence rather than leaf repository configuration authority.
- A future exception for host-derived codegen must not be represented as a pathname or string allowlist. It needs an explicit build-hardware identity, target compatibility decision, artifact provenance/SBOM linkage, reproducibility policy, and rollback story.

## RED

Commit `cdec934c8d7fb8e14eb17cb0885bf1ba9d6fa277` adds hostile fixtures covering:

- build `rustflags = ["-C", "target-cpu=native"]`;
- target compact `-Ctarget-cpu=native`;
- profile `--codegen=target-cpu=native`;
- build `rustdocflags` selecting `target-cpu=native`;
- rustdoc `--doctest-build-arg` forwarding `-C target-cpu=native`;
- unstable split `-Z tune-cpu=native`;
- an explicit `target-cpu=x86-64-v3` control that must remain allowed.

The predecessor classifier had no ambient-host CPU check, so these hostile configurations were not represented by the canonical Cargo authority contract. Because the PR is Draft and no exact-head hosted run is available, this is source-semantic RED evidence rather than an executed hosted RED claim.

## Decision and repair

Commit `3dc8702b74f845750cee88e1a34ca27cbbd4d7d2` adds `_flags_select_ambient_host_cpu()` to the existing canonical Cargo compiler-authority contract. It recognizes split, compact, and long codegen forms of `target-cpu=native`, plus split/compact `-Z tune-cpu=native`, without creating a second Cargo configuration scanner.

The existing build, target, profile-rustflags, rustdocflags, and rustdoc doctest-forwarding paths now reuse that classifier. Policy-specific evidence is emitted as `rustflags:ambient host cpu`, `rustdocflags:ambient host cpu`, or the profile path ending in `:ambient host cpu`; doctest forwarding remains owned by the existing `rustdocflags:doctest compiler authority` marker.

The fixed-CPU control stays accepted. That preserves intentional cross-build optimization choices while removing the runner-hardware dependency from Git-owned build configuration.

## Security and commercial effect

- Exact source review can no longer silently become host-CPU-dependent through repository Cargo flags.
- Reproducibility and artifact-attestation claims cannot be satisfied by two builds that happen to compile for different host CPUs under the same reviewed tree.
- The repair does not claim that CI runner hardware is already attested. It keeps that residual in the CI/release owner where image, runner, CPU, and final artifact provenance can be bound together.
- No WebDriver BiDi/browser domain truth, adapter tuple, navigation authority, recovery state, or protocol identifier moves out of OriginWeave's existing bounded contexts.

## Evidence status

The RED and repair are present on the PR #317 lineage. Fresh hosted executable evidence, exact-head full review, Rust/docstring/test/edge coverage, parent #229 CodeQL closure, non-force parent reconciliation, and immutable release evidence remain independent gates.

## References

The Rust Project Developers. (2026). *Codegen options: target-cpu*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/#target-cpu

The Rust Project Developers. (2026). *Codegen options: tune-cpu*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/#tune-cpu
