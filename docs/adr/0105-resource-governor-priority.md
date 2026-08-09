# ADR 0105: Resource governor and browser-over-model priority

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

OriginWeave may run Chromium rendering, capture, local inference, remote-model orchestration, extraction, evidence processing, and multiple isolated sessions on the same host. CPU, RAM, GPU, VRAM, file descriptors, sockets, and disk are finite safety resources. A model or crawler that starves the browser can corrupt task semantics, crash tabs, lose evidence, or make a state-changing action unverifiable. Enterprise operation also requires predictable admission and degradation rather than best-effort overcommit.

## Decision drivers

- Preserve interactive/browser correctness before optional model throughput.
- Bound CPU, RAM, GPU, VRAM, network, storage, and concurrency per task/tenant.
- Make resource exhaustion an explicit policy/evidence event.
- Prefer deterministic degradation over host instability.
- Support capacity planning and SLOs.

## Assumptions and authority boundaries

The resource governor is a Rust control-plane authority. Browser and model runtimes report telemetry but cannot self-grant additional budget. A task's resource budget is separate from its action capabilities and cannot be enlarged by webpage content or model output.

## Options considered

1. Let the OS scheduler arbitrate all contention: rejected because it cannot express product priority or task budgets.
2. Give model inference priority to maximize agent throughput: rejected because browser starvation can invalidate observation/action correctness.
3. Explicit resource governor with browser correctness above optional model acceleration: selected.

## Decision

Every automated session receives an explicit resource budget and admission decision. The governor accounts for CPU, RAM, GPU/VRAM, browser processes, model processes, network concurrency, and evidence/storage pressure. Browser execution needed to preserve current task correctness and verifiable state has priority over optional local model acceleration. Under GPU/VRAM pressure the model degrades first: reduce concurrency, use CPU or remote inference when policy permits, or pause/fail the model path. Browser workloads are still bounded and may be rejected rather than overcommitted.

## Consequences

Throughput may be lower than unconstrained best effort, but failures become attributable and recoverable. Capacity planning can use explicit budgets. Local-model features need declared fallback semantics. Resource evidence becomes part of operability and buyer-visible reliability.

## Failure and degraded behavior

Admission failure occurs before launching work that cannot fit. Runtime pressure may suspend optional model work, reduce capture fidelity within documented bounds, or fail the task before a state-changing step. The governor must not kill evidence or browser processes in a way that falsely records task success. Recovery starts from an explicit checkpoint or fresh session after resources are available.

## Security / privacy / governance impact

Budgets mitigate resource-exhaustion attacks from hostile pages, crawls, model outputs, and tenants. Tenant quotas prevent noisy-neighbor denial of service. Telemetry must avoid leaking page or secret content and should record quantities, owners, and decisions rather than sensitive payloads.

## Tests and acceptance evidence

Require deterministic admission tests, CPU/RAM/GPU/VRAM pressure tests, browser-vs-model priority tests, concurrent tenant/session tests, crash/restart tests, bounded queue tests, and evidence proving no state-changing action is marked verified when the browser needed for post-condition checking was evicted. Performance tests must record workload and hardware assumptions.

## Migration and rollback

Introduce accounting in observe-only mode before enforcing limits, then enable admission per resource class. Rollback may disable a new optimization or fallback but must retain hard safety bounds already required to prevent host instability.

## Open follow-ups

Define production default budgets, hardware classes, tenant quota APIs, remote-model fallback policy, and capacity/SLO dashboards.

## Supersession / reversal conditions

Supersede only if another scheduler demonstrates equivalent fail-closed resource isolation, tenant fairness, browser correctness, observability, and materially better utilization in representative workloads.

## References

See `docs/OPERABILITY.md`, `docs/TEST_STRATEGY.md`, `docs/erd/README.md`, and the `originweave-resource` crate.
