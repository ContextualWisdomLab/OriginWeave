# Resolution Freshness Authority Trace

- **Documentation status:** Active-PR traceability
- **Protected-main capability status:** **PARTIAL**
- **Primitive implementation lane:** PR #47, `feat/resolution-freshness-authority-main`
- **First-party planning consumer lane:** PR #50, `feat/network-consume-resolution-freshness`
- **Socket-use freshness lane:** PR #54, `fix/network-resolution-freshness-at-use`
- **Governing existing decision boundary:** ADR 0004 and the protected-main destination/rebinding authority model
- **Buyer-visible gap:** bind the interval between a validated resolution answer and actual socket use so DNS-rebinding/TOCTOU exposure is explicit and fail-closed

## Truth boundary

Protected `main` already classifies, approves, pins, and non-expansively revalidates resolved destination addresses. It does **not** yet require a time-bounded resolution authority through the entire first-party direct-socket path.

PR #47 exact head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` contains the reusable production `FreshResolutionSnapshot` primitive and has terminal successful CI/security/SAST/exact-coverage evidence. That primitive is therefore **IMPLEMENTED_ON_ACTIVE_PR** evidence only; it is not protected-main truth.

PR #50 exact head `f8b43bc94444986ab23aa4ef3086e446a0b39295` implements the dependent first-party planning boundary. It keeps the untimed `ConnectionPlan` internal to `originweave-network`, exposes `FreshConnectionPlan` as the ordinary direct-socket planner, requires a `FreshResolutionSnapshot` plus caller-supplied trusted monotonic current time, rejects expired authority at plan authorization, and migrates existing TLS integration helpers through that same fresh boundary. Exact-head CI run `31408474576` passes repository contracts, formatting, workspace check/tests, strict Clippy, rustdoc and exact owned production function/line/region/branch coverage; CodeRabbit exact-head status is success.

PR #54 exact head `ec81031c537f2b662910c1ce78c7ae0e0bfc9c1e` closes a later plan-to-connect TOCTOU discovered after #50: freshness checked only when the plan was created could expire before socket I/O. The active lane retains the exact `FreshResolutionSnapshot` in the single-use plan, exposes `connect_at(current_time)` to re-run freshness immediately before socket use under the caller's trusted monotonic clock domain, and keeps the legacy `connect()` surface fail-closed by adding process-local monotonic elapsed time to the original authorization checkpoint before delegating to `connect_at`. CI run `31418337788` passes repository contracts, formatting, workspace checks/tests, strict Clippy, rustdoc and exact owned production function/line/region/branch coverage; CodeRabbit exact-head status is successful.

PRs #47, #50 and #54 remain **IMPLEMENTED_ON_ACTIVE_PR**, not shipped. #50 remains dependency-gated on #47 and #54 remains dependency-gated on #50. The overall protected-main resolution-to-socket interval therefore remains **PARTIAL** until dependency-ordered integration and fresh protected-main acceptance prove the same authority chain without an untimed planning or delayed-use bypass.

## Current exact-head RCA

### PR #47 primitive

The first production-complete PR #47 head reached all ordinary Rust contracts and security scans, but exact coverage failed at one compiler region while functions, lines, and branches were already complete. Coverage evidence localized the missing region to the generic `FreshResolutionSnapshot::revalidate` instantiation used with a one-address resolver answer: the success path for a one-address contraction was exercised, while the same monomorphized helper's error propagation for a one-address expansion had not been executed.

That was a realistic DNS-rebinding case rather than an impossible instrumentation artifact. The branch added a focused one-address expansion regression requiring `ResolutionSetExpanded`, retained the two-address expansion case, and exact head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` subsequently passed CI including exact production function/line/region/branch coverage, Security Scan, and SAST Semgrep.

The freshness ceiling is executable active-PR evidence rather than an aspirational requirement. `crates/originweave-destination/src/resolution.rs` owns `MAX_RESOLUTION_VALIDITY: Duration = Duration::from_secs(30)`. `FreshResolutionSnapshot::approve` rejects `Duration::ZERO` and any interval above that constant with `DestinationError::InvalidResolutionValidity`; `crates/originweave-destination/tests/resolution_freshness.rs::fresh_resolution_rejects_invalid_or_overflowing_validity` verifies both the zero and greater-than-30-second boundaries plus approval-time overflow. This evidence remains active-PR-only until PR #47 integrates.

### PR #50 planning consumer

PR #50 began from exact PR #47 head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` with a RED consumer contract requiring fresh resolution authority plus one trusted monotonic current time before direct socket planning.

