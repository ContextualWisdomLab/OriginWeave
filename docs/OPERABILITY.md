# OriginWeave Operability Baseline

- **Status:** Proposed authoritative operations baseline
- **Product status:** Pre-alpha; no production SLA is claimed
- **Architecture:** [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- **Threat model:** [`THREAT_MODEL.md`](THREAT_MODEL.md)
- **Release/rollback:** [`RELEASE_AND_ROLLBACK.md`](RELEASE_AND_ROLLBACK.md)

## 1. Purpose

OriginWeave is intended to run long-lived interactive and autonomous browser workloads under explicit security/resource authority. Operability therefore includes more than process uptime: operators must know whether the correct browser/task authority exists, whether a task can make safe progress, what external effect may already have happened, how evidence can be inspected, and how a release can be rolled back without silently weakening security.

This document defines the target operational contract and marks pre-alpha gaps rather than inventing SLA commitments.

## 2. Service modes

Expected deployment modes are:

- interactive OriginWeave Browser;
- headless OriginWeave Runtime;
- embedded module in another CWL product;
- future managed multi-tenant service.

All modes share the Rust authority/evidence contracts. They may use different browser/process/persistence adapters but must not invent weaker semantics.

## 3. Health model

### Liveness

Answers only whether the process/supervisor can make progress and respond to control operations. It does not claim browser readiness, model-provider readiness, destination reachability, or task success.

### Readiness

Requires the configured execution profile's mandatory local dependencies to be ready. For example, an interactive browser profile may require Chromium/process-control health while a model-backed autonomous task may additionally require the selected model route.

A missing optional provider must not make deterministic non-model functions globally unready.

### Task readiness

Evaluated per task using exact policy, session/context, required adapter, secret/provider, resource, tenant and destination authority. One unavailable dependency blocks only tasks that need it.

## 4. State model

Operational task states should remain explicit:

```text
created
authorized
active
observing
planning
awaiting_approval
acting
verifying
paused
recovering
completed
failed
cancelled
quarantined
```

`quarantined` is required when an external side effect may have occurred but current evidence cannot safely determine whether retry would duplicate or conflict with it.

## 5. SLI catalogue

Production profiles define exact computation and aggregation for each **SLI**. Candidate SLIs include:

### Availability and task success

- control-plane request availability;
- session creation success rate;
- delegated task completion rate;
- human-approved action success rate;
- recovery success after browser/adapter crash;
- provider-specific availability separated from product availability.

### Latency

- human input-to-render latency;
- task queue wait;
- observation generation latency;
- policy decision latency;
- connection/TLS/HTTP latency;
- model invocation latency;
- action-to-post-condition verification latency;
- end-to-end task latency by class.

### Safety/correctness

- unauthorized-action attempts blocked;
- stale-node action rejection;
- post-condition failure/uncertainty rate;
- destination/TLS/HTTP policy rejection classes;
- prompt-injection successful-action rate;
- provenance completeness;
- secret/PII leakage detections.

### Resource

- process/task/tab peak RSS;
- CPU worker saturation;
- GPU/VRAM pressure events;
- compositor/frame-health degradation;
- observation cache bytes;
- inference batch/offload/CPU-fallback rate.

### Operations

- restart/crash frequency;
- task cancellation completion time;
- checkpoint age;
- quarantine backlog;
- evidence/export failure rate;
- release/rollback success.

## 6. SLO model

OriginWeave does not publish numerical **SLO** values in pre-alpha documentation without benchmark/operational evidence. Release profiles must define:

- which SLI is covered;
- target/threshold and measurement window;
- workload/hardware/browser/model assumptions;
- excluded planned maintenance;
- error budget policy;
- paging/ticket threshold;
- customer communication and remediation obligation where applicable.

Security invariants such as cross-tenant disclosure or unauthorized high-risk action are not normalized into a permissive error budget; their acceptance threshold is zero for the tested supported conditions.

## 7. Observability

Target integration uses OpenTelemetry-compatible metrics, traces and structured events, but telemetry never becomes a raw-secret store.

### Required correlation identifiers

Where implemented and safe:

```text
tenant_record_id
agent_session_id
browsing_context_id
action_intent_id
policy_decision_id
action_event_id
provenance_record_id
release_version
runtime_instance_id
```

Opaque identifiers are preferred. Do not put cookies, authorization headers, personal values or full unbounded page text into labels/attributes.

### Event families

- task/session lifecycle;
- browser/adapter lifecycle;
- policy/approval decision;
- network authority transition;
- action/post-condition transition;
- resource pressure and mitigation;
- model invocation metadata;
- persistence/evidence outcome;
- operator action and break-glass event;
- release/rollback event.

## 8. Logging contract

Logs are operational summaries, not primary evidence. Primary evidence uses typed credential-free records.

Logs must:

- use structured bounded fields;
- record causal identifiers and safe error classes;
- avoid raw protected values, headers, bodies and unrestricted URLs;
- distinguish expected policy denial from infrastructure failure;
- distinguish retryable transient error from deterministic failure;
- include exact software/build identity where useful.

Operator tooling must support scanning logs/support bundles for synthetic protected test values.

## 9. Alerting and triage

### Page-level conditions

Examples requiring immediate response in a production profile:

- suspected cross-tenant access;
- secret/PII occurrence in disallowed telemetry/evidence;
- unexpected privileged action without matching policy/approval evidence;
- signing/provenance verification failure on deployed artifact;
- widespread crash loop or task corruption;
- evidence integrity/tamper failure.

### Ticket-level conditions

- rising provider/model timeout rate;
- repeated task quarantine;
- persistent resource saturation;
- extension compatibility regression;
- increased stale-node/post-condition failures;
- repeated destination/TLS/HTTP rejects indicating upstream drift.

## 10. Incident response flow

```text
detect
-> identify exact tenant/session/release/authority boundary
-> stop unsafe new admission
-> preserve credential-free evidence
-> quarantine ambiguous tasks/artifacts
-> assess external side effects
-> contain/revoke credentials or routes as needed
-> reproduce at smallest safe boundary
-> remediate and verify
-> rollback/release fixed artifact
-> post-incident control update
```

Do not retry an ambiguous externally mutating action merely to learn whether the first attempt succeeded.

## 11. Break-glass operations

A **break-glass** path is Planned for enterprise operation and must never be implemented as a reusable administrator bypass.

Required contract:

- explicit incident/support reason;
- authenticated eligible operator;
- separate approval/dual control for high-risk data access;
- narrow tenant/task/data scope;
- short expiry;
- no implicit extension to future tasks;
- elevated logging/evidence;
- post-event review;
- immediate revocation path.

Break-glass does not disable origin/destination/TLS/evidence or release integrity controls.

## 12. Quarantine and recovery

Use **quarantine** when state cannot be safely retried or discarded automatically. Examples:

- action request was transmitted but post-condition was not observable;
- browser crashed during externally visible mutation;
- evidence integrity mismatch;
- persistence state disagrees with external service;
- tenant/policy changed during operation and side effects are ambiguous.

A quarantined task may be:

- inspected read-only;
- reconciled against external authoritative state;
- completed manually;
- compensated through a separately authorized action;
- cancelled with evidence;
- resumed only after exact current authority is re-established.

## 13. Cancellation

Cancellation is cooperative but must have a bounded hard-stop escalation for local components. The task record distinguishes:

- cancel requested;
- model/network/browser operation interrupted before external effect;
- external effect may have committed;
- browser context closed;
- resources reclaimed;
- evidence finalized.

A cancelled task cannot silently reuse secret handles or approvals on later restart.

## 14. Retry and backoff

Retry policy is allow-listed by causal error class.

- deterministic validation/permission/origin/identity/malformed-input errors: do not retry;
- selected transient network/provider states: bounded retry under one end-to-end deadline;
- ambiguous external mutation: quarantine/reconcile instead of automatic retry;
- broker/resource unavailability: defer task or fail closed according to task deadline, never substitute weaker authority.

Backoff/circuit breaker belongs to the orchestration/adapter layer and cannot change lower-level security decisions.

## 15. Capacity management

Capacity profiles include:

- max simultaneous browser sessions/contexts;
- CPU worker pool capacity;
- RAM and observation-cache budgets;
- GPU/VRAM reserved interactive capacity;
- model residency/batch limits;
- network/download limits;
- object/evidence storage rates;
- queue depth and maximum task age.

Admission control uses current resource evidence and reviewed budgets. A declared capacity number without a load test is not a supported capacity claim.

## 16. Browser and model upgrade operations

### Chromium

A supported update requires:

- security/update rationale;
- API/protocol diff assessment;
- OriginWeave adapter build/test;
- MV3 compatibility suite;
- real task/observation/action regression;
- sandbox/Site Isolation preservation;
- canary and rollback artifact.

### Model/provider

Model updates are separately versioned from deterministic browser authority. Evaluate routing/tool use/schema compliance, prompt-injection robustness, task success, unsupported actions/claims, cost/latency and provider policy. Provider changes cannot silently expand protected-data eligibility.

## 17. Persistence backup and recovery

Planned durable adapters must define per data class:

- authoritative vs reconstructible data;
- backup frequency and retention;
- encryption/key recovery;
- recovery point objective and recovery time objective when production SLOs exist;
- tenant deletion/legal hold behavior;
- artifact/digest verification after restore;
- evidence continuity and replay.

Raw secrets use a dedicated trusted secret system and are not restored from general audit/evidence backups.

## 18. Data retention and deletion

Retention is purpose/data-class/tenant aware. Operators must be able to identify all declared durable copies of:

- authoritative sensitive values;
- task/session metadata;
- screenshots/downloads;
- WARC source artifacts;
- model request/output artifacts;
- provenance/audit evidence;
- backups.

Deletion/expiry produces metadata receipts where appropriate without retaining the deleted protected value.

## 19. Security operations

- dependency/SAST/security findings are triaged against exact release code;
- vulnerabilities use private coordinated disclosure;
- credentials/keys have rotation/revocation playbooks;
- egress and service identities are monitored against declared configuration;
- privileged operator access is reviewable;
- supply-chain provenance is verified before deployment.

## 20. Deployment/configuration safety

Configuration is versioned, validated and fail closed. Invalid limits, origins, routes, trust roots, model/provider policy, retention, tenant identity or required credentials prevent the affected feature/task from starting. Startup does not silently apply permissive defaults to recover from malformed security configuration.

## 21. Operational acceptance for scheduler/automation fixes

A CI-green workflow change is not operational proof. After protected merge, perform a real scheduled/manual run demonstrating the intended path on the protected exact head. Evidence includes trigger, head SHA, gating reason, credential boundary, action taken/not taken and resulting authoritative repository state.

## 22. Runbook ownership

As product surfaces become concrete, maintain focused runbooks for at least:

- browser/session crash loop;
- task quarantine/reconciliation;
- secret/PII leakage response;
- TLS/destination policy outage;
- model/provider outage;
- extension/Chromium compatibility regression;
- resource saturation;
- persistence/evidence integrity incident;
- release rollback;
- tenant deletion/export;
- break-glass access.

This baseline defines the common rules; surface-specific operational commands must live next to the actual implementation and be tested where automation is safe.
