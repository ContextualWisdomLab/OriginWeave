# Browser Session LLD layout/profile input authority

Status: Proposed until exact-head hosted repository/security checks and independent review complete.

## Problem

LLD can read repository-selected external files after Cargo/rustc have already selected the reviewed Rust source and dependency closure. Four ELF linker options materially affect output layout or LTO decisions:

- `--call-graph-ordering-file=<file>` lays out sections using a supplied call graph;
- `--irpgo-profile=<file>` reads a temporary IRPGO profile for startup/profile-guided ordering;
- `--symbol-ordering-file=<file>` lays out sections according to a supplied symbol-order file;
- `--lto-sample-profile=<file>` reads an LTO sample profile. LLD also exposes the GNU-plugin-compatible `-plugin-opt=sample-profile=<file>` / `--plugin-opt=sample-profile=<file>` alias.

Before this repair, equals-joined spellings such as `--symbol-ordering-file=tools/symbols.order` were not classified by the direct-linker authority contract. Because the file name remains inside the same option token, the existing positional-native-input fallback never saw a separate path token. Git-owned Cargo `rustflags`, `rustdocflags`, or rustdoc doctest compiler forwarding could therefore select unreviewed layout/profile material without tripping the Browser Session provenance boundary.

## Authoritative option grammar

LLVM LLD's ELF option table distinguishes the accepted spellings:

- `call-graph-ordering-file` uses the `Eq` multiclass. `Eq` accepts both one- and two-dash multi-letter spellings and supports separated and `=`-joined operands. Therefore both `-call-graph-ordering-file=<file>` and `--call-graph-ordering-file=<file>` are modeled.
- `irpgo-profile` and `symbol-ordering-file` use `EEq`, whose spelling is double-dash only and supports separated and `=`-joined operands.
- `lto-sample-profile=` uses `JJ`, a double-dash joined option.
- `plugin-opt=sample-profile=` is an alias of `lto-sample-profile` using `J`; `J` accepts both one- and two-dash spellings.

The contract does not invent single-dash aliases for the `EEq` or `JJ` options.

## Owner boundary

Browser Session keeps repository-selected compiler/rustdoc/toolchain/linker execution and input authority in `tests/test_browser_session_cargo_compiler_authority_contract.py`. Production package/source topology and dependency-source authority remain owned by `tests/test_browser_session_trusted_adapter_boundary.py`.

This repair extends the existing direct-linker classifier only. Existing `-Wl,`, `--for-linker=`, `-Xlinker`, `-C link-arg`, `-C link-args`, build/target `rustflags`, build/target `rustdocflags`, and rustdoc doctest forwarding continue to converge on the same owner path.

## RED → repair

- RED `04a6079c025d7a1db83c8ee302f957f9fcc93096`: introduces hostile equals-joined LLD layout/profile file cases plus rustdoc doctest forwarding and an allowed `-Wl,-z,relro` control.
- Repair `04da16344e413cdd966d6104f75c27b5603750ea`: adds the four file-selecting option families to the canonical direct-linker authority classifier.
- Grammar correction RED `565456357d5b7f1a52f5b58d8205f170d4bb39ff`: covers the valid single-dash `-call-graph-ordering-file=` spelling.
- Grammar correction repair `a5c63c7fbe0cef14ee47cd184b87d001c8b296b1`: adds that exact alias without broadening unrelated option matching.
- Alias RED `3797481cdcd241c94dc1bfe5bad0d32d54be0957`: covers both `-plugin-opt=sample-profile=` and `--plugin-opt=sample-profile=`.
- Alias repair `d6dc93f87afb986be88537a8f7a174126c5656e5`: routes those LLD sample-profile aliases through the same layout/profile input classifier.

These are source-semantic RED/repair contracts. They are not hosted executable GREEN until the exact protected evidence lanes actually run.

## Decision

Repository-selected LLD layout/profile files are rejected by default because they can alter code/data placement or LTO decisions while living outside the reviewed Cargo source/dependency closure.

A pathname allowlist is insufficient. A future exception must bind the input to an immutable content digest, reviewed producer/source identity, exact linker/toolchain compatibility, repository/runner containment including symlink resolution, SBOM/provenance evidence, deterministic rebuild evidence, expiry/invalidation rules, and rollback. Profile data that may encode production execution behavior also requires purpose and data-retention review before becoming a governed build input.

## Residual authority

Environment/direct-CLI linker arguments, compiler-driver defaults, toolchain-distributed profiles, runner filesystem contents, externally restored build/cache state, linker distribution/version and `PATH`, and artifacts materialized outside Git remain CI/release supply-chain evidence surfaces. They are not converted into repository-source exceptions by this contract.

## Evidence

LLVM's ELF `Options.td` defines `call-graph-ordering-file`, `irpgo-profile`, `symbol-ordering-file`, `lto-sample-profile`, the `plugin-opt=sample-profile=` alias, and the `Eq`/`EEq`/`J`/`JJ` spelling grammar. `lld/ELF/DriverUtils.cpp` treats these option values as paths when generating reproduction material, corroborating that they are external file inputs rather than scalar tuning values.

### References

LLVM Project. (2026). *LLD ELF option definitions* (`lld/ELF/Options.td`, commit `34eeb2320ff2b991ddb3e3511bbe01ba14478d93`). https://github.com/llvm/llvm-project/blob/34eeb2320ff2b991ddb3e3511bbe01ba14478d93/lld/ELF/Options.td (retrieved September 19, 2026).

LLVM Project. (2026). *LLD ELF driver reproduction utilities* (`lld/ELF/DriverUtils.cpp`, commit `34eeb2320ff2b991ddb3e3511bbe01ba14478d93`). https://github.com/llvm/llvm-project/blob/34eeb2320ff2b991ddb3e3511bbe01ba14478d93/lld/ELF/DriverUtils.cpp (retrieved September 19, 2026).
