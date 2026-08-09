# OriginWeave Data Governance and Privacy Boundary

- **Document status:** Proposed authoritative product baseline
- **Product status:** Pre-alpha
- **Scope:** OriginWeave-owned runtime contracts plus explicit host/enterprise ownership boundaries
- **Related:** [`PRD.md`](PRD.md), [`TRD.md`](TRD.md), [`THREAT_MODEL.md`](THREAT_MODEL.md), [`OPERABILITY.md`](OPERABILITY.md), [`erd/README.md`](erd/README.md), [`traceability/README.md`](traceability/README.md)

## 1. Purpose

OriginWeave must support enterprise web tasks that genuinely require personal, confidential, credential, or customer data without solving privacy risk by blanket masking that makes the task impossible. The product therefore rejects both ambient raw-value propagation and indiscriminate masking.

The governing rule is:

> **Keep authoritative data usable by explicitly authorized business processes while replacing ambient propagation with purpose-bound, field-scoped, just-in-time disclosure and durable evidence that does not copy the protected value.**

This document defines the product data-governance contract. It is not a legal opinion, certification, or claim that every planned enterprise control is already implemented.

## 2. Status discipline

Protected `main` is the implementation authority.

- **Implemented:** protected-main code and tests provide the stated behavior.
- **Accepted architecture:** a governing ADR/architecture decision establishes the direction, but the complete runtime path may not yet exist.
- **In progress:** an active PR implements part of the contract; it is not shipped behavior.
- **Planned:** roadmap-aligned but not yet implemented.
- **Proposed:** requires additional reviewed design or implementation evidence.

Current protected-main foundations include deterministic policy/evidence types, canonical origin authority, bounded resource/evidence contracts, and credential-free evidence principles. Purpose-bound sensitive-data policy and sensitive-access receipts are active development lines and must not be represented as protected-main completion until merged.

## 3. Data classes

OriginWeave uses explicit classifications rather than assuming every value has the same handling requirements. Product integrations should map local classifications into a versioned policy domain that can represent at least:

- `public_data`;
- `internal_data`;
- `personal_data`;
- `sensitive_personal_data`;
- `credential_data`;
- `payment_data`; and
- customer-defined classes with an explicit policy version.

A classification is policy input, not authorization. Classification changes invalidate previously derived disclosure authority when the new class is not covered by the original decision.

## 4. Authority model

Every sensitive-data transition is a distinct resource-access decision. Network location, repository membership, administrator role, possession of a model key, or ownership of a browser session never implies full disclosure.

A disclosure decision must bind, where applicable:

- tenant identity;
- human/workload/service/device identity;
- authenticated session identity;
- delegated task identity;
- exact field identifiers and record scope;
- business purpose;
- action kind;
- canonical destination origin or connector;
- data classification;
- model/provider/region/retention mode when AI is involved;
- requested retention/export behavior;
- approval or break-glass evidence; and
- policy/control version.

Supported decision outcomes are conceptually:

- `deny_access`;
- `opaque_handle_only`;
- `derived_value_only`;
- `partial_field_disclosure`;
- `full_field_disclosure`;
- `human_approval_required`; and
- `dual_control_required`.

An approval-required result is a control state, not execution authority. The exact authority must be re-evaluated after approval and immediately before disclosure.

## 5. Opaque handles and trusted broker boundary

The planner/model receives an opaque handle by default, not the protected value. The trusted broker or trusted browser adapter resolves a handle only after rechecking the current authority.

A future production `sensitive_value_handle` must be bound to at least:

- one tenant;
- one task;
- one field set;
- one business purpose;
- one destination/audience;
- one data classification;
- an exclusive expiry;
- a bounded maximum use count; and
- a policy version or invalidation epoch.

Atomic reservation/increment, trusted current time, revocation, concurrent/replay protection, value resolution, and compensation semantics are trusted-broker responsibilities. A pure policy predicate must never be documented as if it atomically consumes a handle.

Handles must not be serialized into long-term model memory, URLs, screenshots, telemetry, crash dumps, clipboard history, public provenance, or browser-visible page content.

## 6. Model disclosure

Raw protected data may enter a model request only when deterministic or handle-based execution cannot satisfy the approved task and an explicit model-disclosure policy authorizes it.

The decision must bind the exact fields, purpose, provider, model, region, retention/training mode, request lifetime, output schema, and approval class. Provider outage or policy mismatch fails closed; it must not silently fall back to a less trusted provider or region.

Model credentials remain outside model-visible context. `NVIDIA_NIM_API_KEY` is a development/approved runtime credential only where an explicit model-backed path requires it and never becomes an application data-governance shortcut.

Model output is untrusted data. It cannot create new disclosure authority, expand a field set, change a destination, or override deterministic browser/network policy.

## 7. Browser and page disclosure

A trusted browser fill may reveal a value to an approved page only after the runtime revalidates:

