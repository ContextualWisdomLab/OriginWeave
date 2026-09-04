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

Opaque handle use is separately bound to tenant, task, field, purpose, canonical destination, data classification, exclusive expiry time, and maximum use count. A field reclassification therefore invalidates the prior handle authority even when every other identifier is unchanged; the caller must obtain a newly authorized handle for the new classification. `evaluate_handle_use` is intentionally a pure admission predicate: it compares authority, classification, trusted-time input, and broker-recorded prior-use count, but it does not own mutable handle state, resolve a handle, consume a use, or return the protected value. It must never be treated as standalone enforcement by an untrusted caller.

The later trusted broker or browser adapter is the stateful enforcement boundary. Before resolving any protected value, it must obtain trusted time and authoritative, caller-unforgeable handle state; atomically compare the exact scope, classification, exclusive expiry, and current use count; and reserve or increment the use count in the same transaction that grants the use. Concurrent or replayed requests therefore compete for one authoritative count rather than reusing a stale caller-supplied count. Once a use has been successfully reserved, a downstream browser/action failure does not silently refund that use unless a separately specified compensating transaction is both safe and auditable. At the expiry boundary (`now >= expires_at`) no new reservation is permitted. Immediately before release, the broker rechecks that the reserved handle, requested scope, and classification still match and that revocation or lifecycle state has not invalidated the disclosure.

The active sensitive-model policy stack extends this authority kernel without turning metadata into protected-value access. Full-field model disclosure first composes the exact `SensitiveDataRequest` with independently reviewed model route/invocation authority. Only an otherwise authorized request reaches the necessity gate. `ModelDisclosureNecessityEvidence` is bound to that same exact tenant, task, field, purpose, canonical destination, and classification request and carries an exclusive validity horizon. Evidence derived for another request fails closed as `NecessityAuthorityMismatch`; zero or expired evidence also fails closed. A fresh request-bound `NoLowerDisclosurePath` result may preserve an otherwise authorized decision only after the trusted broker/orchestrator has derived it from current executable alternatives. Untrusted page/model content cannot mint necessity authority, and the policy object itself is not proof that the classification was truthfully derived.

The original first kernel intentionally did not implement storage, encryption, tokenization, model disclosure, provider or region policy, retention, audit persistence, break-glass access, or a broker. The active stack now adds metadata-only model route, invocation, and request-bound necessity composition, but it still does not carry protected bytes, authenticate provider/runtime identity, attest physical region or clock provenance, resolve broker values, invoke a model, validate output, execute fallback, or implement storage, encryption, tokenization, retention, audit persistence, break-glass, or durable broker service state. Those remain separate authority and lifecycle boundaries rather than being inferred from this primitive.

## Consequences

- Raw protected bytes are structurally absent from the first policy API.
- A caller with the wrong tenant, task, field, purpose, destination, or classification cannot reuse another disclosure scope or opaque handle.
- A later field reclassification fails closed against an older handle instead of inheriting the old disclosure class.
- Missing, oversized, whitespace-bearing, control-bearing, Unicode, or otherwise malformed tenant, task, field, or purpose identifiers cannot become authority through equality with another invalid scope; destination validity is guaranteed by the canonical `Origin` boundary.
- A stale, reclassified, or exhausted opaque handle fails closed in the pure predicate, while the future broker must enforce classification, expiry, and use-count consumption atomically before value resolution.
- Approval-required disclosure outcomes cannot fall through directly to execution; the exact scope is re-evaluated after approval evidence is obtained.
- Model-disclosure necessity evidence cannot be replayed across a different exact tenant/task/field/purpose/destination/classification request, even when that other request has separate disclosure and invocation authority.
- Later UI, connector, model, export, and browser-fill adapters can reuse the same explicit decision boundary without inheriting ambient authority.
- The complete enterprise gap is not closed by this kernel; independently reusable storage/broker/service contracts, evidence, lifecycle controls, and end-to-end tests are still required.

## Rejected alternatives

### Blanket masking

Rejected because some authorized operational workflows require the real value and a permanent masked copy can diverge from the authoritative record.

### Ambient trusted-network or session access

Rejected because network or session membership is not a sufficient authorization decision and creates confused-deputy and propagation risk.

### Sending every protected value through the model

Rejected because many actions can operate through opaque handles or deterministic trusted adapters. Model disclosure must remain a separately governed exceptional path.

### Classification-free opaque handles

Rejected because a handle issued while a field is classified as ordinary personal data could otherwise be reused after the same field is reclassified as sensitive personal, credential, or payment data. Classification is an authority dimension, not mutable display metadata.

### Caller-managed handle-use counters

Rejected because two concurrent callers can present the same stale `uses_so_far` value and both appear admissible. The mutable count, trusted clock, revocation state, and compare-and-increment operation belong to the trusted broker's authoritative state boundary.

## Verification

Tests must prove exact-scope disclosure, canonical destination behavior, denial on every authority-dimension mismatch, fail-closed behavior for missing or malformed authority, acceptance at the exact 128-byte identifier bound, rejection beyond that bound, rejection of whitespace/control/Unicode identifiers, every supported disclosure result, opaque-handle classification mismatch, expiry, use-count exhaustion, and destination mismatch. Sensitive-model tests must additionally prove that necessity metadata cannot upgrade denied base authority, that request-bound necessity succeeds only for the same exact sensitive request, that cross-request necessity replay fails closed, that zero and exclusive-expiry boundaries fail closed, and that every modeled lower-disclosure alternative blocks full-field model input. The broker slice must add classification-change, concurrency, replay, expiry-boundary, post-reservation failure, revocation, and atomic compare-and-increment tests before any protected-value resolution is described as implemented. Production function, line, region, and branch coverage remains exactly 100%.

## References

Chandramouli, R., & Butcher, Z. (2023). *A zero trust architecture model for access control in cloud-native applications in multi-cloud environments* (NIST Special Publication 800-207A). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207A

European Parliament & Council of the European Union. (2016). *Regulation (EU) 2016/679 of the European Parliament and of the Council of 27 April 2016 (General Data Protection Regulation)*. *Official Journal of the European Union, L 119*, 1–88. https://eur-lex.europa.eu/eli/reg/2016/679/oj

OWASP Foundation. (n.d.). *Logging cheat sheet*. OWASP Cheat Sheet Series. Retrieved August 9, 2026, from https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html
