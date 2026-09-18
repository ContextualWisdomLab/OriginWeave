# Browser Session PGO profile input authority

## Decision

OriginWeave treats repository-selected profile-guided optimization data as compiler input provenance, not as a harmless optimization preference.

Git-owned Cargo `rustflags`, profile `rustflags`, and `rustdocflags` must therefore fail closed when they select `-C profile-use=<path>` or `-C profile-sample-use=<path>`. The same rule applies when rustdoc forwards those arguments to the doctest compiler through `--doctest-build-arg`.

`-C profile-generate=<path>` remains allowed by this contract because it names an output location for newly collected profile data rather than an input consumed to shape the current binary. Approval of a future PGO workflow requires a separate immutable profile-artifact contract rather than weakening this rule.

## Why this is provenance-sensitive

Rust's PGO documentation defines the optimization workflow as collecting runtime profile data and feeding the resulting profile back into a later compilation. `-C profile-use=<profdata>` supplies instrumentation-derived `.profdata`; `-C profile-sample-use=<profile>` supplies sampling-profile data. LLVM uses those data to guide inlining, machine-code layout, register allocation, and related optimization decisions. Two builds from the same reviewed Rust source and dependency graph can therefore produce materially different machine code when the profile input differs.

The current rustc option model represents `profile_use` and `profile_sample_use` as path-bearing compiler inputs. A mutable or runner-local profile path would sit outside the reviewed Cargo package/source closure and outside the native/linker provenance rules already enforced by Browser Session.

## RED → repair

RED `8842bb09d17d1ed3861080b3cf8ba7779a354123` added hostile fixtures proving that the prior canonical Cargo compiler-authority contract accepted:

- build-level `-Cprofile-use=...`;
- target-level split `-C` + `profile-sample-use=...`;
- Cargo profile `--codegen=profile-use=...`.

Repair `76e8e8944475e068f84023758cc8fd6b83275a45` keeps the existing Cargo compiler-authority test as the single policy owner. It adds one `_codegen_option_extends_external_inputs()` classifier and extends `_flags_extend_external_link_inputs()` to parse split, compact, and long `-C`/`--codegen` forms. The rustdoc doctest forwarding path now reuses that same external-input classifier rather than maintaining a parallel list.

Coverage commit `6fa0537538789b4c899466ab1619e7d24bad8b36` adds direct rustdoc and `--doctest-build-arg` hostile fixtures and retains `profile-generate` as a control.

## Boundary and future approval requirements

This contract governs only Git-owned repository configuration already traversed by the canonical Browser Session Cargo authority test. It does not claim control over ambient `RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS`, direct CLI injection, ancestor or `$CARGO_HOME` configuration, runner images, or externally materialized profile artifacts. Those remain CI/release supply-chain evidence surfaces.

If OriginWeave later adopts PGO deliberately, the profile must be a versioned immutable artifact with at least:

- exact source/workload/toolchain identity and generation procedure;
- content digest and immutable storage identity;
- workload/data provenance and purpose constraints;
- compiler/LLVM compatibility evidence;
- SBOM/provenance linkage to the binary that consumes it;
- reproducibility comparison against the non-PGO reference build;
- rollback and expiry/re-generation policy.

A path allowlist alone is insufficient because the profile contents, not only the pathname, influence generated machine code.

## References

The Rust Project Developers. (2026). *Codegen options: profile-use*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/#profile-use

The Rust Project Developers. (2026). *Profile-guided optimization*. The rustc book. https://doc.rust-lang.org/nightly/rustc/profile-guided-optimization.html

The Rust Project Developers. (2026). *CodegenOptions in rustc_session::options*. Rust compiler documentation. https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/options/struct.CodegenOptions.html
