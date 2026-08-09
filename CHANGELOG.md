# Changelog

All notable changes to OriginWeave are documented in this file. The format follows Keep a Changelog, and releases use Semantic Versioning.

## [Unreleased]

### Added

- Rust workspace for independently reusable core, policy, destination, network, TLS, resource, and evidence modules.
- Canonical HTTPS and loopback-origin boundary with case-normalized schemes and hosts, default-port normalization, IPv4/IPv6 handling, browser-special numeric-host rejection, and explicit malformed-input errors.
- Typed browser actions, capabilities, risk classes, execution modes, robots decisions, secret-delivery contracts, immutable canonical action-intent digests, and intent-bound approval scopes.
- Deterministic fail-closed policy evaluation for untrusted instructions, origin grants, crawler restrictions, execution-mode and purpose consistency, approvals, and brokered secrets.
- Session- and context-bound node authority with nonzero browser-session, browsing-context, and document-epoch identities, adapter-local node identifiers, exact canonical-origin binding, and deterministic cross-session, cross-context, cross-origin, and stale-epoch rejection before a future browser adapter acts on an observed node.
- Fail-closed resolved-destination policy with IPv4/IPv6 special-purpose and reviewed cloud-platform endpoint classification, IPv4-mapped canonicalization, explicit class grants, non-empty origin-bound DNS snapshots capped at 256 resolver addresses, concrete connection pinning, DNS-set expansion detection, and per-hop redirect reauthorization.
- Explicit bounded proxy/PAC route authority in `originweave-destination`: direct-only by default, separately allow-listed Chromium-compatible proxy server identifiers and PAC source origins, independent authorization for PAC-selected DIRECT versus proxy routes, exact canonical target/proxy/PAC evidence, and no DNS, socket, PAC execution, CONNECT, authentication, or Chromium side effects.
- Direct-only `originweave-network` TCP boundary with explicit canonical `SocketAddr` authority, zero IPv6 flow and scope metadata unless separately modeled, a non-cloneable single-use plan, a 30-second per-attempt timeout ceiling, at most four attempts, exact `peer_addr` verification before stream exposure, and no hostname re-resolution or ambient proxy inheritance.
- Authenticated `originweave-tls` service-identity boundary that consumes an existing verified TCP stream, requires exact TLS-origin and transport-origin equality, derives RFC 9525 DNS or literal-IP reference identity only from the canonical HTTPS origin, validates WebPKI with explicit roots and fixed time, permits only TLS 1.2 and TLS 1.3, and never reconnects or resolves.
- Bounded TLS policy for total handshake time, ALPN identifiers, trust-root count and bytes, and server-presented certificate count and bytes, with explicit optional-versus-required ALPN behavior and `NotConfigured` revocation evidence.
- Credential-free TLS evidence containing canonical origin, requested and observed peer, DNS/IP reference identity, TLS version, cipher-suite identifier, selected ALPN or explicit absence, leaf certificate and SPKI hashes, server-presented certificate hashes and bounds, trust-bundle identity and hash, validity interval, fixed verification time, revocation configuration, and measured handshake duration.
- Credential-free connection and redirect evidence containing canonical addresses, destination classes, target digests, hop numbers, and approved-address counts.
- Credential-free verified TCP evidence containing the logical origin, requested socket, observed peer, destination class, successful attempt number, and per-attempt timeout.
- Purpose-bound sensitive-access evidence receipts that retain bounded tenant, actor, task, protected-field identifiers, purpose, canonical destination, classification, outcome, policy version, approval reference, and lifecycle times without retaining protected field values or opaque-handle payloads.
- Standard `Display` and `std::error::Error` contracts for destination, redirect, digest, direct-network, and TLS failures, including preserved destination-policy, rustls, and operating-system sources where applicable.
- Real loopback TCP integration proof plus deterministic timeout, refusal, retry, peer-inspection, peer-mismatch, canonicalization, IPv6 metadata, and single-use replay tests.
- Real loopback rustls integration covering trusted DNS SAN, Common-Name fallback rejection, wrong-name and untrusted-root rejection, fixed-time expiry and not-yet-valid failures, exact IPv4 and IPv6 SANs, TLS 1.2/TLS 1.3, required and optional ALPN, and transport-origin binding.
- Cumulative interactive-first RAM, VRAM, batch, local-model, admission, pause, and compositor-pressure mitigation plans, including active-consumer reduction at exact hard limits.
- Universally value-redacted network evidence with explicit path, metadata, and provenance bounds; ambiguous path rejection; validated source URLs; lowercase SHA-256 identifiers; and verification state.
- Rust 1.97.1 build contract, strict Clippy and rustdoc gates, and exact production function, line, region, and branch coverage enforcement.
- Hourly bounded OpenCode product-development workflow using `NVIDIA_NIM_API_KEY`, an unprivileged disposable workspace, loopback-only model broker, independently verified patches, and publication through a dedicated `OPENCODE_PR_TOKEN` that cannot review or merge.
- Architecture, agent, security, contribution, research, database naming, roadmap, quality-gate, TLS service-identity, session/context node-authority, and hourly agent credential-boundary ADR documentation.

