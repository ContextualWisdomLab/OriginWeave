# Changelog

All notable changes to OriginWeave are documented in this file. The format follows Keep a Changelog, and releases use Semantic Versioning.

## [Unreleased]

- Refreshed the 2026-08-27 delivery snapshot against protected `main` `542ca1e9…`: 109 open pull requests (36 non-draft, 73 draft), 9 open issues, zero releases, and zero tags; older exact-head tables remain explicitly dated evidence.
- Refreshed the product-gap queue to 126 open pull requests (54 ready, 72 draft) after #190, #188, #185, #192, #182, #184, #115, #181, #116, #117, #118, #183, #114, #127, #112, #109, #186, #110, #108, #111, #174, and #113 were merged into their immediate stacked prerequisites. PRs #147, #146, #145, #144, #143, #142, #141, #139, #136, #132, #129, and #128 moved to ready after exact-head checks and thread review; these are queue-consolidation results, not protected-main shipment.

### Added
- Added cross-surface platform coherence to the fingerprint kernel: `PresentationPlatform::hints_platform` is the single source of truth mapping each presentation platform to its canonical UA Client Hints platform, and `require_hints_coherence` fails closed on any contradiction so the presentation-platform, UA-token, and UA-CH-platform triad cannot leak a mismatched identity (see ADR 0113).
- Added bounded User-Agent Client Hints surfaces to the fingerprint kernel: ASCII brand/version validation with a 32-character name bound, enumerated architecture/bitness/platform tokens, a non-empty brand-list requirement, and the spec rule that a non-mobile user agent reports an empty model. Control-plane contract only, grounded in the User-Agent Client Hints draft (WICG, 2026); see ADR 0112.
- Added bounded stealth-normalization surfaces to the fingerprint kernel: enumerated canvas-noise classes, canonicalized WebGL renderer tokens, standard-rate Web Audio normalization, bounded WebRTC interface policy, and a fail-closed Canvas/WebGL/WebAudio/WebRtc surface-admission contract. This is a privacy-preserving control-plane contract with no real-browser or anti-evasion claim (see ADR 0111).
- Corrected the 2026-08-26 product-gap snapshot with current #229 presentation-identity evidence, stacked-only #205 integration evidence, current base/head pairs, the 126-PR queue count, explicit root-versus-child merge ordering, and the active GitHub counted-approval gate.
- Refreshed the product and technical gap baseline onto the 2026-08-26 live inventory: 126 open pull requests (54 ready, 72 draft), protected-main promotion of #168/#194/#196/#216/#151, a verified maintenance-loop record (supersession closure of #153, conflict reconciliations on #37/#149/#152/#173/#175, issue #212 option-(b) authorization on #43, Strix vuln-0001 homoglyph remediation on #124), provider-rerun outcome evidence, an organization review-pipeline congestion record, and refreshed merge-order queue guidance. Documentation evidence contracts were aligned to the same snapshot so the baseline, its dated markers, and the pinned exact-head rows cannot silently diverge.

- Added `originweave_core::release_acceptance`, a deterministic fail-closed benchmark release-decision contract that requires one authoritative result for every mandatory suite, bounds explicit buyer-visible limitations, rejects duplicate limitation claim identities, and rejects non-canonical surrounding whitespace rather than normalizing it into an alternate claim spelling.
- Added fail-closed presentation-surface admission so an adapter cannot claim a
  privacy profile while any required page-observable field remains ambient.
