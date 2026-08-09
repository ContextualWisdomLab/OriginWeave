# ADR 0008 — Leaf-certificate validity horizon for delegated tasks

- Status: Proposed
- Date: 2026-08-09
- Owner: `originweave-tls`
- Tracks: #23

## Context

The TLS kernel already authenticates a canonical HTTPS service identity with an explicit trust bundle and one fixed trusted validation time. RFC 5280 defines X.509 certificate validity using `notBefore` and `notAfter`, and RFC 9525 defines application service-identity verification for TLS. Those checks answer whether the certificate chain and service identity are acceptable at the validation instant; they do not promise that the leaf certificate will remain valid for the rest of a delegated browser task.

A long-running or delayed task can therefore start when a valid leaf certificate is close to `notAfter`. OriginWeave needs a product-level guard that can require enough remaining leaf validity for a bounded task horizon without misrepresenting that guard as a PKIX rule.

## Decision

`originweave-tls` will expose a deterministic `LeafValidityHorizon` policy primitive.

- The primitive receives the trusted validation timestamp and leaf `notAfter` timestamp from authenticated TLS evidence.
- It requires the remaining whole-second validity to be at least the configured `Duration`.
- Fractional requested durations round up to the next whole certificate second so the guard never grants less time than requested.
- A negative, expired, or earlier-than-trusted-time `notAfter` value has zero remaining validity and fails a positive horizon.
- Extreme `Duration` inputs saturate their whole-second requirement rather than wrapping.
- Failure is a typed, credential-free `LeafValidityHorizonError`; certificate bytes, subjects, SANs, and secrets are never included.
- A zero horizon preserves compatibility with point-in-time certificate validation.

This PR adds the reusable guard only. It does **not** claim that existing HTTP or Chromium adapters already enforce it. Issue #23 remains open until the governed browser/network composition consumes this guard before a delegated operation begins and realistic loopback TLS acceptance tests prove that integration.

## Authority and dependency direction

`originweave-tls` owns only the deterministic validity-horizon calculation. It must not fetch DNS, OCSP, CRLs, URLs, or ambient time and must not reconnect sockets. Higher-level adapters own task duration and must derive inputs from immutable `TlsConnectionEvidence`; they must not substitute page/model-provided timestamps.

Allowed direction:

```text
browser / HTTP adapter
        |
        v
TlsConnectionEvidence + bounded task horizon
        |
        v
LeafValidityHorizon
```

Forbidden coupling:

- model output cannot change trusted certificate-validation time;
- page content cannot manufacture certificate validity evidence;
- this guard cannot be treated as revocation checking;
- this guard cannot replace RFC 5280 path validation or RFC 9525 identity verification.

## Failure and recovery semantics

An insufficient horizon fails closed for the consuming adapter. Retrying the same certificate, trusted time, and horizon cannot change the result. A retry is meaningful only after independently obtaining a new authenticated TLS connection/evidence or after an authorized task-horizon change. Errors carry only the remaining and required whole-second budgets.

## Security and privacy consequences

The decision reduces the chance that a bounded delegated task begins on an identity credential that is known to expire before the configured horizon. It adds no personal data and no network disclosure. Because the inputs are timestamps already present in credential-free TLS evidence, normal evidence retention and tenant policy remain unchanged.

## Test and release evidence

The merge gate requires tests for:

- zero horizon;
- exact equality at the horizon boundary;
- one-second-short rejection;
- expired/pre-epoch `notAfter` without arithmetic underflow;
- fractional-second conservative rounding;
- extreme-duration saturation without integer wrap;
- standard error diagnostics;
- exact production function/line/region/branch coverage and public rustdoc.

Protected-main integration into the browser/HTTP execution path is a separate acceptance criterion tracked by #23.

## Alternatives considered

1. **Treat point-in-time WebPKI success as sufficient.** Rejected because it provides no product contract for tasks whose authorized duration extends beyond the certificate's remaining lifetime.
2. **Change RFC 5280 validation time to the task end.** Rejected because that would answer a different PKIX question and can incorrectly reject a certificate that is valid now while obscuring the distinction between standards validation and product horizon policy.
3. **Add OCSP/CRL fetching in this slice.** Rejected because revocation acquisition, freshness, privacy, authority, and egress are a separate security boundary already identified on the roadmap.
4. **Use ambient wall-clock time.** Rejected because OriginWeave's TLS authority intentionally uses explicit trusted time for reproducibility and auditability.

## Reversal conditions

Supersede this ADR if the TLS/runtime architecture adopts a single formally verified task-lifetime credential policy that subsumes this guard while preserving explicit trusted time, fail-closed arithmetic, standards separation, and replayable evidence.

## References — APA 7th

Cooper, D., Santesson, S., Farrell, S., Boeyen, S., Housley, R., & Polk, W. (2008). *Internet X.509 public key infrastructure certificate and certificate revocation list (CRL) profile* (RFC 5280). Internet Engineering Task Force. https://doi.org/10.17487/RFC5280

Saint-Andre, P., & Salgueiro, G. (2023). *Service identity in TLS* (RFC 9525). Internet Engineering Task Force. https://doi.org/10.17487/RFC9525
