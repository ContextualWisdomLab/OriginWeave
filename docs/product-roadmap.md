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

Delivered proxy-route authority foundation:

- direct-only routing by default with no ambient proxy inheritance;
- bounded exact allow-lists for Chromium-compatible proxy server identities and PAC source origins;
- separate proxy-server schemes for HTTP, HTTPS, SOCKS4, SOCKS5, and QUIC, including scheme-specific default-port canonicalization;
- separate authority for explicit proxy, PAC-selected proxy, and PAC-selected `DIRECT` routes;
- exact canonical target origin, proxy server identity, and PAC source origin retained as credential-free route evidence;
- no DNS, socket I/O, PAC execution, proxy authentication, CONNECT, or Chromium side effect in the policy layer;
- explicit separation between route authorization and the destination, TCP peer, TLS identity, and live PAC/browser adapters that must enforce the selected route.

Delivered direct TCP foundation:

- an independently reusable `originweave-network` crate;
- one explicit canonical `SocketAddr` authorized by a resolution snapshot;
- a non-cloneable plan consumed by one bounded connection sequence;
- exact single-address `TcpStream::connect_timeout` use without hostname re-resolution;
- direct-only routing without ambient proxy inheritance;
- exact IP-and-port comparison with the operating-system peer before stream exposure;
- credential-free origin, requested-peer, observed-peer, class, attempt, and timeout evidence;
- real loopback integration and deterministic transport-failure tests.

Delivered TLS service-identity foundation:

- an independently reusable `originweave-tls` crate;
- one single-use handshake over an existing `DirectTcpConnection`, with no reconnect, DNS, or proxy inheritance;
- exact equality between the canonical HTTPS origin and the origin that authorized the TCP stream;
- RFC 9525 DNS and literal-IP reference identities, DNS-only SNI, no Common Name fallback, and exact IPv4/IPv6 SAN handling;
- explicit immutable trust roots and fixed trusted verification time;
- TLS 1.2 and TLS 1.3 only, with resumption, 0-RTT, secret extraction, key logging, client certificates, certificate compression, and dangerous verifier hooks disabled;
- bounded total handshake time, ALPN, root input, and server-presented certificate evidence;
- operating-system peer revalidation before, during, and after the handshake;
- credential-free protocol, cipher, ALPN, certificate, SPKI, trust-bundle, validity, revocation-configuration, and timing evidence;
- real loopback rustls integration for trusted identity, wrong name, Common Name non-fallback, untrusted root, fixed-time validity, IPv4/IPv6 SAN, TLS versions, ALPN, and transport-origin binding.

Delivered document-node authority foundation:

- a nonzero `BrowserSessionId` for one active automation session;
- a nonzero `BrowsingContextId` for one independently navigable browser context inside that session;
- a nonzero `DocumentEpoch` identity for one observed document lifetime inside that context;
- an `ObservedNodeHandle` bound to the exact browser session, browsing context, canonical origin, document epoch, and nonzero adapter-local node identifier;
- same-call QueryNodes admission that transfers a SemanticObservation protocol-use proof by ownership into `bind_current_nodes` before translating admitted `locateNodes` `sharedId` values into those handles;
- deterministic rejection of cross-session, cross-context, cross-origin, or stale-document node reuse before a future browser adapter performs an action;
- reusable core contracts without Chromium, WebDriver, selector, script-execution, network, storage, or secret dependencies.

Remaining vertical-slice work:

- launch and terminate ephemeral Chromium user contexts;
- WebDriver BiDi adapter behind a versioned interface;
- session-scoped translation from external protocol identifiers to collision-free internal browser-session, browsing-context, document-epoch, and node identities;
- navigation and accessibility-tree observation;
- typed `navigate`, `observe`, `query`, and `click` actions;
- post-condition verification and audit events;
- crash recovery and task checkpoint;
- perform DNS resolution in a trusted browser-network adapter and prove that Chromium's actual connection consumes the approved snapshot, direct socket authority, and authenticated TLS stream;
- define the exact interval between resolution approval and socket use and close remaining time-of-check/time-of-use races;
- define revocation material acquisition and freshness without overstating the current `NotConfigured` evidence;
- implement trusted PAC evaluation and proxy transport adapters that consume explicit route authority without ambient environment fallback;
- separately authorize and authenticate every intermediate proxy and final target through the applicable destination, transport, and TLS boundaries;
- re-evaluate the implemented origin, resolved-address, capability, transport, TLS, and action-risk gates on every redirect in the live adapter;
- bound connection count, response headers, body bytes, redirects, download bytes, and elapsed time;
- validate download MIME type and declared versus observed content before persistence;
- retain complete DNS, connection, address, TLS, redirect, proxy, HTTP, and policy-decision evidence without exposing credentials.

A syntactically canonical origin is an identity input, not an SSRF boundary. A Chromium proxy server identity is a routing input whose scheme is part of its authority and is not interchangeable with the target web origin. An approved resolver address is a policy decision, an exact TCP peer is transport evidence, an authorized proxy route is a routing decision, and an authenticated TLS stream is service-identity evidence; none substitutes for proof that Chromium used the governed path or for bounded HTTP semantics. Phase 1 cannot claim safe real navigation until the trusted browser adapter composes all of these boundaries.

Commercial proof: one controlled workflow completes repeatedly without selector scripts, raw secrets, unverified success, cross-session, cross-context, cross-origin, or stale-node authority, destination-policy bypass, DNS rebinding, peer substitution, TLS identity confusion, unsafe downgrade, redirect cycles, proxy bypass, or unbounded download behavior.

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

- accessible task, approval, secret, destination, connection, TLS identity, and evidence UI designed in Figma;
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
- cross-session and cross-context node collisions, stale nodes, hidden instructions, cross-origin transitions, CAPTCHA handoff, renderer crash, network failure, and memory pressure;
- DNS rebinding, IPv4-mapped IPv6, alternate numeric hosts, redirect-to-private-address, unsafe downgrade, exact-target redirect cycles, proxy bypass, metadata endpoints, oversized downloads, MIME confusion, connection refusal, timeout, peer-inspection failure, peer mismatch, and partial connection failures;
- trusted and untrusted TLS roots, DNS and IP SANs, Common Name fallback attempts, expiry and future validity, TLS version negotiation, ALPN absence and mismatch, peer mutation, and handshake deadlines;
- task success, safety, provenance completeness, latency, memory, VRAM, connection time, handshake time, and cost.

## Explicit non-goals

- rewriting Blink or V8 in Rust;
- supporting NPAPI, Flash, or obsolete plugin models;
- CAPTCHA bypass or fingerprint-evasion features;
- arbitrary script execution as a default agent action;
- sharing the user's unrestricted default profile with autonomous tasks;
- describing a pure policy, proxy-route, direct TCP, or TLS identity kernel as a supported production browser.
