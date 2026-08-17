# Changelog

All notable changes to OriginWeave are documented in this file. The format follows Keep a Changelog, and releases use Semantic Versioning.

## [Unreleased]

### Added

- Bound explicit extension-to-Agent grants to exclusive trusted-time expiry in addition to extension identity, session, browsing context, and canonical origin, so a same-origin grant cannot be reused at or after the deadline.
- Bound explicit extension-to-Agent grants to the exact canonical origin in addition to extension identity, session, and browsing context, so a same-session navigation or port change cannot reuse the grant.
- Rust workspace for independently reusable core, policy, destination, network, TLS, resource, and evidence modules.
- Canonical HTTPS and loopback-origin boundary with case-normalized schemes and hosts, default-port normalization, IPv4/IPv6 handling, browser-special numeric-host rejection, and explicit malformed-input errors.
- Typed browser actions, capabilities, risk classes, execution modes, robots decisions, secret-delivery contracts, immutable canonical action-intent digests, and intent-bound approval scopes.
- Deterministic fail-closed policy evaluation for untrusted instructions, origin grants, crawler restrictions, execution-mode and purpose consistency, approvals, and brokered secrets.
- Fail-closed resolved-destination policy with IPv4/IPv6 special-purpose and reviewed cloud-platform endpoint classification, IPv4-mapped canonicalization, explicit class grants, non-empty origin-bound DNS snapshots capped at 256 resolver addresses, concrete connection pinning, DNS-set expansion detection, and per-hop redirect reauthorization.
- Direct-only `originweave-network` TCP boundary with explicit canonical `SocketAddr` authority, zero IPv6 flow and scope metadata unless separately modeled, a non-cloneable single-use plan, a 30-second per-attempt timeout ceiling, at most four attempts, exact `peer_addr` verification before stream exposure, and no hostname re-resolution or ambient proxy inheritance.
- Authenticated `originweave-tls` service-identity boundary that consumes an existing verified TCP stream, requires exact TLS-origin and transport-origin equality, derives RFC 9525 DNS or literal-IP reference identity only from the canonical HTTPS origin, validates WebPKI with explicit roots and fixed time, permits only TLS 1.2 and TLS 1.3, and never reconnects or resolves.
- Bounded TLS policy for total handshake time, ALPN identifiers, trust-root count and bytes, and server-presented certificate count and bytes, with explicit optional-versus-required ALPN behavior and `NotConfigured` revocation evidence.
- Credential-free TLS evidence containing canonical origin, requested and observed peer, DNS/IP reference identity, TLS version, cipher-suite identifier, selected ALPN or explicit absence, leaf certificate and SPKI hashes, server-presented certificate hashes and bounds, trust-bundle identity and hash, validity interval, fixed verification time, revocation configuration, and measured handshake duration.
- Bounded single-use `originweave-http` HTTP/1.1 `GET`/`HEAD` authority over the already authenticated TLS stream, with strict RFC 9110/9112 framing, one monotonic exchange deadline, encoded/decoded/chunk/trailer budgets, RFC 9530 digest validation, conservative MIME/disposition handling, redirect metadata without automatic follow, and immutable credential-free evidence.
- Exact HTTP/1.1 reason-phrase octets retained separately on `AuthenticatedHttpResponse` for bounded diagnostics, including obs-text, while remaining excluded from credential-free evidence and all semantic decisions.
- Credential-free connection and redirect evidence containing canonical addresses, destination classes, target digests, hop numbers, and approved-address counts.
- Credential-free verified TCP evidence containing the logical origin, requested socket, observed peer, destination class, successful attempt number, and per-attempt timeout.
- Standard `Display` and `std::error::Error` contracts for destination, redirect, digest, direct-network, and TLS failures, including preserved destination-policy, rustls, and operating-system sources where applicable.
- Real loopback TCP integration proof plus deterministic timeout, refusal, retry, peer-inspection, peer-mismatch, canonicalization, IPv6 metadata, and single-use replay tests.
- Real loopback rustls integration covering trusted DNS SAN, Common-Name fallback rejection, wrong-name and untrusted-root rejection, fixed-time expiry and not-yet-valid failures, exact IPv4 and IPv6 SANs, TLS 1.2/TLS 1.3, required and optional ALPN, and transport-origin binding.
- Cumulative interactive-first RAM, VRAM, batch, local-model, admission, pause, and compositor-pressure mitigation plans, including active-consumer reduction at exact hard limits.
- Universally value-redacted network evidence with explicit path, metadata, and provenance bounds; ambiguous path rejection; validated source URLs; lowercase SHA-256 identifiers; and verification state.
- Rust 1.97.1 build contract, strict Clippy and rustdoc gates, and exact production function, line, region, and branch coverage enforcement.
- Hourly bounded OpenCode product-development workflow using `NVIDIA_NIM_API_KEY`, an unprivileged disposable workspace, loopback-only model broker, independently verified patches, and publication through a dedicated `OPENCODE_PR_TOKEN` that cannot review or merge.
- Architecture, agent, security, contribution, research, database naming, roadmap, quality-gate, and TLS service-identity ADR documentation.
- Authoritative product documentation graph spanning PRD, TRD, ADR lifecycle/index, product-wide UML, conceptual ERD, requirement/decision traceability, threat modeling, product-wide test strategy, operability, API/protocol, release/rollback, and current primary-source standards doctoring, with machine-checkable repository contracts that keep conversation-derived future work distinct from protected-main implementation claims.
- Purpose-bound data-governance and privacy baseline that rejects both blanket masking and ambient raw-value propagation, defines field-scoped just-in-time disclosure, opaque-handle/trusted-broker boundaries, model/provider/region policy, retention/deletion/residency/break-glass controls, truthful CSAP/SOC 2 readiness language, and machine-checkable documentation contracts without inventing an OriginWeave-owned production database.
- Proposed product-wide target-architecture ADRs for the Rust control plane, isolated execution modes, typed actions, semantic observation/stale-node authority, prompt-injection and secret separation, resource-governor priority, provenance evidence, browser/protocol adapters, crawler policy, and hourly automation operational closure; these remain Proposed rather than shipped claims until protected review and merge.

