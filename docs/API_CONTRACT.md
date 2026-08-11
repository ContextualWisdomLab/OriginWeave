# OriginWeave API and Protocol Contract

- **Status:** Proposed product-wide API baseline
- **Product status:** Pre-alpha; no public compatibility guarantee yet
- **Technical requirements:** [`TRD.md`](TRD.md)
- **Conceptual model:** [`erd/README.md`](erd/README.md)
- **Threat model:** [`THREAT_MODEL.md`](THREAT_MODEL.md)

## 1. Purpose

OriginWeave needs one stable authority model even though different deployments may use Chromium internals, WebDriver BiDi, Chrome DevTools Protocol, WebMCP, MCP, desktop IPC or an embedded Rust API. This document defines the direction of the **OriginWeave Protocol** and the rules every public/internal adapter must preserve.

This is a contract baseline, not a claim that the complete network service or SDK is implemented on protected `main`.

## 2. Protocol design goals

- transport-neutral;
- versioned and schema-first;
- tenant/task/session scoped;
- opaque identifiers rather than raw browser/process pointers;
- deterministic authority validation outside model output;
- explicit deadlines, cancellation and idempotency;
- bounded request/response sizes;
- typed errors and degraded states;
- evidence identifiers returned separately from sensitive values;
- state-changing success tied to an observed **post-condition**;
- adapter-specific identifiers translated through scoped registries.

## 3. Non-goals

The protocol does not:

- expose arbitrary JavaScript as the standard action interface;
- expose browser cookies/passwords/API keys to the caller/model;
- make CDP/WebDriver IDs durable OriginWeave IDs;
- accept raw page text as policy;
- collapse origin/destination/route/TCP/TLS/HTTP authority into one URL field;
- treat protocol authentication alone as task authorization;
- guarantee every future Chromium/CDP/WebMCP feature.

## 4. Versioning

Before 1.0, every serialized contract carries a protocol version such as:

```json
{
  "protocol_version": "originweave/0.1"
}
```

Compatibility policy before 1.0:

- additive compatible fields may appear behind explicit version/capability negotiation;
- removing/renaming/reinterpreting an existing field requires a version change;
- security semantics never become permissive merely for backward compatibility;
- adapters reject unknown mandatory capability/version combinations;
- supported WebDriver/CDP/WebMCP/MCP versions are documented independently from the OriginWeave protocol version.

## 5. Identity vocabulary

Planned external wire objects use descriptive identifiers such as:

```text
tenant_record_id
agent_session_id
browser_profile_id
browsing_context_id
page_snapshot_id
semantic_node_id
action_intent_id
policy_decision_id
action_event_id
postcondition_evidence_id
provenance_record_id
secret_handle_id
```

These values are **opaque**. Clients must not infer ordering, ownership, browser process, database placement or authority from their syntax.

External adapter IDs such as CDP `backendNodeId`, WebDriver element references or browser process IDs are never accepted as global authority by themselves.

## 6. Common envelope

A request that can mutate or access tenant-scoped state includes an authenticated transport/session plus a logical envelope similar to:

```json
{
  "protocol_version": "originweave/0.1",
  "tenant_record_id": "opaque",
  "agent_session_id": "opaque",
  "request_id": "opaque",
  "idempotency_key": "opaque",
  "deadline_at": "2026-08-09T10:00:00Z"
}
```

The transport authenticates the caller/workload. The envelope scopes the operation. Neither alone grants an action capability.

## 7. Idempotency

Every externally replayable state-changing command defines explicit **idempotency** behavior.

Rules:

- idempotency keys are scoped by tenant, session, operation kind and semantic request contract;
- a reused key with different semantic content fails deterministically;
- an idempotency record does not authorize the operation; policy is still revalidated as required;
- retries after ambiguous external side effects may return `quarantined`/reconciliation-required rather than re-execute;
- idempotency retention is bounded and declared;
- secret-handle max-use semantics remain independent of request idempotency.

## 8. Deadline and cancellation

Requests use one end-to-end deadline. Lower layers consume the remaining budget rather than reset a fresh unlimited timeout.

Cancellation distinguishes:

- cancellation before any external side effect;
- cancellation while outcome is known not to have committed;
- cancellation after a side effect may have committed;
- local browser/model/network cancellation completion;
- task state requiring quarantine/reconciliation.

## 9. Capability negotiation

A session exposes a bounded capability document, for example:

