# Browser Session Rust native-link attribute authority

## Problem

The Browser Session production-source closure is reviewed, but reviewed Rust source can itself select native-library bytes that are not part of that source closure. Rust's built-in `#[link(...)]` attribute on an `extern` block names a native library for rustc to link. The Rust Reference defines `dylib` as the default, and also supports `static`, macOS `framework`, and Windows `raw-dylib`; link modifiers can further change how those native bytes participate in the artifact.

The existing repository-owned Cargo authority contract already fails closed on Git-owned `-L`, `-l`, `--library`, `--extern`, linker positional inputs, and equivalent modeled forwarding paths. That does not prove the artifact selected by a source-level `#[link(name = ...)]`. A reviewed Rust source tree therefore did not, by itself, prove the final Browser Session native dependency closure.

## Constraint

`tests/test_browser_session_trusted_adapter_boundary.py` remains the single writer for production Cargo package/source topology. `tests/test_browser_session_rust_source_indirection_contract.py` remains the lexical owner for Rust attribute extraction and source-indirection trivia/literal handling. This supplemental contract consumes both; it does not rediscover workspace topology or introduce a second Rust attribute parser.

This is an exact-tree provenance rule, not a claim that Rust FFI or the `link` attribute is unsafe as a language feature. Until a versioned native-artifact contract exists, a source-selected native library would depend on bytes resolved from the target toolchain/search environment rather than on the reviewed OriginWeave source closure.

## RED → repair

Structural RED **`9024d61487d7a0127050b97932589e2044ddb019`** adds hostile fixtures for:

- direct `#[link(name = "review_bypass", kind = "static")]`; and
- nested `#[cfg_attr(unix, link(name = "review_bypass"))]`.

The RED intentionally delegates to the pre-existing Rust-source provenance contract. That predecessor does not classify source-level native-library selection, so both hostile cases are expected to remain unblocked at that generation. Positive controls keep the word `link(...)` inside a documentation string and `#[unsafe(link_section = ...)]` outside this native-library rule.

Minimal repair **`e3571e51db8f3f0dadbbc54baa5de1813e2c66cb`** stays in the supplemental contract. It reuses the canonical production-source closure and the existing balanced Rust attribute/trivia/literal helpers, then fails closed when an attribute meta tree contains a real `link(...)` meta item. No Cargo topology scanner, linker parser, or FFI runtime implementation is duplicated.

Focused review of exact **`e7258ab8136b019b1570e24de8528aa7ba0cb963`** then found a valid root lexical defect in the shared attribute extractor: the outer `_rust_attribute_bodies()` search used `text.find("#", ...)` before lexical filtering. As a result, `#[link(...)]` text inside a line comment or ordinary string could be extracted as if it were a real attribute and rejected by the supplemental contract. This was a source-level false positive, not a reason to weaken the native-library authority rule or add a parallel parser.

Review-driven RED **`5ae81cc7657327ca901d968dba94389c723200cc`** adds explicit controls for both forms:

- `// #[link(name = "review_bypass")]`; and
- an ordinary Rust string containing `#[link(name = "review_bypass")]`.

Canonical root repair **`efb8f1da2f1f3aa98d947e4c6c611e58408cd09b`** changes the shared Rust-source lexical owner instead of working around it in the native-link contract. `_next_rust_attribute_marker()` now scans for the next attribute marker while skipping whitespace/comments, raw strings, conventional strings, and simple character literals with the already-owned lexical helpers; `_rust_attribute_bodies()` consumes that marker. The same repair adds path-attribute controls proving that `#[path = ...]` text inside a line comment or ordinary string is not source indirection. The existing balanced attribute-body parser remains authoritative for both source-indirection and native-link consumers.

The intended post-repair semantics are therefore:

- real direct `#[link(...)]`: fail closed;
- real nested `cfg_attr(..., link(...))`: fail closed;
- `link(...)` inside a real attribute string: allowed unless another modeled meta item grants authority;
- `#[link(...)]` text inside source comments/strings: ignored as lexical data;
- `link_section`: outside this native-library-selection rule; and
- macro-expanded equivalent attributes: still a residual surface requiring compiler-derived or macro-expansion evidence.

