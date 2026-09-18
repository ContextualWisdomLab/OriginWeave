# Browser Session linker runtime-loader authority

## Problem

OriginWeave treats repository-owned Cargo compiler and linker selection as Browser Session provenance because the linked runtime is part of the executable trust boundary. The shared Cargo/rustc/rustdoc authority contract already rejected linker replacement, plugins, scripts, response files, external native inputs, symbol-policy files, and related execution/input selectors, but GNU-compatible ELF runtime-loader controls were not modeled explicitly.

GNU `ld` 2.47 documents `-Ifile` / `--dynamic-linker=file` as selecting the dynamic linker recorded for a dynamically linked ELF executable. It also documents `--no-dynamic-linker` as suppressing the load-time dynamic-linker request. These options therefore change which load-time interpreter participates in process startup, or whether that interpreter contract exists at all; they are not ordinary optimization or presentation flags.

Rust's `-C link-arg` and `-C link-args` append arguments to the linker invocation. On Unix-like targets using a compiler driver, rustc documents `-Clink-arg=-Wl,$ARG` as the route for passing an argument to the underlying linker. Git-owned Cargo `rustflags`, `rustdocflags`, and rustdoc `--doctest-build-arg` forwarding can therefore carry the runtime-loader controls into the final link.

## Constraint

The Browser Session authority owner must fail closed without creating a second Cargo/linker scanner. Production Cargo package/source topology remains owned by `tests/test_browser_session_trusted_adapter_boundary.py`; repository-selected compiler/rustdoc/toolchain/linker execution and input authority remains owned by `tests/test_browser_session_cargo_compiler_authority_contract.py`.

A pathname allowlist is insufficient. A permitted path alone does not establish the loader artifact digest, producer provenance, ABI/toolchain compatibility, filesystem containment, reproducibility, or rollback identity. Likewise, treating `--no-dynamic-linker` as harmless would allow a reviewed build to change its load-time execution model without an explicit provenance decision.

## Alternatives considered

1. Allow the linker defaults plus arbitrary `--dynamic-linker` paths. Rejected because a repository change could select a different ELF interpreter while the reviewed Rust/Cargo source closure remains unchanged.
2. Permit repository-relative loader paths. Rejected because repository-relative spelling does not prove immutable artifact identity, symlink containment, executable compatibility, or runtime deployment identity.
3. Block only long-form `--dynamic-linker=`. Rejected because GNU `ld` also accepts the short `-Ifile` spelling, and runtime-loader suppression is independently authority-changing.
4. Extend the existing direct-linker classifier. Selected because all build/target rustflags, rustdocflags, `-Wl,`, `--for-linker=`, `-Xlinker`, and doctest compiler forwarding already converge on that owner.

## RED → repair

Structural RED commits:

- `0b68811ed2bd9ff0324853f8b24a7fbcb8abe626` adds hostile coverage for long-form and short-form ELF runtime-loader selection through build/target `rustflags`, build `rustdocflags`, and rustdoc doctest forwarding.
- `132b6e15c8de47e2caa8395843c64c6ba77bf521` adds explicit `--no-dynamic-linker` coverage while preserving `--as-needed` as an allowed control.

Minimal repair:

- `0d241f075fc3958838c9d5d5c266357d86efc6d2` adds `_linker_option_controls_runtime_loader()` to the existing direct-linker authority classifier and rejects `-I`, compact `-Ifile`, `--dynamic-linker`, `--dynamic-linker=file`, and `--no-dynamic-linker` through the same shared owner. No second Cargo topology or linker scanner is introduced.

The repair commit changes only the canonical authority contract by 10 added lines relative to the final RED head; no unrelated file content is removed or rewritten.

## Invariant

Repository-owned Cargo configuration must not select or suppress the ELF load-time interpreter through Rust/rustdoc linker forwarding unless a separately reviewed Browser Session provenance contract proves that authority. The command acknowledgement or successful link alone is not evidence that the runtime interpreter is the intended one.

## Evidence required for a future exception

Any future intentional custom runtime-loader contract must bind, at minimum:

- immutable loader artifact identity and cryptographic digest;
- producer/source provenance and SBOM linkage;
- exact target ABI and linker/toolchain compatibility;
- path and symlink containment at build and deployment time;
- deployment/runtime identity proving the recorded interpreter resolves to the reviewed artifact;
- reproducible reference build evidence;
- rollback and expiry/revalidation rules;
- buyer-visible security and operability documentation when the runtime model differs from the platform default.

Environment or direct-CLI overrides, runner image identity, system linker distribution, external deployment filesystem state, and loader artifacts materialized outside the reviewed repository remain CI/release supply-chain evidence surfaces rather than leaf-source exceptions.

## References

Free Software Foundation. (2026). *LD: The GNU linker (GNU Binutils 2.47)*. https://sourceware.org/binutils/docs/ld.html

The Rust Project Developers. (2026). *Codegen options: `link-arg`, `link-args`, and `linker`*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/
