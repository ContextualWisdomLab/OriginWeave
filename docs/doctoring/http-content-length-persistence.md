# HTTP/1.1 Content-Length and persistent-connection boundary

Reviewed: 2026-09-05

## Problem

`originweave-http` is a single-use HTTP/1.1 exchange over an already authenticated TLS stream. A single-use exchange still has to recognize the HTTP message boundary independently of the transport lifetime. The predecessor implementation read exactly the declared `Content-Length` and then performed an additional read that succeeded only when the server closed the TLS stream. On an ordinary HTTP/1.1 persistent connection, that extra read could consume the entire exchange deadline after the complete response had already arrived.

## Normative boundary

RFC 9112 §6.2 states that, when content is present, `Content-Length` supplies the framing information needed to determine where the data and message end. Section 6.3 gives the precedence rules: when a valid `Content-Length` is present without `Transfer-Encoding`, the decimal value defines the expected message body length; closure or timeout is an incompleteness condition only when it occurs before that many octets have been received. Section 9.3 treats HTTP/1.1 persistence as the normal connection model and requires self-defined message lengths for persistent use.

RFC 9112 §6.3 also permits a user agent, after the final response has been completely received, to discard remaining data or inspect whether it belongs to the prior message, while prohibiting processing, caching, or forwarding such data as a separate response because of cache-poisoning risk. OriginWeave does not reuse this single-use connection, so it does not wait for EOF to prove that a length-delimited response is complete. It still rejects surplus bytes already observed inside the parsed response buffer and never reinterprets trailing bytes as another response.

## Decision, RED lineage, and repair

The realistic regression `content_length_response_completes_without_transport_eof` serves `HTTP/1.1 200 OK` with `Content-Length: 5`, sends exactly `hello`, and deliberately keeps the authenticated TLS connection open beyond the HTTP exchange deadline. The required post-condition is a successful `BodyFraming::ContentLength(5)` response before transport closure.

Test-only commit `f70a81d8da15ab418c7c667db9c727dd089bd472` established that regression while production still called `confirm_content_length_termination()` after receiving the exact declared octet count. The source-level causal path was deterministic: the extra read had no message-framing purpose and could only return after another byte, transport EOF, an I/O failure, or the deadline. The repository-native run for the later documentation generation remained queued at the time of repair, so no hosted RED is claimed for the test-only generation.

Commit `b21bfd6b7b766df31dc1e43733a183249fc6230d` applies the minimal production repair: `read_exact_content()` returns as soon as the exact declared length has been received. The now-obsolete EOF/surplus sentinel helper and its unit-only contract are removed. Incomplete bodies still fail before the expected length, surplus bytes already present in the parsed response buffer still fail closed, Transfer-Encoding/Content-Length ambiguity remains rejected by framing authority, bounded decoding remains unchanged, non-empty 205 content remains rejected after ordinary HTTP/1.1 framing, and the single-use authenticated connection is disposed rather than reused for trailing bytes.

This repair is not GREEN until repository-native exact-head CI, coverage, security, and review gates complete successfully on the unchanged head.

## References

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112
