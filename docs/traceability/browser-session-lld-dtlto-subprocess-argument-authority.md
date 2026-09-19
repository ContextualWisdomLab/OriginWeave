# Browser Session LLD DTLTO subprocess argument authority

## Problem

OriginWeave treats repository-owned Rust/Cargo linker selection as Browser Session supply-chain authority. LLVM LLD Distributed ThinLTO (DTLTO) can execute a distributor and a remote compiler, and it also exposes options that forward arbitrary command-line arguments into those subprocesses.

At LLVM `llvm-project@0da016867d1fd3d7938895ec36a9775fe26e1919`, `lld/ELF/Options.td` defines `--thinlto-distributor-arg`, `--thinlto-remote-compiler-prepend-arg`, and `--thinlto-remote-compiler-arg` as two-dash `EEq` options, so both separated and `=`-joined forms are accepted. `lld/docs/DTLTO.md` states that these values are placed on the distributor or remote compiler command line. The upstream ELF DTLTO tests use `--thinlto-distributor-arg` for a Python script path and exercise remote-compiler arguments directly.

Before this repair, the canonical Browser Session compiler/linker authority classifier rejected `--thinlto-distributor=<executable>` and `--thinlto-remote-compiler=<executable>`, but did not classify their argument-forwarding surfaces. An `=`-joined argument remained inside one linker option token, so the positional-native-input fallback could not observe its payload. A separated forwarded argument that itself began with an otherwise-unclassified option could also escape positional-input detection.

That is execution and input provenance authority, not a harmless linker tuning surface. A forwarded argument can alter the subprocess toolchain, load executable compiler plugins, select target/runtime inputs, or otherwise change the native bytes emitted for the Browser Session artifact.

## Boundary and ownership

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected compiler, rustdoc, linker execution, and linker-input authority. `tests/test_browser_session_trusted_adapter_boundary.py` continues to own production Cargo package/source topology. No DTLTO scanner or policy authority is duplicated in another bounded context.

The repair deliberately extends the existing `_linker_option_selects_dtlto_executable()` classifier rather than adding a parallel parser. Its name is retained to avoid needless call-site churn; its docstring now states the broader invariant: DTLTO options that select **or control** a subprocess extend authority.

## RED → repair

- Initial structural RED `fe842827a2781b198075d91352745fd90b3b341a` adds joined distributor/remote-compiler argument cases, rustdoc doctest forwarding, and a typed `-z relro` negative control.
- Structural RED successor `352c22b3b7aba5888e2ffd6f1e56cad93e04dba4` adds the separated `--thinlto-remote-compiler-arg --target=...` spelling so the `EEq` grammar itself is covered rather than only the joined form.
- Minimal causal repair `bb657a7be0149d6c47207723cdd9650fb2e0508d` changes only the canonical DTLTO classifier: separated argument-option names fail closed and joined distributor/compiler selectors plus all three argument-forwarding prefixes fail closed. The repair commit changes one existing file by `+16/-2`; no unrelated production or test topology is rewritten.

A preliminary `--chroot` probe (`8634074f4ebad22a17fddcc4badef8176a51c1a4`) was rejected as a false finding and removed by `2d7c6e653d4cf0c7dad81c2827a3b3686b2ec28f`: current LLD accepts `--chroot` only as a separated option, and OriginWeave's existing positional-native-input fallback already rejects the following path. It is not part of this repair claim.

## Alternatives rejected

Allowing known argument strings was rejected because DTLTO arguments are an open-ended subprocess command-line surface; string allowlisting would not prove the selected compiler/distributor implementation, plugin bytes, target/sysroot contents, or transitive files opened by that subprocess.

Inspecting only the DTLTO executable path was rejected because a fixed executable with mutable or unreviewed arguments can still change code generation and load additional executable code.

Treating every unknown linker option as hostile was also rejected. The existing parser intentionally distinguishes typed linker controls such as `-z relro` from execution/input authority so the boundary remains precise instead of becoming a blanket option ban.

## Evidence required for a future exception

A future DTLTO exception must be owned by CI/release provenance rather than by an ad-hoc source pathname. Evidence must bind the exact distributor and remote-compiler artifact digests, version/toolchain identity, complete forwarded argv, target/sysroot and plugin inputs, working-directory and environment inputs that affect code generation, architecture/ABI, and any files materialized or consumed by distributed backends. It must also provide SBOM/provenance linkage, independent reproducibility or an equivalent deterministic attestation, and explicit cache/invalidation/rollback behavior.

Repository pathname containment by itself is insufficient because it does not prove the bytes executed or the transitive inputs selected by forwarded subprocess arguments.

## Acceptance status

The source-semantic RED → minimal repair lineage is established at the commits above. This document does not claim hosted repository/security GREEN, whole-PR review closure, or release readiness. Those claims require exact-head hosted checks and current-head review after the final reconciled #317 lineage is produced.
