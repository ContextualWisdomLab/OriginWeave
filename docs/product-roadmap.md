# Product Roadmap

## Product north star

A buyer can delegate a bounded web task and receive the result, the exact evidence that supports it, the policy decisions that authorized it, and a replayable record—without giving the model ambient browser authority.

## Phase 0 — Safety kernel

Status: complete as a reusable pre-alpha foundation.

- typed session, purpose, origin, capability, action, risk, secret, robots, and approval contracts;
- fail-closed deterministic action policy;
- cumulative interactive-first resource mitigation;
- universally value-redacted evidence and provenance pointers;
- 100% production function, line, region, and branch coverage plus public documentation;
- an hourly bounded OpenCode product-development workflow separated from review and merge authority.

## Phase 1 — Isolated Chromium vertical slice

Status: in progress.

Delivered destination-policy foundation:

- IPv4 and IPv6 special-purpose classification;
- IPv4-mapped IPv6 canonicalization;
- public-only default and explicit managed class policies;
- non-empty, origin-bound approved resolution snapshots;
- concrete connection-address pinning;
- DNS answer contraction with expansion treated as possible rebinding;
- per-hop redirect origin, resolution, HTTPS downgrade, exact-target cycle, and hop-limit reauthorization;
- credential-free connection and redirect evidence.

Delivered direct TCP foundation:

- an independently reusable `originweave-network` crate;
- one explicit canonical `SocketAddr` authorized by a resolution snapshot;
- a non-cloneable plan consumed by one bounded connection sequence;
- exact single-address `TcpStream::connect_timeout` use without hostname re-resolution;
- direct-only routing without ambient proxy inheritance;
- exact IP-and-port comparison with the operating-system peer before stream exposure;
- credential-free origin, requested-peer, observed-peer, class, attempt, and timeout evidence;
- real loopback integration and deterministic transport-failure tests.

Remaining vertical-slice work:

- launch and terminate ephemeral Chromium user contexts;
- WebDriver BiDi adapter behind a versioned interface;
- navigation and accessibility-tree observation;
- document epoch and stale-node invalidation;
- typed `navigate`, `observe`, `query`, and `click` actions;
- post-condition verification and audit events;
- crash recovery and task checkpoint;
- perform DNS resolution in a trusted browser-network adapter and prove that Chromium's actual connection consumes the approved snapshot and direct socket authority;
- define the exact interval between resolution approval and socket use and close remaining time-of-check/time-of-use races;
- validate TLS server name, certificate chain, validity period, revocation policy, and negotiated ALPN against the logical origin and verified peer;
- define proxy and PAC behavior explicitly so a proxy cannot silently bypass destination policy;
- separately authorize every intermediate proxy and final target;
- re-evaluate the implemented origin, resolved-address, capability, transport, and action-risk gates on every redirect in the live adapter;
- bound connection count, response headers, body bytes, redirects, download bytes, and elapsed time;
- validate download MIME type and declared versus observed content before persistence;
- retain complete DNS, connection, address, TLS, redirect, proxy, and policy-decision evidence without exposing credentials.

A syntactically canonical origin is an identity input, not an SSRF boundary. An approved resolver address is a policy decision, and an exact TCP peer is transport evidence; neither substitutes for TLS identity or proof that Chromium used the governed path. Phase 1 cannot claim safe real navigation until the trusted browser adapter composes all of these boundaries.

Commercial proof: one controlled workflow completes repeatedly without selector scripts, raw secrets, unverified success, destination-policy bypass, DNS rebinding, peer substitution, TLS identity confusion, unsafe downgrade, redirect cycles, proxy bypass, or unbounded download behavior.

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

- accessible task, approval, secret, destination, connection, and evidence UI designed in Figma;
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
- DNS rebinding, IPv4-mapped IPv6, alternate numeric hosts, redirect-to-private-address, unsafe downgrade, exact-target redirect cycles, proxy bypass, metadata endpoints, oversized downloads, MIME confusion, connection refusal, timeout, peer-inspection failure, peer mismatch, and partial connection failures;
- task success, safety, provenance completeness, latency, memory, VRAM, and cost.

## Explicit non-goals

- rewriting Blink or V8 in Rust;
- supporting NPAPI, Flash, or obsolete plugin models;
- CAPTCHA bypass or fingerprint-evasion features;
- arbitrary script execution as a default agent action;
- sharing the user's unrestricted default profile with autonomous tasks;
- describing a pure policy or direct TCP kernel as a supported production browser.
