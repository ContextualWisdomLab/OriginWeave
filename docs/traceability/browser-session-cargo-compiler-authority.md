# Browser Session Cargo compiler authority traceability

Status: Draft contract evidence on PR #317. This document does not claim executable repository/security GREEN.

## Problem

OriginWeave already fails closed on production `build.rs`, Cargo build dependencies, dependency-source overrides, repository-external source paths, and unmodeled Rust source indirection. The remaining Git-owned Cargo configuration boundary is execution and external-input selection around Rust compilation, documentation, linking, and test/run execution.

Cargo permits repository configuration to replace `rustc` with `build.rustc`, execute a program in front of `rustc` with `build.rustc-wrapper`, add a workspace-only wrapper with `build.rustc-workspace-wrapper`, and replace the documentation generator executable with `build.rustdoc`. Cargo target configuration also permits a matching target table to choose `linker`, the executable used for linking, and `runner`, the wrapper used for `cargo run`, `cargo test`, and `cargo bench`. In addition, `build.rustflags` and matching target `rustflags` are passed to `rustc`, while `build.rustdocflags` and matching target `rustdocflags` are passed to `rustdoc`.

Rust documents `-C` and `--codegen` as equivalent short and long codegen-option interfaces. The `linker` codegen option directly selects which linker executable rustc invokes. Rust also documents `link-arg` and `link-args` as arguments appended to the linker invocation. On Unix-like targets where a C compiler is the linker driver, Rust documents that `-Clink-arg=-fuse-ld=$value` is passed to the driver after rustc's own linker-feature arguments and therefore generally takes priority when the driver chooses the actual linker. A repository-owned Cargo flag can therefore re-select the linker behind the nominal compiler driver without using `target.<triple>.linker` or `-C linker=<path>`.

The same nominal-driver boundary has another executable-selection path. Rust documents that Unix-like targets commonly use `cc` or `clang` as the linker driver. GCC documents `-Bprefix` as the first search prefix for driver subprograms including `ld`; if the requested program is found there, that executable is run before the standard prefixes or `PATH` lookup. Repository-owned `-C link-arg=-B...` or `--codegen=link-args=-B ...` can therefore redirect the compiler driver to a different linker executable without changing the nominal driver or using `-fuse-ld=`.

GCC expands `@file` response-file arguments in place, recursively. A repository-owned `-C link-arg=@tools/linker.rsp` can hide `-B...`, `-fuse-ld=...`, or another driver option from a scanner that only inspects the visible Cargo flag. GNU-compatible forwarding does not make that provenance safe: `-Wl,@file`, `--for-linker=@file`, and `-Xlinker @file` delegate parsing to the linker after the compiler-driver surface. Until response-file contents, containment, recursion, and effective driver/linker semantics are represented as reviewed provenance, both driver-level and forwarded linker-level `@file` arguments are opaque extensions of the execution/input boundary and fail closed.

GNU `ld` can dynamically load linker plugins with `-plugin name`, and GCC forwards explicit linker options through `-Wl,option`, `--for-linker=option`, or `-Xlinker option`. A repository-owned Cargo flag can therefore leave the nominal compiler and linker executables unchanged while injecting a repository-selected shared object into the linker process. That is executable-code provenance, not ordinary linker tuning.

GNU linker scripts are explicit native-input authority. `-T`/`--script` can select a script whose `INPUT(...)`/`GROUP(...)` directives add object or archive inputs outside the reviewed Rust production-source closure. The shared Cargo linker-argument classifier fails closed on direct script selection and on `-Wl,`, `--for-linker=`, or `-Xlinker` forwarding of those options.

Ordinary positional linker inputs are now part of the same authority boundary. GNU `ld` treats non-option arguments as input files and may parse an unrecognized input as an implicit linker script. The current shared parser therefore treats every unconsumed non-option token in the modeled direct/GNU-compatible grammar as provenance-bearing input, including suffix-bearing native objects, path-shaped extensionless inputs, and bare extensionless filenames. Explicitly modeled harmless option operands, currently including `-z <keyword>`, are consumed by arity before positional classification.

GCC itself is a driver that invokes preprocessing, compilation, assembly, and linking subprocesses according to spec strings. GCC documents that command-line `-specs=file` overrides built-in specs, while the driver also accepts the option and its file argument as separate argv tokens. The spec-file format can override named spec strings or include other spec files. Repository-owned Cargo `link-arg`/`link-args` can therefore alter which subprocesses or switches the nominal linker driver uses with either `-specs=file` or `-specs file`, without changing the visible `linker=` setting.

