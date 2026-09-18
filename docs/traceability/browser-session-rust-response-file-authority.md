# Browser Session Rust response-file authority

## Problem

Repository-owned Cargo `rustflags` and `rustdocflags` are reviewed compiler/documentation input surfaces. Both `rustc` and `rustdoc` support a top-level `@path` argument that opens a UTF-8 file and loads additional command-line options from it, one option per line. Treating only the visible Cargo flag list as authority therefore leaves an opaque indirection path: a Git-owned response file can introduce `--extern`, `--sysroot`, `-L`, `-l`, codegen linker selection, or other compiler inputs after the repository contract has inspected the outer configuration.

This is distinct from linker response files passed through `-Wl,`, `--for-linker=`, or `-Xlinker`. Those are already handled by the linker-argument classifier. This contract closes the rustc/rustdoc top-level response-file boundary.

## Authority and constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. `tests/test_browser_session_cargo_compiler_authority_contract.py` owns repository-selected compiler/rustdoc execution and input authority. Supplemental tests consume that owner rather than rediscovering Cargo topology.

A top-level Cargo `rustflags` or `rustdocflags` argument whose first character is `@` is fail-closed. The rule applies to both `[build]` and `[target.*]` configuration. An `@` appearing later inside an ordinary argument is not a response-file selector and is not rejected by this rule.

## RED → repair evidence

RED `f031bbc6154567c2b641879d00fa49d3412bc739` adds `tests/test_browser_session_rust_response_file_contract.py`. It covers build/target `rustflags` and build/target `rustdocflags` with top-level `@tools/...args` hostile fixtures while retaining an ordinary argument containing an internal `@` as a control.

Repair `0c3d76ae214328c124fab8a7c8adcb4d2452a493` extends the existing `_rustc_argument_extends_external_inputs()` classifier. No new Cargo topology/config scanner is introduced; the existing build/target rustflags/rustdocflags call sites all inherit the same fail-closed rule.

## Security effect

A reviewed Cargo config can no longer hide compiler or rustdoc execution/input authority behind an opaque top-level response file. The repair preserves the existing explicit classifiers for `--extern`, `--sysroot`, `-L`, `-l`, rustc codegen linker/tool selection, and linker-level response files rather than replacing them with a separate policy path.

This is a repository-source contract, not proof of the ambient execution environment. `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `RUSTDOCFLAGS`, `CARGO_ENCODED_RUSTDOCFLAGS`, direct command-line arguments, ancestor or user Cargo configuration, runner-installed toolchains/sysroots, and Cargo's own internally generated argument files remain CI/release/runtime provenance surfaces. Those surfaces must be controlled by the execution/release owner rather than inferred from Git source closure.

## Primary references

- The Rust Project. (2026). *The rustc book: Command-line arguments — `@path`: load command-line flags from a path*. https://doc.rust-lang.org/rustc/command-line-arguments.html#path-load-command-line-flags-from-a-path
- The Rust Project. (2026). *The rustdoc book: Command-line arguments — `@path`: load command-line flags from a path*. https://doc.rust-lang.org/rustdoc/command-line-arguments.html#path-load-command-line-flags-from-a-path

Both references specify that `@path` opens the named file and reads command-line options from it, one option per line, using UTF-8 with Unix or Windows line endings.
