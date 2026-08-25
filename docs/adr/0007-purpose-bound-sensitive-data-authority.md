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

Tenant, task, field, and purpose identifiers are policy tokens, not arbitrary display text. Each must be 1–128 bytes of ASCII alphanumeric characters plus `.`, `_`, `:`, or `-`. Missing, oversized, whitespace-bearing, control-bearing, Unicode, or otherwise malformed authority identifiers fail closed even when the request and scope contain the same invalid bytes. This bounds memory and serialization surface and prevents malformed identifiers from becoming authority merely through equality. The destination must already have crossed the canonical `Origin` parser boundary, so credentials, paths, malformed or ambiguous hosts, unsupported insecure remote schemes, Unicode/control input, and browser-special numeric-host spellings cannot be smuggled into the sensitive-data authority as arbitrary text.

An exact match may return only the explicitly configured disclosure decision: deny, opaque handle only, derived value only, partial field disclosure, full field disclosure, human approval required, or dual control required. Any authority mismatch or invalid authority fails closed to denial. `HumanApprovalRequired` and `DualControlRequired` are not execution permissions: the caller must collect the required independent approval evidence and re-evaluate the exact same tenant, task, field, purpose, destination, and classification scope before any trusted broker, browser fill, export, or model-disclosure path can proceed. `DenyAccess` terminates the disclosure path.

Opaque handle use is separately bound to tenant, task, field, purpose, canonical destination, data classification, exclusive expiry time, and maximum use count. A field reclassification therefore invalidates the prior handle authority even when every other identifier is unchanged; the caller must obtain a newly authorized handle for the new classification. `evaluate_handle_use` is intentionally a pure admission predicate: it compares exact authority, classification, trusted-time input, and broker-recorded prior-use count, but it does not own mutable handle state, resolve a handle, consume a use, authenticate a workload audience, or return the protected value.

`SensitiveHandleUseState` is the bounded in-process stateful reservation primitive. It owns the current reserved-use count and a monotonic trusted-time floor. A reservation attempt whose trusted time predates the latest time already observed by the state fails closed as `TrustedTimeRollback`. Every non-rollback trusted time advances that floor even when a later scope, expiry, or use-count check denies the attempt, preventing a stale clock value from restoring authority after a later time has already been observed. Denied reservations never consume a use; only an `Authorized` reservation increments the authoritative count.

The trusted broker or browser adapter remains the real enforcement boundary. Before resolving any protected value, it must obtain trusted time and caller-unforgeable lifecycle/use state, persist an equivalent monotonic trusted-time floor together with lifecycle/use state when durability or cross-process coordination exists, atomically compare the exact scope, classification, exclusive expiry, and current use count, and reserve or increment the use count in the same transaction that grants the use. Concurrent or replayed requests must therefore compete for one authoritative count rather than reusing a stale caller-supplied count. Once a use has been successfully reserved, a downstream browser/action failure does not silently refund that use unless a separately specified compensating transaction is both safe and auditable. At the expiry boundary (`now >= expires_at`) no new reservation is permitted. Immediately before release, the broker rechecks that the reserved handle, requested scope, classification, and lifecycle state still permit disclosure. Authenticated audience binding is a separate child authority contract and is not claimed by this bounded reservation slice.

The first kernel intentionally does not implement storage, encryption, tokenization, workload/service authentication, model disclosure, provider or region policy, retention, audit persistence, break-glass access, or a broker. Those remain separate authority and lifecycle boundaries rather than being inferred from this primitive.

## Consequences

- Raw protected bytes are structurally absent from the first policy API.
- A caller with the wrong tenant, task, field, purpose, destination, or classification cannot reuse another disclosure scope or opaque handle.
- A later field reclassification fails closed against an older handle instead of inheriting the old disclosure authority.
- Missing, oversized, whitespace-bearing, control-bearing, Unicode, or otherwise malformed tenant, task, field, or purpose identifiers cannot become authority through equality with another invalid scope; destination validity is guaranteed by the canonical `Origin` boundary.
- An expired or exhausted opaque handle fails closed, and an earlier trusted-time value cannot restore authority after a later trusted time has already been observed by `SensitiveHandleUseState`.
- The future durable broker must persist equivalent lifecycle, use-count, and monotonic trusted-time state transactionally rather than treating caller-provided counters or clock values as authority.
- Approval-required disclosure outcomes cannot fall through directly to execution; the exact scope is re-evaluated after approval evidence is obtained.
- Later UI, connector, model, export, and browser-fill adapters can reuse the same explicit decision boundary without inheriting ambient authority.
- The complete enterprise gap is not closed by this kernel; independently reusable storage/broker/service contracts, authenticated workload identity and audience binding, evidence, lifecycle controls, and end-to-end tests are still required.

## Rejected alternatives

### Blanket masking

Rejected because some authorized operational workflows require the real value and a permanent masked copy can diverge from the authoritative record.

### Ambient trusted-network or session access

Rejected because network or session membership is not a sufficient authorization decision and creates confused-deputy and propagation risk.

### Sending every protected value through the model

Rejected because many actions can operate through opaque handles or deterministic trusted adapters. Model disclosure must remain a separately governed exceptional path.

### Classification-free opaque handles

Rejected because a handle issued while a field is classified as ordinary personal data could otherwise be reused after the same field is reclassified as sensitive personal, credential, or payment data. Classification is an authority dimension, not mutable display metadata.

### Caller-managed handle-use counters or rollback-prone trusted time

Rejected because two concurrent callers can present the same stale `uses_so_far` value and both appear admissible, while a later call carrying an earlier clock value can otherwise restore authority after expiry was already observed. Mutable use count and an equivalent monotonic trusted-time floor belong to the trusted state boundary.

## Verification

Tests must prove exact-scope disclosure, canonical destination behavior, denial on every authority-dimension mismatch, fail-closed behavior for missing or malformed authority, acceptance at the exact 128-byte identifier bound, rejection beyond that bound, rejection of whitespace/control/Unicode identifiers, every supported disclosure result, opaque-handle classification mismatch, expiry, zero-use and use-count exhaustion, destination mismatch, authoritative reservation consumption, denied-reservation non-consumption, trusted-time rollback after expiry observation, and trusted-time-floor advancement on non-rollback denied attempts. The durable broker slice must add concurrency, replay, expiry-boundary, post-reservation failure, revocation, authenticated-audience binding, atomic compare-and-increment, and persistent trusted-time-floor tests before any protected-value resolution is described as implemented. Production function, line, region, and branch coverage remains exactly 100%.

## References

Chandramouli, R., & Butcher, Z. (2023). *A zero trust architecture model for access control in cloud-native applications in multi-cloud environments* (NIST Special Publication 800-207A). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207A

European Parliament & Council of the European Union. (2016). *Regulation (EU) 2016/679 of the European Parliament and of the Council of 27 April 2016 (General Data Protection Regulation)*. *Official Journal of the European Union, L 119*, 1–88. https://eur-lex.europa.eu/eli/reg/2016/679/oj

OWASP Foundation. (n.d.). *Logging cheat sheet*. OWASP Cheat Sheet Series. Retrieved August 9, 2026, from https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html
