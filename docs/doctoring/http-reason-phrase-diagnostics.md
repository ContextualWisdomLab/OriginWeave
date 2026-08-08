# HTTP reason-phrase diagnostics — standards doctoring addendum

**Decision date:** 2026-08-08  
**Applies to:** `originweave-http` bounded HTTP/1.1 response handling  
**Governing ADR:** `docs/adr/0008-http-reason-phrase-diagnostics.md`

## Evidence-to-decision trace

RFC 9112 defines the HTTP/1.1 status line as `HTTP-version SP status-code SP [ reason-phrase ]`. The second separating SP is mandatory even when the optional reason phrase is empty. The reason phrase grammar permits HTAB, SP, visible ASCII, and `obs-text`; therefore a conforming diagnostic value is not necessarily valid UTF-8. RFC 9112 also states that clients should ignore the reason phrase content because it is potentially obsolete and might not convey reliable semantics.

OriginWeave therefore keeps the status code and reason phrase as separate values. The status code alone participates in HTTP semantics. After the existing bounded status-line parser validates the complete line, the exact reason-phrase octets are retained as `Vec<u8>` on `AuthenticatedHttpResponse` for diagnostics and are never interpreted as status, framing, redirect, integrity, MIME, disposition, authorization, or business-policy input.

The bytes are deliberately excluded from `HttpExchangeEvidence`, generic logs, and credential-free provenance. A remote server controls this field and can place arbitrary diagnostic octets in it; copying it into evidence would turn an attacker-controlled string into durable metadata without any product need. The status-line byte budget already bounds the retained value, so no separate unbounded allocation path is introduced.

## Required verification

- Accept `HTTP/1.1 200 OK` and retain `b"OK"` exactly.
- Accept `HTTP/1.1 204` followed by one required `SP` octet and retain an empty byte sequence as the reason phrase.
- Accept valid `obs-text` reason bytes such as `b"O\xffK"` without lossy UTF-8 conversion.
- Reject a status line that omits the mandatory second SP.
- Demonstrate through a real authenticated loopback TLS exchange that the exact final reason phrase reaches the public response accessor and `into_parts()` unchanged.
- Preserve exact 100% production function, line, region, and branch coverage and all existing framing/security tests.

## APA 7th reference

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112