- Added a proposed privacy-preserving presentation-identity kernel with bounded screen, viewport, pixel ratio, processor, platform, language, reduced-motion, standardized named-UTC time-zone, and credential-free digest contracts; real Chromium application and anti-evasion claims remain explicitly unshipped.
- Refreshed the product and technical gap baseline with the 2026-08-24 live inventory: 158 open pull requests (44 ready, 114 draft), refreshed exact base/head evidence for the #208–#222 release, enterprise-approval, BAP, and WARC/PROV chains, the governance issue additions #212 and #215, and a required-check provider-failure record for the fail-closed Strix re-dispatches on #208/#218/#220.
- Added a dated product and technical gap baseline that separates protected-main implementation truth, active pull-request evidence, live review/check blockers, and the next buyer-visible Phase 1 acceptance work.
- Refreshed the product and technical gap baseline with the current open-PR inventory and exact base/head evidence for the newest Chromium, BAP, extraction, WARC, and idempotency slices.
- Bound explicit extension-to-Agent grants to exclusive trusted-time expiry in addition to extension identity, session, browsing context, and canonical origin, so a same-origin grant cannot be reused at or after the deadline.
- Bound explicit extension-to-Agent grants to the exact canonical origin in addition to extension identity, session, and browsing context, so a same-session navigation or port change cannot reuse the grant.
- Rust workspace for independently reusable core, policy, destination, network, TLS, resource, and evidence modules.
- Canonical HTTPS and loopback-origin boundary with case-normalized schemes and hosts, default-port normalization, IPv4/IPv6 handling, browser-special numeric-host rejection, and explicit malformed-input errors.
- Typed browser actions, capabilities, risk classes, execution modes, robots decisions, secret-delivery contracts, immutable canonical action-intent digests, and intent-bound approval scopes.
- Protected main now contains deterministic MCP `2026-07-28` stateless `tools/call` routing with bounded method/tool names, a single reviewed tool-to-action registry shared by routing and discovery metadata, and fail-closed policy binding that grants no ambient authority. The complete MCP adapter, transport serialization, discovery response handling, OAuth, browser I/O, and persistence remain planned.
- Active PR #170 adds conservative MCP `2026-07-28` `tools/list` discovery metadata derived from that protected-main catalog, with `resultType = complete`, zero freshness, private cache scope, no continuation cursor, per-request protocol/client-capability admission, and bounded protocol-version and method metadata validated before cross-field comparison. This remains active-PR evidence only and grants no browser, network, secret, approval, or Agent authority.
- Deterministic fail-closed policy evaluation for untrusted instructions, origin grants, crawler restrictions, execution-mode and purpose consistency, approvals, and brokered secrets.
- Fail-closed resolved-destination policy with IPv4/IPv6 special-purpose and reviewed cloud-platform endpoint classification, IPv4-mapped canonicalization, explicit class grants, non-empty origin-bound DNS snapshots capped at 256 resolver addresses, concrete connection pinning, DNS-set expansion detection, and per-hop redirect reauthorization.
- Bounded resolution-freshness authority with trusted monotonic approval time, capped non-zero validity, half-open use windows, non-expanding revalidation, and credential-free authorization timestamps.
- Direct-only `originweave-network` TCP boundary with explicit canonical `SocketAddr` authority, zero IPv6 flow and scope metadata unless separately modeled, a non-cloneable single-use plan, a 30-second per-attempt timeout ceiling, at most four attempts, exact `peer_addr` verification before stream exposure, and no hostname re-resolution or ambient proxy inheritance.
- Authenticated `originweave-tls` service-identity boundary that consumes an existing verified TCP stream, requires exact TLS-origin and transport-origin equality, derives RFC 9525 DNS or literal-IP reference identity only from the canonical HTTPS origin, validates WebPKI with explicit roots and fixed time, permits only TLS 1.2 and TLS 1.3, and never reconnects or resolves.
- Bounded TLS policy for total handshake time, ALPN identifiers, trust-root count and bytes, and server-presented certificate count and bytes, with explicit optional-versus-required ALPN behavior and `NotConfigured` revocation evidence.
- Deterministic TLS revocation-material freshness authority with a strict signed `thisUpdate`→`nextUpdate` half-open window and typed invalid-window, not-yet-valid, and stale failures, without claiming OCSP/CRL acquisition, cryptographic validation, or certificate revocation status.
- Credential-free TLS evidence containing canonical origin, TCP peers, reference identity, TLS version, cipher-suite identifier, selected ALPN or explicit absence, leaf certificate and SPKI hashes, server-presented certificate hashes and bounds, trust-bundle identity and hash, validity interval, fixed verification time, revocation configuration, and measured handshake duration.
- Credential-free sensitive-handle lifecycle evidence binds issuance, exclusive expiry, bounded uses, observed resolution count, and revocation to the exact credential-free `OpaqueHandleOnly` sensitive-access receipt, preserving tenant, actor, task, field set, purpose, destination, classification, policy version, and decision time without storing opaque handle tokens or protected values.
- Credential-free connection and redirect evidence containing canonical addresses, destination classes, target digests, hop numbers, and approved-address counts.
- Credential-free verified TCP evidence containing the logical origin, requested socket, observed peer, destination class, successful attempt number, and per-attempt timeout.
- Standard `Display` and `std::error::Error` contracts for destination, redirect, digest, direct-network, TLS, and resource-budget failures, including preserved destination-policy, rustls, and operating-system sources where applicable.
- Real loopback TCP integration proof plus deterministic timeout, refusal, retry, peer-inspection, peer-mismatch, canonicalization, IPv6 metadata, and single-use replay tests.
- Real loopback rustls integration covering trusted DNS SAN, Common-Name fallback rejection, wrong-name and untrusted-root rejection, fixed-time expiry and not-yet-valid failures, exact IPv4 and IPv6 SANs, TLS 1.2/TLS 1.3, required and optional ALPN, and transport-origin binding.
- Cumulative interactive-first RAM, VRAM, batch, local-model, admission, pause, and compositor-pressure mitigation plans, including active-consumer reduction at exact hard limits.
- Universally value-redacted network evidence with explicit path, metadata, and provenance bounds; ambiguous path rejection; validated source URLs; lowercase SHA-256 identifiers; and verification state.
- Versioned schema-bound extraction contracts with bounded identifiers and field counts, typed value/cardinality metadata, explicit duplicate-free reviewed source channels, fail-closed schema validation, and deterministic `Display`/`std::error::Error` contracts for public schema failures.
- Rust 1.97.1 build contract, strict Clippy and rustdoc gates, and exact production function, line, region, and branch coverage enforcement.
- Hourly bounded OpenCode product-development workflow using `NVIDIA_NIM_API_KEY`, an unprivileged disposable workspace, loopback-only model broker, independently verified patches, and publication through a dedicated `OPENCODE_PR_TOKEN` that cannot review or merge.
- Architecture, agent, security, contribution, research, database naming, roadmap, quality-gate, and TLS service-identity ADR documentation.
- Resumable BAP lifecycle restoration with monotonic sequence recovery and fail-closed sequence exhaustion.
- Authoritative product documentation graph spanning PRD, TRD, ADR lifecycle/index, product-wide UML, conceptual ERD, requirement/decision traceability, threat modeling, product-wide test strategy, operability, API/protocol, release/rollback, and current primary-source standards doctoring, with machine-checkable repository contracts that keep conversation-derived future work distinct from protected-main implementation claims.
- Purpose-bound data-governance and privacy baseline that rejects both blanket masking and ambient raw-value propagation, defines field-scoped just-in-time disclosure, opaque-handle/trusted-broker boundaries, model/provider/region policy, retention/deletion/residency/break-glass controls, truthful CSAP/SOC 2 readiness language, and machine-checkable documentation contracts without inventing an OriginWeave-owned production database.
- Proposed product-wide target-architecture ADRs for the Rust control plane, isolated execution modes, typed actions, semantic observation/stale-node authority, prompt-injection and secret separation, resource-governor priority, provenance evidence, browser/protocol adapters, crawler policy, and hourly automation operational closure; these remain Proposed rather than shipped claims until protected review and merge.

