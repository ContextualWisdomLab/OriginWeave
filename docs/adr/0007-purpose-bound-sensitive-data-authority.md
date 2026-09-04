# ADR 0007: Purpose-bound sensitive-data authority

- Status: Accepted for the first authority kernel
- Date: 2026-08-09

## Context

Enterprise browser workflows sometimes require real personal or otherwise protected values. Blanket masking can make a legitimate shipment, form fill, customer contact, reconciliation, or incident workflow impossible. Ambient raw access has the opposite failure mode: values can propagate into models, logs, traces, exports, support channels, or unrelated services merely because a caller already has network or session access.

OriginWeave therefore treats each sensitive-data disclosure as a separate resource-access decision. Network location, session ownership, repository membership, administrator status, or possession of a model credential does not grant disclosure authority. This follows the same explicit-authority architecture used for origins, resolved destinations, TCP peers, TLS identities, actions, and approvals.

NIST SP 800-207A models application and service identities as policy inputs rather than relying on network location. The GDPR purpose-limitation and data-minimisation principles likewise require processing to remain tied to specified purposes and limited to what is necessary for them. OWASP logging guidance warns that personal and other sensitive information can become a secondary exposure through application logs. These sources motivate the architecture; this ADR does not itself establish legal compliance or certification.

## Decision

The first implementation lives inside `originweave-policy` as a bounded preparatory authority kernel while the repository's lockfile-governance constraint prevents safely adding the separately versioned `originweave-sensitive-data` workspace crate required by the complete issue. The final issue remains open until that standalone crate and service contract exist.

The kernel carries authority metadata but never the protected value. A disclosure request and its explicit scope are bound to tenant identity, task identity, field identity, declared business purpose, canonical destination `Origin`, and data classification.

Tenant, task, field, purpose, and opaque-handle audience identifiers are policy tokens, not arbitrary display text. Each must be 1–128 bytes of ASCII alphanumeric characters plus `.`, `_`, `:`, or `-`, with at least one alphanumeric character. Missing, oversized, whitespace-bearing, control-bearing, Unicode, or otherwise malformed identifiers fail closed. This bounds memory and serialization surface and prevents malformed identifiers from becoming authority merely through equality. The destination must already have crossed the canonical `Origin` parser boundary, so credentials, paths, malformed or ambiguous hosts, unsupported insecure remote schemes, Unicode/control input, and browser-special numeric-host spellings cannot be smuggled into sensitive-data authority as arbitrary text.

An exact disclosure match may return only the explicitly configured disclosure decision: deny, opaque handle only, derived value only, partial field disclosure, full field disclosure, human approval required, or dual control required. Any authority mismatch or invalid authority fails closed to denial. `HumanApprovalRequired` and `DualControlRequired` are not execution permissions: the caller must collect the required independent approval evidence and re-evaluate the exact same tenant, task, field, purpose, destination, and classification scope before any trusted broker, browser fill, export, or model-disclosure path can proceed. `DenyAccess` terminates the disclosure path.

Opaque handle use is separately bound to tenant, task, field, purpose, canonical destination, data classification, a non-transferable audience identifier, exclusive expiry time, and maximum use count. A field reclassification or audience change therefore invalidates the prior handle authority even when every other identifier is unchanged; a newly authorized handle is required. `evaluate_handle_use` is intentionally a pure admission predicate: it compares exact authority, audience, trusted-time input, and broker-recorded prior-use count, but it does not authenticate the audience, own mutable handle state, resolve a handle, consume a use, or return the protected value.

`SensitiveHandleUseState` is the bounded in-process stateful reservation and revocation primitive. It owns the current reserved-use count, the first authoritative revocation reason, and a monotonic trusted-time floor for its exact authority-and-audience binding. Authority scope and audience equality are evaluated before either lifecycle state or the trusted-time floor: a request outside either binding fails as `ScopeMismatch` or `AudienceMismatch` without reading or mutating revocation or time state. A foreign scope or audience therefore cannot distinguish active from revoked state, probe whether a newer trusted time has been observed, or poison the state by presenting an arbitrarily advanced timestamp. For an exact binding, revocation is terminal and returns `Revoked`; otherwise a trusted time that predates the latest exact-binding time already observed fails closed as `TrustedTimeRollback`. Every non-rollback exact-binding trusted time advances the floor even when expiry or use-count policy later denies the attempt. Denied reservations never consume a use; only an `Authorized` reservation increments the authoritative count. Revocation is first-wins and records one of task completion, policy change, key rotation, session termination, or suspicious use; later duplicate revocation calls cannot rewrite the original lifecycle reason.

The trusted broker or browser adapter remains the real enforcement boundary. It must derive `audience_id` from authenticated, caller-unforgeable workload or service identity rather than accepting self-asserted audience text as authority. Before resolving any protected value, it must obtain trusted time and caller-unforgeable lifecycle/use state, persist an equivalent monotonic trusted-time floor together with audience binding, revocation, and use state when durability or cross-process coordination exists, atomically compare the exact scope, authenticated audience, classification, exclusive expiry, revocation state, and current use count, and reserve or increment the use count in the same transaction that grants the use. Concurrent or replayed requests must compete for one authoritative count rather than reusing a stale caller-supplied count. Once a use has been successfully reserved, a downstream browser/action failure does not silently refund that use unless a separately specified compensating transaction is both safe and auditable. At the expiry boundary (`now >= expires_at`) no new reservation is permitted. Immediately before release, the broker rechecks that the reserved handle, requested scope, authenticated audience, classification, and lifecycle state still permit disclosure.

