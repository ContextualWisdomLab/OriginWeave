# ADR 0011: Bind bounded HTTP/1.1 semantics to the authenticated TLS stream

- Status: Proposed
- Date: 2026-08-09
- Supersedes: historical unmerged `docs/adr/0007-bounded-http11-semantics.md` from PR #11

## Context

OriginWeave separates canonical origin, approved resolved destination, exact operating-system TCP peer, and TLS service identity. Those authorities do not make an HTTP exchange safe. An authenticated peer can still send ambiguous framing, conflicting lengths, malformed chunks, oversized fields, incomplete messages, decompression bombs, unsafe redirect metadata, misleading MIME metadata, hostile filenames, or integrity metadata that is malformed or does not match the content.

A general-purpose client would also introduce connector, DNS, proxy, redirect, pooling, cookie, credential, retry, and runtime behavior that is outside the authority already proven by the destination, network, and TLS crates. The HTTP boundary therefore must consume one existing `AuthenticatedTlsConnection` and must not create a second path to the network.

PR #11 established the original HTTP design and TDD evidence, but it diverged from protected-main work. PR #37 reconstructs that unique behavior on current OriginWeave lineage without transferring predecessor checks or approvals. This ADR records the decision under an unused current-main number rather than colliding with the accepted sensitive-data ADR 0007.

## Decision drivers

- Preserve the exact origin -> destination -> TCP peer -> TLS identity authority chain.
- Reject request smuggling and ambiguous HTTP/1.1 framing.
- Bound all attacker-controlled retained bytes and decode expansion.
- Keep redirects as metadata requiring a fresh authority chain.
- Keep download persistence, rendering, cookies, authentication, proxy/PAC, DNS, Chromium, and model execution outside this crate.
- Emit useful credential-free evidence without retaining arbitrary field values or response bodies in audit metadata.
- Make one authenticated exchange deterministic and independently reusable by OriginWeave and other CWL hosts.

## Assumptions and authority boundaries

The caller supplies an already authenticated TLS connection whose origin and peer evidence have passed the earlier OriginWeave boundaries. The caller also supplies an origin-form target, a supported method, bounded non-authoritative request fields, and an `HttpClientPolicy` whose values can only reduce reviewed product maxima.

The HTTP crate does not resolve, connect, reconnect, proxy, follow redirects, persist files, execute content, invoke a browser, or call a model. A successful HTTP response proves only that one bounded message was exchanged over the already authenticated stream; it does not prove business authorization, content safety, legal permission, or rendering safety.

## Options considered

### General-purpose HTTP client

Rejected for the authority kernel. Hidden connector, proxy, redirect, pool, cookie, and retry behavior would enlarge the trusted computing base and make exact-stream evidence harder to prove.

### Treat TLS close as the universal response delimiter

Rejected. HTTP/1.1 chunked transfer coding is self-delimiting; a valid persistent peer may keep the connection open after the terminal zero chunk. Close-delimited responses still require authenticated TLS EOF.

### Buffer then validate

Rejected. Declared lengths, chunk metadata, trailers, and compressed content are peer-controlled. The implementation must enforce limits before growth where possible and before returning content in all cases.

### Follow redirects internally

Rejected. A redirected target may alter origin, DNS results, route, TCP peer, TLS identity, capability, sensitive-data scope, and policy. It requires a new complete authority evaluation.

## Decision

Create the independently reusable `originweave-http` Rust crate. It performs exactly one HTTP/1.1 `GET` or `HEAD` exchange over one existing `AuthenticatedTlsConnection`.

The serializer owns `Host`, `Connection: close`, and `Accept-Encoding: gzip, deflate`. Caller fields cannot override authority, credentials, framing, proxy, connection, upgrade, cookie, trailer, or response-coding negotiation fields. Origin-form targets reject fragments, absolute/authority form, invalid percent escapes, controls, whitespace, backslashes, and oversized encoded targets.

The parser accepts strict HTTP/1.1 CRLF syntax, a bounded number of informational responses, and one final response. It rejects obsolete field folding, invalid field syntax, `Transfer-Encoding` with `Content-Length`, unsupported transfer coding, conflicting lengths, protocol upgrades, malformed chunks, forbidden trailers, and ambiguous or surplus bytes.

Body framing follows RFC 9110 and RFC 9112 with these product constraints:

1. `HEAD`, informational responses, 204, and 304 expose no content.
2. The only supported transfer-coding list is exactly `chunked`.
3. Canonical repeated/comma-separated content lengths must all agree.
4. Chunked completion occurs at the terminal zero chunk plus bounded trailers and final empty line; peer close is not required.
5. Close-delimited content is complete only after authenticated clean TLS EOF.
6. This first slice is single-use and retains no parser state for connection reuse.

The strict policy bounds request bytes, target bytes, status line, field counts/names/values/section, interim responses, chunk count/line size, trailer count/section, encoded and decoded content, decoded-to-encoded expansion, and one monotonic total exchange timeout. The unfinished chunked wire prefix has an independent derived memory bound checked before retained-buffer growth.

