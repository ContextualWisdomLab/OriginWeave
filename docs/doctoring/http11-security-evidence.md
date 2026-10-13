# HTTP/1.1 Redirect, Download-Metadata, and MIME Security Evidence

This document is an authoritative doctoring addendum for ADR 0011. It records primary-source evidence that changed the bounded HTTP/1.1 redirect, download-metadata, framing, content-coding, and observed-MIME contracts. References use APA 7th style.

## Redirect-reference authority

RFC 3986 distinguishes a relative reference beginning with two slash characters (`//authority/path`) from one beginning with a single slash (`/path`). Section 4.2 calls the former a **network-path reference** and the latter an **absolute-path reference**. The authority component therefore cannot be discarded merely because both forms begin with `/`.

OriginWeave's first bounded HTTP slice intentionally does not own base-URI resolution. Consequently, `Location: //authority/path` is rejected as `InvalidRedirectMetadata` instead of being represented as same-origin relative metadata. A single-leading-slash location remains same-origin metadata. This preserves the invariant that any redirect capable of changing authority must return to canonical origin parsing, destination approval, exact TCP peer proof, TLS authentication, capability/risk policy, and a new HTTP exchange.

Regression evidence includes an explicit `//evil.example/path` case that must fail closed. This is a semantic authority test, not a string-format preference.

## Portable filename handoff

Microsoft's Win32 naming guidance documents the reserved filename characters `<`, `>`, `:`, `"`, `/`, `\`, `|`, `?`, and `*`, together with NUL/control restrictions, reserved device basenames, and trailing-dot/space limitations. A future OriginWeave download adapter may persist bytes on Windows, Unix-like systems, or a provider-neutral object store; the HTTP semantics crate therefore emits only a conservative portable filename record rather than a filesystem-specific path.

Validation occurs after quoted-string or RFC 8187 extended-value decoding. RFC 8187 supersedes RFC 5987 and is the current HTTP header-parameter encoding reference for this contract. This ordering is security-significant: an escaped double quote that is syntactically valid inside `Content-Disposition` must not become an admitted Win32 filename after decoding. Regression tests cover every newly enforced reserved character, including an escaped quote.

The HTTP crate still does not create files. It supplies bounded metadata to a later separately authorized persistence boundary.

## Observed MIME text/binary boundary

The current WHATWG MIME Sniffing Standard defines a **binary data byte** narrowly: `0x00..=0x08`, `0x0B`, `0x0E..=0x1A`, and `0x1C..=0x1F`. In the unknown-MIME algorithm, a bounded resource header that contains none of those bytes is classified as `text/plain`; validity as UTF-8 is not a prerequisite. Bytes in `0x80..=0xFF` therefore cannot be treated as binary merely because a prefix is not valid UTF-8.

OriginWeave's observed classifier follows that byte-level contract after higher-priority reviewed signatures. The implementation still reads only the bounded sniff prefix, and active/scriptable signatures keep their separate conservative risk handling. Regression evidence includes non-UTF-8 high bytes that must remain passive `text/plain` and control-byte content that must remain `application/octet-stream`. This prevents evidence drift caused by conflating text/binary sniffing with Unicode decoding.

The same WHATWG unknown-MIME signature table defines an exact `<?xml` signature whose computed MIME type is `text/xml`. OriginWeave therefore records `text/xml` for that observed signature rather than normalizing it to the related `application/xml` alias. This distinction is evidence-significant: a supplied `Content-Type: text/xml` must compare as an exact essence match when the observed prefix is the standard XML signature. Because that observable classifier output is persisted as evidence, this contract change advances the classifier version from `originweave-mime-signatures-1` to `originweave-mime-signatures-2`.

## Request-target diagnostic and evidence redaction

An HTTP origin-form request target can legitimately contain credentials or other protected material in either query values or path segments. Authorization of the surrounding origin and path shape does not make those bytes safe for logs or immutable evidence. OriginWeave therefore treats the complete encoded path-and-query as wire-only request state: it is retained only by `HttpRequestTarget` long enough to serialize the exact request and is omitted from structural `Debug` output.

Credential-free request evidence records the canonical origin, a domain-separated SHA-256 identifier for the exact encoded target, whether a query component exists, and only the constant root path prefix `/`. No raw path or query bytes, path segments, or encoded-path length are retained in `HttpExchangeEvidence`. The exact target remains cryptographically bound through `target_hash`, so removing human-readable path bytes does not weaken immutable request identity.

Regression evidence uses a request whose path itself contains `credential-value` and whose query contains `q=secret`. The authenticated loopback server must still receive the exact wire request, while the evidence accessor returns only `/` and the evidence `Debug` representation must omit both credential-shaped values. This proves that diagnostic/evidence redaction does not rewrite network behavior or silently replace exact target identity with a lossy string.

