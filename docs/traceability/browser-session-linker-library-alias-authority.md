# Browser Session linker `--library` alias authority

Status: source-semantic repair; hosted exact-head execution evidence is still required before acceptance.

## Problem

The Browser Session Cargo/compiler authority contract already fails closed for native-library selectors such as `-lNAME`, `-l NAME`, `-L`, and `--library-path`. LLVM LLD also accepts the GNU-compatible long form `--library=NAME` (and the corresponding separated alias). At LLVM revision `851eb5a97ba67b4e8ebec39fb821c144db67a10a`, `lld/ELF/Options.td` defines `-l` as `Search for library <libname>` and declares both `library` aliases:

- `Separate<["--", "-"], "library">`
- `Joined<["--", "-"], "library=">`

Primary source: `https://github.com/llvm/llvm-project/blob/851eb5a97ba67b4e8ebec39fb821c144db67a10a/lld/ELF/Options.td`.

Before this repair, the shared classifier rejected compact single-dash `-l...` forms but did not classify joined double-dash `--library=...`. A Git-owned `rustflags`/`rustdocflags` path could therefore forward `--library=review_bypass` through `-C link-arg`, `-Wl,`, `--for-linker=`, or doctest compiler forwarding without being recognized as an external native-library selector.

## Authority boundary

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for Git-owned Cargo compiler/linker input authority. Production Cargo package/source topology remains owned by `tests/test_browser_session_trusted_adapter_boundary.py`; this repair does not duplicate that discovery logic.

## RED → repair

- RED `30026004a1108aa850d084c0ba06551d474b050c` adds `tests/test_browser_session_linker_library_alias_authority_contract.py`. It requires joined `--library=review_bypass` to fail closed through ordinary build `rustflags` and rustdoc doctest compiler forwarding, while keeping `-Wl,-z,relro` as an allowed typed control.
- Repair `a168131b65d25f5a4625b77b259492a1b5e692a9` extends the existing shared external-input classifier with `--library` and `--library=`. The RED-to-repair delta in the canonical authority file is two additions/two replacements; no second Cargo/linker scanner was introduced.

The repair deliberately treats library-name selection as input authority even when the name is not a pathname. The selected bytes still depend on linker search roots, sysroot/toolchain state, and the resolved library artifact. A name allowlist alone does not prove artifact identity or provenance.

## Invariant

Git-owned Cargo flags must not select an additional native library through GNU-compatible long-form `--library` syntax unless that input is covered by an explicit reviewed provenance contract. This applies equally when the option is forwarded through rustdoc doctest compilation.

## Rejected alternatives

- **Allow known library names:** rejected because a stable name does not identify the resolved bytes, producer, search root, ABI, or toolchain.
- **Add a second focused scanner:** rejected because it would split policy ownership from the canonical Cargo/compiler authority contract.
- **Rely on positional-token fallback:** rejected because joined `--library=NAME` keeps the library selector and operand in one option token.

## Acceptance and release evidence

If OriginWeave later needs to permit a repository-selected native library, the same reviewed delta must prove at least:

1. immutable or integrity-verified artifact identity and producer provenance;
2. linker/toolchain, target, ABI, and architecture compatibility;
3. bounded search-root/sysroot resolution with symlink containment;
4. SBOM and build provenance for the resolved artifact;
5. reproducible independent rebuild or equivalent byte-identity evidence;
6. invalidation and rollback behavior when the artifact or toolchain changes.

This source-semantic repair is not hosted GREEN. Exact-head required workflows, repository/security gates, independent current-head review, and the broader Browser Session acceptance chain remain mandatory before merge or release.

## Reference

The LLVM Project. (2026). *LLD ELF option definitions* (revision `851eb5a97ba67b4e8ebec39fb821c144db67a10a`) [Source code]. GitHub. `lld/ELF/Options.td`.
