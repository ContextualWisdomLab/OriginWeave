# ADR 0008: Retain HTTP reason phrases only as bounded diagnostics

- **Status:** Accepted
- **Date:** 2026-08-08
- **Decision owners:** Contextual Wisdom Lab

## Context

OriginWeave issue #9 requires the final HTTP status code and the exact HTTP/1.1 reason-phrase bytes to be retained separately while forbidding behavior from being derived from the reason phrase. The bounded response parser already enforced RFC 9112 status-line syntax, including the mandatory SP after the three-digit status code, but previously discarded the validated reason-phrase octets after parsing.

RFC 9112 defines the reason phrase as optional diagnostic text in the status line and permits HTAB, SP, visible ASCII, and `obs-text`. It also states that clients should ignore the reason phrase content. Converting these octets to UTF-8 text would reject valid `obs-text`; placing arbitrary server-provided text in credential-free evidence would also weaken the evidence boundary.

## Decision

`originweave-http` retains the exact final reason-phrase octets as a bounded `Vec<u8>` owned by `AuthenticatedHttpResponse`.

The contract is deliberately narrow:

1. the parser preserves `line[13..]` only after the complete status line passes the existing RFC 9112 syntax and status-line byte budget;
2. the bytes are exposed through `AuthenticatedHttpResponse::reason_phrase()` and returned separately by `into_parts()`;
3. no HTTP framing, status, redirect, integrity, MIME, disposition, authorization, or business decision may depend on these bytes;
4. the reason phrase is excluded from `HttpExchangeEvidence`, logs, hashes, and other credential-free evidence because a server can place arbitrary octets in it;
5. empty reason phrases and valid `obs-text` remain representable without lossy text conversion.

The status code remains the sole status-line input to HTTP semantics.

## Consequences

### Positive

- OriginWeave now satisfies the issue #9 diagnostic-retention contract without changing HTTP semantics.
- RFC 9112 `obs-text` remains lossless.
- Arbitrary server diagnostics do not contaminate credential-free evidence.
- The existing status-line byte budget bounds retained diagnostic memory.

### Negative

- `AuthenticatedHttpResponse::into_parts()` now returns a three-tuple `(content, reason_phrase, evidence)`, which is a source-level API change for callers of this unreleased crate.
- Callers that choose to display the bytes must perform their own safe presentation or escaping; the crate intentionally does not claim they are UTF-8.

## Verification

- A parser contract proves exact retention for `OK`, an empty reason phrase, and valid non-UTF-8 `obs-text`.
- The real loopback rustls HTTP exchange proves `HTTP/1.1 200 OK` reaches the public response accessor and `into_parts()` unchanged.
- Existing tests continue to prove reason-phrase content does not influence status, framing, integrity, MIME, redirect, or completion behavior.
- Exact production function, line, region, and branch coverage remains a merge gate.

## Reference

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112
