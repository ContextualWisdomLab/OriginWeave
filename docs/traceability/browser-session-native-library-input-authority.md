# Browser Session native-library input authority

## Problem

The Browser Session repository contract already constrains Git-owned Cargo settings that replace Rust tools, target runners/linkers, compiler-driver executables, linker plugins, response files, linker scripts, and rustc-managed native tools. That boundary initially did not constrain top-level rustc `-L` and `-l` flags supplied through repository-owned Cargo `rustflags`.

`-L` changes the search path for external crates and libraries, including native libraries. `-l` asks rustc to link a named native library and supports static archives, dynamic libraries, frameworks, and modifiers such as `+whole-archive`. A reviewed Rust source closure therefore did not prove the final native input closure when a Git-owned `.cargo/config.toml` or `.cargo/config` could add either flag.

A first repair closed those top-level rustc forms but left an equivalent driver path open: rustc `-C link-arg` and `-C link-args` append arguments to the linker invocation. On Unix-like targets rustc commonly uses `cc` or `clang` as the linker driver, so `link-arg=-L...`, `link-args=-L ...`, `link-arg=-l...`, and `link-args=-l ...` can widen the same native-library search/input closure without using top-level rustc `-L`/`-l` syntax.

## Constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. The compiler-authority contract may consume that topology and constrain Git-owned compiler/input authority, but it must not reimplement workspace/package discovery.

This slice applies only to repository-owned Cargo `rustflags`. It does not claim authority over environment-injected `RUSTFLAGS`, direct `cargo rustc -- ...` arguments, build-system flags outside the repository, or external toolchain configuration. Those require their canonical CI/supply-chain owner or a separate reviewed contract.

## RED → repair

RED `657428dd607c2a384f95865ff59caba704a899d9` adds hostile contract cases for:

- `-L native=tools/review-bypass-native`, which widens native-library search to a repository-selected path; and
- `-l static:+whole-archive=review_bypass_native`, which asks rustc to link a native static archive as a complete archive.

The predecessor exact allowed both settings.

Repair `d3a1790c50c393e0328854a2dd7fe4d9aaf3d5c4` keeps production topology ownership unchanged, normalizes the existing Cargo flag representation once, and extends the compiler-authority contract so Git-owned top-level `rustflags` fail closed on `-L`/`-l` in both `[build]` and `[target.<...>]` settings. Unrelated codegen flags remain allowed.

Focused review of exact `e934b3c17261ab26bb13b4f02417416c4202d344` then found the forwarded-driver equivalent. RED `ad5090dfb1e6de9eb1e2875365fa2b65ad37a5d9` adds hostile build- and target-scoped cases for compact and split `-C link-arg` / `--codegen=link-args` forms that forward `-L` or `-l` into the linker driver.

Repair `a0f8525b57837ac119ef7b2af9c1daa759ef12f4` reuses one external-input classifier across top-level rustc flags and the existing linker-driver parser. Direct driver arguments, `-Wl,` / `--for-linker=` forwarded arguments, and the argument following `-Xlinker` now fail closed when they select `-L` or `-l`. Existing response-file, linker-script, plugin, tool-selection, GCC specs/wrapper, and driver-search-path checks remain in the same shared parser. Ordinary non-input linker options such as `-Wl,--as-needed` and `-Wl,-Bsymbolic` remain allowed.

## Decision

Until external/native input provenance is modeled as a versioned reviewed contract, repository-owned Cargo configuration must not widen rustc or compiler-driver external-library search paths or request additional native libraries for Browser Session production packages.

This is an input-provenance rule, not a claim that `-L` or `-l` are unsafe Rust/GCC features. They are rejected here because their resolved artifacts are outside the current exact-head source and artifact review closure.

## Residual surfaces

The following remain separate review surfaces and are not pre-authorized by this decision:

- `--extern` and other direct precompiled-Rust dependency injection;
- positional object/archive inputs forwarded through `-C link-arg` / `link-args` that do not use `-L`/`-l`;
- environment `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS` and direct `cargo rustc` trailing arguments;
- sysroot/rustup/toolchain composition and custom target specifications;
- non-GNU platform-specific native input/control-file mechanisms.

A future allowlist must identify the exact artifact path, digest/provenance, producer, target triple, linkage kind, and reproducible build evidence on the same reviewed exact tree. A path-only allowlist is insufficient.

## Primary evidence

Rust Project. (2026). *Command-line arguments: `-L` and `-l`*. The rustc book. https://doc.rust-lang.org/nightly/rustc/command-line-arguments.html

The rustc documentation states that `-L` adds a path searched for external crates and libraries and can be scoped to `dependency`, `crate`, `native`, `framework`, or `all`. It also states that `-l` links the generated crate to a specified native library and supports static archives, dynamic libraries, frameworks, and linking modifiers including `+whole-archive`.

Rust Project. (2026). *Codegen options: `link-arg` and `link-args`*. The rustc book. https://doc.rust-lang.org/rustc/codegen-options/

The rustc documentation states that `link-arg` appends one extra argument and `link-args` appends multiple extra arguments to the linker invocation. It also states that Unix-like targets commonly use a C compiler as the linker driver.

Free Software Foundation. (2026). *Link options*. Using the GNU Compiler Collection (GCC). https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html

GCC documents `-l` as searching and linking the named library and states that the search directories include those added through `-L`. Those driver semantics make forwarded `-L`/`-l` part of native input selection rather than inert linker metadata.

Rust Project. (2026). *Build scripts*. The Cargo book. https://doc.rust-lang.org/cargo/reference/build-scripts.html

Cargo documents the corresponding native-library and search-path concepts through `cargo::rustc-link-lib` and `cargo::rustc-link-search`, which are passed to rustc as `-l` and `-L` semantics. Build-script authority itself remains separately fail-closed in the Browser Session Cargo build-surface contract.

## Verification state

Both RED→repair generations are structurally present on the active #317 lineage. This dossier does not promote the branch to executable GREEN: current-head hosted repository/security workflows and independent current-head review must still complete on the reconciled lineage before merge or release readiness can be claimed.
