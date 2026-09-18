# Browser Session rustdoc render-file input authority

Status: source-semantic repair evidence; not hosted executable GREEN.

## Problem

OriginWeave treats `tests/test_browser_session_cargo_compiler_authority_contract.py` as the single writer for repository-selected Cargo compiler/rustdoc execution and input authority. The existing contract rejected rustdoc replacement, `@path`, `--extern`, `-L`/`-l`, sysroot, codegen/linker authority, doctest execution programs, and doctest compiler forwarding, but it did not classify rustdoc's rendering file selectors.

Rustdoc documents `--html-in-header`, `--html-before-content`, and `--html-after-content` as reading files and inserting their contents into generated HTML. It also reads file inputs for `--extend-css`/`-e`, `--theme`, and `--check-theme`; current rustdoc source additionally exposes `--markdown-before-content` and `--markdown-after-content` as file-backed rendering inputs. The unstable `--index-page PATH` option is another file-backed surface: rustdoc converts the argument to a path, requires it to be a file, records it as a loaded path, and uses that Markdown file as the generated index page. A Git-owned `build.rustdocflags` or target `rustdocflags` entry could therefore make generated documentation depend on content outside the reviewed Cargo source/dependency closure even when the Rust source and compiler inputs were unchanged.

For a repository that publishes generated documentation, that is a provenance and documentation-integrity gap. It is not treated as Browser Session runtime policy authority, and no claim is made that every such input is executable script content.

## Constraints

- Production Cargo package/source topology remains owned by `tests/test_browser_session_trusted_adapter_boundary.py`.
- This contract must not create a second Cargo configuration/topology scanner.
- Ordinary rustdoc presentation controls that do not make rustdoc read another file, such as `--document-private-items`, `--default-theme`, and `--markdown-css`, remain outside this fail-closed rule.
- Environment `RUSTDOCFLAGS` / `CARGO_ENCODED_RUSTDOCFLAGS`, ancestor or `$CARGO_HOME` configuration, and direct `cargo rustdoc -- ...` remain CI/release environment provenance surfaces.

## Alternatives considered

1. **Allow arbitrary render files when their path is repository-relative.** Rejected. A path spelling does not establish immutable identity, reviewed ownership, symlink containment, or release provenance.
2. **Copy rustdoc option parsing into a new supplemental Cargo scanner.** Rejected because it would violate the existing compiler-authority single-writer boundary.
3. **Fail closed on the file-selecting rustdoc options in the existing owner.** Selected. It is small, deterministic, and preserves the current authority topology.

## Decision

Initial RED commit `2f8233cb7957ffd959d88ed8b3aca44bf3f6f001` added a realistic repository Cargo fixture in `tests/test_browser_session_rustdoc_render_input_authority_contract.py`. Before the repair, `build.rustdocflags = ["--html-in-header", "tools/review-bypass-header.html"]` and equivalent target/render-file selectors were not rejected by the canonical authority helper.

Initial repair commit `ab259680e9bc8c6fde2221cd4c12d2a36721e93a` added a rustdoc file-input classifier to the existing compiler-authority owner and applied it to both build-level and target-level `rustdocflags`. Coverage commit `a1d8a7fe27ed67f2189dd19f276cbc960632441c` exercised the modeled stable/unstable HTML, Markdown, CSS and theme selectors plus safe controls.

A fresh primary-source sweep then found the unstable `--index-page PATH` file input that the first classifier generation had not modeled. Follow-up RED `a17cb3d60e5a09b7e10131dcef9eec39bded3d97` added a Cargo fixture using `-Z unstable-options --index-page tools/review-bypass-index.md`; the prior classifier accepted it. Repair `54041d692aafc9d2c9d55134db9df4810c5b76d0` added `--index-page` to the same canonical selector set. Coverage `5dda25c3d4892d1bb813f86dd9d0d6873a19a10a` added the equals-form target configuration so split and equals spellings are both constrained.

A later documentation-metadata finding broadened the owner name, not the render-file semantics. Repair `47ff4370afdda5487224c437f6883d8947500c3f` renamed the shared classifier to `_flags_select_rustdoc_documentation_input()` and its option set to `RUSTDOC_DOCUMENTATION_INPUT_OPTIONS`, with rejection marker `rustdocflags:documentation input`, so rendered-file and cross-crate metadata inputs share one accurate authority boundary. The separate metadata rationale and RED are documented in `browser-session-rustdoc-doc-meta-input-authority.md`.

The classifier now includes these render-file selectors:

- `--html-in-header`
- `--html-before-content`
- `--html-after-content`
- `--markdown-before-content`
- `--markdown-after-content`
- unstable `--index-page`
- `--extend-css` and its short `-e` form
- `--theme`
- `--check-theme`

The trusted-adapter boundary remains the single writer for Cargo package/source discovery; this change only extends the existing rustdoc input-authority classifier.

## Primary references

- Rust Project. (2026). *The rustdoc book: Command-line arguments*. https://doc.rust-lang.org/rustdoc/command-line-arguments.html
  - documents file-backed HTML inclusion, CSS extension, theme/check-theme, and the distinction between `--markdown-css` and files whose contents rustdoc reads.
- Rust Project. (2026). *rustdoc option definitions (`rustdoc/lib.rs`)*. https://doc.rust-lang.org/beta/nightly-rustc/src/rustdoc/lib.rs.html
  - identifies the HTML/Markdown file selectors, `--extend-css`, and unstable `--index-page PATH` used by current rustdoc.
- Rust Project. (2026). *rustdoc configuration (`rustdoc/config.rs`)*. https://doc.rust-lang.org/beta/nightly-rustc/src/rustdoc/config.rs.html
  - shows `ExternalHtml::load` receiving the HTML/Markdown file option values and separately shows `--index-page` becoming a `PathBuf`, being required to resolve to a file, and being recorded as a loaded path.
- Rust Project. (2026). *The Cargo Book: Configuration*. https://doc.rust-lang.org/cargo/reference/config.html
  - documents `build.rustdocflags` as custom flags passed to rustdoc and Cargo's hierarchical configuration model.

## Security and buyer effect

Repository-reviewed Rust source can no longer silently acquire additional rendered-document content through Git-owned Cargo `rustdocflags` using the modeled rustdoc file selectors, including the unstable custom index page. This narrows the documentation supply-chain boundary and prevents a source review from incorrectly implying that generated documentation is derived only from reviewed repository inputs.

This does **not** prove generated documentation publication, GitHub Pages deployment, CSP behavior, browser rendering, accessibility, or release provenance. Those require their own exact-head build/publish/browser evidence.

## Residual risk and follow-up

- Environment/direct-CLI rustdoc flags and ambient Cargo configuration remain CI/release supply-chain inputs.
- `--markdown-css` writes a stylesheet reference into Markdown-rendered HTML rather than loading the referenced file contents during rustdoc execution. It is intentionally not classified as this file-input surface; external-resource policy for published documentation should be owned by the docs/site publication boundary.
- Rustdoc's unstable `--read-doc-meta-dir` is now classified by the same canonical documentation-input owner, but its directory/merge semantics and output-only `--write-doc-meta-dir` control are documented separately.
- Future rustdoc releases may add file-backed rendering options. Exact toolchain qualification must update this contract when those options become relevant.
- An eventual approved custom render asset contract must identify the artifact immutably, prove repository/release provenance and containment, and connect the generated documentation to SBOM/provenance and rollback evidence rather than relying on a pathname allowlist.
