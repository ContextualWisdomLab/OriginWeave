# Bounded HTTP/1.1 Semantics Design

**Status:** Approved product scope via issue #9; independent non-author review remains required for merge  
**Date:** 2026-08-07  
**Scope:** Perform one fail-closed HTTP/1.1 `GET` or `HEAD` exchange over an already authenticated OriginWeave TLS stream

## Problem

OriginWeave now separates four authorities:

1. a canonical web origin;
2. an approved resolved destination;
3. the exact operating-system TCP peer;
4. the TLS service identity authenticated on that same stream.

Those authorities do not make an HTTP exchange safe. An authenticated server can still send ambiguous framing, conflicting length fields, malformed chunk syntax, unbounded headers, compression bombs, incomplete content, unsafe redirect metadata, misleading MIME labels, or hostile download names. A convenience HTTP client can also reconnect, resolve again, follow redirects, inherit proxy settings, retain credentials, pool a connection, or silently normalize malformed messages. Any of those behaviors would break the evidence chain established by ADRs 0004 through 0006.

OriginWeave therefore needs a separate HTTP semantics boundary before a browser adapter can claim bounded navigation or content acquisition.

## Design approaches considered

### Use a general-purpose asynchronous HTTP client

A client such as Hyper or Reqwest provides mature protocol support, but its normal ownership model includes connectors, pools, redirect adapters, proxy configuration, and runtime integration that are broader than this slice. Proving that no alternate connection path or ambient behavior was used would require disabling and auditing a large surface. This remains an option for a later adapter after the narrow authority contract is stable.

### Use an HTTP head parser and implement framing around it

A parser such as `httparse` would reduce start-line and header parsing code, but OriginWeave would still own every resource limit, duplicate-field rule, response-body framing decision, chunk state, trailer policy, timeout, evidence record, and decoder budget. The external parser's accepted syntax would also become part of the product security contract.

### Implement one small strict parser and exchange state machine

The selected approach implements the reviewed HTTP/1.1 subset directly in safe Rust. It accepts only one explicit syntax profile, has no connector or pool, and makes every limit and state transition visible to tests. The cost is more local code, so exact branch coverage, property tests, fuzz-ready pure functions, and real loopback tests are mandatory.

## Goals

1. Add an independently reusable `originweave-http` Rust crate.
2. Consume one existing `AuthenticatedTlsConnection`; never resolve, connect, reconnect, proxy, pool, or control Chromium.
3. Support only HTTP/1.1 `GET` and `HEAD` in the first slice.
4. Require the request origin to equal the TLS evidence origin.
5. Require negotiated ALPN `http/1.1`; permit explicit ALPN absence only through a separately named loopback-test policy value.
6. Generate request framing, `Host`, `Connection: close`, and accepted content codings inside the crate.
7. Reject caller attempts to inject authority, credentials, framing, connection, or hop-by-hop fields.
8. Parse status lines, fields, interim responses, body framing, chunks, and trailers under explicit byte and count limits.
9. Reject every `Transfer-Encoding` plus `Content-Length` combination and every conflicting content length.
10. Enforce one hard monotonic exchange deadline before and after every blocking read or write.
11. Bound encoded content, decoded content, chunk count, trailer fields, and content-coding expansion.
12. Validate RFC 9530 `Content-Digest` and `Repr-Digest` values for `sha-256` and `sha-512` without treating an unsigned digest as authentication.
13. Record supplied MIME, bounded observed MIME, no-sniff state, and mismatch classification without executing content.
14. Parse a bounded `Content-Disposition` filename into safe metadata without creating a file.
15. Return redirects as evidence; never follow them.
16. Emit immutable credential-free HTTP evidence and deterministic standard errors.
17. Preserve exact 100% production function, line, region, and branch coverage and complete public rustdoc.

## Non-goals

- HTTP/2 or HTTP/3
- connection pooling or reuse
- DNS, TCP, TLS, proxy, or PAC authority
- automatic redirects
- cookies, authentication, client certificates, or credential storage
- cache semantics or conditional requests
- state-changing methods
- range assembly
- multipart upload
- WebSocket, WebTransport, or protocol upgrade
- file persistence, archive extraction, document parsing, or active-content rendering
- Chromium Network Service integration
- browser UI

These remain separate authority and product slices.

## Standards baseline

