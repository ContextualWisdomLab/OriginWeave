# HTTP/1.1 Redirect, Download-Metadata, and MIME Security Evidence

This document is an authoritative doctoring addendum for ADR 0011. It records primary-source evidence that changed the bounded HTTP/1.1 redirect, download-metadata, and observed-MIME contracts. References use APA 7th style.

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

Credential-free request evidence records the canonical origin, a domain-separated SHA-256 identifier for the exact encoded target, whether a query component exists, and a structural marker of the form `<redacted-path:N-bytes>`, where `N` is the encoded path byte count before `?`. The raw path and query bytes are not retained in `HttpExchangeEvidence`. The exact target remains cryptographically bound through `target_hash`, so removing human-readable path bytes does not weaken immutable request identity.

Regression evidence uses a request whose path itself contains `credential-value` and whose query contains `q=secret`. The authenticated loopback server must still receive the exact wire request, while both the evidence accessor and the evidence `Debug` representation must omit those credential-shaped bytes. This proves that diagnostic/evidence redaction does not rewrite network behavior or silently replace exact target identity with a lossy string.

## Verification contract

The exact pull-request head must demonstrate:

- rejection of RFC 3986 network-path redirect references without losing their authority in evidence;
- preservation of admitted single-leading-slash same-origin redirect metadata;
- rejection of all Win32 reserved filename characters after decoding;
- preservation of existing control, path, device-name, bidi, length, dot, and whitespace restrictions;
- WHATWG byte-level text/binary classification, including passive non-UTF-8 high bytes and binary control-byte rejection;
- exact WHATWG XML signature evidence as `text/xml`, with supplied `text/xml` producing `MimeMismatch::Match` and the classifier version reflecting the changed evidence semantics;
- exact request-target wire serialization while credential-free evidence retains only the target digest, query-presence flag, and structural encoded-path byte count;
- request-target and exchange-evidence debug output that cannot expose raw path/query bytes or credential-shaped values;
- Rust formatting, workspace checks, tests, Clippy, and rustdoc;
- exact 100% production function, line, region, statement, and branch coverage;
- Security Scan, SAST, all operationally required current review gates, and branch-protection gates.

## References

Berners-Lee, T., Fielding, R., & Masinter, L. (2005). *Uniform resource identifier (URI): Generic syntax* (RFC 3986). Internet Engineering Task Force. https://doi.org/10.17487/RFC3986

Microsoft. (n.d.). *Naming files, paths, and namespaces*. Microsoft Learn. Retrieved August 7, 2026, from https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file

Reschke, J. (2017). *Indicating character encoding and language for HTTP header field parameters* (RFC 8187). Internet Engineering Task Force. https://doi.org/10.17487/RFC8187

Web Hypertext Application Technology Working Group. (2026). *MIME Sniffing Standard*. https://mimesniff.spec.whatwg.org/
