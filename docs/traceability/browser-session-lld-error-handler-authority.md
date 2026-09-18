# Browser Session LLD error-handler execution authority

## Problem

Browser Session's Cargo/rustc provenance contract already rejects linker replacement, linker plugins, linker scripts, response files, native positional inputs, and external library search inputs. It did not classify LLVM LLD's `--error-handling-script=<path>` option as linker execution authority.

LLD documents `--error-handling-script=<path>` as a user-provided executable that is invoked from linker error handling. The script may be resolved through `PATH` or supplied as a full path, must be executable, and runs in the same environment as the parent linker process. A repository-owned Cargo `rustflags` or `rustdocflags` value can forward this option through rustc's `-C link-arg` / `-C link-args` path. That creates a code-execution edge outside the reviewed Browser Session source and dependency closure even when the selected linker binary itself is unchanged.

This matters operationally because the handler is conditional: a normal link can appear inert while a missing library or undefined symbol causes the linker to execute the selected program. Command acknowledgement or a successful configuration parse therefore cannot be treated as evidence that the build TCB stayed unchanged.

## Constraint and owner boundary

`tests/test_browser_session_cargo_compiler_authority_contract.py` remains the single writer for repository-selected Cargo/rustc/rustdoc/linker execution and input authority. `tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. No linker-policy scanner is duplicated into downstream Browser Session, Navigation, WebDriver BiDi, EgressWeave, Wardnet, or contextual-orchestrator owners.

Ambient `RUSTFLAGS`, `RUSTDOCFLAGS`, direct CLI options, runner `PATH`, the concrete linker distribution/version, and externally restored toolchain/cache state remain CI/release supply-chain evidence surfaces rather than leaf-source authority.

## Alternatives considered

Allowing repository-relative error-handler paths was rejected. Relative path containment says nothing about executable identity after symlink resolution, producer provenance, permissions, mutation between review and execution, or the environment inherited by the process.

Allowing the option only when the file is tracked by Git was rejected. Git tracking alone does not bind the executable artifact, interpreter, transitive runtime, or runner environment used at link time.

Blocking all linker policy options was rejected because modeled non-executable policy such as `--as-needed` does not itself select another executable or external input and remains useful without widening the build TCB.

The selected rule is narrow: fail closed only when forwarded linker arguments select LLD's error-handler executable surface.

## RED → repair

Structural RED `a59f8727061aa1cd5e6d136f61f6e8d969164d8b` adds realistic Cargo fixtures for build `rustflags`, target `rustflags`, build `rustdocflags`, and rustdoc doctest forwarding. It covers both `--error-handling-script=<path>` and split `--error-handling-script,<path>` forms behind `-Wl,`. `--as-needed` remains an allowed control.

Minimal repair `df9c0257982ed136403ce8c3b36999563209ebb0` adds `_linker_option_selects_error_handler()` to the existing direct-linker classifier and reuses the existing build, target, profile, rustdoc, doctest, `-Wl,`, `--for-linker=`, and `-Xlinker` paths. No Cargo topology/config scanner was added.

## Security and release consequence

A future exception requires evidence stronger than a path allowlist:

- immutable executable identity and digest;
- exact LLD/toolchain and runner identity;
- interpreter and transitive runtime provenance when the handler is a script;
- purpose and trigger conditions for each supported error tag;
- environment and secret-exposure analysis;
- SBOM/provenance linkage to the consuming binary;
- reproducible no-handler reference build where applicable;
- rollback and expiry/removal conditions.

Until such a contract exists, repository-selected LLD error handlers fail closed.

## Primary source

LLVM Project. (2026). *Error Handling Script — lld 24.0.0git documentation*. https://lld.llvm.org/error_handling_script.html

The documentation states that LLD executes the user-provided error-handling script in the same environment as the parent process and currently defines `missing-lib` and `undefined-symbol` trigger tags.
