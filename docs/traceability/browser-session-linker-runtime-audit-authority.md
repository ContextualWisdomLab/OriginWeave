# Browser Session linker runtime-audit authority

## Problem

OriginWeave treats repository-selected linker inputs and executable/runtime authority as Browser Session provenance. The shared Cargo/rustc/rustdoc authority contract already rejects linker replacement, plugins, scripts, response files, external native inputs, symbol-policy files, and ELF interpreter selection. GNU-compatible rtld-audit controls remained a separate option-shaped gap: Git-owned linker forwarding could name an audit library that the platform dynamic linker may load when the produced ELF object executes.

GNU `ld` 2.47 documents `--audit AUDITLIB` as adding `AUDITLIB` to the `DT_AUDIT` entry of the dynamic section, and `--depaudit AUDITLIB` / `-P AUDITLIB` as adding it to `DT_DEPAUDIT`. The linker does not check that these named libraries exist. On platforms supporting the rtld-audit interface, the values therefore select runtime code outside the reviewed Rust/Cargo source closure rather than merely changing a link-time diagnostic or optimization. GNU `ld` also documents that multi-letter options can use one or two leading dashes, so the single-dash `-audit=...` / `-depaudit=...` spellings must not become a spelling-based bypass.

Rust's `-C link-arg` and `-C link-args` append arguments to the linker invocation. On Unix-like targets using a compiler driver, rustc documents `-Clink-arg=-Wl,$ARG` for forwarding an option to the underlying linker. Repository-owned Cargo `rustflags`, `rustdocflags`, and rustdoc `--doctest-build-arg` can therefore carry `DT_AUDIT` / `DT_DEPAUDIT` selection into produced artifacts.

## Constraint

The repair must remain inside the existing Browser Session compiler/linker authority owner. `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology, while `tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler/rustdoc/toolchain/linker execution and input authority. A second Cargo/linker scanner is not acceptable.

A pathname allowlist is also insufficient: a permitted soname or path alone does not prove the audit library's immutable bytes, producer provenance, ABI compatibility, deployment-time resolution, dependency closure, or rollback identity.

## Alternatives considered

1. Allow `--audit` / `--depaudit` because the linker does not itself load the named library. Rejected: the option persists runtime-load authority in the ELF dynamic section, so the security boundary is the produced runtime behavior, not only the linker process.
2. Permit repository-relative audit-library names. Rejected: name spelling does not prove immutable runtime artifact identity or deployment resolution.
3. Block only the double-dash forms. Rejected: GNU `ld` accepts multi-letter options with one or two leading dashes, and Solaris-compatible `-P` is a documented dependency-audit selector.
4. Extend the existing direct-linker classifier. Selected because build/target `rustflags`, `rustdocflags`, `-Wl,`, `--for-linker=`, `-Xlinker`, `link-arg`/`link-args`, and doctest compiler forwarding already converge on that owner.

## RED → repair

Structural RED:

- `81b05158621716fdb323b8a77960d84eeb6e633e` adds hostile `--audit=...`, `--depaudit=...`, compact `-P...`, and doctest-forwarded audit-library cases while retaining `--as-needed` as an allowed control.
- `4b25e4e64e437a4305bd83308a58c412e0d93baa` adds the GNU single-dash multi-letter `-audit=...` alias so the contract does not accidentally depend on one spelling.

Minimal repair:

- `942c158d7b9f7f192688d152673b0badfd36eaf7` adds `_linker_option_selects_runtime_audit_library()` to the existing direct-linker authority classifier and consumes it from `_direct_linker_arguments_extend_authority()`. It covers split and joined double-dash forms, the GNU single-dash multi-letter aliases, and split/compact `-P`. The final-RED → repair diff is one canonical authority file with 10 additions and no deletions.

No new Cargo topology or linker scanner is introduced.

## Invariant

Repository-owned Cargo configuration must not persist an unreviewed rtld-audit library into `DT_AUDIT` or `DT_DEPAUDIT` through Rust/rustdoc linker forwarding. A successful link or command acknowledgement is not evidence that the produced executable's runtime audit code is approved.

## Evidence required for a future exception

Any intentional runtime-audit contract must bind, at minimum:

- immutable audit-library artifact identity and cryptographic digest;
- producer/source provenance, dependency closure, and SBOM linkage;
- exact target ABI, libc/dynamic-linker, and toolchain compatibility;
- build- and deployment-time path/soname resolution and symlink containment;
- purpose and least-privilege justification for audit callbacks;
- reproducible build and runtime acceptance evidence against the pinned loader/library combination;
- rollback, expiry, and revalidation policy;
- buyer-visible security and operability documentation when rtld-audit is enabled.

Environment/direct-CLI overrides, externally materialized audit libraries, runner image/toolchain identity, system linker distribution, and deployment filesystem state remain CI/release supply-chain evidence surfaces rather than leaf-source exceptions.

## References

Free Software Foundation. (2026). *LD: The GNU linker (GNU Binutils 2.47)*. https://sourceware.org/binutils/docs/ld.html

The Rust Project Developers. (2026). *Codegen options: `link-arg`, `link-args`, and `linker`*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/