## Decision

Browser Session production Rust source must not introduce `#[link(...)]` native-library selection until the same reviewed delta defines the selected artifact contract. `cfg_attr(..., link(...))` is governed by the same rule because target configuration can make that native dependency active only on a subset of release targets.

If a native FFI dependency becomes product-required, the allow contract must identify at least:

- logical library name and link kind/modifiers;
- exact artifact digest and producer/build provenance;
- target triple, ABI, architecture, and toolchain identity;
- bounded library search roots and symlink/realpath containment;
- static/archive member or dynamic/import-library identity as applicable;
- SBOM and release provenance linkage;
- an independent reproducibility check on a clean environment; and
- invalidation and rollback behavior when the artifact or toolchain changes.

A library-name-only or path-only allowlist is insufficient because the same `name` can resolve to different bytes under a different sysroot, linker search root, runner image, or target platform.

## Security and buyer effect

The rule closes a source-authored native supply-chain path that is independent of Cargo `rustflags`. A code review that sees `#[link(name = "foo")]` can prove intent, but without artifact identity it cannot prove which `foo` bytes entered a static artifact or which runtime library/import library a release depends on. For enterprise browser runtime evidence, that distinction is material to SBOM completeness, provenance, reproducibility, incident response, and rollback.

The contract deliberately does not ban ordinary `unsafe extern` declarations that do not select a native library. FFI declarations and ABI safety remain Rust/runtime review concerns; this slice owns only native-library artifact selection.

The shared lexical repair also matters to buyer-visible evidence quality: comments and diagnostic strings must not create phantom dependency findings that could turn a repository policy gate into a source-text keyword filter rather than a reviewable Rust authority boundary.

## Residual surfaces

Environment/direct-CLI `-L`/`-l` injection, toolchain/sysroot composition, target-default native libraries, linker search roots, deployment-time dynamic loader state, and non-Rust native dependencies remain CI/release supply-chain surfaces. They are not converted into repository-owned authority by this contract.

Source or macro mechanisms that can synthesize attributes after parsing require compiler-derived or macro-expansion evidence if they can introduce equivalent native-library authority; this lexical contract does not claim macro-expansion completeness.

## Primary evidence

Rust Project. (2026). *External blocks: The `link` attribute*. The Rust Reference. https://doc.rust-lang.org/nightly/reference/items/external-blocks.html

The Reference states that `#[link]` specifies the native library the compiler links for an external block, defines `dylib`, `static`, `framework`, and `raw-dylib`, and documents modifiers such as `whole-archive` and `verbatim`.

Rust Project. (2026). *Command-line arguments: `-l`*. The rustc book. https://doc.rust-lang.org/nightly/rustc/command-line-arguments.html

The rustc book documents the native-library `-l` model and explicitly notes that library kind and modifiers can also be specified with a `#[link]` attribute. This connects the source attribute to the same native-input semantics already governed for Git-owned Cargo flags.

Rust Project. (2026). *Foreign function interface: Linking*. The Rustonomicon. https://doc.rust-lang.org/nomicon/ffi.html

The Rustonomicon explains that the `link` attribute instructs rustc how to link native libraries and that static native libraries can be incorporated into output artifacts while dynamic dependencies propagate to the final artifact boundary.

## Verification state

The initial RED, native-link repair, review-driven false-positive RED, canonical lexical root repair, and this traceability successor are structurally present on the active #317 lineage. The focused review finding is treated as valid and repaired at its canonical lexical owner rather than suppressed.

The environment available to this writer cannot resolve `github.com` for a local clone, and the exact head still has no pull-request-triggered hosted workflow execution. Therefore no executable GREEN, full current-head review closure, protected-main integration, 100% owned coverage closure, or release readiness is claimed from this dossier alone.