The first kernel intentionally does not implement storage, encryption, tokenization, workload/service authentication, model disclosure, provider or region policy, retention, audit persistence, break-glass access, or a broker. Those remain separate authority and lifecycle boundaries rather than being inferred from this primitive.

## Consequences

- Raw protected bytes are structurally absent from the first policy API.
- A caller with the wrong tenant, task, field, purpose, destination, classification, or audience cannot reuse another disclosure scope or opaque handle, cannot observe its revocation state, cannot advance that handle state's trusted-time floor, and cannot probe whether the floor has already moved forward.
- A later field reclassification or audience change fails closed against an older handle instead of inheriting the old disclosure authority.
- Missing, oversized, whitespace-bearing, control-bearing, Unicode, or otherwise malformed authority or audience identifiers cannot become authority through equality with another invalid value; destination validity is guaranteed by the canonical `Origin` boundary.
- An expired, exhausted, or revoked opaque handle fails closed, and an earlier exact-binding trusted-time value cannot restore authority after a later exact-binding trusted time has already been observed by `SensitiveHandleUseState`.
- First revocation wins; later duplicate revocation cannot erase or replace the original lifecycle cause, and revoked state cannot regain reservation authority.
- The future durable broker must derive audience from authenticated identity and persist equivalent audience, lifecycle, revocation, use-count, and monotonic trusted-time state transactionally rather than treating caller-provided audience, counters, or clock values as authority.
- Approval-required disclosure outcomes cannot fall through directly to execution; the exact scope is re-evaluated after approval evidence is obtained.
- Later UI, connector, model, export, and browser-fill adapters can reuse the same explicit decision boundary without inheriting ambient authority.
- The complete enterprise gap is not closed by this kernel; independently reusable storage/broker/service contracts, authenticated workload identity, evidence, lifecycle controls, and end-to-end tests are still required.

## Rejected alternatives

### Blanket masking

Rejected because some authorized operational workflows require the real value and a permanent masked copy can diverge from the authoritative record.

### Ambient trusted-network or session access

Rejected because network or session membership is not a sufficient authorization decision and creates confused-deputy and propagation risk.

### Sending every protected value through the model

Rejected because many actions can operate through opaque handles or deterministic trusted adapters. Model disclosure must remain a separately governed exceptional path.

### Classification-free or audience-free opaque handles

Rejected because a handle issued under one data classification or for one authenticated workload could otherwise be reused after reclassification or by another service. Classification and audience are authority dimensions, not mutable display metadata.

### Caller-managed handle-use counters, audience, or rollback-prone trusted time

Rejected because concurrent callers can present the same stale `uses_so_far`, a caller can self-assert a privileged audience, and a later exact-binding call carrying an earlier clock value can otherwise restore authority after expiry was already observed. Mutable use count, authenticated audience binding, revocation state, and an equivalent exact-binding monotonic trusted-time floor belong to the trusted broker/state boundary; mismatched bindings must not be allowed to read or mutate that state.

## Verification

Tests must prove exact-scope disclosure, canonical destination behavior, denial on every authority-dimension mismatch, fail-closed behavior for missing or malformed authority, acceptance at the exact 128-byte identifier bound, rejection beyond that bound, rejection of whitespace/control/Unicode identifiers, every supported disclosure result, opaque-handle classification mismatch, audience mismatch and malformed audience, expiry, zero-use and use-count exhaustion, destination mismatch, authoritative reservation consumption, denied-reservation non-consumption, trusted-time rollback after expiry observation, trusted-time-floor advancement on non-rollback exact-binding expiry/use-limit denials, every supported revocation cause, first-revocation-wins idempotency, terminal revoked behavior, concurrent single-count reservation, and that a scope or audience mismatch can neither expose revocation/rollback state nor advance the trusted-time floor. The durable broker slice must add authenticated-identity derivation, concurrency across processes, replay, expiry-boundary, post-reservation failure, atomic compare-and-increment, persistent revocation, and persistent trusted-time-floor tests before any protected-value resolution is described as implemented. Production function, line, region, and branch coverage remains exactly 100%.

## References

Chandramouli, R., & Butcher, Z. (2023). *A zero trust architecture model for access control in cloud-native applications in multi-cloud environments* (NIST Special Publication 800-207A). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207A

European Parliament & Council of the European Union. (2016). *Regulation (EU) 2016/679 of the European Parliament and of the Council of 27 April 2016 (General Data Protection Regulation)*. *Official Journal of the European Union, L 119*, 1–88. https://eur-lex.europa.eu/eli/reg/2016/679/oj

OWASP Foundation. (n.d.). *Logging cheat sheet*. OWASP Cheat Sheet Series. Retrieved August 9, 2026, from https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html
