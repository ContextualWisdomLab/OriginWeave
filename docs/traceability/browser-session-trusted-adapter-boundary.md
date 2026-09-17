# Browser Session trusted-adapter boundary

- **Status:** active-PR security and composition evidence for PR #317; not protected-main behavior
- **Owner:** OriginWeave Browser Session bounded context
- **Related authority:** `docs/THREAT_MODEL.md`, `SECURITY.md`, ADR 0114, issue #312

## Problem

`DisposableContextPort` is a cross-crate service-provider interface. Its implementation is allowed to return the browser-issued isolation and browsing-context address that Browser Session records before minting `PresentationMutationAuthority`. The Rust type system cannot distinguish a reviewed implementation from malicious code merely because both implement the same public trait. Aggregate-issued request/completion correlation, incarnation binding, duplicate rejection, exact-fact recovery, and monotonic epochs prevent replay, swapping and ABA classes; **request/completion correlation is not adapter authentication**.

Treating an arbitrary in-process implementation as if it were an untrusted web actor would therefore create a false security promise. A nonce or opaque request handed to that implementation can simply be echoed. It does not prove that Chromium created a disposable user-context boundary.

## Trust boundary

OriginWeave's threat model places the Rust control plane and privileged Chromium/browser adapters inside the trusted computing base. `originweave-browser-session` is an internal `publish = false` crate, not an extension SDK that promises isolation from hostile linked Rust code. A malicious crate already executing inside this trusted process is a supply-chain compromise / trusted-code compromise; it is not made safe by making one handle constructor opaque.

This does **not** make every implementation acceptable. Product composition may bind only a reviewed privileged lifecycle adapter; a caller-selected production adapter is not admitted. `DisposableContextPort` is an internal TCB SPI, not caller-selected product policy. Any production source that references this SPI outside the Browser Session owner and any production crate that depends on `originweave-browser-session` is a repository review surface and must be explicitly allowlisted by contract. An allowlist entry is evidence about a production surface that exists on the same exact branch, not permission reserved for a future implementation. Test doubles remain allowed only under test code and grant no shipped product capability.

No external production consumer is approved on the current #317 tree. The intended future consumer is the versioned WebDriver BiDi lifecycle adapter in `crates/originweave-bidi`, but the current BiDi manifest does not depend on Browser Session and `crates/originweave-bidi/src/lifecycle_acl.rs` does not exist. When #316 or a verified successor introduces that adapter, the implementation, manifest dependency, and exact allowlist entries must arrive in the same reviewed delta. Browser Session does not pre-authorize those future paths.

## Enforced repository contract

`tests/test_browser_session_trusted_adapter_boundary.py` enforces the currently supportable boundary:

1. the Browser Session crate remains `publish = false`;
2. the canonical threat model continues to classify privileged browser integration as trusted Zone C code;
3. production references to `DisposableContextPort` outside the Browser Session owner are limited to the explicit reviewed source allowlist;
4. production crate dependencies on `originweave-browser-session` are limited to the explicit reviewed manifest allowlist, including direct package aliases, table syntax, target-specific production dependencies, and workspace-inherited aliases resolved through root `[workspace.dependencies]`;
5. both allowlists exactly describe production surfaces that exist on the current tree, so an absent future source path or dependency cannot be pre-approved;
6. no current production source outside the Browser Session owner references `bind_lifecycle_port` as a caller-selected composition escape hatch, regardless of method-call, UFCS, or whitespace spelling; and
7. the source and manifest review surface is derived from the Cargo workspace's explicit `[workspace].members` plus the workspace-root package when `[package]` is present, rather than from a `crates/*` directory convention. Workspace-member globs fail closed until this contract is explicitly extended and reviewed.

