# Browser Session trusted-adapter boundary

- **Status:** active-PR security and composition evidence for PR #317; not protected-main behavior
- **Owner:** OriginWeave Browser Session bounded context
- **Related authority:** `docs/THREAT_MODEL.md`, `SECURITY.md`, ADR 0114, issue #312

## Problem

`DisposableContextPort` is a cross-crate service-provider interface. Its implementation is allowed to return the browser-issued isolation and browsing-context address that Browser Session records before minting `PresentationMutationAuthority`. The Rust type system cannot distinguish a reviewed implementation from malicious code merely because both implement the same public trait. Aggregate-issued request/completion correlation, incarnation binding, duplicate rejection, exact-fact recovery, and monotonic epochs prevent replay, swapping and ABA classes; **request/completion correlation is not adapter authentication**.

Treating an arbitrary in-process implementation as if it were an untrusted web actor would therefore create a false security promise. A nonce or opaque request handed to that implementation can simply be echoed. It does not prove that Chromium created a disposable user-context boundary.

## Trust boundary

OriginWeave's threat model places the Rust control plane and privileged Chromium/browser adapters inside the trusted computing base. `originweave-browser-session` is an internal `publish = false` crate, not an extension SDK that promises isolation from hostile linked Rust code. A malicious crate already executing inside this trusted process is a supply-chain compromise / trusted-code compromise; it is not made safe by making one handle constructor opaque.

This does **not** make every implementation acceptable. Product composition may bind only a reviewed privileged lifecycle adapter. `DisposableContextPort` is an internal TCB SPI, not caller-selected product policy. Production implementations are repository review surfaces and must be explicitly allowlisted by contract. Test doubles remain allowed only under test code and grant no shipped product capability.

The intended canonical production implementation is the versioned WebDriver BiDi lifecycle adapter in `crates/originweave-bidi/src/lifecycle_acl.rs` once its stack is restacked onto the current Browser Session contract and passes review. No other production implementation is admitted by this dossier.

## Enforced repository contract

`tests/test_browser_session_trusted_adapter_boundary.py` enforces the currently supportable boundary:

1. the Browser Session crate remains `publish = false`;
2. the canonical threat model continues to classify privileged browser integration as trusted Zone C code;
3. production `DisposableContextPort` implementations are limited to the explicit reviewed allowlist;
4. no current production source outside the Browser Session owner directly calls `.bind_lifecycle_port(...)` as a caller-selected composition escape hatch.

The fourth rule intentionally leaves the product composition owner unclaimed until a reviewed runtime/composition lane exists. When that owner is introduced, its exact path must be added deliberately with architecture and security review rather than discovered implicitly through a new call site.

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
- the reviewed production composition path and the versioned BiDi adapter must satisfy the repository allowlist contract when introduced/restacked;
- exact-head repository/security checks and independent review must pass with no unresolved authority finding;
- pinned Chromium must later prove create/use/post-condition/destroy behavior rather than treating a command ACK as success.
