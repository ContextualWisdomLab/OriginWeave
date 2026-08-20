# Bounded HTTP/1.1 exchange doctoring

## Implemented slice

The active HTTP slice adds an independently reusable `originweave-http` crate
over an already authenticated `originweave-tls::AuthenticatedTlsConnection`.
It emits only a generated `GET` or `HEAD` request, requires `http/1.1` ALPN
or an explicit direct-test absent-ALPN policy, consumes one stream, and never
resolves, reconnects, selects a proxy, follows a redirect, persists content,
or grants browser authority.

The parser follows the RFC 9112 message boundary model: strict CRLF lines,
bounded status and header sections, bounded field names and values, explicit
`Content-Length`/`Transfer-Encoding` conflict rejection, bounded `chunked`
decoding with trailers, no-body status/method semantics, bounded
close-delimited reads, and explicit incomplete-response errors. Only
credential-free allow-listed response fields are retained. Content codings
other than `identity`, redirects, integrity fields, MIME sniffing, and download
handoff remain later authority-bound slices.

This document describes the active branch slice; it is not evidence that the
crate is already part of protected `main` or that the complete HTTP product
gap is closed.

## Standards basis

RFC 9112 defines the HTTP/1.1 start-line, header-section, body, framing, and
connection-management rules used by the parser. RFC 9110 supplies shared HTTP
semantics; the implementation keeps method and status semantics separate from
the reason phrase. RFC 9530 remains a later integrity-field boundary and is
not claimed as implemented by this slice.

## References

Fielding, R., Nottingham, M., & Reschke, J. (2022, June). *HTTP/1.1* (RFC
9112). Internet Engineering Task Force. https://datatracker.ietf.org/doc/html/rfc9112

Fielding, R., Ed., Nottingham, M., Ed., & Reschke, J., Ed. (2022, June).
*HTTP semantics* (RFC 9110). Internet Engineering Task Force.
https://datatracker.ietf.org/doc/html/rfc9110

Polli, M., Kinnear, E., & Bishop, M. (2024, February). *Digest fields* (RFC
9530). Internet Engineering Task Force. https://datatracker.ietf.org/doc/html/rfc9530
