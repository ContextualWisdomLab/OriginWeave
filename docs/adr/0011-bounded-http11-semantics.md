# ADR 0011: Bind bounded HTTP/1.1 semantics to the authenticated TLS stream

- Status: Proposed
- Date: 2026-08-09
- Supersedes: historical unmerged `docs/adr/0007-bounded-http11-semantics.md` from PR #11

## Context

OriginWeave separates canonical origin, approved resolved destination, exact operating-system TCP peer, and TLS service identity. Those authorities do not make an HTTP exchange safe. An authenticated peer can still send ambiguous framing, conflicting lengths, malformed chunks, oversized fields, incomplete messages, decompression bombs, unsafe redirect metadata, misleading MIME metadata, hostile filenames, or integrity metadata that is malformed or does not match the content.

A general-purpose client would also introduce connector, DNS, proxy, redirect, pooling, cookie, credential, retry, and runtime behavior that is outside the authority already proven by the destination, network, and TLS crates. The HTTP boundary therefore must consume one existing `AuthenticatedTlsConnection` and must not create a second path to the network.

PR #11 established the original HTTP design and TDD evidence, but it diverged from protected-main work. PR #37 reconstructs that unique behavior on current OriginWeave lineage without transferring predecessor checks or approvals. Stacked content-coding support is being developed as the ordinary-forward successor #327 under issue #326. This ADR remains Proposed until the HTTP lineage reaches protected-main review and implementation evidence; active-PR source is not shipped truth.

## Decision drivers

- Preserve the exact origin -> destination -> TCP peer -> TLS identity authority chain.
- Reject request smuggling and ambiguous HTTP/1.1 framing.
- Bound all attacker-controlled retained bytes and decode expansion.
- Interoperate with standards-valid ordered `Content-Encoding` lists without allowing layer count or expansion accounting to become an amplification primitive.
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

### Reject every stacked content coding

Rejected for the successor interoperability contract. RFC 9110 defines `Content-Encoding` as an ordered list and requires codings to be listed in application order. OriginWeave already advertises `Accept-Encoding: gzip, deflate`; accepting either coding alone while rejecting a standards-valid two-layer composition is an avoidable buyer-visible mismatch once a bounded implementation and evidence model exist.

### Unbounded or caller-configurable coding depth

Rejected. RFC list syntax is not a product resource budget. The first stacked slice admits at most two non-empty codings, matching the two fixed advertised codings. Increasing that maximum or making it independently caller-lowerable while the advertisement remains fixed would create a separate negotiation/resource decision.

### Reset expansion accounting at each coding layer

Rejected. A per-layer denominator would permit effective amplification approaching `ratio^depth`. Every layer output is therefore checked against the original transfer-decoded, still-content-coded byte count.

### Follow redirects internally

Rejected. A redirected target may alter origin, DNS results, route, TCP peer, TLS identity, capability, sensitive-data scope, and policy. It requires a new complete authority evaluation.

## Decision

Create the independently reusable `originweave-http` Rust crate. It performs exactly one HTTP/1.1 `GET` or `HEAD` exchange over one existing `AuthenticatedTlsConnection`.

The serializer owns `Host`, `Connection: close`, and `Accept-Encoding: gzip, deflate`. Caller fields cannot override authority, credentials, framing, proxy, connection, upgrade, cookie, trailer, or response-coding negotiation fields. Origin-form targets reject fragments, absolute/authority form, invalid percent escapes, controls, whitespace, backslashes, and oversized encoded targets.

The parser accepts strict HTTP/1.1 CRLF syntax, a bounded number of informational responses, and one final response. It rejects obsolete field folding, invalid field syntax, `Transfer-Encoding` with `Content-Length`, unsupported transfer coding, conflicting lengths, protocol upgrades, malformed chunks, forbidden trailers, and ambiguous or surplus bytes.

Body framing follows RFC 9110 and RFC 9112 with these product constraints:

1. `HEAD`, informational responses, 204, and 304 expose no content.
2. The only supported transfer-coding list is exactly `chunked`.
3. Repeated or comma-separated `Content-Length` members are accepted only when every decimal member is byte-identical after optional whitespace is trimmed; numerically equal but differently spelled members such as `042` and `42` fail closed as conflicting framing evidence.
4. Chunked completion occurs at the terminal zero chunk plus bounded trailers and final empty line; peer close is not required.
5. Close-delimited content is complete only after authenticated clean TLS EOF.
6. This first slice is single-use and retains no parser state for connection reuse.

The strict policy bounds request bytes, target bytes, status line, field counts/names/values/section, interim responses, chunk count/line size, trailer count/section, encoded and decoded content, decoded-to-encoded expansion, and one monotonic total exchange timeout. The unfinished chunked wire prefix has an independent derived memory bound checked before retained-buffer growth.

