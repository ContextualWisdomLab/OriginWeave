# HTTP/1.1 chunked message-boundary doctoring addendum

**Decision date:** 2026-08-08  
**Applies to:** `originweave-http` bounded HTTP/1.1 response handling  
**Governing ADR:** `docs/adr/0007-bounded-http11-semantics.md`

## Evidence-to-decision trace

RFC 9112 separates HTTP/1.1 message framing from connection lifetime. Section 6.3 specifies that when the final transfer coding is `chunked`, the message body length is determined by decoding the chunked transfer coding until it indicates completion. Section 7.1 defines that completion marker as the last chunk, optional trailer section, and terminating empty line. The same specification explicitly describes chunked transfer coding as enabling messages to remain self-delimited on persistent connections.

OriginWeave previously read chunked responses through the same clean-TLS-EOF helper used for close-delimited responses. That behavior was stricter than the protocol in the wrong dimension: a valid peer that emitted a complete terminal zero chunk and then kept the authenticated connection open could cause the bounded HTTP exchange to time out even though the response message was already complete.

The production path now parses the retained chunked wire prefix after every bounded TLS read. When the parser reports a complete zero-chunk/trailer terminator and no surplus bytes are already present, the response completes immediately. If the parser is incomplete, another read is permitted only under the existing total exchange deadline. A clean TLS EOF before chunked completion remains `IncompleteResponse`, and bytes already present beyond the complete chunked message remain `UnexpectedResponseBytes` because this first slice is deliberately single-use and has no connection-reuse parser state.

Close-delimited framing remains different: authenticated TLS EOF is the message delimiter and therefore remains required before the response can be complete.

This change does not add pooling or reuse authority. OriginWeave still emits `Connection: close`, consumes the authenticated connection for one exchange, and never exposes the stream for a second request. The change only prevents peer close timing from overriding an already unambiguous RFC 9112 chunked message boundary.

## Security and resource consequences

Incremental parsing removes the obsolete need for a synthetic maximum whole-wire buffer derived from worst-case per-chunk overhead. The existing independent limits remain authoritative: chunk-size line bytes, chunk count, encoded content bytes, trailer field count, trailer section bytes, and one monotonic exchange deadline. Each incomplete prefix is validated against those limits before another read can grow the retained wire buffer.

The parser continues to reject unsupported chunk extensions, malformed hexadecimal sizes, missing CRLF, excessive chunks, encoded-content overflow, forbidden or malformed trailers, incomplete terminal sections, and already-buffered bytes after a complete message. No production branch or content bound was removed to obtain interoperability.

## Realistic regression proof

A real loopback rustls server sends a complete chunked response with the body `hello`, flushes it, and intentionally keeps the TLS connection open for one second before sending `close_notify`. The client exchange deadline is 250 ms. Before the production fix, the test failed with `HttpExchangeTimedOut` because the implementation waited for TLS EOF. With incremental message-boundary parsing, the response succeeds before peer close and reports `BodyFraming::Chunked` with two chunks including the terminal zero chunk.

## APA 7th reference

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112