### Changed

- Separated logical origin authority from resolved network destination authority; an origin grant no longer implies permission to connect to every resolver result.
- Separated resolved-address authorization from direct transport evidence; an approved IP now becomes a usable stream only after the operating system reports the exact requested IP and port.
- Separated exact TCP peer proof from authenticated TLS service identity; an observed peer becomes an authenticated HTTPS stream only after explicit-root, fixed-time, SAN-bound WebPKI verification over that same stream.
- Reserved HTTP response-coding negotiation to the serializer that owns the gzip and zlib-deflate decoder set; caller `Accept-Encoding` fields now fail closed instead of combining with the fixed negotiation and advertising unsupported codings.
- Tightened HTTP/1.1 duplicate `Content-Length` handling so comma-separated or repeated decimal members must be byte-identical after optional whitespace is trimmed; numerically equal but differently spelled members such as `042` and `42` now fail closed as conflicting framing evidence.
- Restricted raw ASCII in HTTP origin-form request targets to RFC 3986 path/query characters; disallowed delimiters and unsafe punctuation now fail closed unless supplied through a valid percent escape, while non-ASCII UTF-8 remains percent-encoded.
- Chunked HTTP/1.1 responses now complete at the terminal zero chunk plus bounded trailer terminator instead of waiting for peer TLS closure; close-delimited responses still require authenticated TLS EOF, and the stream remains single-use.
- Overlong chunked trailer lines now fail as `TrailerSectionTooLarge` against the trailer-section budget, while malformed trailer line endings retain their chunked-syntax error classification.
- MIME essence construction now enforces its parameter-free contract: reviewed type/subtype inputs that introduce MIME parameters fail closed instead of manufacturing a parameterized `MimeType`.
- Replaced single resource-pressure directives with a cumulative mitigation plan so simultaneous RAM, VRAM, frame, model, and admission pressure cannot discard required actions.
- Changed generic network capture from finite deny-lists or safe-name allow-lists to unconditional value redaction. Typed metadata values and bodies now require a separate schema-specific capture contract.
- Updated the first Chromium slice to distinguish implemented origin, destination, direct TCP, TLS identity, and bounded HTTP/MIME kernels from the remaining trusted DNS adapter, proxy/PAC execution, browser download handoff, and Chromium integration required before safe navigation can be claimed.
- Separated hourly product PR publication authority from the organization review and merge system, and added live default-branch and release-blocker rechecks immediately before publication.
- Made the agent-development contract work-conserving: completing one bounded slice, RCA, review request, check, merge, or documentation change is an intermediate state; maintenance must return to the live queue, treat waits as item-local, and perform a mandatory exit sweep before terminating while executable OriginWeave work remains.
- Moved autonomous-agent Cargo targets and Python bytecode caches outside the proposed source tree and prefetched locked Cargo dependencies for offline verification.
- Updated research doctoring to pin Chromium canonicalizer evidence to an immutable revision, add RFC 9293, RFC 5280, RFC 8446, RFC 9525, rustls 0.23.42, and Rust `TcpStream` evidence, distinguish the April 2026 Fugu beta from the June 2026 release, and treat vendor benchmark claims as first-party evidence rather than independent validation.

