# Browser Session linker runtime filter authority

Status: Proposed

Owner: OriginWeave Browser Session / Cargo compiler authority contract

## Problem

GNU-compatible ELF linkers can persist runtime implementation indirection in the output artifact without adding a normal Cargo dependency. `-f name` / `--auxiliary=name` creates `DT_AUXILIARY`; `-F name` / `--filter=name` creates `DT_FILTER`. At runtime the dynamic linker can resolve symbols through the named shared object, so repository-owned `rustflags` or `rustdocflags` can select runtime code outside the reviewed Cargo source/dependency closure.

This is not equivalent to ordinary linker tuning. The selected shared object participates in runtime symbol implementation authority.

## Primary sources

- GNU Binutils 2.47 `ld` manual, command-line options: https://sourceware.org/binutils/docs/ld/Options.html
- GNU Binutils 2.47 `ld` manual: https://sourceware.org/binutils/docs/ld/
- rustc code-generation options (`link-arg`, `link-args`): https://doc.rust-lang.org/rustc/codegen-options/index.html

The GNU `ld` manual states that `--auxiliary=name` records `DT_AUXILIARY` and permits the named shared object to provide alternative implementations. `--filter=name` records `DT_FILTER`; when the filter object is used at runtime, the dynamic linker resolves selected symbols to definitions in the named shared object. The same manual specifies that multi-letter options accept one or two leading dashes and that single-letter option operands may be joined to the option.

## Invariant

Git-owned Cargo configuration must not use compiler/linker forwarding to select ELF auxiliary/filter runtime implementations unless the selected runtime artifact is represented by an explicit reviewed provenance contract.

The canonical classifier therefore rejects:

- `-f name` and compact `-fname`;
- `-F name` and compact `-Fname`;
- `--auxiliary name`, `--auxiliary=name`, and GNU single-dash `-auxiliary` forms;
- `--filter name`, `--filter=name`, and GNU single-dash `-filter` forms;
- the same controls when forwarded through `-Wl,`, `--for-linker=`, `-Xlinker`, rustdoc flags, or rustdoc doctest compiler forwarding.

`--as-needed` remains an allowed control because it does not name a new runtime implementation artifact.

## RED → repair evidence

Structural RED: `0168d405a89a5dfec5fff4ed49aef28f43f546b8`

The RED fixture covers build and target `rustflags`, `rustdocflags`, doctest forwarding, GNU single-dash long-option spelling, compact `-f`/`-F` spelling, and an unrelated allowed control.

Minimal canonical repair: `b7adc787523741bd35cedf86704133db7d6da9c3`

The repair changes only `tests/test_browser_session_cargo_compiler_authority_contract.py`: one runtime-filter classifier is added to the existing direct-linker single writer. No second Cargo scanner, source-topology walker, or pathname allowlist is introduced.

## Rejected alternatives

A pathname allowlist is insufficient. A stable path does not prove the selected shared object's digest, producer source, toolchain, ABI compatibility, deployment identity, or rollback state. Treating only `--filter`/`--auxiliary` as relevant while allowing compact `-f`/`-F` or GNU single-dash long-option aliases would also leave equivalent spellings outside the contract.

## Future exception evidence

Any approved exception must bind at least:

- immutable artifact identity and cryptographic digest;
- producer source revision and build provenance;
- target ABI and dynamic-linker compatibility;
- SBOM/provenance linkage to the consuming release;
- deployment path/namespace containment and purpose;
- reproducible fallback or rollback procedure;
- explicit expiry/revalidation conditions.

## Residual authority

Ambient `RUSTFLAGS`/`RUSTDOCFLAGS`, direct CLI arguments, linker distribution/version and `PATH`, runner/container image identity, deployed runtime filesystem contents, loader search paths, and externally materialized shared objects remain CI/release supply-chain evidence surfaces. They are not duplicated into the OriginWeave leaf repository policy contract.