A first production repair added a public `FreshConnectionPlan` wrapper that authorized freshness and then delegated to the existing untimed `ConnectionPlan`. That implementation made the positive/expiry path available but did not close the buyer/security gap because the original public `ConnectionPlan::new(&ResolutionSnapshot, ...)` remained callable. Canonical review therefore rejected the parallel-wrapper design as insufficient rather than weakening the acceptance boundary.

The corrected implementation removed `ConnectionPlan` from the public crate exports while retaining it as a private implementation detail. Exact-head CI run `31407686307` then failed at the intended first-party migration boundary: `cargo check --locked --workspace --all-targets` found exactly three TLS integration tests still importing the now-private stale planner (`handshake_deadline.rs`, `handshake_integration.rs`, and `validity_horizon_integration.rs`). That compile failure was useful evidence because it enumerated remaining first-party bypass consumers instead of hiding them behind a compatibility re-export.

Those integration helpers were migrated to deterministic `FreshResolutionSnapshot` + `FreshConnectionPlan` fixtures with one explicit trusted monotonic clock domain. A later run `31408143459` found only missing end-of-file newlines under rustfmt; that formatting-only defect was corrected without changing the authority contract. Current exact head `f8b43bc94444986ab23aa4ef3086e446a0b39295` then passed CI run `31408474576` end to end, including exact owned function/line/region/branch coverage.

The accepted remedy is therefore realized on the active branch: ordinary first-party direct planning cannot import the untimed planner, while the private implementation remains reusable only after `FreshConnectionPlan` performs freshness authorization. This proves the active planning implementation, but not freshness at a later delayed socket-use instant.

### PR #54 socket-use consumer

PR #54 follows #50 because a plan authorized within the resolution window could be retained until that window expired and then connected. The first failing boundary was therefore no longer public planner construction; it was the time between plan authorization and the exact operating-system connect operation.

The accepted active-branch remedy keeps the admitted freshness snapshot with the non-cloneable single-use plan and revalidates it at the socket-use boundary. `connect_at(current_time)` is the explicit deterministic path and rejects both expiry and an authorization-time regression using the existing destination error taxonomy. The compatibility `connect()` path does not freeze the old authorization timestamp: it anchors a process-local monotonic `Instant` at plan construction, adds actual elapsed time to the admitted authorization time, and delegates to `connect_at`, so delayed legacy callers cannot replay stale authority indefinitely.

The regression suite proves explicit success, deadline expiry, trusted-time regression, unchanged connection-parameter validation, and expiry of the compatibility path with a deliberately short real monotonic interval. Current exact head `ec81031c537f2b662910c1ce78c7ae0e0bfc9c1e` passes CI run `31418337788`. This remains active-PR evidence and does not add DNS lookup, a wall-clock authority, proxy/PAC, or a resolver service.

## Deterministic authority contract

The active stack proves one continuous destination-to-socket authority chain with all of the following properties:

1. approval time is explicit and supplied from one trusted monotonic clock domain;
2. validity is non-zero and capped by the active implementation's repository-owned `MAX_RESOLUTION_VALIDITY` safety budget (30 seconds on PR #47 exact head), with shorter caller-selected intervals permitted;
3. the usable interval is half-open: `approved_at <= now < valid_until`;
4. use before approval, use at/after expiry, arithmetic overflow, unapproved addresses, and set expansion fail closed with typed errors;
5. the ordinary first-party socket planner no longer publicly accepts an untimed `ResolutionSnapshot` as sufficient authority on PR #50 exact head;
6. a single-use plan rechecks the retained freshness authority immediately before socket I/O on PR #54 rather than assuming plan-time admission remains fresh;
7. the compatibility socket path derives a new use time from monotonic elapsed duration and therefore cannot preserve stale plan-time authority indefinitely;
8. credential-free planning evidence records approval, expiry, and authorization times without introducing credentials, resolver internals, or protected values;
9. non-expanding revalidation may renew the bounded interval only while rerunning existing destination-policy validation against the newly supplied answer; and
10. the primitive and planning/use boundaries perform no DNS lookup, wall-clock read, ambient proxy selection, TLS policy mutation, HTTP, browser control, persistence, secret, or model call.

