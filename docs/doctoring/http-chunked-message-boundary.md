# HTTP/1.1 chunked message-boundary doctoring addendum

**Decision date:** 2026-08-08  
**Current-head integrity update:** 2026-09-23  
**Applies to:** `originweave-http` bounded HTTP/1.1 response handling  
**Governing ADR:** `docs/adr/0011-bounded-http11-semantics.md`

## Evidence-to-decision trace

RFC 9112 separates HTTP/1.1 message framing from connection lifetime. Section 6.3 specifies that when the final transfer coding is `chunked`, the message body length is determined by decoding the chunked transfer coding until it indicates completion. Section 7.1 defines that completion marker as the last chunk, optional trailer section, and terminating empty line. The same specification explicitly describes chunked transfer coding as enabling messages to remain self-delimited on persistent connections.

OriginWeave previously read chunked responses through the same clean-TLS-EOF helper used for close-delimited responses. That behavior was stricter than the protocol in the wrong dimension: a valid peer that emitted a complete terminal zero chunk and then kept the authenticated connection open could cause the bounded HTTP exchange to time out even though the response message was already complete.

The production path now retains a bounded chunked wire buffer and a stateful decoder across bounded TLS reads. The decoder advances only after a complete chunk line/body boundary or accepted trailer line. When it reports a complete zero-chunk/trailer terminator and no surplus bytes are already present, the response completes immediately. If the parser is incomplete, another read is permitted only under the existing total exchange deadline. A clean TLS EOF before chunked completion remains `IncompleteResponse`, and bytes already present beyond the complete chunked message remain `UnexpectedResponseBytes` because this slice is deliberately single-use and has no connection-reuse parser state.

Close-delimited framing remains different: authenticated TLS EOF is the message delimiter and therefore remains required before the response can be complete.

This change does not add pooling or reuse authority. OriginWeave still emits `Connection: close`, consumes the authenticated connection for one exchange, and never exposes the stream for a second request. The change only prevents peer close timing from overriding an already unambiguous RFC 9112 chunked message boundary.

## Incremental parser integrity invariant

The 2026-09-23 current-head review found a separate state-integrity defect in the incremental decoder contract. `ChunkedDecoder` retained its cursor and decoded state between calls, but the previous API accepted an arbitrary replacement slice on each call and resumed from the stored cursor without proving that bytes before that cursor were unchanged. The production caller already grows one retained `wire` buffer append-only, so the finding did not demonstrate a remote ability to rewrite bytes already received from TLS. It did show that the reusable decoder authority itself could silently trust a mutated committed prefix if a future caller violated the implicit append-only assumption.

The repaired decoder therefore makes the assumption executable instead of documentary. Before every resumed parse it requires the supplied input to start with the exact bytes already committed by earlier successful parse steps. A mismatch fails closed as `MalformedChunkedBody`. Newly trusted bytes are added to the committed witness only after their chunk/body boundary or trailer syntax has been accepted. Append-only growth remains valid. The regression pair covers both sides: mutation of a committed prefix must fail, while the same first chunk followed by ordinary append-only growth must still complete normally.

The witness is exact bytes rather than a hash. The invariant is therefore deterministic and does not make parser correctness depend on collision resistance. This has a bounded memory cost: during an incomplete chunked response the caller retains the bounded wire prefix and the decoder can additionally retain up to the committed portion of that same prefix, while decoded content remains subject to the independent encoded-content limit. This duplication is accepted for the current repair because it closes the integrity gap without changing HTTP framing semantics or introducing a new cryptographic dependency. It is not a claim that the representation is memory-optimal; a future optimization may replace the duplicated witness with an ownership type that makes append-only mutation impossible by construction, but it must preserve the same fail-closed regression contract and resource limits.

## Security and resource consequences

The retained chunked wire buffer has an independent product memory budget in addition to the parser's semantic limits. Before any retained-buffer growth, OriginWeave enforces the derived maximum

`max_encoded_content_bytes + max_chunk_count × (MAX_CHUNK_LINE_BYTES + 4) + max_trailer_section_bytes + MAX_CHUNK_LINE_BYTES + 4`.

With the reviewed strict defaults (`16 MiB` encoded content, `65,536` chunks, a `16`-byte chunk-size line, and `16 KiB` trailers), the maximum retained chunked wire prefix is exactly `18,104,340` bytes, which is below `18 MiB`. This is a product resource-safety bound, not an additional RFC 9112 protocol-validity limit. Reads are restricted to the remaining retained-buffer capacity plus one sentinel byte; if more wire data arrives after the cap is full, the exchange fails closed before the caller-owned `Vec` can grow beyond that maximum.

The committed-prefix witness introduced by the integrity repair is bounded by the same parser progress and cannot exceed the supplied retained wire prefix. It does increase current worst-case transient per-exchange memory relative to the prior decoder, so concurrency capacity planning must account for both retained wire and committed witness rather than treating `18,104,340` bytes as the whole exchange footprint.

The existing independent semantic limits remain authoritative: chunk-size line bytes, chunk count, encoded content bytes, trailer field count, trailer section bytes, and one monotonic exchange deadline. Each incomplete prefix is validated against those limits before another read can grow the retained wire buffer. The parser continues to reject unsupported chunk extensions, malformed hexadecimal sizes, missing CRLF, excessive chunks, encoded-content overflow, forbidden or malformed trailers, incomplete terminal sections, mutation of already committed bytes, and already-buffered bytes after a complete message. No production branch or content bound was removed to obtain interoperability.

## Realistic regression proof

A real loopback rustls server sends a complete chunked response with the body `hello`, flushes it, and intentionally keeps the TLS connection open for one second before sending `close_notify`. The client exchange deadline is 250 ms. Before the message-boundary production fix, the test failed with `HttpExchangeTimedOut` because the implementation waited for TLS EOF. With incremental message-boundary parsing, the response succeeds before peer close and reports `BodyFraming::Chunked` with two chunks including the terminal zero chunk.

A Rust contract invokes the production retained-wire calculation directly and fixes the strict-default result at `18,104,340` bytes. A second contract exercises the exact append guard and proves that an over-cap byte is rejected before it mutates the retained prefix. The 2026-09-23 committed-prefix regression additionally proves that previously accepted bytes cannot be replaced between incremental parse calls while preserving the normal append-only continuation path. These contracts prevent a future parser or policy refactor from silently restoring an unbounded retained allocation, quadratic reparse, or mutable committed-prefix trust.

## APA 7th reference

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112