The implementation follows:

- RFC 9110 for HTTP semantics;
- RFC 9112 for HTTP/1.1 message syntax and framing;
- RFC 9530 for `Content-Digest` and `Repr-Digest`;
- RFC 8941 Structured Fields syntax needed by RFC 9530 digest dictionaries;
- the WHATWG MIME Sniffing Living Standard snapshot reviewed on 17 July 2026;
- RFC 6266 for `Content-Disposition` semantics, with a stricter local filename safety policy.

RFC 9530 normatively binds its Structured Fields processing to RFC 8941. RFC 9651 later obsoletes RFC 8941, but its additional Date and Display String bare-item types are not silently imported into this version-bound digest parser. They remain rejected until a separately reviewed protocol-version decision adopts them.

The standards permit behavior that OriginWeave intentionally rejects. Product limits and syntax restrictions are safety policy, not claims that rejected messages are invalid for every HTTP deployment.

## Dependencies

Production dependencies are pinned through `Cargo.lock`:

- `originweave-core` and `originweave-tls` path dependencies;
- `sha2 = 0.10.9` for target and evidence hashes plus SHA-256/SHA-512 digest validation;
- `base64 = 0.22.1` for RFC 8941 byte-sequence decoding;
- `flate2 = 1.1.9`, default features disabled, explicit `rust_backend`, for bounded gzip and zlib-wrapped deflate decoding.

`flate2` uses the portable pure-Rust backend selected explicitly rather than inheriting a mutable default. OriginWeave performs all output and ratio accounting outside the decoder.

Test-only dependencies reuse the local `originweave-destination`, `originweave-network`, `originweave-tls`, `rustls`, and `rcgen` test authority to run real loopback HTTPS exchanges.

## Crate boundary

```text
originweave-http
├── src/lib.rs            public API and crate safety contract
├── src/policy.rs         request, field, body, decoder, and deadline budgets
├── src/target.rs         canonical origin-bound request target
├── src/field.rs          strict field-name/value validation and indexed lookup
├── src/request.rs        deterministic GET/HEAD serialization
├── src/response_head.rs  status line, interim response, and field parsing
├── src/framing.rs        RFC 9110/9112 response body-length decision
├── src/chunked.rs        bounded chunk and trailer state machine
├── src/content.rs        encoded-body collection and bounded content decoding
├── src/integrity.rs      RFC 9530 digest dictionary parsing and validation
├── src/mime.rs           supplied/observed MIME and no-sniff classification
├── src/disposition.rs    bounded filename metadata and hostile-name rejection
├── src/evidence.rs       credential-free HTTP exchange evidence
├── src/exchange.rs       single-use deadline-bound I/O orchestration
└── src/error.rs          deterministic standard error taxonomy
```

The pure parsing modules take byte slices or bounded readers and perform no network access. `exchange.rs` is the only production module allowed to read or write the authenticated stream.

## Public API

### Request method

```rust
pub enum HttpMethod {
    Get,
    Head,
}
```

The enum serializes to uppercase method tokens and exposes whether response semantics suppress content.

### Request target

```rust
pub struct HttpRequestTarget { /* private */ }

impl HttpRequestTarget {
    pub fn parse(origin: Origin, path_and_query: &str) -> Result<Self, HttpError>;
    pub fn origin(&self) -> &Origin;
    pub fn path_and_query(&self) -> &str;
    pub fn target_hash(&self) -> &str;
}
```

The constructor accepts only origin-form targets:

- the first byte is `/`;
- no fragment exists;
- no control, whitespace, backslash, invalid percent escape, or raw non-ASCII byte exists;
- the UTF-8 source is converted to a strictly ASCII HTTP target by preserving valid percent escapes and percent-encoding non-ASCII scalar UTF-8 bytes with uppercase hexadecimal;
- the encoded target is at most 8 KiB;
- evidence retains a domain-separated SHA-256 identifier, a query-present flag, and a bounded path prefix without query values.

The first slice does not normalize dot segments or change percent-encoded octets. The canonical `Origin` remains a separate authority value.

### Caller request fields

```rust
pub struct RequestField { /* private */ }

impl RequestField {
    pub fn new(name: &str, value: &[u8]) -> Result<Self, HttpError>;
}
```

