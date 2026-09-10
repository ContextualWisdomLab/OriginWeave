# Browser Session lifecycle authority trace

- Status: IMPLEMENTED_ON_ACTIVE_PR
- Owning bounded context: `originweave-browser-session`
- Governing proposal: ADR 0114
- Requirement owner: issue #312
- Integration prerequisites: #229 presentation-ownership witnesses; canonical browser/sandbox owner path under #212/#148

## Problem and invariant

Browser-session and browsing-context identifiers are addresses. They are not evidence that the current Browser Session aggregate exclusively owns presentation mutation or cleanup. They may also be reused across separate aggregate incarnations, so an aggregate-local epoch does not by itself prevent cross-aggregate authority aliasing.

The active implementation establishes this fail-closed chain:

```text
validated BrowserSessionId
→ BrowserSession::start
→ DisposableContextPort creates a fresh task-owned isolation boundary + browsing context
→ adapter returns DisposableIsolationId + BrowsingContextId
→ aggregate records exact isolation handle + monotonic context epoch
→ opaque PresentationMutationAuthority(session, isolation, context, epoch)
→ exact-authority validation before adapter I/O
→ destruction receives the stored isolation handle, not reconstructed session/context authority
→ adapter proves exact disposable boundary destruction
→ context state Destroyed
→ normal BrowserSession::end is admitted
```

Creation failure is also causal evidence. `CreateFailedClean` is allowed only when the adapter proves that no disposable browser state was created. `CreateFailedUncertain`, duplicate browsing-context output, duplicate isolation output, or an invalid creation-time error enters `RecoveryRequired`, marks active owned contexts uncertain, and blocks all active-only transitions. This prevents a partial create from being followed by a false normal `end()`.

A raw `BrowsingContextId`, stale epoch, foreign session, foreign isolation, unknown context, destruction failure, lost transport, or recovery-required session cannot enter the successful chain. Destruction failure and transport loss invalidate active authority rather than treating a remote acknowledgement as cleanup evidence.

## Standards trace

The latest published WebDriver BiDi Working Draft at the time of this decision is 9 September 2026. A user context has a user-context id defined as a unique string set on creation. The browser module defines `browser.createUserContext`; `browsingContext.create` accepts a `userContext`; and `browser.removeUserContext` removes the selected user context after closing its navigables.

OriginWeave does not make the protocol identifier itself a policy authority. `DisposableIsolationId` is lifecycle addressability carried through the domain so cleanup cannot be reconstructed from aliasable session/context identifiers. A WebDriver BiDi implementation of `DisposableContextPort` must map the isolation id one-to-one to the specification-defined unique user-context id and must prove removal of that exact boundary. An unchecked random adapter token without that browser-lifecycle mapping does not satisfy the port contract. A successful command ACK is insufficient evidence that the disposable boundary is actually gone.

The active `originweave-bidi` adapter remains runtime-qualified against its separately documented 3 September 2026 revision. Tracking the 9 September publication here does not silently repin that runtime contract.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| raw context cannot mint authority | `BrowserSession::presentation_authority`; `disposable_creation_is_the_only_raw_context_entry_to_authority` |
| authority is session/isolation/context/epoch bound | `PresentationMutationAuthority`; `epoch_advance_invalidates_old_and_cross_session_authority` |
| same external session/context/epoch cannot cross aggregate isolation | `BrowserSession::context_for_authority_mut`; `two_aggregate_alias_cannot_cross_mutation_or_destruction_boundary` |
| destruction is scoped by the already-validated stored isolation handle | `BrowserSession::destroy_disposable_context`; `two_aggregate_alias_cannot_cross_mutation_or_destruction_boundary` |
| proved-clean versus uncertain creation is typed | `DisposableContextPortError`; `creation_failure_is_typed_clean_or_recovery_required` |
| duplicate adapter output requires recovery | `BrowserSession::create_disposable_context`; `duplicate_adapter_output_requires_recovery` |
| cleanup failure invalidates authority | `BrowserSession::destroy_disposable_context`; `destroy_failure_quarantines_authority_and_transport_loss_is_idempotent` |
| transport loss invalidates active contexts | `BrowserSession::record_transport_loss`; `transport_loss_invalidates_still_active_contexts` |
| normal end requires proved destruction | `BrowserSession::end`; `successful_destruction_is_required_before_normal_end` |

The 10 September 2026 exact-head RED on predecessor `6486e916dceb4ab5f33f7b390cd76fd4673d6007` is part of this trace: CI `34440868057` failed rustfmt and exact coverage. The coverage artifact `10138258867` (`sha256:dc38bd6a2a2cb307f6b3fd34332cac04a71aa47e4bae83173afa00a99a85adea`) isolated two unexecuted `DisposableContextHandle` accessors and a structurally unreachable second context lookup after authority validation. The repair exercises the accessors and retains one validated mutable record across destroy I/O instead of testing or excluding an impossible branch.

Exact-head CI/coverage for the repair is required before this dossier can be cited as verified active-PR implementation. Protected-main integration is required before any capability maturity is promoted beyond `IMPLEMENTED_ON_ACTIVE_PR`.

## Buyer acceptance still open

This slice does not yet prove:

- actual WebDriver BiDi `browser.createUserContext`/`browsingContext.create` integration and one-to-one `DisposableIsolationId` mapping;
- observed `browser.removeUserContext` post-condition for the exact owned isolation boundary;
- reconciliation of `RecoveryRequired` after a partial create or duplicate response;
- conversion of domain authority into the BiDi presentation/screen-area private witnesses;
- pinned Chromium post-condition observation after presentation mutation;
- browser crash/restart reconciliation of uncertain disposable contexts;
- 3/3 complete #299 Agent Task browser trials;
- protected-main release, SBOM, provenance, reproducibility, or rollback evidence.

## Reference

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