Rustc and rustdoc external-input flags are also provenance-bearing. Git-owned Cargo `rustflags` and `rustdocflags` can use `-L` to widen library search paths or `--extern` to select an external crate; rustc `-l` can request native libraries. These repository-owned forms are fail-closed until exact artifact provenance is modeled. This is distinct from Cargo's own dependency wiring: the canonical production topology remains owned by `tests/test_browser_session_trusted_adapter_boundary.py`.

If any of these settings enters a reviewed Browser Session production workspace without a separate provenance contract, the effective build/test/documentation execution or external-input path is no longer represented by the existing exact-tree source closure.

## Constraints

- `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology and dependency-source override discovery.
- This contract does not authorize a future wrapper, custom compiler, custom rustdoc executable, linker, runner, generated source path, response file, linker plugin, GCC specs file, linker script, external crate/native input, or adapter implementation.
- Environment-owned `RUSTC`/`RUSTC_WRAPPER`/`RUSTDOC`/`RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS`/`RUSTDOCFLAGS`/`CARGO_ENCODED_RUSTDOCFLAGS` authority belongs to the CI/runtime owner. This repository contract covers Git-owned `.cargo/config.toml` and `.cargo/config` files.
- Ordinary Cargo settings, rustflags, rustdocflags, and linker arguments that do not select a modeled executable, alter modeled driver subprocess authority, dynamically load modeled linker code, or widen a modeled external-input surface are not rejected merely because they occur under `[build]` or `[target]`.
- Command-line `cargo rustc`/`cargo rustdoc` flags, environment/toolchain configuration, non-GNU linker/plugin/control-file mechanisms, non-`-B` driver/tool search-path mechanisms, and linker option grammars/operands not yet explicitly modeled remain separate review surfaces.

## RED

Commit `ba9c9299d9a81d26ec68ed1c39a9dc11b08ddcb6` added a hostile repository fixture with `build.rustc-wrapper`. The predecessor source/config boundary rejected Cargo source overrides and build scripts but did not classify compiler-wrapper execution.

Commit `a39caf95a38862f4cb4bcb68115b5e385bd6c26e` added a hostile `build.rustdoc` fixture. The predecessor compiler-authority contract accepted that setting because its execution-key set covered only `rustc` and the two rustc wrapper keys.

Commit `883ad62125af37e9afb2551803267284e21b7ea3` added target-specific hostile fixtures using `target.<triple>.linker` and `target.<cfg>.runner`. The predecessor contract inspected only `[build]`.

Commit `8b9ab4c34033b9e5990caad79169351e3d771039` added hostile Cargo-owned rustflags that selected `rustc -C linker=<path>`. Commit `b0489000709243b27a9c849d2cada8e5fadd254e` added the corresponding rustdocflags path. Focused review of exact `a3434ebf7d8169a6cf5276d983d4a0c978a9720a` then found the long `--codegen` bypass, preserved by RED `053b6cacd0d7d36dc619165aada99c1ac485b043`.

Commit `0b457b3936ef3fdd0546de06f279fc5cb2756e13` preserves the `-fuse-ld=` driver-selection RED. Commit `ed5661090417e1b95943720f101f54a3a925ac96` preserves the GNU-compatible `-B` driver-search RED. Commit `659e0b3914f19fd19c2d5d9dbeb9f40a43c00517` preserves the driver response-file RED.

Commit `6e94d6b86ac96a677ad6195c818bc94f2199e77b` preserves linker-plugin execution REDs through GNU-compatible forwarding. Commit `f778fe6a3f5c0b5f97e1eee15ef42ef6b25fcc71` preserves joined GCC `-specs=<file>` RED; focused review of exact `c9440bdb2b1b610b6a52024a176acddbcd5e6cc5` identified the split `-specs <file>` bypass, preserved by `eb1ef86f6456e99bd581599b4bb474f9fa48fc02`.

Commit `43375ef80b1bcf6a0421f900a2f9cbf519741849` preserves explicit GNU linker-script native-input REDs. Focused review of exact `c0204371a1d8e06e3b68ed07437d5581f9a94762` found the forwarded linker-response-file gap, preserved and repaired by `b0af9ed4251d4195c4f71893b9550cdbb38ab0b8` with coverage for `-Wl,@file`, `--for-linker=@file`, and `-Xlinker @file`.

Commit `657428dd607c2a384f95865ff59caba704a899d9` preserves repository-owned rustc `-L`/`-l` external-input REDs, and `ad5090dfb1e6de9eb1e2875365fa2b65ad37a5d9` preserves the equivalent compiler-driver forwarding gap. Commit `dbd9faa623f01652c2e75904af8bec614c2e6fc9` preserves Git-owned Cargo `--extern` external-crate input authority.

Commit `e547203368da2aec62e2c94ab491eb0749d7d7b5` preserves suffix-bearing positional native-input REDs. Commit `8f5e49f5fe63d99773d25f13a3ab50648e02ac92` preserves the path-shaped extensionless input gap. Commit `ed86b335b4aa6cde223fe2e614bb26d39f789d8a` proves the remaining bare extensionless filename gap across direct and GNU-compatible forwarding forms.

Commit `5928b1a614c8620203675b609b75f5489338e06d` preserves the rustdoc external-input gap: build-level `rustdocflags --extern` and target-level `rustdocflags -Lnative=...` passed the predecessor because external-input classification was applied only to `rustflags`.

These are source-level RED fixtures; no hosted-run result is inferred from the Draft branch.

## Decision and repair

Commit `80aaa562672211582d5b2de69edc79a984559fb3` introduced a separate execution-authority contract that consumes the canonical trusted-adapter production topology contract and then inspects Git-owned Cargo configuration. `345759105a0f0d2e88142df9961ea724b1055734` extended it to `build.rustdoc`; `e7390cdb12c483570940411c857554540f754763` added matching target `linker` and `runner` rejection.

Commit `743a5321bb72d83541b34f12ce83628f95f581f6` closed Cargo-owned rustflags that select a linker. Commit `e157b9c5f33467425df794f42e704503de697232` reused the same narrow parser for Cargo-owned rustdocflags. Commit `7ee4b253ff4e213e949468274d126db214a63e4a` closed the review-discovered long `--codegen` bypass.

Commit `d9e7d8ab4047bb25d3c6db0e195ec06240495aa9` adds `-fuse-ld=` driver selection. Commit `17a665ca0c895635cc52f3014a43cb9dea86e1b1` adds `-B` driver-program search selection. Commit `92ce887209f58b33ecd478ee09212bda70890894` classifies driver-level `@file` as opaque provenance.

Commit `0ae660fa0c182dc9776aef95d9ed2d2bea7bfa2a` keeps one parser across multiple `link-arg` values and GNU-compatible `-Wl,`, `--for-linker=`, and `-Xlinker` forwarding while rejecting plugin loading. Commit `29dd6bb6548d4e003970743e7602753c3505cf83` adds joined `-specs=<file>` authority; `688680c92e48bc5ad999327c46b366e5d27eeb5f` closes split `-specs <file>`.

Commit `8975224e86ea5ef9821d168efeaafa20702ec659` adds explicit GNU linker-script selection. Commit `b0af9ed4251d4195c4f71893b9550cdbb38ab0b8` closes GNU-forwarded linker response files without banning ordinary forwarded controls.

Commit `d3a1790c50c393e0328854a2dd7fe4d9aaf3d5c4` closes top-level Git-owned Cargo rustc `-L`/`-l`; `a0f8525b57837ac119ef7b2af9c1daa759ef12f4` extends the same external-input classifier to driver/direct-forwarding forms. Commit `098a596029c0cf339c070604a265593540822956` closes Git-owned Cargo `--extern` external-crate inputs.

Commit `e73d221cf28aa25ae6fbd941db95d5d923c7e883` first closes common suffix-bearing positional native inputs. `cb95d5d37050bb7da70f1782da2e63a9b4583734` closes path-shaped extensionless inputs. Commit `5f070bf0b87ae513cf06badda29914e579f850ee` replaces those shape heuristics with an arity-aware direct-linker parser: every unconsumed non-option token in the modeled direct/GNU-compatible grammar fails closed, while explicitly modeled option operands such as `-z relro` remain allowed.

Commit `e8edb487b779a1c4ff22bf2ca4c62ae7e52abd8b` applies the existing external-input classifier to build- and target-level `rustdocflags`, closing repository-owned rustdoc `--extern`/`-L` forms without adding another Cargo topology/config scanner. The dedicated rustdoc trace is `docs/traceability/browser-session-rustdoc-external-input-authority.md`.

The modeled fail-closed execution/input-authority surfaces now include:

- `build.rustc`, `build.rustc-wrapper`, `build.rustc-workspace-wrapper`, and `build.rustdoc`;
- `target.<triple-or-cfg>.linker` and `target.<triple-or-cfg>.runner`;
- build/target `rustflags` and `rustdocflags` that select a linker through `-C`/`--codegen`;
- driver linker reselection through `-fuse-ld=` or `-B`;
- driver/linker response files (`@file`) in the modeled direct/GNU forwarding forms;
- linker plugin loading, explicit `-T`/`--script`, GCC `-specs=<file>` / `-specs <file>`, and modeled rustc-managed native-tool selectors;
- Git-owned Cargo rustc/rustdoc external-input widening through modeled `-L`, `-l`, and `--extern` forms;
- direct/GNU-forwarded positional inputs, including suffix-bearing objects/archives, path-shaped extensionless inputs, and bare extensionless filenames/implicit-script candidates.

A separate contract is retained instead of expanding Cargo package/source discovery because executable selection, driver subprocess authority, dynamically loaded linker code, and external-input argument authority are not package topology. A blanket rustflags/rustdocflags or linker-argument ban remains rejected because non-authority flags are common and do not justify widening this boundary. Path-only allowlists remain insufficient because they do not establish immutable executable/artifact identity, recursively expanded arguments/includes, transitive native inputs, behavior, or provenance.

## Security effect and residual risk

The repair closes the modeled Git-owned execution/input-provenance paths that could place an unmodeled executable between Cargo and `rustc`, replace rustdoc, choose/interpose the linker or runner, alter GCC driver subprocess rules, load linker code, inject explicit/implicit scripts or positional native inputs, widen external-library search paths, or add external crates through repository-owned Cargo flags while the reviewed Rust source closure remained unchanged.

It does **not** prove CI environment variables, direct `cargo rustc` / `cargo rustdoc` trailing arguments, runner images, sysroot/rustup/toolchain composition, external binaries, non-GNU linker/plugin/control-file grammars, non-`-B` driver/tool search-path mechanisms, or unmodeled arguments with equivalent semantics trustworthy. Those controls remain with their canonical CI/supply-chain owners or future focused contracts. Path-shaped and bare extensionless positional inputs, Git-owned Cargo `--extern`, and Git-owned Cargo rustdoc `--extern`/`-L` are no longer residual gaps in this repository-owned Cargo boundary.

## Acceptance and follow-up

1. Obtain independent current-head review of the newest positional-input, rustdoc external-input, and lifecycle-contract repairs together with retained execution/input authority contracts.
2. After #229 exact-head required evidence becomes terminal, reconcile #317 by ordinary non-force ancestry while preserving all valid parent and child deltas.
3. Regenerate executable repository/security evidence on the reconciled exact head.
4. Review environment/direct-CLI injection, non-GNU control/input grammars, unmodeled option arities, and toolchain/sysroot composition separately; add a contract only when a realistic execution/provenance escape is demonstrated.
5. If any blocked executable, response file, linker plugin, specs file, linker script, or external artifact is required, replace fail-closed only with an explicit design covering immutable identity, recursive argument/source provenance, SBOM/attestation, rollback, and buyer-visible evidence.

## References

The Cargo Project. (n.d.). *Configuration*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/reference/config.html

The Cargo Project. (n.d.). *cargo rustdoc*. *The Cargo Book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/cargo/commands/cargo-rustdoc.html

Free Software Foundation. (n.d.). *Directory options*. *Using the GNU Compiler Collection (GCC)*. Retrieved September 18, 2026, from https://gcc.gnu.org/onlinedocs/gcc/Directory-Options.html

Free Software Foundation. (n.d.). *Link options*. *Using the GNU Compiler Collection (GCC)*. Retrieved September 18, 2026, from https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html

Free Software Foundation. (n.d.). *Overall options*. *Using the GNU Compiler Collection (GCC)*. Retrieved September 18, 2026, from https://gcc.gnu.org/onlinedocs/gcc/Overall-Options.html

Free Software Foundation. (n.d.). *Spec files*. *GNU Compiler Collection (GCC) Internals*. Retrieved September 18, 2026, from https://gcc.gnu.org/onlinedocs/gccint/Spec-Files.html

Free Software Foundation. (n.d.). *Plugins*. *GNU ld*. Retrieved September 18, 2026, from https://sourceware.org/binutils/docs/ld/Plugins.html

Free Software Foundation. (n.d.). *Scripts*. *GNU ld*. Retrieved September 18, 2026, from https://sourceware.org/binutils/docs/ld/Scripts.html

Free Software Foundation. (n.d.). *Implicit linker scripts*. *GNU ld*. Retrieved September 18, 2026, from https://sourceware.org/binutils/docs/ld/Implicit-Linker-Scripts.html

The Rust Project Developers. (n.d.). *Command-line arguments*. *The rustc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/command-line-arguments.html

The Rust Project Developers. (n.d.). *Command-line arguments*. *The rustdoc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustdoc/command-line-arguments.html

The Rust Project Developers. (n.d.). *Codegen options*. *The rustc book*. Retrieved September 18, 2026, from https://doc.rust-lang.org/rustc/codegen-options/