Field names are lowercase ASCII tokens after validation. Field values can contain interior HTAB, SP, visible ASCII, and bytes `0x80..=0xff`; every other control byte is rejected. Leading or trailing optional whitespace is rejected before serialization so the generated field value cannot depend on recipient-side trimming. The caller cannot supply:

```text
host
connection
proxy-connection
keep-alive
transfer-encoding
content-length
trailer
te
upgrade
authorization
proxy-authorization
cookie
```

Duplicate caller fields are rejected. The first slice never serializes a request body.

### Policy

```rust
pub enum AlpnHttp11Policy {
    RequireHttp11,
    PermitAbsentForManagedLoopback,
}

pub enum IntegrityRequirement {
    Optional,
    RequireSupportedDigest,
}

pub struct HttpClientPolicy { /* private */ }

impl HttpClientPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        exchange_timeout: Duration,
        max_request_bytes: usize,
        max_status_line_bytes: usize,
        max_header_field_count: usize,
        max_header_name_bytes: usize,
        max_header_value_bytes: usize,
        max_header_section_bytes: usize,
        max_interim_response_count: usize,
        max_chunk_count: usize,
        max_trailer_field_count: usize,
        max_trailer_section_bytes: usize,
        max_encoded_content_bytes: usize,
        max_decoded_content_bytes: usize,
        max_content_expansion_ratio: usize,
        alpn_policy: AlpnHttp11Policy,
        integrity_requirement: IntegrityRequirement,
    ) -> Result<Self, HttpError>;
}
```

The public defaults are deliberately conservative:

```rust
pub const MAX_HTTP_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(120);
pub const DEFAULT_MAX_REQUEST_BYTES: usize = 16_384;
pub const DEFAULT_MAX_STATUS_LINE_BYTES: usize = 8_192;
pub const DEFAULT_MAX_HEADER_FIELD_COUNT: usize = 128;
pub const DEFAULT_MAX_HEADER_NAME_BYTES: usize = 256;
pub const DEFAULT_MAX_HEADER_VALUE_BYTES: usize = 8_192;
pub const DEFAULT_MAX_HEADER_SECTION_BYTES: usize = 65_536;
pub const DEFAULT_MAX_INTERIM_RESPONSE_COUNT: usize = 8;
pub const DEFAULT_MAX_CHUNK_COUNT: usize = 65_536;
pub const DEFAULT_MAX_TRAILER_FIELD_COUNT: usize = 32;
pub const DEFAULT_MAX_TRAILER_SECTION_BYTES: usize = 16_384;
pub const DEFAULT_MAX_ENCODED_CONTENT_BYTES: usize = 16 * 1024 * 1024;
pub const DEFAULT_MAX_DECODED_CONTENT_BYTES: usize = 32 * 1024 * 1024;
pub const DEFAULT_MAX_CONTENT_EXPANSION_RATIO: usize = 32;
pub const MAX_MIME_SNIFF_BYTES: usize = 1_445;
pub const MAX_SAFE_FILENAME_BYTES: usize = 255;
```

Each configurable count or byte limit is `1..=DEFAULT_*`. A caller can reduce, not expand, the reviewed product maximum. Timeout is `1ns..=120s`. Ratios are `1..=32`.

### Exchange plan

```rust
pub struct HttpExchangePlan { /* private */ }

impl HttpExchangePlan {
    pub fn new(
        connection: AuthenticatedTlsConnection,
        method: HttpMethod,
        target: HttpRequestTarget,
        fields: &[RequestField],
        policy: HttpClientPolicy,
    ) -> Result<Self, HttpError>;

    pub fn execute(self) -> Result<AuthenticatedHttpResponse, HttpError>;
}
```

The plan is non-cloneable and single-use. Before bytes are emitted it verifies:

1. target origin equals TLS evidence origin;
2. requested and observed peers in TLS evidence remain equal;
3. TLS protocol is authenticated;
4. ALPN equals `http/1.1`, or is absent only under the managed-loopback policy and a loopback peer;
5. caller fields are unique and non-authoritative;
6. serialized request bytes fit policy.

### Response

