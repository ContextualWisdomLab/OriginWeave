# Resolution Freshness Authority Trace

- **Documentation status:** Active-PR traceability
- **Protected-main capability status:** **PARTIAL**
- **Active implementation lane:** PR #47, `feat/resolution-freshness-authority-main`
- **Governing existing decision boundary:** ADR 0004 and the protected-main destination/rebinding authority model
- **Buyer-visible gap:** bound the interval between a validated resolution answer and use of that authority so DNS-rebinding/TOCTOU exposure is explicit and fail-closed

## Truth boundary

Protected `main` at `67af7c87589edc2039545af335c95064d9b8391c` already classifies, approves, pins, and non-expansively revalidates resolved destination addresses. It does **not** yet encode a resolution approval timestamp, bounded validity interval, or expiry decision in the destination authority consumed by the socket path.

PR #47 is therefore **IMPLEMENTED_ON_ACTIVE_PR only after its production change exists and exact-head tests pass**. Its initial test-only head is intentionally RED and must never be cited as shipped implementation evidence.

The complete DNS-rebinding/TOCTOU path is not closed merely by introducing a deterministic freshness primitive. First-party direct socket planning must subsequently require the fresh authority at the action linearization boundary, and that consumer must use one trusted monotonic clock domain. Until that integration is protected-main evidence, the capability remains **PARTIAL**.

## Required deterministic authority

The active slice is expected to prove a reusable destination-policy primitive with all of the following properties:

1. approval time is explicit and supplied from one trusted monotonic clock domain;
2. validity is non-zero and capped by a repository-owned product safety budget;
3. the usable interval is half-open: `approved_at <= now < valid_until`;
4. use before approval, use at/after expiry, arithmetic overflow, and unapproved addresses fail closed with typed errors;
5. credential-free connection evidence records approval, expiry, and authorization times without introducing hostnames, credentials, resolver internals, or raw protected values beyond the existing origin/address evidence contract;
6. non-expanding revalidation may renew the bounded interval only while rerunning the existing destination-policy validation against the newly supplied answer; and
7. the primitive performs no DNS lookup, socket I/O, wall-clock read, proxy selection, TLS, HTTP, browser control, persistence, or model call.

## Architecture and ADR assessment

This bounded primitive does not by itself introduce a new component, persistence owner, wire protocol, browser adapter, or trust domain. It tightens the already Accepted destination/rebinding authority governed by ADR 0004. Therefore a new ADR, UML deployment view, or physical ERD object would be false precision at this stage.

A new or superseding ADR becomes appropriate only if integration changes the governing boundary—for example, if resolution freshness becomes durable cross-process state, is delegated to a separate resolver service, changes trusted-clock ownership, or introduces a new externally versioned protocol.

## Evidence progression

| Evidence state | Allowed maturity claim |
|---|---|
| Test-only PR head with unresolved production API | `PLANNED` test contract / intentional RED only |
| Active PR production primitive + exact-head tests/coverage | `IMPLEMENTED_ON_ACTIVE_PR` for the primitive; overall path remains `PARTIAL` |
| Protected-main primitive, but direct socket consumer can still bypass freshness | `PARTIAL` |
| Protected-main direct socket path requires exact fresh authority and exact-head integration tests prove expiry/rebinding behavior | `IMPLEMENTED_ON_PROTECTED_MAIN` for the bounded resolution-to-socket interval |
| Browser/network adapter proves the same clock and authority chain under real navigation | additional integration/release evidence; not implied by the lower-layer primitive |

## Required follow-through

- keep PRD/TRD/network-authority traceability from calling the TOCTOU boundary closed while PR #47 is active or while the socket consumer can bypass freshness;
- integrate freshness into the first-party `originweave-network` connection-plan boundary test-first after the destination primitive stabilizes;
- require exact 100% owned production function/line/region/branch coverage and complete rustdoc on each changed head;
- update the network authority UML only if the executable consumer changes the current sequence semantics materially; and
- retain the existing conceptual ERD unless a real persistence owner is introduced.