### Security

- Raw page content cannot become a trusted instruction.
- Raw secrets are rejected and secret-capable actions require an opaque broker handle.
- Crawler mode is read-only, must pair with the public-crawl purpose, and fails closed without an applicable robots-policy decision.
- State-changing actions are same-origin by default.
- R3 and R4 approvals are bound to the exact action, target origin, and immutable digest of the complete canonical action intent; R5 legal consent is non-delegable.
- Shortened, integer, hexadecimal, and legacy octal-looking IPv4 host spellings are rejected so the policy origin cannot diverge from Chromium host interpretation.
- IPv4-mapped IPv6 is canonicalized before destination classification and pin comparison so mapped private or loopback addresses cannot bypass IPv4 policy.
- The default destination policy permits only public addresses and denies unspecified, loopback, private, shared, link-local, metadata, documentation, benchmarking, multicast, broadcast, transition, and protocol-reserved destinations.
- Azure platform IP `168.63.129.16` and Amazon EKS Pod Identity endpoints `169.254.170.23` and `fd00:ec2::23` are classified as metadata or platform services before broader public, link-local, or unique-local rules.
- Resolver answers are rejected when empty or larger than 256 addresses, preventing an unbounded resolver response from entering policy state.
- `localhost` may approve only loopback addresses, while literal IPv4 and IPv6 origins may approve only the exact canonical address encoded in the origin.
- Resolver answers must remain a non-empty subset of the origin-bound approved address set; any newly introduced address fails closed as a possible DNS-rebinding event.
- Every redirect rechecks target-origin authority, target-bound resolution, HTTPS downgrade, complete-target cycle state, and hop capacity before policy state changes.
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
- HTTP/1.1 status, framing, content coding, integrity, MIME, disposition, and redirect decisions never depend on the untrusted reason phrase; exact reason bytes are available only as bounded diagnostics and are not copied into credential-free evidence.
- Safe `Content-Disposition` filename metadata rejects Windows reserved device names using both ASCII digits and the Windows-recognized ISO/IEC 8859-1 superscript digits in `COM`/`LPT` names before any later download handoff.
- Every generic network header and query value is redacted before evidence leaves the trusted boundary, including conventionally benign field names containing attacker-controlled bytes.
- Evidence capture enforces count and byte bounds and rejects credential-bearing source URLs, query strings, fragments, controls, whitespace, malformed percent escapes, encoded separators, dot segments, and backslash paths.
- Hard RAM and VRAM pressure pauses the active agent and rejects new admission; hard VRAM pressure also offloads a resident local model.
- The hourly product agent has no Git metadata or repository authority. A separate post-verification publisher opens one PR and cannot approve or merge it.
- The unprivileged OpenCode user is restricted to loopback egress during model execution, preventing runner-wide allow-listed endpoints from becoming direct source-exfiltration channels.

[Unreleased]: https://github.com/ContextualWisdomLab/OriginWeave/compare/main...HEAD