## Architecture and ADR assessment

The primitive and its first-party planning/socket consumers tighten the already Accepted destination/rebinding authority governed by ADR 0004. They do not introduce a new component, persistence owner, wire protocol, browser adapter, or trust domain. Therefore a new ADR, deployment component, or physical ERD object would be false precision at this stage.

The durable network-authority sequence is now `resolver answer -> destination/origin validation -> fresh resolution approval -> trusted monotonic plan authorization -> socket-use freshness recheck -> exact socket candidate -> observed TCP peer -> TLS/HTTP authority`. That is a sequence refinement within the existing network-authority component graph, not a new topology. A new or superseding ADR becomes appropriate only if later integration changes ownership—for example, durable cross-process freshness state, a separate resolver service, a different trusted-clock owner, or a new externally versioned protocol.

## Evidence progression

| Evidence state | Allowed maturity claim |
|---|---|
| Test-only primitive/consumer head with unresolved production API | intentional RED contract only; not implementation evidence |
| Active PR #47 production primitive + unchanged exact-head CI/security/100% coverage | `IMPLEMENTED_ON_ACTIVE_PR` for the primitive; overall protected-main path remains `PARTIAL` |
| Active PR #50 adds a freshness wrapper while an ordinary untimed planner remains public | implementation progress only; bypass still makes the consumer incomplete |
| Active PR #50 hides the untimed planner and exact compile evidence finds stale first-party consumers | valid structural remedy with migration still incomplete |
| Active PR #50 exact head `f8b43bc...` migrates first-party consumers and passes exact CI/coverage | `IMPLEMENTED_ON_ACTIVE_PR` for planning; delayed socket-use freshness still requires #54 |
| Active PR #54 exact head `ec81031c...` rechecks freshness immediately before socket I/O and passes exact CI/coverage | `IMPLEMENTED_ON_ACTIVE_PR` for socket-use freshness; dependency-gated and non-shipped |
| PR #47 + #50 + #54 exact heads are individually gate-clean but none are on protected main | active-PR evidence only; no shipped claim |
| Protected-main primitive/planner, but delayed socket use can outlive freshness | `PARTIAL` |
| Protected-main direct socket path requires exact fresh authority and rechecks it at use, with tests proving pre-approval/expiry/rebinding/delay behavior | `IMPLEMENTED_ON_PROTECTED_MAIN` for the bounded resolution-to-socket interval |
| Browser/network adapter proves the same clock and authority chain under real navigation | additional integration/release evidence; not implied by lower-layer primitives |

## Required follow-through

- keep PR #47 as active/non-shipped evidence until repository governance integrates it;
- keep PR #50 Draft and dependency-gated while #47 remains active; do not transfer its green evidence to protected main;
- keep PR #54 Draft and dependency-gated while #50 remains active; do not transfer its green evidence to #50 or protected main;
- preserve the structural invariant that ordinary first-party direct planning cannot import an untimed `ConnectionPlan`;
- preserve the socket-use invariant that a delayed call cannot reuse plan-time freshness without a new trusted monotonic use-time check;
- keep PRD/TRD/traceability from calling the DNS-rebinding/TOCTOU interval closed while any prerequisite remains active;
- reconcile the existing network-authority UML with the stable durable freshness sequence without encoding temporary branch-only identifiers as timeless architecture;
- retain the existing conceptual ERD unless a real persistence owner is introduced; and
- after all three layers integrate, rerun protected-main operational/release acceptance before promoting capability maturity.
