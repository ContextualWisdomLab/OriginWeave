# Resolution Freshness Authority Trace

- **Documentation status:** Active-PR traceability
- **Protected-main capability status:** **PARTIAL**
- **Primitive implementation lane:** PR #47, `feat/resolution-freshness-authority-main`
- **First-party consumer lane:** PR #50, `feat/network-consume-resolution-freshness`
- **Governing existing decision boundary:** ADR 0004 and the protected-main destination/rebinding authority model
- **Buyer-visible gap:** bind the interval between a validated resolution answer and socket use so DNS-rebinding/TOCTOU exposure is explicit and fail-closed

## Truth boundary

Protected `main` already classifies, approves, pins, and non-expansively revalidates resolved destination addresses. It does **not** yet require a time-bounded resolution authority at the first-party direct-socket planning boundary.

PR #47 exact head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` contains the reusable production `FreshResolutionSnapshot` primitive and has terminal successful CI/security/SAST/exact-coverage evidence. That primitive is therefore **IMPLEMENTED_ON_ACTIVE_PR** evidence only; it is not protected-main truth.

PR #50 is the dependent consumer lane. Its intended contract requires the first-party direct-socket planning boundary to consume the fresh snapshot together with one caller-supplied trusted monotonic current time, reject expired authority before socket I/O, and retain only credential-free freshness timestamps needed to prove the planning decision. PR #50 remains Draft and non-shipped. Current exact head `18d3b19523de61f59dd47a11c2c82d6451512272` adds a production `FreshConnectionPlan`, but leaves the original public `ConnectionPlan::new(&ResolutionSnapshot, ...)` bypass available and leaves the acceptance test intentionally calling that old path. Exact-head CI therefore fails at the intended policy boundary rather than proving the consumer complete. The complete resolution-to-socket interval remains **PARTIAL**.

Neither an active primitive nor an alternative wrapper may be cited as shipped implementation evidence while the first-party untimed planning path remains callable. The overall DNS-rebinding/TOCTOU boundary becomes protected-main implemented only after the primitive and the first-party consumer integrate under one exact trusted clock/authority chain and no ordinary first-party socket plan can bypass freshness.

## Current exact-head RCA

### PR #47 primitive

The first production-complete PR #47 head reached all ordinary Rust contracts and security scans, but exact coverage failed at one compiler region while functions, lines, and branches were already complete. Coverage evidence localized the missing region to the generic `FreshResolutionSnapshot::revalidate` instantiation used with a one-address resolver answer: the success path for a one-address contraction was exercised, while the same monomorphized helper's error propagation for a one-address expansion had not been executed.

That was a realistic DNS-rebinding case rather than an impossible instrumentation artifact. The branch added a focused one-address expansion regression requiring `ResolutionSetExpanded`, retained the two-address expansion case, and exact head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` subsequently passed CI including exact production function/line/region/branch coverage, Security Scan, and SAST Semgrep.

The freshness ceiling is also executable active-PR evidence rather than an aspirational requirement. `crates/originweave-destination/src/resolution.rs` owns `MAX_RESOLUTION_VALIDITY: Duration = Duration::from_secs(30)`. `FreshResolutionSnapshot::approve` rejects `Duration::ZERO` and any interval above that constant with `DestinationError::InvalidResolutionValidity`; `crates/originweave-destination/tests/resolution_freshness.rs::fresh_resolution_rejects_invalid_or_overflowing_validity` verifies both the zero and greater-than-30-second boundaries plus approval-time overflow. This evidence remains active-PR-only until PR #47 integrates.

### PR #50 consumer

PR #50 began from exact PR #47 head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` with a test contract that intentionally does not compile against the old `ConnectionPlan::new(&ResolutionSnapshot, ...)` API. Predecessor CI also exposed and removed a setup-only `OriginError` formatting defect, leaving the production API mismatch as the valid RED boundary.

Current exact head `18d3b19523de61f59dd47a11c2c82d6451512272` implements a separate `FreshConnectionPlan`. That wrapper first calls `FreshResolutionSnapshot::authorize_connection`, then delegates to the existing untimed `ConnectionPlan`, and stores approval/expiry/authorization timestamps. This is useful implementation progress, but it does **not** yet satisfy the accepted consumer contract because the existing public `ConnectionPlan::new(&ResolutionSnapshot, ...)` remains a first-party route that can bypass freshness entirely.

CI run `31400013559` checked out exactly `18d3b19523de61f59dd47a11c2c82d6451512272`; repository contracts and rustfmt passed, then `cargo check --locked --workspace --all-targets` failed in `crates/originweave-network/tests/fresh_resolution_plan.rs`. The test still supplies a `FreshResolutionSnapshot` plus trusted current time to `ConnectionPlan::new` and requires freshness evidence accessors. Rust correctly reports the old four-argument untimed signature and missing freshness accessors. Production coverage also fails because the workspace does not compile.

