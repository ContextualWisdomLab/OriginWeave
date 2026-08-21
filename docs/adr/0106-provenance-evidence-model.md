# ADR 0106: Provenance-native evidence model

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

OriginWeave must not merely act; it must let a user, operator, auditor, or downstream system establish what was observed, authorized, executed, and verified. Browser logs alone are not sufficient because they collapse observation, policy, action, network identity, approvals, and post-conditions. At the same time, evidence can itself contain sensitive or attacker-controlled content. The product needs a durable model compatible with web-archive and provenance concepts without claiming that every conceptual record is already persisted.

## Active implementation boundary

The extraction lane currently implements one bounded, verified in-memory WARC 1.1
`resource` record contract over already-authorized bytes. It binds the WARC target
URI to independently verified provenance, computes a SHA-256 block digest, and
emits deterministic record bytes. This is active-PR evidence rather than a claim
of durable storage, tenant retention, request/response capture, or PROV export.

## Decision drivers

- `Browse. Act. Prove.` requires evidence as a first-class product output.
- Source observation, policy decision, action execution, and verification must remain distinct authorities.
- Evidence should support WARC-style capture and PROV-style derivation where useful.
- Credentials and unnecessary PII must not be copied into evidence.
- Crash recovery and export need stable identities and integrity metadata.

## Assumptions and authority boundaries

Evidence records describe events and artifacts; they do not retroactively authorize them. Browser/network payloads are untrusted content even when archived. Policy decisions and approval evidence are control-plane records. Raw secrets are outside the evidence model. Conceptual entities in the ERD are not automatically claims of database persistence.

## Options considered

1. Store only human-readable logs: rejected because provenance, machine verification, and linkage are weak.
2. Capture complete browser/network data indiscriminately: rejected because privacy, retention, and secret exposure become unacceptable.
3. Typed provenance graph with selective WARC/content records and explicit policy/action links: selected.

## Decision

OriginWeave maintains provenance-native evidence with stable identifiers for sessions, snapshots, network exchanges, content records, policy decisions, approvals, action events, artifacts, and verification outcomes. WARC-compatible records may preserve eligible web exchanges or content; PROV-style relations describe derivation and responsibility without making either external format the sole internal authority. Evidence binds logical origin, resolved destination, TCP peer, TLS identity, HTTP semantics, browser/session/document identity, action intent, policy outcome, approval reference, and post-condition where applicable. Sensitive values are omitted, redacted, tokenized, or represented by opaque handles/fingerprints according to policy.

WARC and PROV are interoperability/export contracts, not substitutes for OriginWeave's internal authorization or evidence schema. A WARC record can contain untrusted or sensitive payload bytes and therefore inherits capture, retention, encryption, and export policy. A PROV entity/activity/agent relation records derivation or responsibility; it cannot manufacture authentication, authorization, durable completion, or tenant ownership not established by the producing system.

### Versioned extraction-schema binding

A versioned `ExtractionSchema` is the binding contract for typed extraction before any capture persistence or export format is allowed to claim semantic authority. Each schema version contains an ordered, non-empty set of unique `ExtractionField` definitions. Schema-version and field identifiers are bounded to 128 encoded bytes, begin with a lowercase ASCII letter, and thereafter admit only lowercase ASCII letters, digits, `_`, or `-`. One schema admits at most 256 fields.

Every extraction field binds its stable identifier to a value type, cardinality, required/optional status, deterministic normalization rule, and a non-empty duplicate-free set of reviewed source-channel classes. `Verbatim` is the compatibility default used by the existing constructor. `TrimTextWhitespace` is admitted only for text fields and `Rfc3339Utc` only for timestamp fields; type-incompatible normalization fails closed. A `ModelInterpretation` source channel is classification metadata only and does not grant model execution, approval, disclosure, browser, network, secret, or storage authority.

At this value-object boundary, the version identifier is immutable schema identity; there is deliberately no registry that silently treats two different field contracts as compatible merely because their version strings compare or sort in a particular way. Callers changing a field identifier, value type, cardinality, required status, normalization rule, or admitted source-channel set must use a distinct reviewed schema version and perform any migration/compatibility decision at an explicit higher layer. The current schema object does not itself read browser data, materialize extracted values, persist artifacts, execute models, or change governance policy. Those capabilities require separately authorized runtime boundaries and are not implied by schema construction.