```rust
pub struct AuthenticatedHttpResponse { /* private */ }

impl AuthenticatedHttpResponse {
    pub fn content(&self) -> &[u8];
    pub fn evidence(&self) -> &HttpExchangeEvidence;
    pub fn redirect(&self) -> Option<&RedirectMetadata>;
    pub fn supplied_mime(&self) -> Option<&MimeType>;
    pub fn observed_mime(&self) -> &MimeType;
    pub fn disposition(&self) -> Option<&SafeContentDisposition>;
    pub fn into_parts(self) -> (Vec<u8>, HttpExchangeEvidence);
}
```

A response is observable only after framing is complete, all budgets hold, content decoding completes, required integrity validation succeeds, metadata parsing finishes, and the total deadline remains valid. No partial response type implements this successful API.

## Deterministic request serialization

The crate serializes exactly:

```text
<METHOD> <origin-form target> HTTP/1.1\r\n
Host: <canonical authority>\r\n
Connection: close\r\n
Accept-Encoding: gzip, deflate\r\n
<validated caller fields>\r\n
\r\n
```

The Host value includes a non-default explicit port and brackets IPv6 as already represented by the canonical origin. No request body, `Content-Length`, `Transfer-Encoding`, or trailer is emitted.

## Response head parsing

A private bounded buffer reads until the first `CRLF CRLF`. The parser:

- rejects bare CR, bare LF, NUL, and a section that exceeds the configured maximum;
- requires `HTTP/1.1`, one SP, exactly three decimal status digits, and the mandatory second SP before the optional reason phrase, including when the reason phrase is empty;
- accepts reason bytes for evidence-free diagnostics but never uses them for semantics;
- rejects whitespace before a field name and every line beginning with SP or HTAB;
- validates field names as non-empty RFC token bytes;
- trims optional whitespace from field values and rejects forbidden controls;
- preserves duplicate fields as ordered entries for field-specific validation;
- counts bytes before allocating owned field values;
- handles at most eight non-101 interim responses;
- rejects `101 Switching Protocols` because upgrades are out of scope.

Unknown fields are parsed for framing safety but not automatically exposed. Evidence records their lowercase names and byte counts, not their values. Only a fixed non-sensitive metadata allow-list can retain values.

## Framing decision

`determine_body_framing(method, status, fields, maximum_encoded_bytes)` returns one of:

```rust
pub enum BodyFraming {
    NoContent,
    ContentLength(u64),
    Chunked,
    CloseDelimited,
}
```

Rules are applied in this order:

1. `HEAD`, informational responses, `204`, and `304` have no content regardless of received framing fields; malformed framing fields are still rejected.
2. A successful `CONNECT` would become a tunnel, so it is unreachable with the permitted method set.
3. Any message containing both `Transfer-Encoding` and `Content-Length` is rejected.
4. `Transfer-Encoding` is accepted only when the combined coding list is exactly `chunked`; all other or repeated codings are rejected.
5. Multiple or comma-separated `Content-Length` values are accepted only when every canonical decimal value is identical and fits `u64` and the encoded-content budget.
6. Without either field, the body is close-delimited because the request explicitly asks for connection close.

A close-delimited response is complete only when the TLS reader reaches clean end-of-stream. A rustls unexpected-EOF error remains an incomplete-response failure.

## Chunked state machine

The chunk parser alternates:

```text
size line -> data -> CRLF -> size line ... -> zero size -> trailer section -> complete
```

- size lines are limited to 16 bytes, which is sufficient to represent every admitted `usize` chunk size while sharply bounding syntax overhead;
- only one or more hexadecimal digits followed by CRLF are accepted;
- chunk extensions are rejected in the first slice;
- size arithmetic uses checked conversion and checked cumulative addition;
- every chunk increments the bounded count, including the zero chunk;
- each data block must be followed by CRLF;
- trailers use the same field validator under smaller count and byte budgets;
- `transfer-encoding`, `content-length`, `host`, `connection`, `trailer`, `content-encoding`, and `content-type` are forbidden in trailers;
- the parser returns encoded content bytes and separately indexed trailers.

With the default 16 MiB encoded-content budget, 65,536-chunk limit, 16-byte chunk-size-line budget, and 16 KiB trailer budget, the pre-parse chunked wire bound is 18,104,340 bytes, below 18 MiB. This is a product memory budget, not a protocol-validity claim.

## Deadline-bound I/O

`HttpExchangePlan::execute` captures inherited read and write timeouts. A monotonic deadline is created before request serialization is written. Before and after every blocking operation:

