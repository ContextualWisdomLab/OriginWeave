# HTTP/1.1 Content-Length and persistent-connection boundary

Reviewed: 2026-09-05

## Problem

`originweave-http` is a single-use HTTP/1.1 exchange over an already authenticated TLS stream. A single-use exchange still has to recognize the HTTP message boundary independently of the transport lifetime. The current implementation reads exactly the declared `Content-Length` and then performs an additional read that succeeds only when the server closes the TLS stream. On an ordinary HTTP/1.1 persistent connection, that extra read can consume the entire exchange deadline after the complete response has already arrived.

## Normative boundary

RFC 9112 §6.2 states that, when content is present, `Content-Length` supplies the framing information needed to determine where the data and message end. Section 6.3 gives the precedence rules: when a valid `Content-Length` is present without `Transfer-Encoding`, the decimal value defines the expected message body length; closure or timeout is an incompleteness condition only when it occurs before that many octets have been received. Section 9.3 treats HTTP/1.1 persistence as the normal connection model and requires self-defined message lengths for persistent use.

RFC 9112 §6.3 also permits a user agent, after the final response has been completely received, to discard remaining data or inspect whether it belongs to the prior message, while prohibiting processing, caching, or forwarding such data as a separate response because of cache-poisoning risk. OriginWeave does not reuse this single-use connection, so it does not need to wait for EOF to prove that a length-delimited response is complete. It must still reject surplus bytes already observed inside the parsed response buffer and must never reinterpret trailing bytes as another response.

## Decision and RED

The realistic regression `content_length_response_completes_without_transport_eof` serves `HTTP/1.1 200 OK` with `Content-Length: 5`, sends exactly `hello`, and deliberately keeps the authenticated TLS connection open beyond the HTTP exchange deadline. The expected post-condition is a successful `BodyFraming::ContentLength(5)` response before transport closure.

The test is intentionally introduced before changing production framing code. The current generation therefore remains a RED candidate until repository-native execution demonstrates the expected timeout caused by the extra terminator read. A subsequent causal repair should remove the EOF dependency after the exact declared length has been received while retaining incomplete-body failure, in-buffer surplus rejection, Transfer-Encoding/Content-Length ambiguity rejection, bounded decoding, and single-use connection disposal.

## References

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112