Content-coding admission parses the complete repeated/combined `Content-Encoding` list before decoder work. Empty list elements are ignored within the existing bounded header bytes, as RFC 9110 requires recipients to tolerate a reasonable number of them; they do not count toward coding depth or create decoder invocations. Missing, all-empty, and the retained sole explicit `identity` compatibility form mean no transform. `identity` mixed with another non-empty coding, unknown codings, and more than two non-empty codings fail before decoding.

The admitted non-empty coding chain contains only `gzip` and `deflate`, preserves exact field/member encounter order, and has fixed maximum depth two. Decoding runs in reverse application order. Gzip retains RFC 1952 concatenated-member semantics inside one HTTP coding layer. Deflate first attempts the RFC 9110 zlib-wrapped form; only an ordinary decoding-format failure may enter the bounded raw RFC 1951 compatibility path for that exact declared `deflate` layer. Size or expansion failures never trigger fallback, and nested layers do not introduce combinatorial retries.

Every decoder output is checked against the existing decoded-byte ceiling and against the original transfer-decoded coded-body byte count. The ratio denominator never resets at a layer boundary. The current implementation materializes bounded `Vec<u8>` intermediates rather than streaming them; under the strict defaults its coarse payload-buffer peak is approximately 80 MiB: at most 16 MiB of original coded content plus one prior intermediate up to 32 MiB plus one new output up to 32 MiB, excluding allocator, decoder, and TLS overhead. This is a documented first-slice bound, not a streaming claim.

`HttpExchangeEvidence` retains the exact admitted non-empty wire/application-order coding sequence and the decoder outcome for each layer without retaining response content. A standards gzip layer, standards zlib-wrapped deflate layer, and raw-DEFLATE-compatible declared deflate layer are distinct typed evidence states, so an impossible gzip/raw-DEFLATE combination cannot be constructed. Compatibility outcome is not rewritten into a synthetic wire coding and wire order is not reconstructed from the terminal decoder result.

RFC 9530 integrity validation remains on the existing byte-domain boundary: transfer coding has been removed, but content coding has not. Stacked decoding does not move `Content-Digest` or `Repr-Digest` verification to final plaintext and does not collapse their distinct HTTP content/representation semantics. RFC 9530 Structured Fields parsing remains on the RFC 8941 definition to which that field specification was bound; RFC 9651-only Date and Display String bare-item syntax remains outside this slice.

MIME handling records supplied type separately from a conservative versioned observed classification. `Content-Disposition` may yield only bounded portable metadata; it does not create a file. Redirect handling returns bounded/hash-oriented metadata and never follows the redirect. Network-path redirect references that could carry an authority are never collapsed into same-origin path metadata.

`HttpExchangeEvidence` records the inherited origin/peer/TLS summary plus method, bounded/hash-oriented target information, status, response field names and byte counts, framing, ordered content-coding/decoder evidence, byte budgets, chunk/trailer decisions, integrity status, MIME/disposition/redirect classifications, completeness, elapsed time, and configured resource limits. It does not retain credentials, cookies, arbitrary request/response field values, response content, query values, unsafe filenames, certificates, or raw redirect locations.

## Consequences

### Positive

- One authenticated stream yields one deterministic bounded HTTP result.
- HTTP framing cannot silently create a second network authority path.
- Persistent peers are interoperable for completed chunked messages without weakening close-delimited completeness.
- Standards-valid two-layer `gzip`/`deflate` compositions can be represented without unbounded decoder depth or per-layer expansion-budget reset.
- Coding audit evidence preserves declared wire order and actual compatibility outcome independently.
- Resource limits and evidence are explicit and testable.
- Redirect and download metadata return to later policy/persistence authorities rather than bypassing them.

### Negative

- HTTP/1.1 only and `Connection: close` reduce performance and compatibility.
- Strict parsing rejects some legacy-but-tolerated messages.
- Content-coding depth is intentionally capped at two; Brotli and Zstandard remain unsupported.
- The first stacked slice materializes bounded intermediate buffers. Its approximate 80 MiB strict-default payload-buffer ceiling excludes allocator, decoder, and TLS overhead and therefore is not a complete process-RSS bound.
- Authentication, cookies, proxying, caching, HTTP/2/3, streaming downloads, and browser integration remain separate future work.

## Failure and degraded behavior

Any malformed syntax, framing conflict, unknown/mixed/excess content coding, limit breach, incomplete response, timeout, unclean close where EOF is semantic, decoder failure, integrity mismatch, unsafe metadata, or timeout-restoration failure returns a typed error and withholds a successful response. Parser-level coding rejection occurs before decoder work. Resource-limit failures do not activate raw-DEFLATE fallback. The single-use authenticated stream is consumed on success or failure and is never reused after ambiguous state.

