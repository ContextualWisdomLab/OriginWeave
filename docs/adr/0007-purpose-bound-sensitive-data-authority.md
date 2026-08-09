# ADR 0007: Purpose-bound sensitive-data authority

- Status: Accepted for the first authority kernel
- Date: 2026-08-09

## Context

Enterprise browser workflows sometimes require real personal or otherwise protected values. Blanket masking can make a legitimate shipment, form fill, customer contact, reconciliation, or incident workflow impossible. Ambient raw access has the opposite failure mode: values can propagate into models, logs, traces, exports, support channels, or unrelated services merely because a caller already has network or session access.

OriginWeave therefore treats each sensitive-data disclosure as a separate resource-access decision. Network location, session ownership, repository membership, administrator status, or possession of a model credential does not grant disclosure authority. This follows the same explicit-authority architecture used for origins, resolved destinations, TCP peers, TLS identities, actions, and approvals.

NIST SP 800-207A models application and service identities as policy inputs rather than relying on network location. The GDPR purpose-limitation and data-minimisation principles likewise require processing to remain tied to specified purposes and limited to what is necessary for them. OWASP logging guidance warns that personal and other sensitive information can become a secondary exposure through application logs. These sources motivate the architecture; this ADR does not itself establish legal compliance or certification.

## Decision

The first implementation lives inside `originweave-policy` as a bounded preparatory authority kernel while the repository's lockfile-governance constraint prevents safely adding the separately versioned `originweave-sensitive-data` workspace crate required by the complete issue. The final issue remains open until that standalone crate and service contract exist.

The kernel carries authority metadata but never the protected value. A disclosure request and its explicit scope are bound to:

- tenant identity;
- task identity;
- field identity;
- declared business purpose;
- destination;
- data classification.

All authority identifiers must be present before policy can grant disclosure or opaque-handle use. Two equally incomplete scopes are not valid authority merely because their missing identifiers compare equal; incomplete authority fails closed.

An exact match may return only the explicitly configured disclosure decision: deny, opaque handle only, derived value only, partial field disclosure, full field disclosure, human approval required, or dual control required. Any authority mismatch or incomplete authority fails closed to denial.

Opaque handle use is separately bound to tenant, task, field, purpose, destination, exclusive expiry time, and maximum use count. The policy function only authorizes handle use; it does not resolve the handle or return the protected value. Resolution belongs in a later trusted broker or browser adapter that rechecks the same authority immediately before disclosure.

The first kernel intentionally does not implement storage, encryption, tokenization, model disclosure, provider or region policy, retention, audit persistence, break-glass access, or a broker. Those remain separate authority and lifecycle boundaries rather than being inferred from this primitive.

## Consequences

- Raw protected bytes are structurally absent from the first policy API.
- A caller with the wrong tenant, task, field, purpose, destination, or classification cannot reuse another disclosure scope.
- Missing tenant, task, field, purpose, or destination identifiers cannot become authority through equality with another incomplete scope.
- A stale or exhausted opaque handle fails closed before any value resolution can occur.
- Later UI, connector, model, export, and browser-fill adapters can reuse the same explicit decision boundary without inheriting ambient authority.
- The complete enterprise gap is not closed by this kernel; independently reusable storage/broker/service contracts, evidence, lifecycle controls, and end-to-end tests are still required.

## Rejected alternatives

### Blanket masking

Rejected because some authorized operational workflows require the real value and a permanent masked copy can diverge from the authoritative record.

### Ambient trusted-network or session access

Rejected because network or session membership is not a sufficient authorization decision and creates confused-deputy and propagation risk.

### Sending every protected value through the model

Rejected because many actions can operate through opaque handles or deterministic trusted adapters. Model disclosure must remain a separately governed exceptional path.

## Verification

Tests must prove exact-scope disclosure, denial on every authority-dimension mismatch, fail-closed behavior for incomplete authority, every supported disclosure result, opaque-handle expiry, use-count exhaustion, and destination/audience mismatch. Production function, line, region, and branch coverage remains exactly 100%.

## References

Chandramouli, R., & Butcher, Z. (2023). *A zero trust architecture model for access control in cloud-native applications in multi-cloud environments* (NIST Special Publication 800-207A). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207A

European Parliament & Council of the European Union. (2016). *Regulation (EU) 2016/679 of the European Parliament and of the Council of 27 April 2016 (General Data Protection Regulation)*. *Official Journal of the European Union, L 119*, 1–88. https://eur-lex.europa.eu/eli/reg/2016/679/oj

OWASP Foundation. (n.d.). *Logging cheat sheet*. OWASP Cheat Sheet Series. Retrieved August 9, 2026, from https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html
