# TLS Revocation-Material Freshness Authority Trace

- **Documentation status:** Active-PR traceability
- **Protected-main capability status:** **PARTIAL**
- **Active implementation lane:** PR #48, `feat/tls-revocation-freshness-main`
- **Governing existing boundary:** protected-main TLS service-identity authority, ADR 0006, ADR 0008, and the revocation-distribution/freshness roadmap gap
- **Buyer-visible gap:** prevent stale independently verified revocation material from being treated as current authority while preserving the fact that OriginWeave does not yet make an unrevoked-certificate claim

## Truth boundary

Protected `main` authenticates the requested HTTPS service over the already verified TCP stream, but its TLS evidence records revocation as `NotConfigured`. It does not fetch, parse, validate, cache, or enforce OCSP/CRL material and it does not claim that a certificate is unrevoked.

PR #48 adds a reusable **freshness primitive** for revocation material only. The primitive can classify independently verified material as usable inside its signed `thisUpdate` to `nextUpdate` interval. That active-PR implementation is not protected-main truth, and passing the freshness check does not prove signature validity, path validity, responder authority, non-revocation, successful distribution, or complete TLS authentication policy.

The complete revocation path therefore remains **PARTIAL** until a separately reviewed adapter acquires and cryptographically verifies revocation material, composes freshness into the authentication decision, defines failure/cache/recovery semantics, and proves the resulting behavior on protected main.

## Required deterministic authority

The bounded primitive is expected to preserve these properties:

1. `thisUpdate` and `nextUpdate` are supplied only after independent cryptographic verification by a higher-layer adapter;
2. the signed interval is non-empty and ordered;
3. the usable interval is half-open: `thisUpdate <= trusted_time < nextUpdate`;
4. trusted time before `thisUpdate` and at/after `nextUpdate` fails closed with typed bounded errors;
5. the primitive performs no OCSP/CRL fetch, DNS, socket connection, TLS handshake mutation, parsing, signature verification, cache operation, browser control, persistence, or model call; and
6. no evidence or documentation converts freshness into an `unrevoked` claim.

## Architecture and ADR assessment

The active primitive tightens an existing TLS evidence/policy concern without introducing a new deployed component, persistence owner, wire protocol, network path, or secret boundary. A new ADR is therefore not required merely because the helper type exists.

A new or superseding ADR becomes appropriate if OriginWeave later chooses a concrete revocation architecture that changes trust ownership—for example, stapled OCSP versus independently fetched OCSP/CRL, cache authority and freshness policy, hard-fail versus explicitly bounded degraded behavior, responder/path validation ownership, or a separate revocation service.

No new physical ERD object is justified by this active in-memory primitive. UML should change only when the executable TLS/revocation data or control path changes materially.

## Evidence progression

| Evidence state | Allowed maturity claim |
|---|---|
| Protected main records `RevocationStatus::NotConfigured` | `PARTIAL`; no revocation enforcement or unrevoked claim |
| Active PR freshness primitive with exact-head tests/coverage | `IMPLEMENTED_ON_ACTIVE_PR` for freshness classification only |
| Protected-main freshness primitive without verified material acquisition/composition | `PARTIAL` |
| Protected-main adapter verifies responder/material authenticity, freshness, cache/failure policy, and binds the result into TLS authentication | implementation evidence for the chosen bounded revocation policy |
| Protected-main integration/recovery/operational tests prove the complete path | required additional release evidence; not implied by the helper primitive |

## Required follow-through

- keep PRD/TRD/TLS evidence from implying revocation enforcement while protected main remains `NotConfigured`;
- define revocation-material acquisition, authenticity, cache, freshness, failure, privacy, and recovery semantics before calling the TLS revocation boundary implemented;
- require exact 100% owned production function/line/region/branch coverage and complete rustdoc on every changed head;
- add or supersede an ADR only when the concrete revocation architecture changes a durable trust or deployment decision; and
- retain the conceptual ERD unless executable persistence ownership actually appears.
