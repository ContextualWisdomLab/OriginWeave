# ADR 0008 — Leaf-certificate validity horizon for delegated tasks

- Status: Accepted pending protected-main integration
- Date: 2026-08-09
- Owner: `originweave-tls`
- Tracks: #23

## Context

The TLS kernel authenticates a canonical HTTPS service identity with an explicit trust bundle and one fixed trusted validation time. RFC 5280 defines X.509 certificate validity using `notBefore` and `notAfter`, and RFC 9525 defines application service-identity verification for TLS. Those checks answer whether the certificate chain and service identity are acceptable at the validation instant; they do not promise that the leaf certificate will remain valid for the rest of a delegated task.

A long-running or delayed task can therefore begin when an otherwise valid leaf certificate is close to `notAfter`. OriginWeave needs a product-level guard that requires enough remaining leaf validity for the caller's bounded task horizon without misrepresenting that guard as a PKIX rule.

## Decision

`originweave-tls` layers a deterministic leaf-validity horizon onto the existing fixed-time WebPKI handshake boundary.

- `LeafValidityHorizon` remains a reusable calculation over trusted validation time and leaf `notAfter` evidence.
- `TlsClientPolicy` carries `minimum_leaf_validity`; existing callers default to zero for point-in-time compatibility.
- A caller may opt into a nonzero horizon with `with_minimum_leaf_validity` before the handshake.
- The configured horizon is bounded to `MAX_MINIMUM_LEAF_VALIDITY`, currently seven days. This is an OriginWeave product limit, not an X.509 or PKIX limit; longer-lived work must obtain fresh transport authority instead of stretching one TLS authentication into indefinite task authority.
- Fractional requested durations round up to the next whole certificate second so the policy never grants less time than requested.
- A negative, expired, or earlier-than-trusted-time `notAfter` value has zero remaining validity and fails a positive horizon.
- Extreme standalone `LeafValidityHorizon` inputs saturate their whole-second requirement rather than wrapping.
- After normal rustls/WebPKI verification, peer revalidation, timeout restoration, and immutable TLS evidence construction, the policy compares the configured horizon with the authenticated leaf validity interval.
- If the remaining leaf validity is too short, authentication returns typed `TlsError::InsufficientLeafValidity` and the authenticated stream is not exposed to the consuming adapter.
- Configuration beyond the reviewed seven-day product maximum returns typed `TlsError::InvalidMinimumLeafValidity` before network execution.
- Errors contain only duration/timestamp-derived budget values; certificate contents, subjects, SANs, credentials, and secrets are never included.

This satisfies the bounded `originweave-tls` vertical slice in #23. HTTP, Chromium, or future protocol adapters remain responsible for choosing the nonzero horizon appropriate to the delegated operation; the TLS kernel does not invent task duration or read ambient time.

## Authority and dependency direction

`originweave-tls` owns deterministic certificate-horizon enforcement once a caller supplies a reviewed task horizon. It must not fetch DNS, OCSP, CRLs, URLs, or ambient time and must not reconnect sockets. Higher-level adapters own task duration and construct `TlsClientPolicy` from trusted control-plane state; page content and model output are not authority for the validation timestamp or certificate evidence.

Allowed direction:

```text
trusted task/controller horizon
        |
        v
TlsClientPolicy.minimum_leaf_validity
        |
        v
fixed-time WebPKI handshake + TlsConnectionEvidence
        |
        v
LeafValidityHorizon enforcement
        |
        +---- insufficient ----> typed failure, stream withheld
        |
        `---- sufficient ------> authenticated stream + evidence
```

Forbidden coupling:

- model output cannot change trusted certificate-validation time;
- page content cannot manufacture certificate validity evidence;
- this guard cannot be treated as revocation checking;
- this guard cannot replace RFC 5280 path validation or RFC 9525 identity verification;
- a long task cannot bypass the product maximum by claiming one handshake as durable authority.

## Failure and recovery semantics

An insufficient horizon fails closed before the authenticated stream escapes the TLS boundary. Retrying the same certificate, trusted time, and horizon cannot change the result. A retry is meaningful only after independently obtaining new authenticated transport evidence or after an authorized task-horizon change. Excessive configured horizons are rejected before the handshake. Errors remain deterministic and credential-free.

## Security and privacy consequences

The decision prevents a configured delegated operation from starting on a leaf certificate already known to expire before its required horizon. It adds no personal data and no network disclosure. Inputs are fixed control-plane duration plus timestamps already present in credential-free TLS evidence, so normal evidence retention and tenant policy remain unchanged.

The zero default intentionally preserves compatibility. Product adapters that promise a bounded delegated-task lifetime must explicitly select a nonzero horizon; zero must not be described as providing task-lifetime assurance.

## Test and release evidence

The merge gate requires permanent tests for:

- zero compatibility margin;
- exact equality at the horizon boundary;
- one-second-short rejection;
- expired/pre-epoch `notAfter` without arithmetic underflow;
- fractional-second conservative rounding;
- extreme-duration saturation without integer wrap;
- exact seven-day policy maximum and fail-closed excessive configuration;
- typed deterministic error diagnostics;
- a realistic loopback rustls certificate whose point-in-time WebPKI validation succeeds but whose configured nonzero task horizon fails before stream exposure;
- exact production function/line/region/branch coverage and public rustdoc.

Protected-main integration remains required before the repository may claim the capability is shipped.

## Alternatives considered

1. **Treat point-in-time WebPKI success as sufficient.** Rejected because it provides no product contract for tasks whose authorized duration extends beyond the certificate's remaining lifetime.
2. **Change RFC 5280 validation time to the task end.** Rejected because that answers a different PKIX question and obscures the distinction between standards validation and product horizon policy.
3. **Leave the guard as an optional post-handshake helper.** Rejected because a caller could expose the authenticated stream before applying the task-horizon policy. The reviewed policy is now enforced inside `TlsHandshakePlan::authenticate` before stream exposure.
4. **Use an unbounded caller duration.** Rejected because it lets one transport authentication become de facto indefinite task authority. The policy caps the horizon at seven days and requires fresh authority beyond that bound.
5. **Add OCSP/CRL fetching in this slice.** Rejected because revocation acquisition, freshness, privacy, authority, and egress are separate security boundaries.
6. **Use ambient wall-clock time.** Rejected because OriginWeave's TLS authority intentionally uses explicit trusted time for reproducibility and auditability.

## Reversal conditions

Supersede this ADR if the TLS/runtime architecture adopts a single formally verified task-lifetime credential policy that subsumes this guard while preserving explicit trusted time, fail-closed arithmetic, standards separation, bounded authority, and replayable evidence.

## References — APA 7th

Cooper, D., Santesson, S., Farrell, S., Boeyen, S., Housley, R., & Polk, W. (2008). *Internet X.509 public key infrastructure certificate and certificate revocation list (CRL) profile* (RFC 5280). Internet Engineering Task Force. https://doi.org/10.17487/RFC5280

Saint-Andre, P., & Salz, R. (2023). *Service identity in TLS* (RFC 9525). Internet Engineering Task Force. https://doi.org/10.17487/RFC9525