## Security / privacy / governance impact

The decision reduces request-smuggling, decompression-amplification, SSRF-authority-confusion, redirect, unsafe-download-name, and accidental evidence-disclosure risk. Fixed coding depth, original-coded expansion accounting, exact-layer fallback, and content-free typed evidence make the added interoperability auditable without transferring policy authority to a codec. It does not replace upstream destination/TLS authority or downstream content/rendering/business authorization. No model credential or protected value belongs in this layer.

## Tests and acceptance evidence

Acceptance requires deterministic parser/boundary tests and real authenticated loopback TLS integration. For stacked content coding that includes both coding orders, repeated-field and comma-list forms, ignored/all-empty list members, mixed identity, unknown/depth-overflow rejection before decoder work, malformed outer and inner layers, truncated members, trailing bytes, cumulative expansion against the original coded length, zero-byte coded input, raw-DEFLATE fallback on the exact declared layer, coded-byte digest semantics, and exact wire-order decoder evidence. The captured request must prove the client actually advertised `Accept-Encoding: gzip, deflate`.

The causal predecessor RED is #327 exact `dc68914cd4721ebd4e65e1f7034a665c39042dfb`, where hosted Production coverage run `35901066797` executed the authenticated-TLS matrix and exposed the old single-coding rejection. Current source repair is active-PR evidence only until the unchanged exact head completes repository contracts, formatting, Clippy, rustdoc, exact owned production function/line/region/branch coverage, applicable Security/SAST/CodeQL, independent review, and the live protected-branch ruleset. Queued, skipped, predecessor, or status-only evidence is not acceptance.

## Migration and rollback

The crate and the stacked-coding successor are additive within the still-unshipped HTTP lineage. Consumers do not gain a second network path or new persistence authority. Rollback of the stacked successor restores the bounded single-coding behavior while leaving origin, destination, network, TLS, framing, digest, MIME, redirect, and evidence authorities unchanged. Rollback must not increase coding depth, expansion limits, advertised codings, or reintroduce a convenience client. If the fixed advertisement changes later, coding admission and request negotiation must be reviewed as one policy rather than drift independently.

## Open follow-ups

- Reconcile current canonical architecture/PRD/TRD/ADR index after the HTTP branch incorporates current main.
- Add adapter-level browser navigation only after a real pinned Chromium vertical slice proves the same authority chain.
- Replace the bounded intermediate-buffer pipeline with a streaming decoder only after equivalent size/ratio, integrity-byte-domain, decoder-outcome evidence, cancellation, and recovery behavior are proven.
- Measure representative compressed buyer payloads and retain exact p95/resource evidence before making performance claims about stacked decoding.
- Add separately authorized streaming download sinks for content larger than the in-memory first-slice budget.
- Evaluate Brotli, Zstandard, coding depth above two, and HTTP/2 or HTTP/3 only as separate interoperability/resource decisions rather than silently widening this implementation.

## Supersession / reversal conditions

Supersede this ADR if OriginWeave adopts a different transport abstraction, connection-reuse authority, proxy-integrated HTTP path, content-decoding pipeline, or protocol-unified HTTP/1.1-2-3 kernel that preserves equivalent or stronger explicit authority, resource, integrity-byte-domain, and evidence contracts.

## References

Berners-Lee, T., Fielding, R., & Masinter, L. (2005). *Uniform resource identifier (URI): Generic syntax* (RFC 3986; STD 66). Internet Engineering Task Force. https://doi.org/10.17487/RFC3986

Deutsch, P. (1996). *DEFLATE compressed data format specification version 1.3* (RFC 1951). Internet Engineering Task Force. https://doi.org/10.17487/RFC1951

Deutsch, P. (1996). *GZIP file format specification version 4.3* (RFC 1952). Internet Engineering Task Force. https://doi.org/10.17487/RFC1952

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP semantics* (RFC 9110; STD 97). Internet Engineering Task Force. https://doi.org/10.17487/RFC9110

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112

Nottingham, M., & Kamp, P.-H. (2021). *Structured field values for HTTP* (RFC 8941). Internet Engineering Task Force. https://doi.org/10.17487/RFC8941

Nottingham, M., & Kamp, P.-H. (2024). *Structured field values for HTTP* (RFC 9651). Internet Engineering Task Force. https://doi.org/10.17487/RFC9651

Polli, R., & Pardue, L. (2024). *Digest fields* (RFC 9530). Internet Engineering Task Force. https://doi.org/10.17487/RFC9530