Supported content coding is identity, one gzip layer, or one zlib-wrapped deflate layer. Unsupported or stacked coding fails closed. RFC 9530 `Content-Digest` and `Repr-Digest` support SHA-256 and SHA-512 using the RFC 8941 Structured Fields grammar to which RFC 9530 was originally bound. RFC 9651 now obsoletes RFC 8941; this first slice intentionally remains on the RFC 8941 baseline and rejects RFC 9651-only Date and Display String bare-item syntax until a separate reviewed compatibility change updates the parser and its evidence contract. Integrity remains corruption evidence, not authentication.

MIME handling records supplied type separately from a conservative versioned observed classification. `Content-Disposition` may yield only bounded portable metadata; it does not create a file. Redirect handling returns bounded/hash-oriented metadata and never follows the redirect. Network-path redirect references that could carry an authority are never collapsed into same-origin path metadata.

`HttpExchangeEvidence` records the inherited origin/peer/TLS summary plus method, bounded/hash-oriented target information, status, response field names and byte counts, framing, content coding, byte budgets, chunk/trailer decisions, integrity status, MIME/disposition/redirect classifications, completeness, elapsed time, and configured resource limits. It does not retain credentials, cookies, arbitrary request/response field values, response content, query values, unsafe filenames, certificates, or raw redirect locations.

## Consequences

### Positive

- One authenticated stream yields one deterministic bounded HTTP result.
- HTTP framing cannot silently create a second network authority path.
- Persistent peers are interoperable for completed chunked messages without weakening close-delimited completeness.
- Resource limits and evidence are explicit and testable.
- Redirect and download metadata return to later policy/persistence authorities rather than bypassing them.

### Negative

- HTTP/1.1 only and `Connection: close` reduce performance and compatibility.
- Strict parsing rejects some legacy-but-tolerated messages.
- The first slice materializes a bounded decoded body in memory.
- Authentication, cookies, proxying, caching, HTTP/2/3, streaming downloads, and browser integration remain separate future work.

## Failure and degraded behavior

Any malformed syntax, framing conflict, limit breach, incomplete response, timeout, unclean close where EOF is semantic, decoder failure, integrity mismatch, unsafe metadata, or timeout-restoration failure returns a typed error and withholds a successful response. The single-use authenticated stream is consumed on success or failure and is never reused after ambiguous state.

## Security / privacy / governance impact

The decision reduces request-smuggling, decompression, SSRF-authority-confusion, redirect, unsafe-download-name, and accidental evidence-disclosure risk. It does not replace upstream destination/TLS authority or downstream content/rendering/business authorization. No model credential or protected value belongs in this layer.

## Tests and acceptance evidence

Acceptance requires deterministic parser/boundary tests, real loopback TLS integration, persistent-peer chunked completion, truncation and unclean-close failures, total-deadline tests, exact retained-wire budget tests, MIME/disposition/redirect hostile cases, known-answer digest vectors, current Rust formatting/Clippy/rustdoc, exact owned production function/line/region/branch coverage, Security Scan, SAST, current review cleanup, and current branch policy on one unchanged head.

## Migration and rollback

The crate is additive. No caller should replace a previously trusted HTTP path until its adapter explicitly consumes this authority. Rollback removes the new crate/adapters and leaves origin, destination, network, and TLS authorities unchanged. A rollback must not restore a convenience client inside a privileged authority boundary without a superseding ADR.

## Open follow-ups

- Reconcile current canonical architecture/PRD/TRD/ADR index after the HTTP branch incorporates current main.
- Add adapter-level browser navigation only after a real pinned Chromium vertical slice proves the same authority chain.
- Design separately authorized streaming download sinks for content larger than the in-memory first-slice budget.
- Evaluate HTTP/2 and HTTP/3 as separate protocol authorities rather than silently widening this implementation.

## Supersession / reversal conditions

Supersede this ADR if OriginWeave adopts a different transport abstraction, connection-reuse authority, proxy-integrated HTTP path, or protocol-unified HTTP/1.1-2-3 kernel that preserves equivalent or stronger explicit authority and evidence contracts.

## References

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP semantics* (RFC 9110; STD 97). Internet Engineering Task Force. https://doi.org/10.17487/RFC9110

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112

Berners-Lee, T., Fielding, R., & Masinter, L. (2005). *Uniform resource identifier (URI): Generic syntax* (RFC 3986; STD 66). Internet Engineering Task Force. https://doi.org/10.17487/RFC3986

Nottingham, M., & Reschke, J. (2021). *Structured field values for HTTP* (RFC 8941). Internet Engineering Task Force. https://doi.org/10.17487/RFC8941

Nottingham, M., & Kamp, P.-H. (2024). *Structured field values for HTTP* (RFC 9651). Internet Engineering Task Force. https://doi.org/10.17487/RFC9651

Polli, R., Pardue, L., & Oku, K. (2023). *Digest fields* (RFC 9530). Internet Engineering Task Force. https://doi.org/10.17487/RFC9530