- browser session and browsing context;
- current document epoch;
- canonical page origin;
- actionable node/field identity;
- delegated task and purpose;
- field classification;
- handle expiry/use state;
- current approval/policy version; and
- post-condition expectations.

Protected values must not be exposed through hidden DOM, accessibility labels, page logs, screenshots, clipboard state, or unrelated form fields merely because the approved page can receive one value.

A successful input command is not durable evidence that the intended business operation succeeded. The trusted adapter must observe the expected post-condition and record the action result separately from disclosure authority.

## 8. Data stores and persistence ownership

OriginWeave core currently does not claim a production application database. The conceptual ERD therefore distinguishes runtime/domain entities from persistence that is host-owned or planned.

Where an embedding product persists sensitive information, recommended conceptual records include:

- `sensitive_data_record`;
- `sensitive_field_definition`;
- `sensitive_data_request`;
- `sensitive_access_decision`;
- `sensitive_value_handle`;
- `sensitive_access_evidence`;
- `business_purpose_record`;
- `field_disclosure_policy`;
- `tenant_security_policy`;
- `model_disclosure_policy`;
- `provider_region_policy`;
- `encryption_key_reference`;
- `retention_policy_record`;
- `approval_evidence_record`;
- `break_glass_event`;
- `control_evidence_mapping`;
- `audit_sequence_record`; and
- `deletion_receipt_record`.

Physical persistence requires a separate accepted ADR, tenant model, migrations, backup/restore, retention, encryption, authorization, audit, and rollback design. All owned database objects use descriptive two-or-more-word `snake_case` names.

## 9. Encryption, tokenization, and derived values

Protected values at rest require encryption appropriate to the deployment boundary. Product architecture should support versioned key references, rotation, revocation, customer-managed-key policy where required, and cryptographic-deletion evidence without putting key material into evidence records.

Reversible tokenization, deterministic encryption, or format-preserving transforms are allowed only for an explicit interoperability or join requirement with documented residual risk. Hashing must not be described as anonymization where the original value space is enumerable or linkable.

Derived values are independently classified. A summary, embedding, screenshot, token, model output, or provenance artifact is not automatically non-sensitive merely because it differs from the source value.

## 10. Logging and observability

Raw protected values are excluded by default from:

- application logs;
- tracing/span attributes;
- metric labels;
- exception messages;
- GitHub issues and CI logs;
- crash/support bundles;
- analytics events;
- screenshots and clipboard history;
- model caches outside the approved disclosure contract;
- WARC/PROV exports unless a separate authorized evidence-retention policy explicitly allows protected content.

Observability uses identifiers, classifications, policy versions, outcomes, counts, durations, and bounded reason codes instead of protected values.

Credential-free evidence is not the same as public evidence. Tenant, actor, task, revision, provider, and destination metadata can remain confidential and must follow access and retention policy.

## 11. Sensitive-access evidence

A `sensitive_access_evidence` record must carry enough metadata to reconstruct the decision without copying the protected value. The evidence model should include, where applicable:

- decision/request identifier;
- tenant, actor, workload, device, and task identifiers;
- field identifiers and classification;
- business purpose and canonical destination;
- decision outcome and policy/control versions;
- approval or break-glass reference;
- handle issuance/resolution/revocation metadata;
- model/provider/region/retention mode when applicable;
- encryption-key reference/rotation epoch without key material;
- decision, disclosure, completion, and retention-deadline times; and
- success/denial/partial/policy-change outcome.

Tamper-evident sequencing is a planned enterprise evidence capability. A local immutable Rust value object is not by itself a durable append-only audit service.

## 12. Retention, deletion, and legal hold

Retention is attached to the artifact class, purpose, tenant policy, and legal/contractual requirement rather than a global timer.

The lifecycle must distinguish:

- task-local ephemeral material;
- operational records;
- authorized model artifacts;
- export artifacts;
- audit/control evidence;
- customer-configured retention; and
- legal hold.

Deletion/revocation must propagate to caches, temporary files, search/vector derivatives, exports, model caches, and backup-expiry workflows as applicable. Deletion receipts retain identifiers and completion evidence but not the deleted value.

Backup expiry and legal-hold behavior are deployment responsibilities and must not be invented by a core-library API.

## 13. Data residency and provider policy

Tenant policy may restrict storage, browser execution, model inference, support access, export, or evidence retention by region/provider. A destination or provider that violates the current region policy is denied rather than used as a fallback.

Residency rules apply to derived artifacts too. Model responses, embeddings, screenshots, WARC bundles, diagnostics, and exported provenance cannot silently cross a region boundary simply because the original record remained local.

## 14. Break-glass and support access

Cross-tenant or privileged support access is disabled by default. A break-glass path must require:

