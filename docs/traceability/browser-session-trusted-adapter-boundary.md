# Browser Session trusted-adapter boundary

- **Status:** active-PR security and composition evidence for PR #317; not protected-main behavior
- **Owner:** OriginWeave Browser Session bounded context
- **Related authority:** `docs/THREAT_MODEL.md`, `SECURITY.md`, ADR 0114, issue #312

## Problem

`DisposableContextPort` is a cross-crate service-provider interface. Its implementation is allowed to return the browser-issued isolation and browsing-context address that Browser Session records before minting `PresentationMutationAuthority`. The Rust type system cannot distinguish a reviewed implementation from malicious code merely because both implement the same public trait. Aggregate-issued request/completion correlation, incarnation binding, duplicate rejection, exact-fact recovery, and monotonic epochs prevent replay, swapping and ABA classes; **request/completion correlation is not adapter authentication**.

Treating an arbitrary in-process implementation as if it were an untrusted web actor would therefore create a false security promise. A nonce or opaque request handed to that implementation can simply be echoed. It does not prove that Chromium created a disposable user-context boundary.

## Trust boundary

OriginWeave's threat model places the Rust control plane and privileged Chromium/browser adapters inside the trusted computing base. `originweave-browser-session` is an internal `publish = false` crate, not an extension SDK that promises isolation from hostile linked Rust code. A malicious crate already executing inside this trusted process is a supply-chain compromise / trusted-code compromise; it is not made safe by making one handle constructor opaque.

This does **not** make every implementation acceptable. Product composition may bind only a reviewed privileged lifecycle adapter; a caller-selected production adapter is not admitted. `DisposableContextPort` is an internal TCB SPI, not caller-selected product policy. Any production source that references this SPI outside the Browser Session owner and any production crate that depends on `originweave-browser-session` is a repository review surface and must be explicitly allowlisted by contract. Test doubles remain allowed only under test code and grant no shipped product capability.

The intended canonical production consumer is the versioned WebDriver BiDi lifecycle adapter in `crates/originweave-bidi/src/lifecycle_acl.rs` once its stack is restacked onto the current Browser Session contract and passes review. Its crate manifest is the only reserved external Browser Session dependency. No other external production reference or crate dependency is admitted by this dossier.

## Enforced repository contract

`tests/test_browser_session_trusted_adapter_boundary.py` enforces the currently supportable boundary:

1. the Browser Session crate remains `publish = false`;
2. the canonical threat model continues to classify privileged browser integration as trusted Zone C code;
3. production references to `DisposableContextPort` outside the Browser Session owner are limited to the explicit reviewed source allowlist;
4. production crate dependencies on `originweave-browser-session` are limited to the explicit reviewed manifest allowlist; and
5. no current production source outside the Browser Session owner references `bind_lifecycle_port` as a caller-selected composition escape hatch, regardless of method-call, UFCS, or whitespace spelling.

The fifth rule intentionally leaves the product composition owner unclaimed until a reviewed runtime/composition lane exists. When that owner is introduced, its exact path must be added deliberately with architecture and security review rather than discovered implicitly through a new call site.

### Scanner false-negative repair

The first repository contract recognized only the literal unqualified Rust form `impl DisposableContextPort for ...` and the exact method spelling `.bind_lifecycle_port(`. Those are style conventions, not security boundaries: valid Rust can name the trait through a qualified path or alias and can invoke the binding function through UFCS or with different whitespace.

Test-first commit `100c00487488bbc281106ddf6fe4ae1b60feb16b` adds hostile qualified-trait, aliased-trait, UFCS, and whitespace spellings and exposes those false negatives. Minimal contract repair `7b2334b1df92d03629b7931ce71cd58134b93f4c` makes direct source review spelling-resilient: any external production source containing the SPI token is reviewed, and any production source outside the owner containing the binding API token is rejected until an explicit composition owner is approved.

A second review found the remaining cross-file alias case: one reviewed module could import or re-export the trait under another name while a different module implements only that alias and therefore contains no `DisposableContextPort` token. Test-first commit `6ce9c1f3b13fe157cfae822a0958c2b8dc2dabd8` records that direct source scanning cannot prove this case. Commit `cb9fb54e4a799919a425f3636cb0a5f1daacfb24` adds the compensating crate-boundary invariant: every production `Cargo.toml` that can link Browser Session must itself be reviewed and allowlisted. A cross-file alias therefore cannot create a new production adapter from an unreviewed crate without first widening an explicit dependency review surface.

These repairs change no Rust production behavior or trust classification; they make the existing single-writer/TCB policy enforceable across ordinary Rust spelling and module-layout choices.

## Authority invariant

A `DisposableContextHandle` remains lifecycle addressability, not standalone authority. Browser Session alone owns `PresentationMutationAuthority` issuance and validates session incarnation, isolation identity, browsing-context identity, monotonic epoch and lifecycle state before later adapter I/O. `BrowserSessionIncarnation` is part of the authority binding and provides sequential-ABA protection in authorization validation; the isolation identity does not carry that responsibility by itself.

A reviewed adapter must create a fresh disposable browser boundary, keep remote addressability scoped to the same Browser Session incarnation, settle creation only for the exact aggregate-issued attempt, and prove exact-boundary destruction. Command acknowledgement alone is not creation, ownership, destruction or browser-observed post-condition evidence.

## Rejected fixes

- **Caller-visible nonce or opaque request as adapter authentication:** rejected because the implementation receives the value and can echo it without performing browser I/O.
- **Generic public `TrustedPort` marker trait:** rejected because arbitrary Rust code can implement an unsealed marker and the name creates no security property.
- **Sealing `DisposableContextPort` inside `originweave-browser-session`:** not adopted because the canonical versioned browser adapter lives in a separate crate; Rust has no friend-crate visibility, so sealing here would either break the adapter boundary or force protocol code into the Browser Session owner.
- **Moving deterministic browser policy into WebDriver BiDi/MCP:** rejected; adapters translate qualified browser state and never become policy authority.
- **`--no-sandbox` or browser-process weakening:** unrelated and forbidden.

## Remaining acceptance

This dossier resolves the threat-model ambiguity; it does not by itself make #317 merge-ready. Before the Browser Session stack advances:

- the active branch must inherit every still-valid #229 delta by ordinary non-force adoption;
- `ARCHITECTURE.md` must explicitly include `BrowserSessionIncarnation` in `PresentationMutationAuthority` binding and sequential-ABA responsibility;
- the reviewed production composition path and the versioned BiDi adapter must satisfy the source-reference and crate-dependency allowlist contracts when introduced/restacked;
- exact-head repository/security checks and independent review must pass with no unresolved authority finding;
- pinned Chromium must later prove create/use/post-condition/destroy behavior rather than treating a command ACK as success.
