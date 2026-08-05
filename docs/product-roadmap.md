# Product Roadmap

## Product north star

A buyer can delegate a bounded web task and receive the result, the exact evidence that supports it, the policy decisions that authorized it, and a replayable record—without giving the model ambient browser authority.

## Phase 0 — Safety kernel

Status: in development.

- typed session, purpose, origin, capability, action, risk, secret, robots, and approval contracts;
- fail-closed deterministic policy;
- interactive-first resource directives;
- redacted evidence and provenance pointers;
- 100% production coverage and public documentation.

## Phase 1 — Isolated Chromium vertical slice

- launch and terminate ephemeral Chromium user contexts;
- WebDriver BiDi adapter behind a versioned interface;
- navigation and accessibility-tree observation;
- document epoch and stale-node invalidation;
- typed `navigate`, `observe`, `query`, and `click` actions;
- post-condition verification and audit events;
- crash recovery and task checkpoint.

Commercial proof: one controlled workflow completes repeatedly without selector scripts, raw secrets, or unverified success.

## Phase 2 — Agent-native scraping

- structured-data extraction;
- redacted network-response capture;
- DOM, accessibility, layout, iframe, and shadow-DOM semantic view;
- incremental diffs and bounded observation cache;
- WARC and PROV-JSON-LD adapters;
- schema-bound extraction with field-level source evidence;
- RFC 9309 crawler policy, rate limits, retention, and purpose records.

Commercial proof: extracted fields meet precision/recall and provenance-completeness thresholds across a versioned benchmark corpus.

## Phase 3 — External agent platform

- Browser Agent Protocol 1.0;
- local and remote MCP server;
- contextual-orchestrator adapter;
- single-model versus multi-agent compute policy;
- provider-neutral model gateway and NVIDIA NIM live tests;
- task cancellation, idempotency, retries, and dead-letter evidence.

Commercial proof: multiple orchestrators use the same runtime without bypassing browser policy.

## Phase 4 — Resource and extension compatibility

- real process RSS, JS heap, cache, GPU, and frame telemetry;
- fixed CPU worker-pool and oversubscription controls;
- phase-scheduled local inference and GPU rendering;
- 4/6/8/12/24 GiB VRAM profiles and CPU fallback;
- Manifest V3 install/update/service-worker/DNR/native-messaging test farm;
- extension-to-agent isolation and contamination tests.

Commercial proof: declared performance and extension compatibility are reproducible on supported hardware and platforms.

## Phase 5 — Enterprise product

- accessible task, approval, secret, and evidence UI designed in Figma;
- SSO, SCIM, tenant isolation, managed policies, data residency, and immutable audit;
- encrypted profiles and regional object storage;
- observability, SLOs, incident response, upgrade, and rollback;
- SBOM, attestations, reproducible release, procurement, and support package.

Commercial proof: a regulated enterprise can approve, operate, audit, and renew the product.

## Benchmark program

Each phase expands a stable benchmark suite:

- controlled malicious and benign pages;
- Mind2Web-derived task diversity;
- WebArena/VisualWebArena-style repeatability;
- WASP prompt-injection scenarios;
- stale nodes, hidden instructions, cross-origin transitions, CAPTCHA handoff, renderer crash, network failure, and memory pressure;
- task success, safety, provenance completeness, latency, memory, VRAM, and cost.

## Explicit non-goals

- rewriting Blink or V8 in Rust;
- supporting NPAPI, Flash, or obsolete plugin models;
- CAPTCHA bypass or fingerprint-evasion features;
- arbitrary script execution as a default agent action;
- sharing the user's unrestricted default profile with autonomous tasks;
- describing a prototype as a supported production browser.
