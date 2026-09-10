# Browser Session lifecycle authority trace

- Status: IMPLEMENTED_ON_ACTIVE_PR
- Owning bounded context: `originweave-browser-session`
- Governing proposal: ADR 0114
- Requirement owner: issue #312
- Integration prerequisites: #229 presentation-ownership witnesses; #314/#316 WebDriver BiDi ACL after this foundation is exact-head GREEN

## Problem and invariant

Browser-session, user-context/isolation, browsing-context, and adapter-selected identifiers are addresses. They are not evidence that the current Browser Session aggregate exclusively owns lifecycle or presentation mutation.

The active implementation establishes this chain:

```text
validated BrowserSessionId
→ BrowserSession::start allocates non-reused BrowserSessionIncarnation
→ BrowserSession::bind_lifecycle_port consumes one concrete DisposableContextPort
→ BoundBrowserSession<P> owns aggregate + exact port; no public raw port accessor exists
→ aggregate validates Active + reserves monotonic BrowserContextEpoch
→ aggregate privately constructs DisposableContextCreateRequest(session, incarnation, attempt epoch)
→ exact owned port creates a remote candidate but must keep it non-authorizing
→ aggregate validates returned isolation/context against current ownership
→ aggregate privately constructs DisposableContextCreateCompletion(attempt, Accepted|Rejected)
→ accepted candidate may become adapter-authorizing; rejected candidate remains quarantined
→ aggregate records accepted exact handle + epoch
→ opaque PresentationMutationAuthority(session, incarnation, isolation, context, epoch)
→ exact authority validation before destroy I/O
→ aggregate privately constructs DisposableContextDestroyRequest(session, incarnation, stored handle)
→ exact owned port proves destruction
→ context Destroyed
→ normal BrowserSession end admitted
```

`BoundBrowserSession` is the lifecycle composition boundary. Public create/destroy methods accept no arbitrary port argument, and there is **no public raw port accessor**. Application code cannot recover `&P` and invoke an inherent shared-reference method with interior mutation or remote I/O.

`DisposableContextCreateRequest`, `DisposableContextCreateCompletion`, and `DisposableContextDestroyRequest` have private construction paths. The create request carries the already-reserved context epoch as a **per-create transaction** identity. The exact attempt is settled only after Browser Session validates the returned handle.

## Transactional remote creation

A protocol adapter may stage a successful remote create result as pending when it receives the create request. It must not make that result authorizing yet.

Browser Session examines the returned `DisposableContextHandle`:

- if ownership validation succeeds, `DisposableContextCreateCompletion::Accepted` settles that exact attempt before normal presentation authority is returned;
- if the handle aliases an existing isolation or browsing context, `Rejected` settles that exact attempt and the aggregate enters `RecoveryRequired`;
- if exact completion cannot be proven, Browser Session stores the complete handle as `UnsettledAdapterHandle`, enters recovery, and mints no normal authority.

Protocol-specific tuple contents and pending/accepted/quarantined storage remain #314/#316 responsibilities. Browser Session owns only the attempt identity, domain validation, and accept/reject decision.

## Lossless recovery evidence

`CreateFailedClean` is valid only when no disposable browser state exists. `CreateFailedUncertain(Some(isolation))` retains the exact known isolation identity. Duplicate output stores the complete offending handle. Completion failure retains an unsettled complete handle. Failed or unproven destruction records the exact owned handle. None of this evidence grants browser command authority.

## Orthogonal transport liveness

Transport liveness is tracked independently from ownership recovery. If transport loss occurs after `RecoveryRequired`, the aggregate keeps recovery evidence and separately records `transport_lost = true`. Repeated loss reports are idempotent.

## Sequential ABA safety

Aggregate A may create `(S,U,C,epoch=1)`, prove destruction, and end. Aggregate B can later start with the same external values and also begin at epoch 1. A's retained authority still fails because B has a different `BrowserSessionIncarnation`. The bound port receives the incarnation inside aggregate-issued lifecycle capabilities.

## Standards trace

The design dossier references the 9 September 2026 WebDriver BiDi Working Draft. `browser.createUserContext` creates a user context, `browsingContext.create` can create a browsing context inside it, and `browser.removeUserContext` removes the selected user context after closing its navigables.

OriginWeave does not treat those protocol identifiers as policy authority or assume historical non-reuse after removal. A command ACK is insufficient proof that the disposable boundary is actually gone.

## Source and executable evidence

| Invariant | Source / test |
|---|---|
| independent Browser Session bounded context | `crates/originweave-browser-session/`; `tests/test_browser_session_lifecycle_contract.py` |
| lifecycle port ownership is structural | `BoundBrowserSession`; `bound_port_is_structural_and_not_swappable` |
| no public raw port accessor | absence of `BoundBrowserSession::lifecycle_port`; repository contract |
| binding performs no arbitrary adapter callback | `BrowserSession::bind_lifecycle_port`; `lifecycle_binding_invokes_no_adapter_callback_before_authorized_create` |
| no self-asserted adapter id authority | absence of `DisposableContextPortId` / `port_id()` |
| create requests are aggregate-issued and attempt-scoped | `DisposableContextCreateRequest::attempt_epoch`; transaction hostile fixture |
| per-create transaction settles accepted/rejected candidates | `DisposableContextCreateCompletion`; `accepted_and_rejected_create_candidates_are_correlated_by_exact_attempt` |
| completion failure fails closed | `UnsettledAdapterHandle`; internal completion-failure tests |
| raw context cannot mint presentation authority | `BrowserSession::presentation_authority`; `bound_creation_is_the_only_raw_context_entry_to_authority` |
| sequential ABA authority is rejected before I/O | `BrowserSessionIncarnation`; `stale_authority_cannot_cross_sequential_session_incarnations` |
| lossless recovery evidence | `BrowserSessionRecoveryEvidence`; recovery tests |
| unproven destruction retains exact handle | `destroy_failure_requires_recovery_before_any_new_authority` |
| transport liveness remains orthogonal | `BrowserSession::record_transport_loss` |
| normal end requires proved destruction | `BrowserSession::end` |
| incarnation exhaustion fails closed | `allocate_incarnation` |

Exact `9cde981899950b900698a17e7fa739af59f6bb4f` / CI `34531025582` is historical RED for this successor: production exact coverage passed, but canonical formatting failed, and the raw port accessor plus missing transaction completion remained. Historical GREEN never transfers.

Protected-main integration is required before capability maturity can be promoted beyond `IMPLEMENTED_ON_ACTIVE_PR`.

## Buyer acceptance still open

This slice does not yet prove actual WebDriver BiDi lifecycle integration, observed removal post-condition, protocol-specific pending/accepted/quarantined binding, separately authorized recovery reconciliation, Browser Session authority conversion into BiDi presentation private witnesses, Chromium post-condition observation, crash/process-restart reconciliation, #299 3/3 browser trials, or protected-main release/SBOM/provenance/reproducibility/rollback.

## Reference

Browser Testing and Tools Working Group. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
