# Browser Session linker runtime filter authority

Status: Proposed

Owner: OriginWeave Browser Session / Cargo compiler authority contract

## Problem

GNU-compatible ELF linkers can persist runtime implementation indirection in the output artifact without adding a normal Cargo dependency. `-f name` / `--auxiliary=name` creates `DT_AUXILIARY`; `-F name` / `--filter=name` creates `DT_FILTER`. At runtime the dynamic linker can resolve symbols through the named shared object, so repository-owned `rustflags` or `rustdocflags` can select runtime code outside the reviewed Cargo source/dependency closure.

This is not equivalent to ordinary linker tuning. The selected shared object participates in runtime symbol implementation authority.

## Primary sources

- GNU Binutils `ld` manual, command-line options: https://sourceware.org/binutils/docs/ld/Options.html
- GNU Binutils `ld` manual: https://sourceware.org/binutils/docs/ld/
- rustc code-generation options (`link-arg`, `link-args`): https://doc.rust-lang.org/rustc/codegen-options/index.html

The GNU `ld` manual states that `--auxiliary=name` records `DT_AUXILIARY` and permits the named shared object to provide alternative implementations. `--filter=name` records `DT_FILTER`; when the filter object is used at runtime, the dynamic linker resolves selected symbols to definitions in the named shared object. The same manual specifies that multi-letter options accept one or two leading dashes and that single-letter option operands may be joined to the option.

The same option surface also defines `-fini=name`, which only selects the symbol used for `DT_FINI`. It does not name an external shared object. Compact `-f<name>` handling therefore must not collapse the documented `-fini=` spelling into auxiliary-library authority.

## Invariant

Git-owned Cargo configuration must not use compiler/linker forwarding to select ELF auxiliary/filter runtime implementations unless the selected runtime artifact is represented by an explicit reviewed provenance contract.

The canonical classifier therefore rejects:

- `-f name` and compact `-f<auxiliary-name>`;
- `-F name` and compact `-F<filter-name>`;
- `--auxiliary name`, `--auxiliary=name`, and GNU single-dash `-auxiliary` forms;
- `--filter name`, `--filter=name`, and GNU single-dash `-filter` forms;
- the same controls when forwarded through `-Wl,`, `--for-linker=`, `-Xlinker`, rustdoc flags, or rustdoc doctest compiler forwarding.

`-fini=<symbol>` and `--as-needed` remain allowed controls because neither names a new runtime implementation artifact.

## RED → repair evidence

Structural security RED: `0168d405a89a5dfec5fff4ed49aef28f43f546b8`

The security RED fixture covers build and target `rustflags`, `rustdocflags`, doctest forwarding, GNU single-dash long-option spelling, compact `-f`/`-F` spelling, and an unrelated allowed control.

Minimal canonical security repair: `b7adc787523741bd35cedf86704133db7d6da9c3`

The repair changes only `tests/test_browser_session_cargo_compiler_authority_contract.py`: one runtime-filter classifier is added to the existing direct-linker single writer. No second Cargo scanner, source-topology walker, or pathname allowlist is introduced.

Fresh compatibility review of the GNU primary source found that the initial compact-`-f` predicate also matched the documented `-fini=<symbol>` option. Compatibility RED `355161387984dc1277fefd1e1bdc2138360953f3` adds `-Wl,-fini=originweave_fini` as an allowed control and therefore fails against the over-broad initial classifier. Minimal correction `28592541f884c08d2f9cb6cfa888508214951115` excludes the documented `-fini=` spelling before compact auxiliary parsing; the hostile auxiliary/filter cases remain unchanged.

## Rejected alternatives

A pathname allowlist is insufficient. A stable path does not prove the selected shared object's digest, producer source, toolchain, ABI compatibility, deployment identity, or rollback state. Treating only `--filter`/`--auxiliary` as relevant while allowing compact `-f`/`-F` or GNU single-dash long-option aliases would leave equivalent spellings outside the contract. Conversely, treating every token beginning with `-f` as auxiliary-library selection is too broad because GNU `ld` separately defines `-fini=`.

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
