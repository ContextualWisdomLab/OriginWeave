# Resolution Freshness Authority Trace

- **Documentation status:** Active-PR traceability
- **Protected-main capability status:** **PARTIAL**
- **Active implementation lane:** PR #47, `feat/resolution-freshness-authority-main`
- **Governing existing decision boundary:** ADR 0004 and the protected-main destination/rebinding authority model
- **Buyer-visible gap:** bind the interval between a validated resolution answer and use of that authority so DNS-rebinding/TOCTOU exposure is explicit and fail-closed

## Truth boundary

Protected `main` already classifies, approves, pins, and non-expansively revalidates resolved destination addresses. It does **not** yet encode a resolution approval timestamp, bounded validity interval, or expiry decision in the destination authority consumed by the socket path.

PR #47 now contains the reusable production `FreshResolutionSnapshot` primitive and realistic tests, so the primitive is **IMPLEMENTED_ON_ACTIVE_PR** evidence. It is not protected-main truth. It is still non-shipped evidence. The complete DNS-rebinding/TOCTOU path is not closed merely by introducing a deterministic freshness primitive: first-party direct socket planning must subsequently require the fresh authority at the action linearization boundary and that consumer must use one trusted monotonic clock domain. Until that integration reaches protected main, the overall capability remains **PARTIAL**.

The initial test-only head is historical RED evidence only and must never be cited as shipped implementation evidence.

## Current exact-head RCA

The first production-complete head reached all ordinary Rust contracts and security scans, but exact coverage failed at one compiler region while functions, lines, and branches were already complete. The uploaded coverage evidence localized the only missing region to the generic `FreshResolutionSnapshot::revalidate` instantiation used with a one-address resolver answer: the success path for a one-address contraction was exercised, while the same monomorphized helper's error propagation for a one-address expansion had not been executed.

That is a realistic DNS-rebinding case rather than an impossible instrumentation artifact. The current branch therefore adds a focused regression in which a fresh resolver answer contains exactly one new public address and must fail closed with `ResolutionSetExpanded`. The existing two-address expansion test is retained because answer cardinality is part of the generic call shape and both are realistic resolver behaviors.

Fresh exact-head CI for that new test must be terminally successful before the branch can be called coverage-clean or moved out of Draft. Predecessor-head security/check results do not transfer to the new head.

## Deterministic authority contract

The active slice proves a reusable destination-policy primitive with all of the following properties:

1. approval time is explicit and supplied from one trusted monotonic clock domain;
2. validity is non-zero and capped by a repository-owned product safety budget;
3. the usable interval is half-open: `approved_at <= now < valid_until`;
4. use before approval, use at/after expiry, arithmetic overflow, and unapproved addresses fail closed with typed errors;
5. credential-free connection evidence records approval, expiry, and authorization times without introducing credentials, resolver internals, or raw protected values beyond the existing origin/address evidence contract;
6. non-expanding revalidation may renew the bounded interval only while rerunning the existing destination-policy validation against the newly supplied answer; and
7. the primitive performs no DNS lookup, socket I/O, wall-clock read, proxy selection, TLS, HTTP, browser control, persistence, or model call.

## Architecture and ADR assessment

This bounded primitive does not by itself introduce a new component, persistence owner, wire protocol, browser adapter, or trust domain. It tightens the already Accepted destination/rebinding authority governed by ADR 0004. Therefore a new ADR, UML deployment view, or physical ERD object would be false precision at this stage.

A new or superseding ADR becomes appropriate only if integration changes the governing boundary—for example, if resolution freshness becomes durable cross-process state, is delegated to a separate resolver service, changes trusted-clock ownership, or introduces a new externally versioned protocol.

## Evidence progression

| Evidence state | Allowed maturity claim |
|---|---|
| Test-only PR head with unresolved production API | `PLANNED` test contract / intentional RED only |
| Active PR production primitive with ordinary tests but a non-passing exact coverage gate | `IMPLEMENTED_ON_ACTIVE_PR` for the primitive, but not gate-clean; overall path remains `PARTIAL` |
| Active PR production primitive + unchanged exact-head CI/security/100% coverage | gate-clean active-PR evidence only; overall path remains `PARTIAL` |
| Protected-main primitive, but direct socket consumer can still bypass freshness | `PARTIAL` |
| Protected-main direct socket path requires exact fresh authority and integration tests prove expiry/rebinding behavior | `IMPLEMENTED_ON_PROTECTED_MAIN` for the bounded resolution-to-socket interval |
| Browser/network adapter proves the same clock and authority chain under real navigation | additional integration/release evidence; not implied by the lower-layer primitive |

## Required follow-through

- require terminal success on the current exact PR #47 head before changing Draft/review readiness;
- keep PRD/TRD/network-authority traceability from calling the TOCTOU boundary closed while PR #47 is active or while the socket consumer can bypass freshness;
- integrate freshness into the first-party `originweave-network` connection-plan boundary test-first after the destination primitive stabilizes;
- require exact 100% owned production function/line/region/branch coverage and complete rustdoc on each changed head;
- update the network authority UML only if the executable consumer changes the current sequence semantics materially; and
- retain the existing conceptual ERD unless a real persistence owner is introduced.