## Consequences

Capture becomes a designed product surface rather than incidental logging. Storage and retention need budgets. Consumers can distinguish a model claim from source evidence and an action request from verified completion. Export adapters can target WARC, provenance graphs, audit streams, or buyer-specific schemas.

A schema consumer can also determine the exact field/type/cardinality/normalization/source contract it reviewed rather than relying on free-form extraction instructions. Schema evolution is explicit instead of being inferred from mutable field definitions; runtime compatibility, migrations, durable storage, and extracted-value validation remain separate implementation work until those boundaries are delivered.

## Failure and degraded behavior

If mandatory evidence cannot be recorded durably enough for a governed state-changing action, the action fails before execution or reports an explicit unverifiable failure; it is never marked proved. Read-only operations may degrade to reduced evidence only when the API contract declares that mode. Corrupt or incomplete evidence is quarantined rather than silently accepted.

Invalid or oversized extraction identifiers, empty or duplicate field sets, missing or duplicate source channels, and type-incompatible normalization rules fail during schema construction. A caller must not reinterpret such a failure as an empty/default-success schema or silently substitute another source channel.

## Security / privacy / governance impact

Evidence is tenant-scoped, selectively disclosed, encrypted as appropriate, retention-bounded, and auditable. Credential-bearing headers, cookies, secret values, and sensitive form data are excluded or transformed according to explicit schema policy. Integrity metadata and immutable artifact identities support tamper detection without claiming external certification. `docs/DATA_GOVERNANCE.md` defines the disclosure/retention boundary for protected content and derived artifacts.

The extraction-schema contract does not modify governance authority. It describes admissible typed fields and reviewed evidence-channel classes only. In particular, declaring `NetworkResponse` or `ModelInterpretation` does not authorize network access, model execution, protected-data disclosure, approvals, retention, or export; those remain governed by their existing owning boundaries.

## Tests and acceptance evidence

Require provenance-link tests, credential-leak tests, integrity/corruption tests, crash-recovery tests, WARC/export conformance where implemented, PROV relation/schema tests where implemented, retention/deletion tests, tenant-isolation tests, and end-to-end checks that state-changing actions link request, policy, approval, execution, and post-condition as separate records. Export tests must prove that disabled or unauthorized source bodies never appear merely because metadata provenance is exportable.

The extraction-schema boundary additionally requires tests for the identifier grammar and limits, field-count bound, duplicate identifiers, source-channel presence and uniqueness, every reviewed value/cardinality/source-channel variant, deterministic normalization selection, incompatible normalization rejection, and the backward-compatible `Verbatim` constructor default.

## Migration and rollback

Introduce stable evidence identifiers and schema versions before changing export formats. Migrations preserve old evidence semantics or explicitly mark unavailable fields. Rollback may revert an exporter but cannot collapse mandatory action and policy evidence into opaque logs.

Extraction contract changes that alter field identity or semantics require a new reviewed schema version rather than mutating the meaning of an existing version. Rolling back a consumer may stop accepting a newer version, but it must not reinterpret that newer contract as an older one or silently discard required fields.

## Open follow-ups

Finalize canonical evidence schemas, content-retention defaults, signing/attestation strategy, cross-system export identifiers, and buyer-controlled disclosure policies. Add the runtime that validates concrete extracted values against an `ExtractionSchema`, plus explicit migration/compatibility policy when durable schema registration is introduced.

## Supersession / reversal conditions

Supersede only if a different evidence model provides equal or better derivation, integrity, privacy, interoperability, crash recovery, and machine-verifiable separation of observation, policy, action, and verification.

## References

International Organization for Standardization. (2017). *Information and documentation—WARC file format* (ISO 28500:2017). https://www.iso.org/standard/68004.html

Lebo, T., Sahoo, S., & McGuinness, D. (Eds.). (2013). *PROV-O: The PROV ontology* (W3C Recommendation). World Wide Web Consortium. https://www.w3.org/TR/prov-o/

Moreau, L., & Missier, P. (Eds.). (2013). *PROV-DM: The PROV data model* (W3C Recommendation). World Wide Web Consortium. https://www.w3.org/TR/prov-dm/

## Related documents

See ADR 0003, `docs/erd/README.md`, `docs/API_CONTRACT.md`, `docs/OPERABILITY.md`, and `docs/DATA_GOVERNANCE.md`.
