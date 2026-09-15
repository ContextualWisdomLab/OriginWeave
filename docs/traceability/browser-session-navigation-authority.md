# Browser Session navigation authority trace

- Status: `IMPLEMENTED_ON_ACTIVE_PR`
- Owning bounded context: `originweave-browser-session`
- Production owner: #317
- Acceptance successors: #318, #321
- Protocol adapter / WebDriver BiDi correlation owner: #316
- Governing proposals: ADR 0114, ADR 0116
- Standards provenance: `WD-webdriver-bidi-20260909`

## Domain boundary

Browser Session owns deterministic navigation authority state for an already-owned browsing context. WebDriver BiDi navigation ids, remote context ids, protocol event order, replay qualification, transport liveness, and pending/accepted/quarantined adapter tuples remain #316 concerns. A raw protocol id or an LLM judgment cannot manufacture Browser Session authority.

The active #317 production lineage implements this contract:

```text
Active Browser Session
+ exact BrowserSessionIncarnation
+ exact live BrowsingContextId
+ exact current BrowserContextEpoch
        |
        | record_observed_navigation(...)
        | zero adapter I/O; no presentation epoch spent
        v
opaque NavigationSettlementAuthority
+ monotonic navigation_generation
+ presentation authority revoked
        |
        +-- first qualified commit -----------------------> Pending(committed=true)
        |                                                   |
        +-- positive complete observation -----------------+
        +-- typed Aborted / Failed ------------------------+--> Eligible
        +-- download start --------------------------------+       |
                                                                | explicit, single-use
                                                                | reestablish_presentation_authority
                                                                | reserves next BrowserContextEpoch
                                                                v
                                                            Established
```

A newer qualified navigation start supersedes an older pending witness or unused eligibility for the same owned context without spending a presentation epoch. Old, foreign, cross-context, cross-incarnation, destroyed-generation, and superseded witnesses fail closed.

## Invariants and source mapping

| Invariant | Owner source / evidence |
|---|---|
| Navigation admission validates aggregate trust, incarnation, live ownership, and exact context epoch before mutation | `BrowserSession::begin_observed_navigation`; owner hostile tests |
| Admission performs zero adapter I/O and does not spend a presentation epoch | `BoundBrowserSession::record_observed_navigation`; presentation-epoch conservation tests |
| Browser Session, not adapter ids, mints terminal authority | private fields of `NavigationSettlementAuthority`; `tests/test_browser_session_navigation_owner_surface_contract.py` |
| Navigation generation is monotonic and independent from presentation epoch | `next_navigation_generation`; generation exhaustion tests |
| First commit is non-terminal | `mark_observed_navigation_committed`; #318 commit tests |
| Positive settlement, typed negative termination, and download start share one current-witness terminal closure | `close_observed_navigation`; #318 terminal/download tests |
| Duplicate or superseded commit/terminal evidence is non-authorizing | `current_pending_navigation_mut`; #318 replay tests |
| Terminal closure makes one context-local re-establishment opportunity | `PresentationNavigationState::Eligible`; #318 sibling/context-local tests |
| Re-establishment is explicit, single-use, and the only navigation transition that spends the next presentation epoch | `reestablish_presentation_authority_for_context`; #318 epoch/single-use tests |
| Navigation-invalidated presentation authority cannot authorize mutation or authority-based cleanup | `context_for_authority_mut`; cleanup hostile tests |
| Exact lifecycle owner may still clean up the retained context without reopening presentation authority | `destroy_owned_disposable_context`; cleanup hostile tests |
| RecoveryRequired and TransportLost dominate stale capability inspection | `require_active`; recovery/liveness tests |
| Proven destruction removes only that exact owned generation; sibling progress/eligibility survives | hot ownership map removal; #318/#321 sibling tests |
| Same raw context recreated later cannot inherit prior navigation or presentation authority | monotonic context epoch + session incarnation; #321 ABA matrix |

## Adapter anti-corruption boundary

#316 may correlate `browsingContext.navigationStarted`, `navigationCommitted`, completion/abort/failure/download observations, remote context destruction, and transport/session loss to Browser Session commands only after protocol qualification. It must retain protocol identifiers for addressability and provenance, not promote them into policy authority.

The adapter must bind a remote candidate to the exact aggregate-issued create attempt before accepted state becomes authorizing. Navigation evidence for a rejected, quarantined, superseded, destroyed, or prior-incarnation tuple cannot rewrite Browser Session state. Silent rebind after transport/session loss is prohibited.

Browser Session does not own Chromium/WebDriver transport truth. #316 does not own Browser Session policy truth. No source copy, cross-service SQL, mutable dependency, or duplicate state machine is permitted across this ACL.

## Acceptance state

#317 production semantics are implemented on an active PR, not protected-main shipment. #318 owns the broad navigation acceptance matrix and doctoring; #321 owns same-raw-id/sibling-recreation ABA acceptance. Those child PRs must remain ordinary non-force descendants of the exact #317 owner and do not replace owner-side contracts.

Repository GREEN requires current exact-head repository contracts, canonical formatting, locked Rust tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage at 100%, and applicable protected checks. Queued, skipped, predecessor, or runner-less jobs are not GREEN.

Real-browser GREEN is separate. A protocol command acknowledgement is insufficient. Acceptance requires pinned Chromium/WebDriver BiDi evidence for navigation, policy-authorized interaction, browser/page-observed post-condition, reset, destruction/cleanup, crash/recovery behavior, and provenance on the current integrated head.

## Standards trace

The immutable W3C WebDriver BiDi Working Draft used for this active-PR contract is the 9 September 2026 publication: `https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/`. Protocol event vocabulary and user-context/browsing-context addressability come from that standard; OriginWeave's opaque authority and lifecycle invariants are internal domain controls and are not claimed as W3C requirements.

## Release status

Status remains `IMPLEMENTED_ON_ACTIVE_PR`. Do not promote it to Accepted, released, or buyer-complete until protected-main integration and immutable release evidence exist. #316 integration and real pinned-Chromium post-condition evidence remain open.
