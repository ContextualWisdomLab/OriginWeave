# Browser Session lifecycle authority trace

- Status: IMPLEMENTED_ON_ACTIVE_PR
- Owning bounded context: `originweave-browser-session`
- Governing proposal: ADR 0114
- Requirement owner: issue #312
- Integration prerequisites: #229 presentation-ownership witnesses; #314/#316 WebDriver BiDi ACL after this foundation is exact-head GREEN

## Problem and invariant

Browser-session, user-context/isolation, browsing-context, and adapter-selected identifiers are addresses. They are not evidence that the current Browser Session aggregate exclusively owns lifecycle or presentation mutation. A retained authority must not regain meaning if a later aggregate reuses the same remote identifiers and local epoch, and a caller must not be able to redirect a valid lifecycle request into a second adapter instance by choosing or replaying an adapter id.

The active implementation establishes this chain:

```text
validated BrowserSessionId
→ BrowserSession::start allocates non-reused BrowserSessionIncarnation
→ BrowserSession::bind_lifecycle_port consumes one concrete DisposableContextPort
→ BoundBrowserSession<P> owns aggregate + exact port; binding invokes no adapter callback
→ aggregate validates Active + reserves monotonic context epoch
→ aggregate privately constructs DisposableContextCreateRequest(session, incarnation)
→ exact owned port creates task-owned isolation boundary + browsing context
→ aggregate records exact handle + epoch
→ opaque PresentationMutationAuthority(session, incarnation, isolation, context, epoch)
→ exact authority validation before destroy I/O
→ aggregate privately constructs DisposableContextDestroyRequest(session, incarnation, stored handle)
→ exact owned port proves destruction
→ context Destroyed
→ normal BrowserSession end admitted
```

`BoundBrowserSession` is the lifecycle composition boundary. Public create/destroy methods accept no arbitrary port argument, no mutable port accessor is exposed, and `DisposableContextPort` has no identity-preflight callback. The previous public `DisposableContextPortId`/`port_id()` design was removed because the value was self-asserted and the callback itself could have side effects before authority.

`DisposableContextCreateRequest` and `DisposableContextDestroyRequest` have private construction paths. Their getters expose only addressability needed by a reviewed adapter. A caller that knows those values cannot reconstruct the request.

## Lossless recovery evidence

`DisposableContextCreateError::CreateFailedClean` is valid only when no disposable browser state exists. `CreateFailedUncertain(Some(isolation))` retains the exact known user-context/isolation identity as `BrowserSessionRecoveryEvidence::PartialCreationIsolation`; `None` remains representable when no identity was obtained. Both uncertain cases enter `RecoveryRequired` and mint no authority.

Duplicate browsing-context or isolation output stores the complete offending `DisposableContextHandle` as `DuplicateAdapterHandle` before recovery quarantine. Failed or unproven destruction records `UnprovenDestruction` with the exact owned handle. Recovery evidence authorizes no browser command.

Protocol-specific complete BiDi tuples, pending → accepted/quarantined mapping, and remote target liveness remain #314/#316 responsibilities; they are not copied into Browser Session domain truth.

## Orthogonal transport liveness

Transport liveness is tracked independently from ownership recovery. If transport loss occurs after `RecoveryRequired`, the aggregate keeps `RecoveryRequired`, preserves all recovery evidence, and separately records `transport_lost = true`. The first loss report is observable; repeated reports are idempotent. If loss occurs while `Active`, the lifecycle state becomes `TransportLost` and active context records become uncertain.

## Sequential ABA safety

Aggregate A may create `(S,U,C,epoch=1)`, prove destruction, and end. Aggregate B can later start with the same external `S`; the browser may return the same `U/C`, and B also begins at local epoch 1. A's retained authority still fails before B adapter I/O because B has a different `BrowserSessionIncarnation`.

The bound port receives the incarnation inside aggregate-issued create/destroy requests. The caller cannot replace the bound adapter after creation to reinterpret that current incarnation against a different adapter-local map.

## Standards trace

The design dossier references the 9 September 2026 WebDriver BiDi Working Draft. A user context has a user-context id set on creation. `browser.createUserContext` creates it, `browsingContext.create` can create a browsing context inside it, and `browser.removeUserContext` removes the selected user context after closing its navigables.

OriginWeave does not turn that protocol identifier into policy authority or assume historical non-reuse after removal. `DisposableIsolationId` remains lifecycle addressability. A successful command ACK is insufficient evidence that the disposable boundary is actually gone.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| lifecycle port ownership is structural | `BoundBrowserSession`; `bound_port_is_structural_and_not_swappable` |
| binding performs no arbitrary adapter callback | `BrowserSession::bind_lifecycle_port`; `lifecycle_binding_invokes_no_adapter_callback_before_authorized_create` |
| no self-asserted adapter id authority | absence of `DisposableContextPortId` / `port_id()`; repository contract |
| create/destroy requests are aggregate-issued | `DisposableContextCreateRequest`; `DisposableContextDestroyRequest`; `aggregate_issued_request_is_reachable_only_through_owned_port_binding` |
| raw context cannot mint presentation authority | `BrowserSession::presentation_authority`; `bound_creation_is_the_only_raw_context_entry_to_authority` |
| authority includes non-reused BrowserSessionIncarnation | `PresentationMutationAuthority`; `sequential_incarnation_reuse_rejects_stale_authority` |
| lossless recovery evidence for known partial identity | `BrowserSessionRecoveryEvidence`; `creation_failure_preserves_known_recovery_identity` |
| duplicate adapter handle retained without speculative cleanup | `create_disposable_context_with_port`; `duplicate_adapter_output_preserves_offending_handle` |
| unproven destruction retains exact handle | `destroy_disposable_context_with_port`; `destroy_failure_requires_recovery_before_any_new_authority` |
| transport liveness remains orthogonal to recovery | `BrowserSession::record_transport_loss`; `destroy_failure_retains_handle_and_transport_loss_orthogonally` |
| sequential ABA authority is rejected before I/O | `BrowserSession::context_for_authority_mut`; `stale_authority_cannot_cross_sequential_session_incarnations` |
| normal end requires proved destruction | `BrowserSession::end`; `normal_end_requires_proven_destruction_and_ignores_late_transport_report` |
| incarnation exhaustion fails closed | `allocate_incarnation`; `incarnation_allocator_fails_closed_before_wrap` |

The pre-authority adapter-callback RED was captured on exact `d43a4d86c8487ebdb9db9f1c4650fb7ee6225afc` in CI `34524654914`: the hostile fixture observed one identity callback where zero was required. The same predecessor also retained the self-selected scalar identity defect. The bound-session successor must earn fresh exact-head formatting, tests, Clippy, rustdoc, and function/line/region/branch 100% evidence; historical GREEN does not transfer.

Protected-main integration is required before capability maturity can be promoted beyond `IMPLEMENTED_ON_ACTIVE_PR`.

## Buyer acceptance still open

This slice does not yet prove actual WebDriver BiDi `browser.createUserContext`/`browsingContext.create` integration, observed `browser.removeUserContext` post-condition, protocol-specific pending/accepted/quarantined binding, separately authorized recovery reconciliation, Browser Session authority conversion into BiDi presentation private witnesses, pinned Chromium post-condition observation, crash/process-restart reconciliation, #299 3/3 browser trials, or protected-main release/SBOM/provenance/reproducibility/rollback.

## Reference

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