```json
{
  "supported_operations": [
    "browser.navigate",
    "browser.observe",
    "browser.query",
    "browser.act",
    "browser.extract",
    "browser.checkpoint"
  ],
  "observation_channels": [
    "structured_data",
    "accessibility_dom_layout"
  ],
  "adapter_profiles": {
    "webdriver_bidi": "planned",
    "cdp": "planned",
    "webmcp": "experimental_optional"
  }
}
```

Supported operation does not mean the current task is authorized to use it. Per-task policy/capability/origin/risk checks remain separate.

## 10. Session operations

Planned high-level surface:

### `browser.create_session`

Inputs:

- requested execution mode;
- execution purpose;
- profile/isolation request;
- resource-budget profile;
- allowed policy profile reference.

Outputs:

- `agent_session_id`;
- effective execution mode/purpose;
- profile isolation class;
- supported capability metadata;
- session evidence reference.

Does not return raw profile/cookie state.

### `browser.close_session`

Closes or begins closing browser contexts, revokes task-scoped handles/approvals according to policy and finalizes evidence. Closing a session does not erase audit evidence whose retention is separately governed.

## 11. Navigation operations

### `browser.navigate`

Accepts a canonical target contract, not ambient browser URL mutation.

Preconditions include:

- session/mode/purpose valid;
- navigation capability;
- canonical origin authority;
- destination/route/network/TLS/HTTP authority through the actual adapter path;
- deadline/resource admission.

Output includes resulting browsing context/document authority and evidence references. Redirects follow the independently governed chain; one approved origin does not grant every redirect.

### `browser.go_back`

Requires current context validity and returns a new/updated document epoch when navigation changes the actionable document.

## 12. Observation operations

### `browser.observe`

Request selects bounded channels/fields, for example:

```json
{
  "observation_request": {
    "channels": ["structured", "accessibility", "dom", "layout"],
    "maximum_nodes": 2000,
    "maximum_text_bytes": 262144
  }
}
```

Output is a `page_snapshot` plus typed `semantic_node` records and source/evidence references. Sensitive values are excluded or represented by separately authorized handles/derived values.

### `browser.observe_changes`

Returns incremental changes after a known snapshot/epoch. If the underlying actionable document changed incompatibly, returns an explicit stale/epoch-changed result rather than attempting to merge incompatible snapshots.

## 13. Query operations

### `browser.query`

Queries the OriginWeave semantic observation contract; callers do not need to synthesize unrestricted CSS/XPath/JavaScript.

Query predicates may include role, accessible name, state, structured field, source channel and scoped layout attributes. Results contain opaque semantic-node handles bound to the current session/context/origin/document epoch.

## 14. Action operations

### `browser.act`

Request is a typed action intent, conceptually:

```json
{
  "action_kind": "click_node",
  "semantic_node_id": "opaque",
  "expected_origin": "https://example.com",
  "risk_class": "R2",
  "post_condition": {
    "kind": "dialog_visible",
    "timeout_ms": 5000
  }
}
```

The caller/model cannot self-authorize `risk_class`, capability or approval simply by setting fields; the deterministic control plane computes/revalidates effective policy.

Immediately before dispatch, the runtime revalidates:

- session/context/document/node authority;
- task capability and purpose;
- origin/destination/secret scope as relevant;
- exact approval evidence where required;
- resource/deadline state.

Success response is not produced until the declared/derived **post-condition** is observed. Outcomes include `succeeded`, `failed`, `timed_out`, `cancelled`, `quarantined` and typed policy/stale errors.

## 15. Secret/sensitive operations

### `browser.fill_secret`

The caller supplies an `secret_handle_id` or other approved sensitive handle, never the raw value. The trusted broker rechecks audience, tenant, task, field, purpose, origin, expiry, revocation, max uses and approval before filling through a trusted adapter.

The response contains only access/evidence metadata and post-condition state.

### `sensitive.resolve_derived`

Where policy permits, a caller may request a derived safe representation rather than the original value. The derivation and retention policy are typed and versioned; no generic “unmask” method exists.

### Selective model disclosure

A full protected field is not eligible for model input merely because sensitive-data policy and a reviewed model route both authorize it. The trusted broker/orchestrator must first derive current necessity from executable alternatives. If an opaque handle, deterministic transform, local rule, structured tool, or approved derived value can satisfy the task, the policy boundary returns a typed lower-disclosure-path denial rather than full-field model authorization. A `no lower disclosure path` assertion is metadata, not self-authenticating proof: untrusted page/model content cannot mint it, and protected bytes remain resolved only inside the trusted value boundary after policy and lifecycle revalidation.

