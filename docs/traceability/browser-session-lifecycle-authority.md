# Browser Session lifecycle authority trace

- Status: IMPLEMENTED_ON_ACTIVE_PR
- Owning bounded context: `originweave-browser-session`
- Governing proposal: ADR 0114
- Requirement owner: issue #312
- Integration prerequisites: #229 presentation-ownership witnesses; canonical browser/sandbox owner path under #212/#148

## Problem and invariant

Browser-session, user-context/isolation, and browsing-context identifiers are addresses. They are not evidence that the current Browser Session aggregate exclusively owns presentation mutation or cleanup. A retained authority must not regain meaning if a later aggregate reuses the same remote identifiers and local epoch.

The active implementation now establishes this chain:

```text
validated BrowserSessionId
→ BrowserSession::start allocates non-reused BrowserSessionIncarnation
→ DisposableContextPort receives session id + incarnation
→ adapter creates fresh task-owned isolation boundary + browsing context
→ adapter returns DisposableIsolationId + BrowsingContextId
→ aggregate records exact handle + monotonic context epoch
→ opaque PresentationMutationAuthority(session, incarnation, isolation, context, epoch)
→ exact authority validation before adapter I/O
→ destruction receives the same incarnation + stored handle
→ adapter proves exact disposable boundary destruction
→ context Destroyed
→ normal BrowserSession::end admitted
```

`BrowserSessionIncarnation` is process-local and monotonic. Presentation authority is not persisted across process restart, so restart invalidates outstanding authority rather than requiring a durable counter. Within one running process, the incarnation is checked by the aggregate and passed through the lifecycle port; an adapter that ignores it does not satisfy the ACL contract.

## Lossless recovery evidence

`DisposableContextCreateError::CreateFailedClean` is valid only when no disposable browser state exists. `CreateFailedUncertain(Some(isolation))` retains the exact known user-context/isolation identity as `BrowserSessionRecoveryEvidence::PartialCreationIsolation`; `None` remains representable when no identity was obtained. Both uncertain cases enter `RecoveryRequired` and mint no authority.

Duplicate browsing-context or isolation output stores the complete offending `DisposableContextHandle` as `DuplicateAdapterHandle` before recovery quarantine. OriginWeave deliberately does not auto-destroy duplicate output because ownership may be foreign. Failed or unproven destruction records `UnprovenDestruction` with the exact owned handle. Recovery evidence authorizes no browser command; it exists only for a later reviewed reconciliation path.

## Orthogonal transport liveness

Transport liveness is tracked independently from ownership recovery. If transport loss occurs after `RecoveryRequired`, the aggregate keeps `RecoveryRequired`, preserves all recovery evidence, and separately records `transport_lost = true`. The first loss report is observable; repeated reports are idempotent. If loss occurs while `Active`, the lifecycle state becomes `TransportLost` and active context records become uncertain.

This avoids conflating “ownership uncertain while transport may still be usable for separately authorized reconciliation” with “ownership uncertain and the transport is gone.”

## Sequential ABA safety

The sequential ABA hostile case is explicit: aggregate A creates `(S,U,C,epoch=1)`, proves destruction, and ends. Aggregate B later starts with the same external `S`; the adapter may return the same `U/C`, and B also begins at local epoch 1. A's retained authority must still fail before any B adapter I/O. B receives a different `BrowserSessionIncarnation`, and only B's newly minted authority is accepted.

The port also receives the incarnation on create/destroy. This closes the prior gap where an aggregate-only nonce could protect token comparison while the browser adapter still keyed destruction by aliasable raw identifiers.

## Standards trace

The design dossier references the 9 September 2026 WebDriver BiDi Working Draft. A user context has a user-context id set on creation. `browser.createUserContext` creates it, `browsingContext.create` can create a browsing context inside it, and `browser.removeUserContext` removes the selected user context after closing its navigables.

OriginWeave does not turn that protocol identifier into policy authority or assume historical non-reuse after removal. `DisposableIsolationId` remains lifecycle addressability. A successful command ACK is insufficient evidence that the disposable boundary is actually gone.

The active `originweave-bidi` adapter remains separately runtime-qualified against its documented 3 September 2026 revision. Tracking the 9 September publication here does not silently repin that runtime contract.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| raw context cannot mint authority | `BrowserSession::presentation_authority`; `disposable_creation_is_the_only_raw_context_entry_to_authority` |
| authority includes non-reused BrowserSessionIncarnation | `PresentationMutationAuthority`; `sequential_incarnation_reuse_rejects_stale_authority` |
| lifecycle port receives the same incarnation | `DisposableContextPort`; `stale_authority_cannot_cross_sequential_session_incarnations` |
| lossless recovery evidence for known partial identity | `BrowserSessionRecoveryEvidence`; `creation_failure_preserves_known_recovery_identity` |
| duplicate adapter handle retained without speculative cleanup | `BrowserSession::create_disposable_context`; `duplicate_adapter_output_preserves_offending_handle` |
| unproven destruction retains exact handle | `BrowserSession::destroy_disposable_context`; `destroy_failure_requires_recovery_before_any_new_authority` |
| transport liveness remains orthogonal to recovery | `BrowserSession::record_transport_loss`; `destroy_failure_retains_handle_and_transport_loss_orthogonally` |
| sequential ABA authority is rejected before I/O | `BrowserSession::context_for_authority_mut`; `stale_authority_cannot_cross_sequential_session_incarnations` |
| normal end requires proved destruction | `BrowserSession::end`; `normal_end_requires_proven_destruction_and_ignores_late_transport_report` |
| incarnation exhaustion fails closed | `allocate_incarnation`; `incarnation_allocator_fails_closed_before_wrap` |

Earlier exact-head evidence remains historical only. Exact `ab04f9522e97e1ecd6d914c48cb6f77f087eac3b` was repository GREEN in CI `34463908909` after repairing repository-contract drift, but it still contained the three Browser Session defects above.

The sequential ABA RED was then captured on exact `ec145963ad8fe19c9416f2b3856b94660082dbf7` in CI `34469580144`: Python repository contracts and canonical formatting passed; the Rust `Run tests` step failed at the newly added hostile sequential-incarnation test. That RED is the causal predecessor for the incarnation-aware domain/port repair. No earlier GREEN transfers to the repaired successor.

Protected-main integration is required before capability maturity can be promoted beyond `IMPLEMENTED_ON_ACTIVE_PR`.

## Buyer acceptance still open

This slice does not yet prove:

- actual WebDriver BiDi `browser.createUserContext`/`browsingContext.create` integration and incarnation-scoped mapping;
- observed `browser.removeUserContext` post-condition for the exact owned boundary;
- a separately authorized reconciliation service consuming `BrowserSessionRecoveryEvidence`;
- Browser Session authority conversion into BiDi presentation/screen-area private witnesses;
- pinned Chromium post-condition observation after presentation mutation;
- crash/process-restart reconciliation of uncertain disposable contexts;
- 3/3 complete #299 Agent Task browser trials;
- protected-main release, SBOM, provenance, reproducibility, or rollback evidence.

## Reference

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