- explicit reason;
- appropriate approval or dual control;
- exact resource/purpose scope;
- short expiry;
- heightened monitoring;
- non-transferable identity;
- post-event review; and
- evidence that cannot be reused as a standing role grant.

Break-glass does not bypass network/service identity, tenant isolation, retention, or evidence requirements.

## 15. WARC, PROV, screenshots, and export

OriginWeave's evidence-first product direction supports WARC-compatible source capture and PROV-compatible derivation export, but neither format creates a privacy exemption.

A capture/export policy must classify and authorize:

- raw HTTP bodies;
- headers/cookies/credentials;
- screenshots;
- DOM/semantic snapshots;
- extracted values;
- model inputs/outputs;
- action evidence; and
- derived provenance.

Public/shareable evidence uses synthetic or approved public fixtures. Production protected content requires a separate purpose, audience, retention, encryption, and export authority.

## 16. Tenant and service identity

Every storage, queue, cache, object, search/vector, model, connector, export, support, and audit boundary must enforce tenant/resource identity in the embedding deployment.

Service-to-service disclosure uses authenticated workload/service identity and explicit authorization. An authorized service must still be prevented from acting as a confused deputy for an unauthorized tenant, task, destination, or connector.

OriginWeave does not treat a shared static credential as service identity.

## 17. Accessibility and data minimization

Accessibility cannot be implemented by leaking protected values into hidden DOM or accessible names. Authorized UI disclosure must preserve keyboard and assistive-technology use while limiting exposed fields to the current task step.

UI surfaces should explain why access was granted, what category of data is in use, and when authority expires without exposing protected values in telemetry or unrelated status text.

## 18. Compliance evidence boundary

OriginWeave designs controls and evidence for CSAP/SOC 2 readiness but does not claim certification from code or documentation alone.

A future `control_evidence_mapping` must distinguish:

1. product capability;
2. configured control;
3. operating control;
4. collected evidence;
5. management assertion; and
6. independent certification/examination result.

CSAP marks may be claimed only for the certified cloud service boundary. SOC 2 statements depend on the actual system description, included Trust Services Criteria, operating period, management assertion, and independent examination.

## 19. Degraded behavior

Data-governance failures are fail-closed for the affected disclosure:

- unavailable policy -> no protected disclosure;
- unavailable broker -> no handle resolution;
- stale/expired handle -> no resolution;
- approval unavailable -> approval-required action remains blocked;
- provider/region mismatch -> no model fallback outside policy;
- audit/evidence failure where evidence is mandatory -> no high-risk completion claim;
- post-condition uncertainty -> action outcome is `unknown`/reconciliation-required rather than success.

These failures block only the affected task/action. They do not justify weakening another tenant's policy or disabling unrelated deterministic browser functionality.

## 20. Test and acceptance contract

Release-relevant data-governance tests must include realistic flows, not only string redaction:

- authorized shipping/form fill uses the required full value at the exact approved origin without exposing it to planner/model/log/evidence;
- wrong tenant/task/field/purpose/origin/classification fails closed;
- stale, expired, over-used, revoked, or replayed handle fails;
- concurrent handle use cannot cross audience or use-count authority;
- provider/model/region/retention policy changes invalidate an otherwise valid model disclosure;
- break-glass requires reason/approval/expiry and cannot create a durable role grant;
- log/trace/metric/crash/support/screenshot/clipboard/WARC/PROV scanners contain no unapproved protected bytes;
- key rotation/revocation and retention/deletion transitions have deterministic evidence;
- hostile Unicode and serialization ambiguity cannot broaden identifiers or policy authority;
- production functions/lines/regions/branches remain exactly 100% covered for OriginWeave-owned Rust behavior.

Synthetic or explicitly approved fixtures are mandatory; production personal data is forbidden in repository tests.

## 21. Standards and research traceability

The governing references live in [`doctoring.md`](doctoring.md), the product-baseline addendum, feature-specific doctoring, and the ADR corpus. This document specifically relies on the product principles established by:

- NIST SP 800-207 and SP 800-207A for explicit identity/resource policy rather than implicit network trust;
- NIST AI 600-1 for generative-AI risk-management inputs without replacing deterministic authorization;
- current KISA CSAP program requirements as a certification-readiness input, never a self-certification claim; and
- AICPA Trust Services Criteria as an assurance/control-evidence reference rather than a product feature checklist.

Material legal or certification claims must be revalidated against the law, program criteria, contracts, deployment region, and operating procedures effective at the release date.

## 22. Rollback and supersession

This document may be superseded only through reviewed changes that preserve or explicitly migrate:

- the no-ambient-disclosure principle;
- purpose/field/destination/classification binding;
- broker/opaque-handle separation;
- protected-value exclusion from generic telemetry/evidence;
- host-vs-core persistence ownership; and
- truthful certification/compliance claims.

A rollback must not reintroduce blanket masking as the only privacy mechanism or ambient raw-value propagation as the convenience path.