```text
remaining = deadline - now
if remaining == 0: HttpExchangeTimedOut
set relevant socket timeout to remaining
perform one bounded read or write
recheck deadline
```

Because `rustls::StreamOwned` owns the underlying `TcpStream`, the crate updates `stream.sock` timeouts while it exclusively owns the wrapper. Timeout-like I/O errors become `HttpExchangeTimedOut`; other errors retain their source. The stream is consumed even on failure. The implementation attempts to restore inherited socket timeouts on every path; an existing exchange failure remains the primary error, while a restoration failure is reported only when the exchange itself succeeded. The first slice does not return the connection for reuse.

## Content decoding

`Content-Encoding` is parsed as a comma-separated ordered list. The first slice accepts:

- absent or `identity`;
- exactly one `gzip` coding;
- exactly one `deflate` coding using the RFC zlib wrapper.

Multiple codings, raw deflate fallback, Brotli, Zstandard, unknown codings, and repeated identity are rejected.

The encoded body is already bounded. Decoding uses an 8 KiB scratch buffer and checks after every decoder read:

```text
decoded_bytes <= max_decoded_content_bytes
decoded_bytes <= max(encoded_bytes, 1) * max_content_expansion_ratio
```

Checked arithmetic is mandatory. A zero-byte encoded body can produce only a zero-byte decoded body. Decoder errors retain their source without including content.

## Digest validation

The crate parses the RFC 8941 Structured Fields dictionary profile required by RFC 9530:

```text
algorithm-key = :base64-byte-sequence:;optional-parameters
```

Repeated field lines are processed in message order. Header field members are processed before trailer field members, and a later occurrence of the same dictionary key replaces the earlier occurrence. RFC 8941 item parameters are syntax-checked and retained only as extensible metadata with no security meaning for digest verification. Inner lists and non-byte-sequence dictionary values remain outside this digest profile.

RFC 8941 byte-sequence parsing uses the standard Base64 alphabet. Missing `=` padding is accepted by synthesizing/accepting fewer padding bytes as permitted by RFC 8941 recipient behavior. Invalid alphabet characters and impossible Base64 lengths fail parsing. OriginWeave also keeps the base64 crate's strict rejection of non-zero trailing bits as a local fail-closed interoperability restriction. Serialization, when introduced, must emit canonical RFC 4648 padding.

RFC 9530 is version-bound to RFC 8941. RFC 9651-only Date (`@`) and Display String (`%"..."`) parameter bare-item types are intentionally rejected until a separately reviewed protocol-version change adopts them.

Supported active keys are `sha-256` and `sha-512`.

- `Content-Digest` is calculated over the HTTP message content after transfer coding removal and before content-coding decoding.
- `Repr-Digest` is calculated over the selected representation data as defined by RFC 9110. `Content-Encoding` is a characteristic of the representation and is part of `representation-data`, so a content-coded selected representation is digested in its coded form after transfer coding removal, not after content-coding decoding. RFC 9530 Appendix B demonstrates this with a Brotli-coded response representation whose `Repr-Digest` is computed over the Brotli-encoded bytes. The first slice therefore validates `Repr-Digest` over the framed encoded representation bytes for a complete status-`200` response without `Content-Range`; other representation contexts record `UnsupportedContext`.
- If multiple supported algorithm keys remain after ordered dictionary merging, every supported value must match.
- Unsupported-only dictionaries are `UnsupportedAlgorithm` unless policy requires a supported digest, in which case the exchange fails.
- An unsigned digest is integrity evidence against accidental corruption, not authentication or malicious-tamper protection.

Digest values can appear in headers or trailers. They are combined through the ordered dictionary merge above; byte-identical header/trailer duplication is not required. A valid later trailer digest can replace an earlier value, and a mismatching later supported digest fails verification.

## MIME classification

`Content-Type` is parsed into lowercase type/subtype tokens and bounded parameters. Invalid supplied syntax is a typed failure rather than silently replaced.

Observed MIME is determined from at most 1,445 bytes of decoded content using a versioned minimal WHATWG-compatible signature table for:

- HTML;
- XML;
- PDF;
- SVG;
- JavaScript text patterns only when supplied metadata allows text sniffing;
- ZIP;
- PNG;
- JPEG;
- GIF;
- WebP;
- UTF-8/plain text;
- `application/octet-stream` fallback.