The sixth rule intentionally leaves the product composition owner unclaimed until a reviewed runtime/composition lane exists. When that owner is introduced, its exact path must be added deliberately with architecture and security review rather than discovered implicitly through a new call site. The seventh rule prevents both an explicit workspace member outside `crates/*` and a future workspace-root package from gaining Browser Session linkage or SPI access without entering the same repository review surface.

### Scanner false-negative repair

The first repository contract recognized only the literal unqualified Rust form `impl DisposableContextPort for ...` and the exact method spelling `.bind_lifecycle_port(`. Those are style conventions, not security boundaries: valid Rust can name the trait through a qualified path or alias and can invoke the binding function through UFCS or with different whitespace.

Test-first commit `100c00487488bbc281106ddf6fe4ae1b60feb16b` adds hostile qualified-trait, aliased-trait, UFCS, and whitespace spellings and exposes those false negatives. Minimal contract repair `7b2334b1df92d03629b7931ce71cd58134b93f4c` makes direct source review spelling-resilient: any external production source containing the SPI token is reviewed, and any production source outside the owner containing the binding API token is rejected until an explicit composition owner is approved.

A second review found the remaining cross-file alias case: one reviewed module could import or re-export the trait under another name while a different module implements only that alias and therefore contains no `DisposableContextPort` token. Test-first commit `6ce9c1f3b13fe157cfae822a0958c2b8dc2dabd8` records that direct source scanning cannot prove this case. Commit `cb9fb54e4a799919a425f3636cb0a5f1daacfb24` adds the compensating crate-boundary invariant: every production `Cargo.toml` that can link Browser Session must itself be reviewed and allowlisted. A cross-file alias therefore cannot create a new production adapter from an unreviewed crate without first widening an explicit dependency review surface.

Cargo permits the dependency key itself to be renamed with `package = "originweave-browser-session"` and also permits table-style dependency declarations. Test-first commit `1e46b302254596397a6c4b0bb9ced1be02a33ea1` adds both forms and exposes the narrower manifest-key matcher. Commit `b24b9beb7a0804b90339a0e9e6abce3dd701fc4b` makes the manifest review fail closed on the canonical package token wherever it appears in a production crate manifest, covering direct keys, package aliases, and table syntax.

A third review found a governance hole in the allowlist itself. The contract pre-listed the future BiDi source path and manifest even though neither current production surface existed. That meant a later change could introduce exactly those surfaces without modifying the security contract, turning a supposedly explicit review surface into latent permission. Test-first commit `6727474a15e85f66917821169cafcba89dbcfbdb` requires both allowlists to equal the surfaces actually discovered on the current tree and therefore fails on those future reservations. Commit `6f2265e8d99cccd7bef89b2aaa4581854f093087` removes the reservations. A future BiDi adapter must now widen the allowlist in the same reviewed change that introduces its source and dependency.

A fourth review found that Cargo workspace inheritance could bypass the manifest scanner without ever spelling the canonical package name in the consuming crate. A root declaration such as `browser_session = { package = "originweave-browser-session", ... }` under `[workspace.dependencies]` can be consumed by a member as `browser_session = { workspace = true }`; the previous per-member regex saw only the alias. Test-first commit `97b925c0d7c957f99e9b798f45decac70877cd40` records that escape. Commit `aeea79c5d57a1cb1c7c5d3f760a2d728214d8a09` parses Cargo TOML, resolves workspace-inherited dependency aliases to their canonical package, and applies the same review surface to target-specific production dependencies. Dev-only dependencies remain outside the shipped adapter-composition surface.

A fifth review found that the hardened scanner still discovered production manifests and Rust sources with `crates/*` filesystem globs instead of Cargo's authoritative workspace membership. Cargo permits explicit workspace members at arbitrary relative paths, so a later `plugins/browser-adapter` member could link Browser Session and reference the lifecycle SPI while remaining invisible to the fixed directory glob. Structural RED `93635d092cfcaf613e88770da003b45b6018fa23` adds a hostile workspace member outside `crates/*` and demonstrates that escape. Minimal repair `9aca127b8ae18a6af38353c4023a6c5361c75e85` parses root `[workspace].members`, verifies each explicit member manifest exists, derives production Rust scanning from those members, and fails closed on workspace-member glob syntax until the contract is deliberately extended. The current repository already uses an explicit workspace-member list, so this widens review coverage without changing production Rust or the trust classification.

