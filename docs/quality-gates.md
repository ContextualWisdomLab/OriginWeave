# Quality and Release Gates

## Pull-request gates

A PR is mergeable only when all apply:

- one coherent scope and current architecture alignment;
- realistic failing test observed before implementation;
- complete focused and full verification on the exact head;
- production function, line, region, and branch coverage each at 100%;
- all public Rust APIs documented and rustdoc warnings denied;
- format, check, test, Clippy, and documentation jobs pass;
- dependency and GitHub Action references are locked or commit-pinned;
- generated build outputs are ignored and absent from `git ls-files`;
- review threads are resolved with code or evidence;
- required independent approval and repository security checks pass;
- README, architecture, ADR, doctoring, and changelog are updated when affected.

## Safety-kernel gates

A policy, evidence, or resource change additionally requires:

- browser-special numeric-host and canonical-origin equivalence tests;
- R3 and R4 approval tests bound to action kind, target origin, and the complete canonical intent digest;
- proof that generic header and query values are universally redacted before evidence leaves the trusted boundary;
- exact field-count, name, value, path, source URL, and locator boundary tests;
- malformed percent-escape, encoded separator, and literal or encoded dot-segment cases;
- simultaneous RAM, VRAM, frame, batch, model-residency, and admission-pressure tests;
- proof that crossing a hard memory limit reduces the active consumer and rejects new work.

## Destination and direct-transport gates

A destination or network change additionally requires:

- reviewed IANA and cloud-platform endpoint classification fixtures;
- IPv4-mapped IPv6 canonicalization and special-purpose bypass tests;
- non-empty bounded origin-bound DNS snapshots;
- exact connection-address pinning and DNS answer expansion rejection;
- redirect origin, target-bound resolution, HTTPS downgrade, complete-target cycle, and hop-budget tests;
- an explicit canonical `SocketAddr` with no hostname reconnect or ambient proxy path;
- port, timeout, and attempt-count boundary tests;
- a real loopback TCP connection whose operating-system peer exactly matches the requested IP and port;
- deterministic timeout, refusal, retry, peer-inspection failure, and peer-mismatch tests;
- a compile-fail proof that consumed connection authority cannot be replayed.

## TLS service-identity gates

A TLS change additionally requires:

- one existing verified direct TCP stream; no DNS, reconnect, or proxy inheritance;
- exact equality between TLS origin and transport-authority origin;
- explicit immutable roots and fixed trusted verification time;
- TLS 1.2 and TLS 1.3 only, with obsolete versions absent from production configuration;
- disabled resumption, 0-RTT, secret extraction, key logging, client certificates, certificate compression, and dangerous custom verifier hooks unless a later ADR explicitly changes the boundary;
- bounded handshake deadline, ALPN inputs, trust roots, and server-presented certificate evidence;
- operating-system peer revalidation before, during, and after the handshake;
- real loopback rustls client/server integration over `DirectTcpConnection`;
- trusted DNS SAN success and proof that Common Name never replaces SAN;
- wrong-name, untrusted-root, expired, and not-yet-valid rejection;
- exact IPv4 and IPv6 SAN success;
- TLS 1.2 and TLS 1.3 negotiation tests;
- explicit required-ALPN rejection and optional-ALPN absence evidence;
- credential-free protocol, cipher, ALPN, certificate, SPKI, trust-bundle, validity, revocation-configuration, and timing evidence;
- documentation that server-presented certificate hashes are not mislabeled as a reconstructed validation path;
- no revocation-validation claim while the evidence status is `NotConfigured`.

## Browser vertical-slice gates

A browser or scraping feature additionally requires:

- isolated profile and origin capability tests;
- stale document-epoch and navigation tests;
- post-condition verification rather than command-return success;
- prompt-injection and hidden-content cases;
- secret non-disclosure in prompts, traces, logs, and provenance;
- Chromium crash/restart and task-checkpoint recovery;
- bounded request, response, snapshot, download, and artifact sizes;
- DNS-resolution, rebinding, redirect, TCP-peer, TLS-identity, proxy, private-address, link-local, metadata-endpoint, and partial-connection tests;
- MIME and declared-versus-observed content validation before persistence;
- keyboard and screen-reader-compatible approval and evidence UI;
- repeatable real-site or controlled-browser task benchmarks.

## Performance evidence

Report distributions, not isolated best runs:

- input and action latency;
- DNS, TCP connection, TLS handshake, HTTP header, body, and total elapsed time;
- compositor frame time and dropped frames;
- process and task peak RSS;
- JavaScript heap and semantic snapshot bytes;
- peak VRAM and CPU fallback reason;
- bytes transferred and disk spill;
- token use and model calls per task;
- task success, unauthorized-action rate, stale-node rate, and recovery success.

## Release gates

A supported release requires:

- explicit semantic version and dated `CHANGELOG.md` section;
- all default-branch checks green after merge;
- SBOM, source and build provenance, checksums, and signed/attested artifacts;
- reproducible build evidence;
- supported-platform compatibility matrix;
- Manifest V3 compatibility evidence for declared extension APIs;
- security threat model and external review appropriate to the release scope;
- privacy, retention, telemetry, upgrade, rollback, and incident procedures;
- documented support and vulnerability-response policy.

No workflow, issue label, test name, or configuration string can substitute for this evidence.