## 16. Extraction operations

### `browser.extract`

Inputs:

- versioned extraction schema;
- allowed source channels;
- field-level evidence requirements;
- bounded record/byte limits.

Outputs:

- typed values;
- verification state/confidence where applicable;
- source/evidence identifiers;
- model/provider/prompt provenance when interpretation was needed.

A value without the required evidence is omitted, rejected or explicitly marked unsupported according to the schema contract; it is not silently promoted to verified.

## 17. Download operations

### `browser.download`

Requires action/destination/resource policy and returns a `download_artifact_id` plus digest/MIME/source evidence. Filenames are metadata only until a persistence adapter sanitizes and authorizes a destination. Content is not executed by the protocol layer.

## 18. Checkpoint and recovery operations

### `browser.checkpoint`

Creates a bounded recoverable task/browser checkpoint where the adapter supports it. The response identifies what state is persisted and what external side effects are not replayable.

### `browser.reconcile`

Planned operator/runtime operation for quarantined tasks. It compares external authoritative state with OriginWeave evidence before an action can be compensated, retried or manually completed.

## 19. Evidence operations

### `evidence.get`

Returns a caller-authorized typed evidence record. Sensitive generic fields remain redacted. Evidence access itself is tenant/policy scoped.

### `evidence.export`

Produces a versioned evidence/provenance bundle under export/retention policy. WARC/PROV/JSON/other encodings are adapters to the logical evidence model.

## 20. Error model

Wire errors have stable machine codes and safe human messages, for example:

```json
{
  "error": {
    "code": "stale_document_epoch",
    "message": "The observed node no longer belongs to the current document.",
    "retryable": false,
    "evidence_id": "opaque"
  }
}
```

Properties:

- no raw credentials/protected bodies;
- deterministic validation/policy/identity errors marked nonretryable;
- transient infrastructure/provider errors distinguishable;
- error cause layer preserved;
- an error code never reveals another tenant's resource existence.

## 21. Pagination/streaming/backpressure

Large observations/extractions/downloads use bounded pagination or streaming with declared maximum chunk/message sizes. The runtime can stop producers when resource budgets are hit. Client refusal to consume data does not create unbounded buffering.

## 22. Concurrency

- sessions/contexts are independently scoped;
- node/action validation occurs at execution time;
- secret handle use is atomic at the trusted broker;
- one protocol request cannot hold an unbounded global lock;
- mutable task transitions use explicit compare/version or equivalent serialization semantics;
- duplicated/concurrent state-changing requests obey idempotency/quarantine rules.

## 23. Authentication vs authorization

Protocol transport authentication proves the calling human/workload identity. Authorization additionally depends on tenant, session, purpose, capability, origin, action/risk, data scope, destination and approvals. “Authenticated” is never equivalent to “can browse/fill/export anything.”

## 24. Adapter mappings

### WebDriver BiDi

Maps session/user-context/browsing-context capabilities into scoped OriginWeave identities. BiDi element/context identifiers remain adapter-local.

### Chrome DevTools Protocol

Maps selected versioned Network/DOMSnapshot/Accessibility/Tracing and other explicitly reviewed domains. `Runtime.evaluate` is not automatically mapped to standard `browser.act`.

### WebMCP

Maps site-provided tool descriptors and results into untrusted typed observations/tool proposals. Experimental upstream status is preserved; missing WebMCP falls through to other observation channels.

### Model Context Protocol

Exposes the bounded high-level OriginWeave operations to orchestrators. MCP server credentials do not expose raw Chromium, cookie or secret state.

## 25. Public Rust API

The Rust crates remain independently reusable. Wire DTOs must not force core crates to depend on HTTP/MCP/CDP. Translation layers convert between transport schemas and stable core value objects.

## 26. Change control

A change to identity scope, action semantics, error/retry behavior, idempotency, secret/sensitive disclosure, post-condition truth, protocol capability, evidence semantics or version compatibility requires TRD/traceability updates and normally a dedicated ADR when it changes a governing authority boundary.

No external SDK can be declared stable until the wire schemas, compatibility policy and conformance tests are published on a protected release.
