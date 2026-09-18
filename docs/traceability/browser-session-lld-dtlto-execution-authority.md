# Browser Session LLD Distributed ThinLTO execution authority

## Problem

Browser Session's Cargo/rustc provenance contract already fails closed on repository-selected linker replacement, linker plugins, linker scripts, response files, error-handler executables, native positional inputs, and external library search inputs. LLVM LLD's Distributed ThinLTO interface adds two more executable-selection surfaces that were not modeled explicitly:

- `--thinlto-distributor=<path>` selects the file LLD executes as the distributor process.
- `--thinlto-remote-compiler=<path>` selects the compiler that the distributor process invokes for remote backend compilations.

LLD documents DTLTO as distributing ThinLTO backend compilations through an external distribution system during the traditional link step. The remote compiler must match the LLD version. Repository-owned Cargo `rustflags`, `rustdocflags`, profile rustflags, or rustdoc doctest forwarding can pass these options through rustc `-C link-arg` / `-C link-args`, so the effective build TCB can gain distributor/compiler executables without changing Cargo package topology or the selected linker binary.

## Constraint and owner boundary

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected Cargo/rustc/rustdoc/linker execution and input authority. `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. No DTLTO scanner is copied into downstream Navigation, WebDriver BiDi, EgressWeave, Wardnet, contextual-orchestrator, or other canonical owners.

Ambient `RUSTFLAGS`/`RUSTDOCFLAGS`, direct CLI options, LLD version/distribution, runner `PATH`, external distributor configuration, remote-worker images/toolchains, and restored caches remain CI/release supply-chain evidence surfaces rather than leaf-source authority.

## Alternatives considered

Allowing repository-relative distributor/compiler paths was rejected. Relative containment does not prove executable identity after symlink resolution, mutation between review and execution, the distributor's transitive runtime, or the remote worker/toolchain identity.

Allowing only Git-tracked executables was rejected. Git tracking does not bind the actual interpreter/binary, remote execution system, remote compiler version, environment, or worker artifact set used at link time.

Blocking all ThinLTO options was rejected. Numeric/policy controls such as `--thinlto-jobs=<n>` do not themselves select a new executable and need not widen the execution TCB.

The selected rule is narrow: fail closed when forwarded direct-linker arguments select the DTLTO distributor or remote compiler executable paths.

## RED → repair

Structural RED `e86b8f52099a0e04ebf53595f7f2110cfd033fb6` adds realistic Cargo fixtures for build `rustflags`, target `rustflags`, build `rustdocflags`, and rustdoc doctest forwarding. It covers both documented executable selectors while preserving `--thinlto-jobs=2` as an allowed non-executable control.

Minimal repair `de49c23712ee1defdb73ca440304ede2c5ddf413` adds the DTLTO executable selector to the existing direct-linker classifier. The existing `-Wl,`, `--for-linker=`, `-Xlinker`, build, target, profile, rustdoc, and doctest-forwarding paths consume the same classifier; no second Cargo topology/config scanner was introduced.

The initial fixture path/helper spelling used `dtlt`. Naming-only successors `d4df3c86079297018387a2be17685a0fe5f67923`, `1ce0ebaca2c22d9a2eb9abccd48a8e23909ba5cb`, and `69ccddcc77195fc9e66edf93a82eaaf90735e05f` normalize the test path, test method names, and shared helper to the canonical `DTLTO` acronym without changing policy semantics.

## Security and release consequence

A future DTLTO exception requires evidence stronger than a path allowlist:

- immutable distributor and remote-compiler artifact identities and digests;
- exact LLD/LLVM/compiler version compatibility;
- remote worker image/runtime identity and isolation boundary;
- distributor arguments and transitive executable/tool inputs;
- input/output transfer semantics and integrity checks for remote compilation;
- environment/credential exposure analysis;
- SBOM and provenance linking each remote backend result to the consuming binary;
- deterministic or independently reproducible reference evidence where applicable;
- failure recovery, rollback, cache invalidation, and expiry/removal conditions.

Until that contract exists, repository-selected DTLTO distributor and remote-compiler executable paths fail closed.

## Primary source

LLVM Project. (2026). *Integrated Distributed ThinLTO (DTLTO) — lld 24.0.0git documentation*. https://lld.llvm.org/DTLTO.html

The documentation states that `--thinlto-distributor=<path>` specifies the file to execute as the distributor process and `--thinlto-remote-compiler=<path>` specifies the compiler the distributor invokes; the compiler must match the LLD version. It also warns that options introducing extra input/output files can cause miscompilation if the distribution system does not correctly transfer them.