The first implementation does not claim the complete WHATWG algorithm. Its type is named `ObservedMimeClassification`, its table version is evidence, and unsupported ambiguity fails to the safe binary fallback. HTML, XML, SVG, JavaScript, and PDF are marked `ActiveOrScriptable` for downstream policy.

`X-Content-Type-Options: nosniff` is parsed case-insensitively. The evidence result distinguishes `Match`, `Mismatch`, `SuppliedOnly`, `ObservedOnly`, and `BinaryFallback`. No-sniff prevents a downstream renderer from replacing a supplied executable type based on observation; this crate only records the condition.

## Content-Disposition safety

The first slice accepts `inline` or `attachment` with at most one `filename` and one RFC 8187-style UTF-8 `filename*`. `filename*` takes precedence only when valid UTF-8 and percent decoding succeed.

A safe filename:

- is 1–255 UTF-8 bytes after NFC-preserving validation;
- contains no ASCII control, DEL, path separator, colon, NUL, bidi override/isolate control, or leading/trailing whitespace;
- is not `.`, `..`, an absolute path, a Windows drive path, a UNC path, or a reserved Windows device basename;
- has no trailing dot or space;
- contains no percent-encoded path separator after decoding;
- is returned as metadata only.

The crate records an extension-versus-MIME mismatch classification but does not rewrite, persist, or execute a file.

## Redirect metadata

For status `300`, `301`, `302`, `303`, `305`, `307`, or `308`, a single valid `Location` value produces:

```rust
pub struct RedirectMetadata {
    location_hash: String,
    target_origin: Option<Origin>,
    is_relative: bool,
}
```

The raw location is not retained in evidence. Parsing supports an absolute HTTPS/managed-loopback HTTP origin or a relative reference sufficient for the caller to resolve under its separate canonical-target policy. Userinfo, control bytes, fragments containing sensitive material, multiple Location fields, and oversized values fail. The crate never follows the target.

## Evidence

`HttpExchangeEvidence` contains:

```text
origin
requested_peer
observed_peer
TLS protocol and ALPN summary
method
target_hash
query_present
HTTP version
status_code
interim_response_count
ordered response field names and byte counts
body_framing
encoded_content_bytes
decoded_content_bytes
content_coding
chunk_count
trailer field names and byte counts
content_digest_status
representation_digest_status
supplied MIME
observed MIME and classifier version
no-sniff state
MIME mismatch class
safe disposition metadata or absence
redirect metadata or absence
response_complete = true
exchange_duration
exchange_timeout
all configured limits
```

No cookie, authorization value, query value, response body, raw Location, certificate body, secret, or unsafe filename is copied into evidence.

## Error contract

`HttpError` distinguishes:

- invalid policy and limit values;
- origin/TLS authority mismatch;
- unexpected ALPN;
- invalid target or caller field;
- request-size overflow;
- write zero, read EOF, timeout, other I/O, and timeout-restoration failure;
- malformed or oversized status line;
- malformed, duplicate, forbidden, excessive, or oversized fields;
- excessive interim responses;
- transfer/content-length ambiguity;
- unsupported transfer coding;
- conflicting or excessive content length;
- malformed or excessive chunks and trailers;
- incomplete response;
- encoded/decoded/expansion budget violation;
- unsupported or invalid content coding;
- malformed, unsupported, required-absent, or mismatched digest;
- invalid MIME or disposition metadata;
- redirect metadata ambiguity.

Public `Display` values are deterministic and exclude message content, field values, URLs with query values, certificates, cookies, authorization data, and filesystem paths. I/O and decoder errors remain available through `source()`.

## Testing strategy

### Pure contract tests

- every policy boundary at `0`, `1`, maximum, and maximum plus one;
- target encoding, invalid percent escapes, fragments, controls, and size boundaries;
- field token/value validation, blocked names, duplicate names, OWS generation rules, and byte accounting;
- status and field parser grammar including bare LF, obs-fold, whitespace-before-name, and obs-text;
- framing matrix for method, status, transfer coding, and content length;
- chunk and trailer state transitions with truncated input at every byte boundary;
- decoder output and ratio boundaries;
- digest dictionary grammar, ordered duplicate-key merging, padded and unpadded RFC 8941 Byte Sequences, and SHA-256/SHA-512 known-answer vectors;
- MIME signatures, no-sniff state, and mismatch classification;
- hostile Unix/Windows/Unicode filenames;
- deterministic error display/source behavior.