### Changed

- Separated logical origin authority from resolved network destination authority; an origin grant no longer implies permission to connect to every resolver result.
- Separated proxy-server routing identity from web-origin identity: HTTP, HTTPS, SOCKS4, SOCKS5, and QUIC proxy schemes retain their own canonical authority, so an ordinary remote HTTP proxy is representable without weakening the web-origin HTTPS requirement.
- Separated resolved-address authorization from direct transport evidence; an approved IP now becomes a usable stream only after the operating system reports the exact requested IP and port.
- Separated exact TCP peer proof from authenticated TLS service identity; an observed peer becomes an authenticated HTTPS stream only after explicit-root, fixed-time, SAN-bound WebPKI verification over that same stream.
- Restricted direct TCP retries to an explicit transient operating-system error allow-list; deterministic permission, input, and local-address failures now stop after the first attempt while retaining the original error source.
- Replaced single resource-pressure directives with a cumulative mitigation plan so simultaneous RAM, VRAM, frame, model, and admission pressure cannot discard required actions.
- Changed generic network capture from finite deny-lists or safe-name allow-lists to unconditional value redaction. Typed metadata values and bodies now require a separate schema-specific capture contract.
- Updated the first Chromium slice to distinguish implemented origin, destination, direct TCP, and TLS identity kernels from the remaining trusted DNS adapter, proxy/PAC, HTTP budget, MIME, download, and Chromium integration required before safe navigation can be claimed.
- Separated hourly product PR publication authority from the organization review and merge system, added live default-branch and release-blocker rechecks immediately before publication, exhaustively paginate release-blocker results in bounded 100-item API pages before filtering pull-request entries so labeled PRs cannot mask a real blocking issue on any later page, and made a missing dedicated `OPENCODE_PR_TOKEN` fail closed after a verified change instead of producing a green publication no-op.
- Moved autonomous-agent Cargo targets and Python bytecode caches outside the proposed source tree and prefetched locked Cargo dependencies for offline verification.
- Split deterministic open-PR, release-blocker, and dry-run evaluation from the conditional NVIDIA credential step so stopped runs never receive `NVIDIA_NIM_API_KEY`; made a missing `NVIDIA_NIM_API_KEY` fail closed after deterministic governance selects the model-backed path instead of silently skipping all remaining work with a green result; replaced post-model raw-key rematerialization with a runner-only length, SHA-256, and rolling-hash fingerprint used solely for exact leak detection; bounded untrusted `PR_MESSAGE.md` before byte-wise leak scanning; stat-size-check model-controlled workspace files against the one-mebibyte per-file bound before any full byte comparison used to discover changed files; added an evidence-first RCA, feasibility, materially distinct corrective-action, and exact-command revalidation contract; reset every fallback model to the pristine source tree; classified model timeouts, model or tool failures, and credential-broker failures before retry; emitted bounded broker diagnostics when broker failure makes retry infeasible; made final cleanup use the privilege required for the UID-65532-owned model configuration; and expanded the job budget to 180 minutes so all three advertised 35-minute model attempts plus independent verification can actually execute without weakening fail-closed egress.
- Distinguished loopback broker liveness from NVIDIA provider viability: the trusted broker now exposes credential-free request-generation counters for upstream `401`/`403` authentication or authorization rejection and `429` rate limiting, cross-model fallback stops when current-attempt evidence proves either condition, and every pre/post-attempt `/statusz` read is locally schema-validated. Missing or malformed broker telemetry now records `credential_broker_unavailable`, emits bounded diagnostics, and prevents a model from starting instead of escaping the RCA path through shell `errexit`. Generation binding prevents late predecessor responses from poisoning a later pristine attempt, while transient rate limiting is retried only by a later fresh scheduled invocation rather than by consuming another same-provider model slot in the finite run.
- Updated research doctoring to pin Chromium canonicalizer evidence to an immutable revision, add RFC 9293, RFC 5280, RFC 8446, RFC 9525, rustls 0.23.42, Rust `TcpStream`, WebDriver, and WebDriver BiDi evidence, distinguish the April 2026 Fugu beta from the June 2026 release, and treat vendor benchmark claims as first-party evidence rather than independent validation.

