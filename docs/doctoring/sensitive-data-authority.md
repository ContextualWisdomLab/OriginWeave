# Purpose-bound sensitive-data authority doctoring

- **Status:** Implemented policy-kernel evidence; broker/storage/lifecycle remain planned under issue #10.
- **Decision:** [`../adr/0007-purpose-bound-sensitive-data-authority.md`](../adr/0007-purpose-bound-sensitive-data-authority.md)

## Evidence boundary

OriginWeave treats protected-value disclosure as explicit resource authorization rather than as a consequence of network location, session ownership, administrator status, repository membership, or possession of a model credential. The first Rust policy kernel carries only authority metadata and binds disclosure/opaque-handle admission to tenant, task, field, business purpose, canonical destination origin, and data classification.

This evidence supports the architectural direction; it does **not** prove legal compliance, CSAP certification, SOC 2 conformity, or that the planned trusted broker/storage lifecycle is implemented.

## Primary and authoritative references — APA 7th

Chandramouli, R., & Butcher, Z. (2023). *A zero trust architecture model for access control in cloud-native applications in multi-cloud environments* (NIST Special Publication 800-207A). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207A

European Parliament & Council of the European Union. (2016). *Regulation (EU) 2016/679 of the European Parliament and of the Council of 27 April 2016 (General Data Protection Regulation)*. *Official Journal of the European Union, L 119*, 1–88. https://eur-lex.europa.eu/eli/reg/2016/679/oj

OWASP Foundation. (n.d.). *Logging cheat sheet*. OWASP Cheat Sheet Series. Retrieved August 9, 2026, from https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html

Rose, S., Borchert, O., Mitchell, S., & Connelly, S. (2020). *Zero trust architecture* (NIST Special Publication 800-207). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207

## Test traceability

- `crates/originweave-policy/tests/sensitive_data_policy.rs` — exact authority dimensions, malformed identifier fail-closed behavior, canonical origin binding, disclosure outcomes, expiry/use-count boundaries.
- `crates/originweave-policy/tests/handle_classification.rs` — field reclassification invalidates prior opaque-handle authority.
- The later broker slice must add atomic compare-and-increment, replay, concurrent-use, trusted-time, revocation, and post-reservation failure tests before value resolution is described as implemented.