## Content-Length message boundary

RFC 9112 makes the determined message-body length authoritative: when `Content-Length` governs framing, the recipient reads exactly that amount as the response body. A persistent transport can remain open after a self-delimited message; transport EOF is not part of that message's completion condition. OriginWeave therefore returns once the exact declared octet count has been authenticated and received instead of waiting for a post-body sentinel read or TLS closure.

This does not admit bytes that are already buffered beyond the declared message boundary. Such surplus remains a fail-closed `UnexpectedResponseBytes` condition because it is unambiguously present in the same bounded read state. Bytes that arrive only after the self-delimited response has completed are not reinterpreted as current content or as a second response; this HTTP authority is single-use and does not pipeline or reuse the stream.

Regression evidence uses a real loopback TLS server that sends `Content-Length: 5`, exactly `hello`, and then keeps the transport open beyond the HTTP exchange deadline. The exchange must complete at the declared body boundary. Separate regressions preserve failure for fewer-than-declared bytes and for surplus bytes already present with the completed response.

## Gzip member semantics

RFC 9110 defines the HTTP `gzip` content coding by reference to RFC 1952. RFC 1952 defines a gzip file as a series of members concatenated without additional framing between them. A second valid gzip member is therefore part of the same coded representation, not trailing garbage.

OriginWeave decodes the complete RFC 1952 member sequence with `flate2`'s multi-member gzip decoder while applying the same aggregate decoded-byte and expansion-ratio budgets used for every other supported content coding. The contract still fails closed when bytes after a valid member sequence cannot be parsed as another gzip member. Regression evidence constructs two independently encoded gzip members and requires their decoded payloads to be concatenated in order; a separate malformed-trailing-bytes case remains a typed `ContentDecodingFailed` error.

## Reset Content response semantics

RFC 9110 Section 15.3.6 forbids content in a `205 Reset Content` response, while RFC 9112 Section 6.3 does not place 205 among the status codes whose messages end at the header boundary. OriginWeave therefore applies the declared, chunked, or close-delimited wire framing first and rejects any non-empty 205 content as `UnexpectedResponseBytes`. This prevents a segmented prohibited body from being mistaken for a successful empty response.

## Verification contract

The exact pull-request head must demonstrate:

- rejection of RFC 3986 network-path redirect references without losing their authority in evidence;
- preservation of admitted single-leading-slash same-origin redirect metadata;
- rejection of all Win32 reserved filename characters after decoding;
- preservation of existing control, path, device-name, bidi, length, dot, and whitespace restrictions;
- WHATWG byte-level text/binary classification, including passive non-UTF-8 high bytes and binary control-byte rejection;
- exact WHATWG XML signature evidence as `text/xml`, with supplied `text/xml` producing `MimeMismatch::Match` and the classifier version reflecting the changed evidence semantics;
- exact request-target wire serialization while credential-free evidence retains only the target digest, query-presence flag, and constant root prefix `/`;
- request-target and exchange-evidence debug output that cannot expose raw path/query bytes or credential-shaped values;
- exact `Content-Length` completion without transport-EOF dependence, while truncation and already-buffered surplus remain fail closed;
- decoding of complete RFC 1952 multi-member gzip representations under the existing decoded-byte and expansion-ratio budgets, while malformed trailing bytes remain rejected;
- RFC 9112 framing followed by semantic rejection of non-empty `205 Reset Content` responses;
- Rust formatting, workspace checks, tests, Clippy, and rustdoc;
- exact 100% production function, line, region, statement, and branch coverage;
- Security Scan, SAST, all operationally required current review gates, and branch-protection gates.

## References

Berners-Lee, T., Fielding, R., & Masinter, L. (2005). *Uniform resource identifier (URI): Generic syntax* (RFC 3986). Internet Engineering Task Force. https://doi.org/10.17487/RFC3986

Deutsch, L. P. (1996). *GZIP file format specification version 4.3* (RFC 1952). Internet Engineering Task Force. https://doi.org/10.17487/RFC1952

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP semantics* (RFC 9110). Internet Engineering Task Force. https://doi.org/10.17487/RFC9110

Microsoft. (n.d.). *Naming files, paths, and namespaces*. Microsoft Learn. Retrieved August 7, 2026, from https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file

Reschke, J. (2017). *Indicating character encoding and language for HTTP header field parameters* (RFC 8187). Internet Engineering Task Force. https://doi.org/10.17487/RFC8187

Web Hypertext Application Technology Working Group. (2026). *MIME Sniffing Standard*. https://mimesniff.spec.whatwg.org/