### Security

- Raw page content cannot become a trusted instruction.
- Raw secrets are rejected and secret-capable actions require an opaque broker handle.
- Crawler mode is read-only, must pair with the public-crawl purpose, and fails closed without an applicable robots-policy decision.
- State-changing actions are same-origin by default.
- R3 and R4 approvals are bound to the exact action, target origin, and immutable digest of the complete canonical action intent; R5 legal consent is non-delegable.
- Observed node handles cannot be reused across browser sessions or browsing contexts, or after the canonical origin or document epoch changes, preventing stale or colliding adapter-local node identifiers from silently crossing an isolated profile, task, tab, frame, navigation, or document boundary.
- Shortened, integer, hexadecimal, and legacy octal-looking IPv4 host spellings are rejected so the policy origin cannot diverge from Chromium host interpretation.
- IPv4-mapped IPv6 is canonicalized before destination classification and pin comparison so mapped private or loopback addresses cannot bypass IPv4 policy.
- The default destination policy permits only public addresses and denies unspecified, loopback, private, shared, link-local, metadata, documentation, benchmarking, multicast, broadcast, transition, and protocol-reserved destinations.
- Azure platform IP `168.63.129.16` and Amazon EKS Pod Identity endpoints `169.254.170.23` and `fd00:ec2::23` are classified as metadata or platform services before broader address-range rules.
- Resolver answers are rejected when empty or larger than 256 addresses, preventing an unbounded resolver response from entering policy state.
- `localhost` may approve only loopback addresses, while literal IPv4 and IPv6 origins may approve only the exact canonical address encoded in the origin.
- Resolver answers must remain a non-empty subset of the origin-bound approved address set; any newly introduced address fails closed as a possible DNS-rebinding event.
- Every redirect rechecks target-origin authority, target-bound resolution, HTTPS downgrade, complete-target cycle state, and hop capacity before policy state changes.
- Proxy route policy permits only exact canonical server identities and PAC source origins; route approval does not grant destination, TCP peer, proxy authentication, CONNECT, TLS, or final-target authority.
- Direct TCP plans reject port zero, zero or excessive timeouts, excessive attempts, unapproved IPs, non-canonical IPv4-mapped IPv6 sockets, and IPv6 flow or scope metadata not represented in destination authority before connection I/O.
- Direct connection code accepts only an explicit `SocketAddr`, never a hostname, and does not read proxy environment variables.
- Established streams are discarded when peer inspection fails or the observed remote IP or port differs from the approved socket.
- TLS accepts only an already verified direct stream, never a hostname or new socket, and requires the TLS origin to match the transport-authority origin exactly.
- DNS TLS identity requires an applicable subjectAltName and never falls back to Common Name; literal IPv4 and IPv6 origins require exact IP subjectAltName entries.
- TLS uses an explicit immutable trust-root bundle and fixed verification time, and permits only TLS 1.2 and TLS 1.3.
- TLS resumption, 0-RTT, secret extraction, key logging, client certificates, certificate compression, and dangerous custom verifier hooks are disabled in the first slice.
- The operating-system peer is rechecked before, during, and after the deadline-bound TLS handshake.
- ALPN selection is restricted to the caller's bounded allow-list, while absence is either explicitly recorded or rejected by policy.
- Revocation is reported as not configured; the product makes no OCSP or CRL validation claim without supplied revocation evidence.
- Every generic network header and query value is redacted before evidence leaves the trusted boundary, including conventionally benign field names containing attacker-controlled bytes.
- Evidence capture enforces count and byte bounds and rejects credential-bearing source URLs, query strings, fragments, controls, whitespace, malformed percent escapes, encoded separators, dot segments, and backslash paths.
- Hard RAM and VRAM pressure pauses the active agent and rejects new admission; hard VRAM pressure also offloads a resident local model.
- The hourly product agent has no Git metadata or repository authority. A separate post-verification publisher opens one PR and cannot approve or merge it.
- The unprivileged OpenCode user is restricted to loopback egress during model execution, preventing runner-wide allow-listed endpoints from becoming direct source-exfiltration channels.

[Unreleased]: https://github.com/ContextualWisdomLab/OriginWeave/compare/main...HEAD