### Changed

- Removed unsupported uniform seed-based presentation selection; the privacy
  kernel now validates explicit coherent profiles and leaves default selection
  unavailable until cited cohort evidence defines a defensible anonymity set.

- Recorded the merged central Strix adapter repair while retaining exact-head
  acceptance reruns as required evidence before closing the provider blocker.

- Refreshed the product-gap baseline with exact current presentation and
  WebDriver BiDi heads, non-draft stack state, and the zero-release/tag truth.

- Classified proposed ADR 0110 consistently as branch-only documentation
  evidence until the presentation-identity line integrates into protected main.

- Coupled macOS presentation derivation and manual validation to integer device
  scale classes so the privacy kernel cannot emit that contradictory identity.

- Labeled the dated product-gap observation explicitly as KST so UTC-hosted
  review does not misread a same-instant snapshot as future evidence.

- Replaced an invalid uppercase-digest test fixture that resembled a Telegram
  credential while preserving the lowercase SHA-256 rejection contract.

- Clarified that presentation identity has active-PR kernel evidence while its
  Chromium adapter remains planned, without mixing proposal and implementation
  labels in the same technical-design section.
- Aligned the hourly product-development branch-coverage toolchain and its one-shot materializer with the reviewed `nightly-2026-08-18` pin, and corrected the official Dependabot Rust-toolchain reference.
- Refreshed the product gap baseline to the 2026-08-27 protected-main and complete open-PR inventory, recorded the shared Strix provider incompatibility, and added the presentation-identity integration gap without promoting local or active-PR evidence to shipped behavior.
- Separated logical origin authority from resolved network destination authority; an origin grant no longer implies permission to connect to every resolver result.
- Separated resolved-address authorization from direct transport evidence; an approved IP now becomes a usable stream only after the operating system reports the exact requested IP and port.
- Separated exact TCP peer proof from authenticated TLS service identity; an observed peer becomes an authenticated HTTPS stream only after explicit-root, fixed-time, SAN-bound WebPKI verification over that same stream.
- Replaced single resource-pressure directives with a cumulative mitigation plan so simultaneous RAM, VRAM, frame, model, and admission pressure cannot discard required actions.
- Changed generic network capture from finite deny-lists or safe-name allow-lists to unconditional value redaction. Typed metadata values and bodies now require a separate schema-specific capture contract.
- Updated the first Chromium slice to distinguish implemented origin, destination, direct TCP, and TLS identity kernels from the remaining trusted DNS adapter, proxy/PAC, HTTP budget, MIME, download, and Chromium integration required before safe navigation can be claimed.
- Separated hourly product PR publication authority from the organization review and merge system, and added live default-branch and release-blocker rechecks immediately before publication.
- Made the agent-development contract work-conserving: completing one bounded slice, RCA, review request, check, merge, or documentation change is an intermediate state; maintenance must return to the live queue, treat waits as item-local, and perform a mandatory exit sweep before terminating while executable OriginWeave work remains.
- Hardened the dated baseline evidence collector with fail-fast isolated artifacts, paginated branch and collaborator rules, and post-collection exact-head revalidation.
- Flattened every paginated workflow-run page in the baseline merge verdict so exact-head evidence cannot silently discard later runs.
- Hardened the baseline evidence procedure with exact-head legacy status and workflow-run capture, counted approval binding, required-workflow recording, merge verdict artifacts, and bounded moving-head retries.
- Moved autonomous-agent Cargo targets and Python bytecode caches outside the proposed source tree and prefetched locked Cargo dependencies for offline verification.
- Updated research doctoring to pin Chromium canonicalizer evidence to an immutable revision, add RFC 9293, RFC 5280, RFC 8446, RFC 9525, rustls 0.23.42, and Rust `TcpStream` evidence, distinguish the April 2026 Fugu beta from the June 2026 release, and treat vendor benchmark claims as first-party evidence rather than independent validation.
- Tightened the product-baseline contract so the BiDi opening path and VPN/profile evidence retain their explicit not-shipped status within their own documentation sections.
- Refreshed the product and technical gap baseline against the 2026-08-21 live inventory: 150 open pull requests, 110 drafts, and the new hardened-runner/MV3 evidence gap issue #206.
- Tightened the baseline completion-gap contract so superseded inventory counts (including the 2026-08-21 150/40/110 snapshot) can no longer pass as current evidence.
- Refreshed the baseline's merge-authority statement to the live ruleset: two approving reviews are required, while the collaborator inventory still contains only the solo maintainer.
- Corrected the baseline evidence collector to flatten every paginated input, apply current reviewer and last-push approval semantics, and discard verdicts when either the PR head or base moves.