The root cause is therefore **not** a formatting, runner, dependency, or coverage-instrumentation defect, and the smallest correct remedy is **not** to weaken the acceptance test to call `FreshConnectionPlan` while leaving an ordinary untimed `ConnectionPlan` public. The product boundary requires eliminating or structurally constraining the bypass: for example, make the externally consumable direct planner itself require `FreshResolutionSnapshot` + trusted time, or make the untimed planner an internal implementation detail reachable only after freshness authorization. Any chosen remedy must preserve current socket-validation semantics, use the same trusted monotonic clock domain, and reacquire exact-head CI/security/coverage evidence.

## Deterministic authority contract

The active work is intended to prove one continuous destination-to-socket authority chain with all of the following properties:

1. approval time is explicit and supplied from one trusted monotonic clock domain;
2. validity is non-zero and capped by the active implementation's repository-owned `MAX_RESOLUTION_VALIDITY` safety budget (30 seconds on PR #47 exact head), with shorter caller-selected intervals permitted;
3. the usable interval is half-open: `approved_at <= now < valid_until`;
4. use before approval, use at/after expiry, arithmetic overflow, unapproved addresses, and set expansion fail closed with typed errors;
5. the first-party socket planner cannot accept an untimed `ResolutionSnapshot` as sufficient authority once the consumer integration is complete;
6. credential-free planning evidence records approval, expiry, and authorization times without introducing credentials, resolver internals, or protected values;
7. non-expanding revalidation may renew the bounded interval only while rerunning existing destination-policy validation against the newly supplied answer; and
8. the primitive and planning boundary perform no DNS lookup, wall-clock read, ambient proxy selection, TLS, HTTP, browser control, persistence, secret, or model call.

## Architecture and ADR assessment

The primitive and its first-party consumer tighten the already Accepted destination/rebinding authority governed by ADR 0004. They do not introduce a new component, persistence owner, wire protocol, browser adapter, or trust domain. Therefore a new ADR, deployment component, or physical ERD object would be false precision at this stage.

The network-authority UML should be reconciled when the PR #50 consumer stabilizes because the executable sequence changes materially from `resolution snapshot -> socket plan` to `fresh resolution authority + trusted monotonic use time -> socket plan`. A new or superseding ADR becomes appropriate only if integration changes the governing ownership boundary—for example, durable cross-process freshness state, a separate resolver service, a different trusted-clock owner, or a new externally versioned protocol.

## Evidence progression

| Evidence state | Allowed maturity claim |
|---|---|
| Test-only consumer head with unresolved production API | intentional RED contract only; not implementation evidence |
| Active PR #47 production primitive + unchanged exact-head CI/security/100% coverage | `IMPLEMENTED_ON_ACTIVE_PR` for the primitive; overall path remains `PARTIAL` |
| Active PR #50 adds a freshness wrapper while an ordinary untimed planner remains public | implementation progress only; bypass still makes the overall path `PARTIAL` |
| Active PR #50 production consumer structurally requires fresh authority but has non-passing gates | `IMPLEMENTED_ON_ACTIVE_PR` only for code already present and testable; overall path remains `PARTIAL` |
| PR #47 + #50 exact heads are individually gate-clean but neither is on protected main | active-PR evidence only; no shipped claim |
| Protected-main primitive, but direct socket consumer can still bypass freshness | `PARTIAL` |
| Protected-main direct socket path requires exact fresh authority and tests prove pre-approval/expiry/rebinding behavior | `IMPLEMENTED_ON_PROTECTED_MAIN` for the bounded resolution-to-socket interval |
| Browser/network adapter proves the same clock and authority chain under real navigation | additional integration/release evidence; not implied by lower-layer primitives |

## Required follow-through

- keep PR #47 as active/non-shipped evidence until repository governance integrates it;
- on PR #50, preserve the valid RED at the public first-party planning API and remove or structurally constrain the untimed `ResolutionSnapshot` bypass rather than weakening the test to accept a parallel wrapper;
- require terminal exact-head workspace/tests/Clippy/rustdoc/100% function-line-region-branch coverage plus security evidence after that production boundary changes;
- keep PRD/TRD/traceability from calling the DNS-rebinding/TOCTOU interval closed while either prerequisite is active or any ordinary socket path can bypass freshness;
- update network-authority sequence documentation after the executable consumer signature/evidence contract stabilizes, without encoding temporary branch-only details as protected-main truth;
- retain the existing conceptual ERD unless a real persistence owner is introduced; and
- after both layers integrate, rerun protected-main operational/release acceptance before promoting the capability maturity.
