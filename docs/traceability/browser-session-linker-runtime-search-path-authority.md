# Browser Session linker runtime search-path authority

Status: Proposed

Owner: OriginWeave Browser Session / Cargo compiler authority contract

## Problem

GNU-compatible ELF linkers can change which shared objects participate in the produced program without changing the reviewed Cargo dependency graph. `-rpath=dir` embeds a runtime library search directory in the output and passes it to the runtime linker. `-rpath-link=dir` changes the link-time search order used to locate additional shared libraries required by shared objects already included in the link. The GNU `ld` manual warns that `--rpath-link` can override a search path compiled into a shared library and thereby select a different library than the runtime linker otherwise would have used.

Repository-owned `rustflags` or `rustdocflags` can forward these controls through compiler-driver linker arguments. That makes search-path selection part of Browser Session build/runtime provenance rather than an ordinary tuning preference.

## Primary sources

- GNU Binutils `ld` manual, command-line options: https://sourceware.org/binutils/docs/ld/Options.html
- rustc code-generation options (`link-arg`, `link-args`): https://doc.rust-lang.org/rustc/codegen-options/index.html

The GNU `ld` manual states that `-rpath` directories are included in the executable and used by the runtime linker, while `-rpath-link` directories are effective only at link time. It also defines one- or two-dash spellings for multi-letter options and permits `=` or separate operands.

## Invariant

Git-owned Cargo configuration must not select ELF runtime or link-time shared-library search directories unless those directories and the artifacts they can resolve are represented by an explicit reviewed provenance contract.

The canonical direct-linker classifier therefore rejects:

- `-rpath dir`, `-rpath=dir`, `--rpath dir`, and `--rpath=dir`;
- `-rpath-link dir`, `-rpath-link=dir`, `--rpath-link dir`, and `--rpath-link=dir`;
- the same controls when forwarded through `-Wl,`, `--for-linker=`, `-Xlinker`, rustdoc flags, or rustdoc doctest compiler forwarding.

`--enable-new-dtags` remains an allowed control because it changes the dynamic-tag form used for an already selected runtime path but does not itself select a directory or external shared object.

## RED → repair evidence

Structural RED: `12abc14ec3c98caa5662686857409160bf41a02e`

The RED fixture covers build/target `rustflags`, build/target `rustdocflags`, doctest forwarding, one- and two-dash multi-letter spellings, joined and split operands, and an unrelated allowed control.

Minimal canonical repair: `a13f803e8d350abddfaec6dc89378fec438f211d`

The repair changes only `tests/test_browser_session_cargo_compiler_authority_contract.py`: `_linker_option_selects_runtime_search_path()` is added to the existing direct-linker single writer and reused by all existing forwarding paths. No second Cargo scanner, source-topology walker, path allowlist, or runtime loader policy is introduced.

## Rejected alternatives

A pathname allowlist is insufficient. A directory name alone does not bind the concrete shared object eventually selected from that directory, its digest, producer source, ABI, deployment state, symlink resolution, or rollback identity. Allowing `-rpath` while blocking only `-L` would also leave runtime resolution authority outside the reviewed closure; allowing `-rpath-link` would leave link-time transitive shared-library selection outside it.

## Future exception evidence

Any approved exception must bind at least:

- the exact allowed directory/namespace and containment rule;
- immutable identities and cryptographic digests for resolvable shared objects;
- producer source revisions and build provenance;
- target ABI, loader/linker compatibility, and SONAME/DT_NEEDED expectations;
- SBOM/provenance linkage to the consuming release;
- deployment and symlink-resolution evidence;
- reproducible fallback or rollback procedure;
- explicit expiry/revalidation conditions.

## Residual authority

Ambient `LD_RUN_PATH`, `LD_LIBRARY_PATH`, `/etc/ld.so.conf`, default linker scripts/search directories, loader cache/state, runner/container image identity, linker distribution/version and configuration, direct CLI arguments, deployed runtime filesystem contents, and externally materialized shared objects remain CI/release supply-chain evidence surfaces. They are not duplicated into the OriginWeave leaf repository policy contract.
