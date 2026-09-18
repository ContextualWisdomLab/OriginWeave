# Browser Session rustdoc doctest execution authority

Status: Draft; source-semantic contract current on PR #317. Hosted executable evidence is still required before this generation is GREEN.

## Problem

Cargo can pass repository-owned `[build].rustdocflags` and matching `target.<triple|cfg>.rustdocflags` directly to `rustdoc`. Rustdoc can then select external programs that participate in documentation-test execution:

- `--test-runtool <program>` executes the specified wrapper instead of the doctest executable.
- nightly `--test-builder <program>` replaces the default rustc-like program used to compile doctests.
- nightly `--test-builder-wrapper <program>` wraps the selected test builder and may be repeated.

Those selectors can change executable provenance without changing the reviewed Cargo package/source graph, compiler package dependencies, or Browser Session domain code. They therefore belong to the existing Cargo-selected execution/input authority boundary rather than to rustdoc semantics owned by Browser Session.

## Ownership and constraints

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source/dependency topology. `tests/test_browser_session_cargo_compiler_authority_contract.py` owns repository-selected Cargo execution and external-input authority. This repair reuses that owner and does not add a second topology/configuration scanner.

Browser Session does not own rustdoc, Cargo, the Rust toolchain, or doctest scheduling. It only requires that Git-owned configuration cannot silently replace or wrap programs that compile or execute Browser Session documentation tests.

## Evidence and repair

RED `ab2fffacffaa061387440c104d4f2b93b7b9675d` adds hostile build/target `rustdocflags` fixtures for `--test-runtool`, `--test-builder`, and `--test-builder-wrapper`. Ordinary doctest arguments that do not select an external executable, including `--test-args` and `--test-run-directory`, remain controls.

Repair `d9c55cfecd2710fcdb585f1ba971cef413e631d5` adds `_flags_select_rustdoc_test_execution()` to the canonical Cargo compiler-authority contract and applies it to build-level and target-level `rustdocflags`. The rule is fail-closed for the three executable selectors in split or `--option=value` form; it is not a blanket ban on doctest flags.

Primary references:

- Cargo Book, Configuration: `build.rustdocflags` and target `rustdocflags` are custom flags passed to rustdoc; environment/direct-command sources have separate precedence.
- rustdoc book, Command-line arguments: `--test-runtool` executes a chosen wrapper instead of the doctest executable.
- rustdoc book, Unstable features: `--test-builder` selects the rustc-like program used to compile doctests and `--test-builder-wrapper` wraps that program.

## Security effect

Repository review now covers the Git-owned Cargo paths that could otherwise select a doctest runner, test compiler, or compiler wrapper while leaving Browser Session source and dependency topology unchanged. Approval of any such executable later must be an explicit provenance decision, not an incidental rustdoc flag.

## Residual execution provenance

This source contract intentionally does not claim authority over:

- `RUSTDOCFLAGS`, `CARGO_ENCODED_RUSTDOCFLAGS`, `CARGO_BUILD_RUSTDOCFLAGS`, or target-specific environment overrides;
- direct `cargo rustdoc -- ...` / manual rustdoc invocation;
- runner-image, PATH, rustup/toolchain, default rustdoc/rustc identity, or externally supplied wrapper binaries;
- immutable artifact identity, SBOM/attestation, sandbox policy, compatibility qualification, and rollback for a future approved doctest execution program.

Those surfaces require CI/release environment evidence from their canonical owners. A future approved runner/builder/wrapper must be versioned and immutable, tied to the exact toolchain and reviewed policy, and covered by executable tests before this fail-closed rule is relaxed.

## Acceptance

This generation is source-semantic only until the exact reconciled head has hosted repository/security execution and current-head independent review. A command acknowledgement, static inspection, or predecessor workflow result is not executable GREEN.