### Security

- Explicit proxy server identifiers require ASCII decimal port tokens before numeric range parsing, preventing Rust-specific leading-plus spellings from widening proxy authority.
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
- TLS trust-bundle policy identifiers must contain at least one ASCII alphanumeric character; punctuation-only labels are rejected while `.`, `_`, `:`, and `-` remain permitted.
- TLS resumption, 0-RTT, secret extraction, key logging, client certificates, certificate compression, and dangerous custom verifier hooks are disabled in the first slice.
- The operating-system peer is rechecked before, during, and after the deadline-bound TLS handshake.
- ALPN selection is restricted to the caller's bounded allow-list, while absence is either explicitly recorded or rejected by policy.
- Revocation is reported as not configured; the product makes no OCSP or CRL validation claim without supplied revocation evidence.
- Every generic network header and query value is redacted before evidence leaves the trusted boundary, including conventionally benign field names containing attacker-controlled bytes.
- Evidence capture enforces count and byte bounds and rejects credential-bearing source URLs, query strings, fragments, controls, whitespace, malformed percent escapes, encoded separators, dot segments, and backslash paths.
- Network-evidence paths and provenance source URL paths accept only RFC 3986 literal `pchar` syntax plus validated percent-encoded octets and slash separators, preventing raw general delimiters such as `[` and `]` or other invalid URI-presentation bytes from entering either evidence surface.
- Hard RAM and VRAM pressure pauses the active agent and rejects new admission; hard VRAM pressure also offloads a resident local model.
- The hourly product agent has no Git metadata or repository authority. A separate post-verification publisher opens one PR and cannot approve or merge it.
- The unprivileged OpenCode user is restricted to loopback egress during model execution, preventing runner-wide allow-listed endpoints from becoming direct source-exfiltration channels.

[Unreleased]: https://github.com/ContextualWisdomLab/OriginWeave/compare/main...HEAD

<!-- dispatch-refresh 2026-08-27T03:05Z -->
