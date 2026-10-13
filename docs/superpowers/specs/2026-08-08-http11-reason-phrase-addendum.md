# HTTP/1.1 Reason-Phrase Retention Addendum

**Status:** Proposed amendment to `2026-08-07-http11-semantics-design.md`, pending qualifying independent non-author review  
**Date:** 2026-08-08  
**Scope:** Preserve exact final reason-phrase octets without expanding HTTP authority

## Proposed amendment

The original HTTP/1.1 design already requires the status parser to accept bounded reason-phrase bytes for diagnostics and never derive semantics from them. This addendum makes the corresponding public response contract explicit.

`AuthenticatedHttpResponse` exposes:

```rust
pub fn content(&self) -> &[u8];
pub fn reason_phrase(&self) -> &[u8];
pub fn evidence(&self) -> &HttpExchangeEvidence;
pub fn into_parts(self) -> (Vec<u8>, Vec<u8>, HttpExchangeEvidence);
```

The middle vector returned by `into_parts()` is the exact final HTTP/1.1 reason phrase after syntax validation. It is bytes rather than `String` because RFC 9112 permits `obs-text`. The value is bounded by the existing status-line byte budget.

The reason phrase is deliberately not part of `HttpExchangeEvidence`. It is remote-controlled diagnostic data and cannot influence status semantics, framing, redirects, integrity checks, MIME/disposition classification, authorization, completion, or product policy. The numeric status code remains the only status-line semantic input.

This amendment does not add network, redirect-following, persistence, rendering, authentication, cookie, proxy, or browser authority.

## Acceptance tests

1. Retain `b"OK"` from `HTTP/1.1 200 OK`.
2. Retain an empty value from `HTTP/1.1 204` followed by one required `SP` octet while still requiring the second status-line separator.
3. Retain valid non-UTF-8 `obs-text` octets losslessly.
4. Prove the public accessor and `into_parts()` through a real loopback rustls HTTP exchange.
5. Keep exact production function, line, region, and branch coverage at 100% and preserve strict Clippy/rustdoc gates.
6. Treat this amendment as accepted only after qualifying independent review and merge of the protected pull-request head.

## Reference

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112