A sixth review checked Cargo's workspace-root package rule rather than assuming every package must appear in `[workspace].members`. If the workspace root later gains a `[package]` table, that root package is part of the workspace even when `members = []`; the fifth-generation helper would have ignored the root manifest and `src/**/*.rs`, allowing a root package to link Browser Session or reference/bind its lifecycle SPI without entering the review surface. Structural RED `89f1ce04a7ba4dfbb8157a849e419f842cb1a18c` adds that hostile root-package fixture. Minimal repair `36f13665553323f70f44f25a6bdf52a0b4b178ac` includes the root manifest whenever `[package]` is present while preserving explicit-member validation and the member-glob fail-closed rule. The repository is currently a virtual workspace, so this is prospective fail-closed coverage rather than a production topology change.

These repairs change no Rust production behavior or trust classification; they make the existing single-writer/TCB policy enforceable across ordinary Rust spelling, Cargo aliasing, workspace inheritance, module-layout choices, explicit workspace-member placement, workspace-root package placement, and future composition changes.

## Authority invariant

A `DisposableContextHandle` remains lifecycle addressability, not standalone authority. Browser Session alone owns `PresentationMutationAuthority` issuance and validates session incarnation, isolation identity, browsing-context identity, monotonic epoch and lifecycle state before later adapter I/O. `BrowserSessionIncarnation` is part of the authority binding and provides sequential-ABA protection in authorization validation; the isolation identity does not carry that responsibility by itself.

A reviewed adapter must create a fresh disposable browser boundary, keep remote addressability scoped to the same Browser Session incarnation, settle creation only for the exact aggregate-issued attempt, and prove exact-boundary destruction. Command acknowledgement alone is not creation, ownership, destruction or browser-observed post-condition evidence.

## Rejected fixes

- **Caller-visible nonce or opaque request as adapter authentication:** rejected because the implementation receives the value and can echo it without performing browser I/O.
- **Generic public `TrustedPort` marker trait:** rejected because arbitrary Rust code can implement an unsealed marker and the name creates no security property.
- **Sealing `DisposableContextPort` inside `originweave-browser-session`:** not adopted because the canonical versioned browser adapter lives in a separate crate; Rust has no friend-crate visibility, so sealing here would either break the adapter boundary or force protocol code into the Browser Session owner.
- **Hard-coding `crates/*` as the production composition boundary:** rejected because Cargo workspace/package membership, not directory placement, determines which production crates are built together.
- **Moving deterministic browser policy into WebDriver BiDi/MCP:** rejected; adapters translate qualified browser state and never become policy authority.
- **`--no-sandbox` or browser-process weakening:** unrelated and forbidden.

## Remaining acceptance

This dossier resolves the threat-model ambiguity; it does not by itself make #317 merge-ready. Before the Browser Session stack advances:

- the active branch must inherit every still-valid #229 delta by ordinary non-force adoption;
- `ARCHITECTURE.md` must explicitly include `BrowserSessionIncarnation` in `PresentationMutationAuthority` binding and sequential-ABA responsibility;
- any reviewed production composition path and versioned BiDi adapter must introduce its source, crate dependency, and allowlist widening together on the exact reviewed tree rather than relying on a reserved future entry;
- any future change from explicit Cargo workspace members to member globs, or introduction of a root package, must remain inside this security contract's reviewed manifest/source surface rather than silently widening trust;
- exact-head repository/security checks and independent review must pass with no unresolved authority finding;
- pinned Chromium must later prove create/use/post-condition/destroy behavior rather than treating a command ACK as success.