### Real loopback HTTPS tests

Use the same managed loopback destination, direct TCP, explicit CA, and rustls server authority as `originweave-tls` tests. Scenarios include:

- `GET` with exact content length;
- `HEAD`, `204`, and `304` with no exposed content;
- valid chunked response and trailers;
- close-delimited response with clean TLS shutdown;
- conflicting lengths and transfer-plus-length rejection;
- malformed status, fields, chunk sizes, missing CRLF, and premature close;
- every byte/count/time budget;
- gzip and zlib-deflate decoding plus expansion bomb;
- digest header and trailer success/mismatch, including later trailer replacement of a duplicate header key;
- MIME agreement and mislabeled active content;
- safe and hostile content disposition;
- redirect evidence with no second connection;
- a hard already-elapsed deadline;
- exact preservation of TLS origin and peer evidence.

### Static governance tests

Repository tests prove that production `originweave-http` contains no:

```text
TcpStream::connect
connect_timeout
ToSocketAddrs
lookup_host
reqwest
hyper connector
proxy environment access
cookie jar
filesystem write
process execution
Chromium/CDP/BiDi/MCP/model call
unsafe block
```

They also require the crate, ADR, design, plan, doctoring references, README/architecture boundary text, changelog entry, exact-head CI, 100% coverage gate, and public rustdoc.

## Documentation and architecture updates

The implementation adds ADR 0007 and updates:

- `AGENTS.md` and `CLAUDE.md` when the governing repository contract changes and the human task authorizes that governance scope;
- `ARCHITECTURE.md` with a sequence diagram from TLS through HTTP evidence and redirect reauthorization;
- `README.md` with supported and explicitly unsupported HTTP behavior;
- `CHANGELOG.md` under `Unreleased`;
- `docs/product-roadmap.md` and `docs/quality-gates.md`;
- `docs/doctoring.md` with APA 7th evidence-to-decision trace;
- `tests/test_repository_contract.py` and `tests/test_http_governance.py`.

## Consequences

### Positive

- One authenticated stream produces one unambiguous, bounded, replayable HTTP result.
- No general HTTP client can silently reconnect, proxy, pool, follow, or retain credentials.
- The same crate can be consumed by OriginWeave, naruon, or another CWL module.
- Framing, integrity, MIME, and metadata decisions become typed evidence rather than log prose.
- Redirects cannot bypass the existing origin, destination, TCP, TLS, capability, or risk boundaries.

### Negative

- HTTP/1.1 only; many modern servers prefer HTTP/2.
- Connection close per request is inefficient but keeps authority and parser state narrow.
- The strict syntax profile rejects some interoperable legacy messages.
- The bounded body is materialized in memory after network streaming; larger downloads require a later bounded sink API.
- The observed MIME table is intentionally smaller than the complete WHATWG algorithm.
- Cookies, authentication, caching, proxying, pooling, and browser integration remain unavailable.

## Commercial proof

A deterministic loopback suite proves that an authenticated TLS stream yields a successful HTTP result only when start lines, fields, framing, content, decoding, integrity, MIME metadata, redirect metadata, and every time/space budget are valid. Every malformed or excessive input fails closed without reconnecting, following, persisting, executing, or leaking protected values.

## References

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP semantics* (RFC 9110). Internet Engineering Task Force. https://doi.org/10.17487/RFC9110

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP/1.1* (RFC 9112; STD 99). Internet Engineering Task Force. https://doi.org/10.17487/RFC9112

Nottingham, M., & Kamp, P.-H. (2021). *Structured field values for HTTP* (RFC 8941). Internet Engineering Task Force. https://doi.org/10.17487/RFC8941

Polli, R., & Pardue, L. (2024). *Digest fields* (RFC 9530). Internet Engineering Task Force. https://doi.org/10.17487/RFC9530

Reschke, J. (2011). *Use of the Content-Disposition header field in the Hypertext Transfer Protocol (HTTP)* (RFC 6266). Internet Engineering Task Force. https://doi.org/10.17487/RFC6266

WHATWG. (2026, July 17). *MIME sniffing: Living Standard*. https://mimesniff.spec.whatwg.org/
