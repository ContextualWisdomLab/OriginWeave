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

For Linux browser-task RSS, the resource boundary accepts measurements supplied by a trusted platform adapter and can read exactly one `/proc/<pid>/status` for an explicitly supplied nonzero PID. The caller or trusted browser/process adapter owns proof that the PID belongs to the governed browser task; `originweave-resource` does not discover Chromium processes, aggregate children, inspect cgroups, measure GPU/heap state, or provide cross-platform sampling. Linux documents `VmRSS` as process resident-memory information and also warns that RSS accounting may be asynchronous and imprecise, so this measurement is operational pressure telemetry rather than exact billing, forensic attribution, or process-ownership proof (Linux Kernel Documentation, n.d.).

## Options considered

1. Let the OS scheduler arbitrate all contention: rejected because it cannot express product priority or task budgets.
2. Give model inference priority to maximize agent throughput: rejected because browser starvation can invalidate observation/action correctness.
3. Explicit resource governor with browser correctness above optional model acceleration: selected.

## Decision

Every automated session receives an explicit resource budget and admission decision. The governor accounts for CPU capacity, RAM, GPU/VRAM, browser-process priority, model-process priority/residency, network concurrency, evidence/cache bytes, temporary/durable artifact pressure, file descriptors, and bounded queue/concurrency limits. Browser execution needed to preserve current task correctness and verifiable state has priority over optional local model acceleration. Under GPU/VRAM pressure the model degrades first: reduce concurrency or batch size, free model caches, use CPU or remote inference when policy permits, pause the model path, or fail it. Browser workloads are still bounded and may be rejected rather than overcommitted.

A resource-budget version and mitigation decision are evidence-bearing control-plane values. Admission and mitigation evidence record quantities, units, resource owner/task, policy version, trigger, and applied mitigation without carrying page content. A `cpu_worker_limit` is a count of OriginWeave-controlled CPU compute workers/admitted execution slots, not a percentage of total host CPU and not authority to change Chromium's internal scheduler.

## Consequences

Throughput may be lower than unconstrained best effort, but failures become attributable and recoverable. Capacity planning can use explicit budgets. Local-model features need declared fallback semantics. Resource evidence becomes part of operability and buyer-visible reliability.

## Failure and degraded behavior

Admission failure occurs before launching work that cannot fit. Runtime pressure may suspend optional model work, reduce capture fidelity within documented bounds, or fail the task before a state-changing step. The governor must not kill evidence or browser processes in a way that falsely records task success. Recovery starts from an explicit checkpoint or fresh session after resources are available.

## Security / privacy / governance impact

Budgets mitigate resource-exhaustion attacks from hostile pages, crawls, model outputs, and tenants. Tenant quotas prevent noisy-neighbor denial of service. Telemetry must avoid leaking page or secret content and should record quantities, owners, policy versions, priorities, and decisions rather than sensitive payloads.

## Tests and acceptance evidence

Require deterministic admission tests; CPU-worker, RAM, GPU/VRAM, network-concurrency, evidence/storage-pressure, file-descriptor, and queue pressure tests; browser-vs-model process-priority tests; concurrent tenant/session tests; crash/restart tests; bounded queue tests; and evidence proving no state-changing action is marked verified when the browser needed for post-condition checking was evicted. GPU tests must prove model offload/pause occurs before protected foreground browser capacity is sacrificed under the configured policy. Performance tests must record workload, browser build, model/runtime, and hardware assumptions.

## Migration and rollback

Introduce accounting in observe-only mode before enforcing limits, then enable admission per resource class. Rollback may disable a new optimization or fallback but must retain hard safety bounds already required to prevent host instability and must not silently widen a previously enforced task budget.

## Open follow-ups

Define production default budgets, hardware classes, tenant quota APIs, remote-model fallback policy, supported telemetry adapters, and capacity/SLO dashboards.

## Supersession / reversal conditions

Supersede only if another scheduler demonstrates equivalent fail-closed resource isolation, tenant fairness, browser correctness, observability, and materially better utilization in representative workloads.

## References

Chromium. (n.d.). *MemoryInfra*. Chromium source documentation. Retrieved August 9, 2026, from https://chromium.googlesource.com/chromium/src/+/HEAD/docs/memory-infra/README.md

Chromium. (n.d.). *Key concepts in Chrome memory*. Chromium source documentation. Retrieved August 9, 2026, from https://chromium.googlesource.com/chromium/src.git/+/HEAD/docs/memory/key_concepts.md

Linux Kernel Documentation. (n.d.). *The /proc filesystem*. Retrieved August 15, 2026, from https://www.kernel.org/doc/html/latest/filesystems/proc.html

V8 Project Authors. (2018, June 11). *Concurrent marking in V8*. https://v8.dev/blog/concurrent-marking

## Related documents

See `docs/OPERABILITY.md`, `docs/TEST_STRATEGY.md`, `docs/erd/README.md`, and the `originweave-resource` crate.