# Bounded HTTP/1.1 Semantics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an independently reusable Rust `originweave-http` crate that performs one strict, resource-bounded HTTP/1.1 `GET` or `HEAD` exchange over an existing authenticated TLS stream and emits credential-free evidence.

**Architecture:** Pure modules validate request targets and fields, parse response heads, determine RFC 9110/9112 framing, decode bounded bodies, validate RFC 9530 digests, and classify MIME/disposition metadata. A single-use `HttpExchangePlan` owns `AuthenticatedTlsConnection`, enforces one monotonic deadline, never reconnects or follows redirects, and returns a response only after all framing, budget, integrity, and metadata checks succeed.

**Tech Stack:** Rust 1.97.1, edition 2024, `originweave-core`, `originweave-tls`, `sha2` 0.10.9, `base64` 0.22.1, `flate2` 1.1.10 with explicit pure-Rust backend, rustls/rcgen loopback tests, Python repository-governance contracts, cargo-llvm-cov 0.8.6 on pinned nightly.

## Global Constraints

- Production arithmetic, parsing, policy, and I/O orchestration are safe Rust; `unsafe` is forbidden.
- The crate opens no socket, performs no DNS, reads no proxy/PAC setting, follows no redirect, writes no file, executes no process, and calls no browser or model API.
- The first slice supports only HTTP/1.1 `GET` and `HEAD` and consumes exactly one `AuthenticatedTlsConnection`.
- All configurable limits can reduce but never exceed the reviewed product maxima in the approved design.
- Every public type, variant, constant, method, and error has rustdoc; rustdoc warnings are denied.
- Production function, line, region, and branch coverage are exactly 100% on the pull-request head.
- All tests are deterministic, run without external network access, and use synthetic loopback certificates and content.
- All database examples contain at least two semantic words and use `snake_case`.
- References and evidence-to-decision trace use APA 7th formatting in `docs/doctoring.md`.
- Commits are small, reviewable, and preserve a failing-test-first record.

---

## File map

```text
crates/originweave-http/Cargo.toml
crates/originweave-http/src/lib.rs
crates/originweave-http/src/error.rs
crates/originweave-http/src/policy.rs
crates/originweave-http/src/target.rs
crates/originweave-http/src/field.rs
crates/originweave-http/src/request.rs
crates/originweave-http/src/response_head.rs
crates/originweave-http/src/framing.rs
crates/originweave-http/src/chunked.rs
crates/originweave-http/src/content.rs
crates/originweave-http/src/integrity.rs
crates/originweave-http/src/mime.rs
crates/originweave-http/src/disposition.rs
crates/originweave-http/src/evidence.rs
crates/originweave-http/src/exchange.rs
crates/originweave-http/tests/policy_contract.rs
crates/originweave-http/tests/request_contract.rs
crates/originweave-http/tests/response_contract.rs
crates/originweave-http/tests/content_contract.rs
crates/originweave-http/tests/metadata_contract.rs
crates/originweave-http/tests/error_contract.rs
crates/originweave-http/tests/exchange_integration.rs
tests/test_http_governance.py
docs/adr/0007-bounded-http11-semantics.md
```

### Task 1: Establish the crate and fail-closed repository contract

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/originweave-http/Cargo.toml`
- Create: `crates/originweave-http/src/lib.rs`
- Create: `crates/originweave-http/src/error.rs`
- Create: `tests/test_http_governance.py`
- Modify: `tests/test_repository_contract.py`

**Interfaces:**
- Consumes: workspace package metadata and strict lint configuration.
- Produces: crate module names, dependency boundary, initial `HttpError`, and repository-level static prohibitions used by every later task.

- [ ] **Step 1: Write failing repository tests**

Add `originweave-http` to the expected workspace set and require the design, plan, ADR, crate modules, README/architecture boundary, and static prohibitions. The production source scan must reject these substrings outside comments and test modules:

```python
FORBIDDEN_HTTP_PRODUCTION_TOKENS = (
    "TcpStream::connect",
    "connect_timeout",
    "ToSocketAddrs",
    "lookup_host",
    "std::fs::write",
    "File::create",
    "reqwest",
    "hyper::client",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "gh pr",
    "COPILOT_GITHUB_TOKEN",
)
```

Run:

```bash
python3 -m unittest tests.test_repository_contract tests.test_http_governance -v
```

Expected: FAIL because the crate, ADR, and workspace member do not exist.

- [ ] **Step 2: Create the crate skeleton**

`crates/originweave-http/Cargo.toml`:

```toml
[package]
name = "originweave-http"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
homepage.workspace = true

[dependencies]
base64 = "=0.22.1"
flate2 = { version = "=1.1.10", default-features = false, features = ["rust_backend"] }
originweave-core = { path = "../originweave-core" }
originweave-tls = { path = "../originweave-tls" }
sha2 = "=0.10.9"

[dev-dependencies]
originweave-destination = { path = "../originweave-destination" }
originweave-network = { path = "../originweave-network" }
rcgen = "=0.14.8"
rustls = { version = "=0.23.42", default-features = false, features = ["ring", "std", "tls12"] }

[lints]
workspace = true
```

`src/lib.rs` must start with:

```rust
//! Perform one bounded HTTP/1.1 exchange over an authenticated OriginWeave TLS stream.
#![forbid(unsafe_code)]
#![deny(missing_docs)]
```

Declare every module from the file map so repository tests can enforce the intended boundary.

- [ ] **Step 3: Add an initial deterministic error type**

Create the first variants required by Tasks 2 and 3:

```rust
#[derive(Debug)]
pub enum HttpError {
    InvalidPolicy,
    InvalidRequestTarget,
    InvalidRequestField,
    ForbiddenRequestField,
    DuplicateRequestField,
    RequestTooLarge { byte_count: usize, maximum_bytes: usize },
}
```

Implement deterministic `Display` and `std::error::Error` with no request target, field value, or content bytes.

- [ ] **Step 4: Run RED-to-GREEN repository tests**

Run:

```bash
python3 -m compileall -q scripts tests
python3 -m unittest discover -s tests -p 'test_*.py'
cargo fmt --all --check
cargo check --locked --workspace --all-targets
```

Expected: repository tests pass; locked Cargo check fails until `Cargo.lock` is regenerated with the pinned dependency graph. Regenerate using the pinned Rust toolchain, review only `originweave-http`, `flate2`, `miniz_oxide`, `crc32fast`, `adler2`, and `simd-adler32` additions, then rerun until PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/originweave-http tests

git commit -m "test: establish bounded HTTP authority contracts"
```

### Task 2: Implement policy, method, and canonical request target

**Files:**
- Modify: `crates/originweave-http/src/error.rs`
- Create: `crates/originweave-http/src/policy.rs`
- Create: `crates/originweave-http/src/target.rs`
- Create: `crates/originweave-http/tests/policy_contract.rs`
- Create: `crates/originweave-http/tests/request_contract.rs`

**Interfaces:**
- Consumes: `originweave_core::Origin`.
- Produces:

```rust
pub enum HttpMethod { Get, Head }
pub enum AlpnHttp11Policy { RequireHttp11, PermitAbsentForManagedLoopback }
pub enum IntegrityRequirement { Optional, RequireSupportedDigest }
pub struct HttpClientPolicy;
pub struct HttpRequestTarget;
```

- [ ] **Step 1: Write boundary tests for every policy field**

For each maximum, test `0`, `1`, exact maximum, and maximum plus one. Assert that a caller can only reduce a reviewed maximum. Include timeout `0ns`, `1ns`, `120s`, and `120s + 1ns`; expansion ratio `0`, `1`, `32`, and `33`.

Run:

```bash
cargo test --locked -p originweave-http --test policy_contract
```

Expected: FAIL because policy types do not exist.

- [ ] **Step 2: Implement `HttpClientPolicy`**

Use private fields and a constructor matching the approved design. Add `HttpClientPolicy::strict_defaults()` returning the exact default constants. Expose read-only accessors and one `pub(crate) fn into_parts(self) -> PolicyParts` for exchange ownership.

Validation must use checked comparisons and return field-specific typed errors:

```rust
HttpError::InvalidExchangeTimeout { timeout, maximum_timeout }
HttpError::InvalidPolicyLimit { limit_name: &'static str, value, maximum }
HttpError::InvalidExpansionRatio { ratio, maximum_ratio }
```

- [ ] **Step 3: Write failing target tests**

Cover:

```text
/
/path
/path?query=value
/non-ascii/한글 -> /non-ascii/%ED%95%9C%EA%B8%80
valid preserved %2f and %2F escapes
fragment rejection
bare percent and non-hex escape rejection
space, tab, CR, LF, NUL, backslash rejection
absolute-form and authority-form rejection
8 KiB encoded boundary and overflow
query-present evidence without query-value accessor
stable domain-separated sha256: target hash
```

- [ ] **Step 4: Implement `HttpRequestTarget` and `HttpMethod`**

Use uppercase percent encoding for UTF-8 bytes and preserve valid existing percent triplets without decoding. Store:

```rust
origin: Origin,
encoded_path_and_query: String,
target_hash: String,
query_present: bool,
path_prefix: String,
```

`path_prefix` ends before `?`, is at most 256 bytes on a UTF-8 boundary, and is the only human-readable target evidence.

- [ ] **Step 5: Run focused tests and commit**

```bash
cargo test --locked -p originweave-http --test policy_contract --test request_contract
cargo clippy --locked -p originweave-http --all-targets -- -D warnings

git add crates/originweave-http

git commit -m "feat: add bounded HTTP policy and request targets"
```

### Task 3: Validate fields and serialize deterministic requests

**Files:**
- Modify: `crates/originweave-http/src/error.rs`
- Create: `crates/originweave-http/src/field.rs`
- Create: `crates/originweave-http/src/request.rs`
- Modify: `crates/originweave-http/tests/request_contract.rs`

**Interfaces:**
- Consumes: `HttpMethod`, `HttpRequestTarget`, `HttpClientPolicy`.
- Produces:

```rust
pub struct RequestField;
pub(crate) struct FieldBlock;
pub(crate) fn serialize_request(
    method: HttpMethod,
    target: &HttpRequestTarget,
    fields: &[RequestField],
    maximum_bytes: usize,
) -> Result<Vec<u8>, HttpError>;
```

- [ ] **Step 1: Write field grammar tests**

Test every RFC token punctuation byte, lowercase normalization, empty name, non-ASCII name, colon, whitespace, CTL, DEL, valid HTAB/SP/VCHAR/obs-text values, CR/LF/NUL rejection, maximum name/value sizes, forbidden names, and case-insensitive duplicates.

- [ ] **Step 2: Implement `RequestField`**

Store lowercase name and opaque value bytes privately. Expose only `name()` publicly; `value()` remains crate-private so general callers cannot accidentally log it through debug output. Implement a custom `Debug` that prints name and byte count only.

- [ ] **Step 3: Write exact serialization tests**

Assert byte-for-byte output for DNS, explicit port, IPv4, and bracketed IPv6 origins. Verify generated ordering:

```text
request line
Host
Connection: close
Accept-Encoding: gzip, deflate
caller fields in input order
empty line
```

Assert no body and no generated content length. Test exact request-size boundary and overflow.

- [ ] **Step 4: Implement serialization**

Derive the Host authority from `Origin::as_str()` after removing the scheme prefix. Build into a pre-sized `Vec<u8>` using checked additions before extension. Reject duplicate caller fields before allocating the final request.

- [ ] **Step 5: Verify and commit**

```bash
cargo test --locked -p originweave-http --test request_contract
cargo fmt --all --check
cargo clippy --locked -p originweave-http --all-targets -- -D warnings

git add crates/originweave-http

git commit -m "feat: serialize authority-bound HTTP requests"
```

### Task 4: Parse response heads and interim responses

**Files:**
- Modify: `crates/originweave-http/src/error.rs`
- Create: `crates/originweave-http/src/response_head.rs`
- Create: `crates/originweave-http/tests/response_contract.rs`

**Interfaces:**
- Produces:

```rust
pub(crate) struct ResponseHead {
    pub(crate) status_code: u16,
    pub(crate) fields: FieldBlock,
}

pub(crate) enum HeadParseResult {
    Incomplete,
    Complete { head: ResponseHead, consumed: usize },
}

pub(crate) fn parse_response_head(
    input: &[u8],
    policy: &HttpClientPolicy,
) -> Result<HeadParseResult, HttpError>;
```

- [ ] **Step 1: Write parser tests before implementation**

Cover all status codes `100..=999`, invalid digit counts, HTTP/1.0, missing spaces, bare CR/LF, NUL, reason controls, empty field name, whitespace before name, obs-fold, duplicate ordered fields, obs-text values, exact byte/count limits, incomplete prefixes, and trailing body bytes with an exact consumed count.

- [ ] **Step 2: Implement a byte-index state machine**

Do not convert the entire head to UTF-8. Locate CRLF pairs, validate each line, and clone only validated name/value ranges after all count and size checks for that line. Return `Incomplete` only while input remains within the maximum; once the maximum cannot permit a terminator, return the relevant size error.

- [ ] **Step 3: Add interim-response logic tests**

A pure helper consumes multiple complete heads from a byte buffer. It accepts up to the policy count of `100..=199` except `101`, then returns the final head and total consumed bytes. Test exact limit and limit plus one.

- [ ] **Step 4: Implement and verify**

```bash
cargo test --locked -p originweave-http --test response_contract
cargo clippy --locked -p originweave-http --all-targets -- -D warnings
```

- [ ] **Step 5: Commit**

```bash
git add crates/originweave-http

git commit -m "feat: parse strict HTTP response heads"
```

### Task 5: Determine framing and parse chunked content

**Files:**
- Modify: `crates/originweave-http/src/error.rs`
- Create: `crates/originweave-http/src/framing.rs`
- Create: `crates/originweave-http/src/chunked.rs`
- Modify: `crates/originweave-http/tests/response_contract.rs`
- Create: `crates/originweave-http/tests/content_contract.rs`

**Interfaces:**
- Produces:

```rust
pub enum BodyFraming { NoContent, ContentLength(u64), Chunked, CloseDelimited }
pub(crate) fn determine_body_framing(
    method: HttpMethod,
    status_code: u16,
    fields: &FieldBlock,
    maximum_encoded_bytes: usize,
) -> Result<BodyFraming, HttpError>;

pub(crate) struct ChunkedResult {
    pub(crate) content: Vec<u8>,
    pub(crate) trailers: FieldBlock,
    pub(crate) chunk_count: usize,
    pub(crate) consumed: usize,
}
```

- [ ] **Step 1: Write the complete framing matrix**

Tests include:

```text
HEAD, 1xx, 204, 304 -> NoContent
TE + CL -> error even for no-content semantics
TE exactly chunked -> Chunked
TE gzip, chunked; chunked, chunked; chunked with invalid list -> error
one CL -> ContentLength
repeated and comma-list identical CL -> same length
conflicting CL -> error
leading sign, whitespace inside digits, overflow -> error
CL above policy -> error
neither -> CloseDelimited
```

- [ ] **Step 2: Implement field-specific indexed lookup and framing**

Parse content lengths with checked decimal arithmetic. Split comma lists only where the field grammar permits. Never call `parse::<usize>()` on unbounded input.

- [ ] **Step 3: Write chunk parser tests**

Use known complete messages and truncate at every byte boundary. Test lowercase/uppercase hex, zero chunk, required CRLF, extension rejection, size overflow, encoded-body overflow, chunk-count overflow, trailer count/bytes, forbidden trailer names, and bytes after the complete trailer terminator.

- [ ] **Step 4: Implement the pure chunk state machine**

Use checked `usize` arithmetic. Allocate output only after checking the next cumulative content size. Return `Incomplete` for a bounded prefix and a typed failure for malformed syntax.

- [ ] **Step 5: Verify and commit**

```bash
cargo test --locked -p originweave-http --test response_contract --test content_contract
cargo fmt --all --check
cargo clippy --locked -p originweave-http --all-targets -- -D warnings

git add crates/originweave-http

git commit -m "feat: enforce HTTP body framing and chunk bounds"
```

### Task 6: Decode bounded content codings

**Files:**
- Modify: `crates/originweave-http/src/error.rs`
- Create: `crates/originweave-http/src/content.rs`
- Modify: `crates/originweave-http/tests/content_contract.rs`

**Interfaces:**
- Produces:

```rust
pub enum ContentCoding { Identity, Gzip, Deflate }
pub(crate) struct DecodedContent {
    pub(crate) bytes: Vec<u8>,
    pub(crate) coding: ContentCoding,
}
pub(crate) fn decode_content(
    encoded: &[u8],
    fields: &FieldBlock,
    policy: &HttpClientPolicy,
) -> Result<DecodedContent, HttpError>;
```

- [ ] **Step 1: Write coding selection tests**

Cover absent, identity, gzip, deflate, case-insensitive tokens, optional whitespace, duplicate identity, multiple codings, unknown coding, empty list member, and oversized field value.

- [ ] **Step 2: Write known-answer decoder tests**

Generate deterministic small gzip and zlib-deflate fixtures in test code. Test empty, text, binary, corrupt checksum/stream, decoded limit exact/overflow, ratio exact/overflow, and encoded-zero semantics.

- [ ] **Step 3: Implement bounded decoder reads**

Use `flate2::read::GzDecoder` and `flate2::read::ZlibDecoder`. Read into `[u8; 8192]`; after every read use checked additions and enforce both limits before extending output. Map decoder errors into `HttpError::ContentDecodingFailed { source }`.

- [ ] **Step 4: Verify and commit**

```bash
cargo test --locked -p originweave-http --test content_contract
cargo clippy --locked -p originweave-http --all-targets -- -D warnings

git add crates/originweave-http Cargo.lock

git commit -m "feat: bound HTTP content decoding"
```

### Task 7: Validate RFC 9530 digest fields

**Files:**
- Modify: `crates/originweave-http/src/error.rs`
- Create: `crates/originweave-http/src/integrity.rs`
- Modify: `crates/originweave-http/tests/content_contract.rs`

**Interfaces:**
- Produces:

```rust
pub enum IntegrityAlgorithm { Sha256, Sha512 }
pub enum IntegrityStatus {
    Absent,
    Verified(Vec<IntegrityAlgorithm>),
    UnsupportedAlgorithm,
    UnsupportedContext,
}
pub(crate) fn validate_content_digest(
    fields: &FieldBlock,
    trailers: &FieldBlock,
    content_bytes: &[u8],
    requirement: IntegrityRequirement,
) -> Result<IntegrityStatus, HttpError>;
```

- [ ] **Step 1: Write RFC 8941 dictionary-subset tests**

Cover one and multiple members, SP around comma, lowercase keys, invalid uppercase key, duplicate key, missing `=`, missing colons, invalid base64, parameters, inner lists, bare strings, empty dictionary, excessive bytes, and header/trailer conflict.

- [ ] **Step 2: Add known-answer digest tests**

Use RFC 9530 SHA-256 and SHA-512 examples plus local empty/text/binary vectors. Assert that every supported member must match and that unsupported-only dictionaries become an error only under `RequireSupportedDigest`.

- [ ] **Step 3: Implement the parser and validator**

Use `base64::engine::general_purpose::STANDARD` with exact padding. Compare digest bytes without formatting them. Error display contains algorithm identifiers but never expected or actual digest bytes.

- [ ] **Step 4: Implement conservative `Repr-Digest` context**

For status `200`, no `Content-Range`, and a full decoded representation, validate against decoded bytes and record `Verified`. All other contexts return `UnsupportedContext` unless the field is malformed, which remains an error.

- [ ] **Step 5: Verify and commit**

```bash
cargo test --locked -p originweave-http --test content_contract
cargo fmt --all --check
cargo clippy --locked -p originweave-http --all-targets -- -D warnings

git add crates/originweave-http

git commit -m "feat: verify HTTP digest fields"
```

### Task 8: Classify MIME, disposition, and redirect metadata

**Files:**
- Modify: `crates/originweave-http/src/error.rs`
- Create: `crates/originweave-http/src/mime.rs`
- Create: `crates/originweave-http/src/disposition.rs`
- Create: `crates/originweave-http/tests/metadata_contract.rs`

**Interfaces:**
- Produces:

```rust
pub struct MimeType;
pub enum ContentRiskClass { Passive, ActiveOrScriptable, ArchiveOrContainer, UnknownBinary }
pub enum MimeMismatch { Match, Mismatch, SuppliedOnly, ObservedOnly, BinaryFallback }
pub struct ObservedMimeClassification;
pub struct SafeContentDisposition;
pub struct RedirectMetadata;
```

- [ ] **Step 1: Write supplied MIME tests**

Cover valid lowercase normalization, parameters, quoted values, duplicate content type, invalid token, CTL, excessive bytes, and nosniff token normalization.

- [ ] **Step 2: Write signature-classification tests**

Fixtures cover HTML with leading whitespace/BOM, XML, SVG, PDF, ZIP, PNG, JPEG, GIF87a/GIF89a, WebP, valid UTF-8 plain text, binary NUL, empty input, and exactly 1,445-byte observation bounds.

- [ ] **Step 3: Implement a versioned conservative classifier**

Expose classifier version `originweave-mime-signatures-1`. Default ambiguity to `application/octet-stream`. Mark HTML/XML/SVG/JavaScript/PDF active or scriptable and ZIP as archive/container.

- [ ] **Step 4: Write and implement disposition tests**

Test inline/attachment, quoted filename, UTF-8 `filename*`, precedence, invalid percent encoding, controls, separators, colon, absolute/drive/UNC paths, dot segments, bidi controls, leading/trailing whitespace, trailing dot/space, reserved Windows device names, exact byte limit, and extension/MIME mismatch.

- [ ] **Step 5: Write and implement redirect metadata tests**

Test supported redirect statuses, one Location, duplicate Location, relative/absolute values, userinfo, control bytes, fragment, size limit, target-origin extraction, stable location hash, and proof that no network function is called.

- [ ] **Step 6: Verify and commit**

```bash
cargo test --locked -p originweave-http --test metadata_contract
cargo clippy --locked -p originweave-http --all-targets -- -D warnings

git add crates/originweave-http

git commit -m "feat: classify HTTP content metadata"
```

### Task 9: Build evidence and the single-use exchange state machine

**Files:**
- Modify: `crates/originweave-http/src/error.rs`
- Create: `crates/originweave-http/src/evidence.rs`
- Create: `crates/originweave-http/src/exchange.rs`
- Modify: `crates/originweave-http/src/lib.rs`
- Create: `crates/originweave-http/tests/error_contract.rs`
- Create: `crates/originweave-http/tests/exchange_integration.rs`

**Interfaces:**
- Produces the approved public API:

```rust
pub struct HttpExchangePlan;
pub struct AuthenticatedHttpResponse;
pub struct HttpExchangeEvidence;
```

- [ ] **Step 1: Write constructor fail-closed tests**

Construct valid TLS evidence, then test wrong target origin, requested/observed peer mismatch adapter, h2 ALPN, absent ALPN under strict policy, absent ALPN on non-loopback under managed-loopback policy, duplicate fields, and oversized serialized request. No test should observe emitted bytes for constructor failures.

- [ ] **Step 2: Implement constructor and ALPN authority**

Consume `AuthenticatedTlsConnection` only after validating target and fields. The managed-loopback exception requires both requested and observed peers to be loopback and ALPN evidence `Absent`.

- [ ] **Step 3: Write deterministic I/O helper tests**

Private helper seams must cover timeout query/update failure, timeout-like read/write error, non-timeout I/O, write zero, read EOF, deadline elapsed before and after I/O, timeout restoration failure, and short writes. Production uses `write_all` semantics implemented as a checked loop so write-zero is typed.

- [ ] **Step 4: Implement exchange orchestration**

Algorithm:

```text
serialize request
capture socket timeouts
create deadline
write all request bytes under remaining write timeout
read bounded head bytes until final head
select framing
read exact/chunked/close-delimited encoded body
restore socket timeouts
bound and decode content
validate digest fields
classify MIME/disposition/redirect metadata
build evidence
return successful response
```

Each read uses a fixed 8 KiB buffer or the exact remaining bounded length, whichever is smaller. `Vec::reserve` receives only policy-bounded checked sizes.

- [ ] **Step 5: Implement complete error contract tests**

Instantiate every public error variant, assert deterministic Display text, verify which variants expose an underlying source, and scan output for fixture content, cookie values, authorization values, query strings, raw digests, and filesystem paths.

- [ ] **Step 6: Implement evidence accessors and redaction tests**

Every field has an accessor. Tests assert exact origin/peer inheritance and scan Debug/Display/evidence JSON-like diagnostic output for prohibited raw values.

- [ ] **Step 7: Verify and commit**

```bash
cargo test --locked -p originweave-http --all-targets
cargo fmt --all --check
cargo clippy --locked -p originweave-http --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p originweave-http --no-deps

git add crates/originweave-http

git commit -m "feat: execute bounded authenticated HTTP exchanges"
```

### Task 10: Prove behavior with real loopback HTTPS adversaries

**Files:**
- Modify: `crates/originweave-http/tests/exchange_integration.rs`

**Interfaces:**
- Consumes: public origin, destination, network, TLS, and HTTP APIs only.
- Produces: buyer-visible proof that no alternate transport or incomplete response can become successful evidence.

- [ ] **Step 1: Build a reusable loopback server fixture**

The fixture:

```rust
struct LoopbackHttpServer {
    socket_address: SocketAddr,
    root_der: Vec<u8>,
    connection_count: Arc<AtomicUsize>,
    thread: JoinHandle<()>,
}
```

It accepts exactly one TLS connection, reads the complete HTTP request, records request bytes through a bounded channel, writes caller-supplied response fragments with optional delays, optionally sends close_notify, and exits. It never uses production parsing code to construct adversarial responses.

- [ ] **Step 2: Add successful exchange scenarios**

Cover content length, HEAD, 204, 304, chunked trailers, clean close-delimited content, gzip, deflate, content/repr digest, MIME match, safe disposition, and redirect metadata. Assert `connection_count == 1` and exact request bytes.

- [ ] **Step 3: Add framing and syntax adversaries**

Cover TE+CL, conflicting CL, unsupported TE, obs-fold, whitespace before name, bare LF, invalid chunk, missing CRLF, forbidden trailer, duplicate Location, and 101 upgrade.

- [ ] **Step 4: Add resource and timing adversaries**

Cross every configured limit by one unit, send an expansion bomb, delay before first byte and between fragments, and force an already elapsed deadline. Assert typed failures and no successful response/evidence.

- [ ] **Step 5: Add integrity and metadata adversaries**

Cover digest mismatch/malformed/unsupported, invalid content type, mislabeled active content with nosniff, hostile filenames, extension spoofing, and an invalid redirect target.

- [ ] **Step 6: Verify and commit**

```bash
cargo test --locked -p originweave-http --test exchange_integration -- --nocapture
cargo test --locked --workspace --all-targets

git add crates/originweave-http/tests/exchange_integration.rs

git commit -m "test: prove HTTP authority on real TLS streams"
```

### Task 11: Finish ADR, product documentation, and exact gates

**Files:**
- Create: `docs/adr/0007-bounded-http11-semantics.md`
- Modify: `README.md`
- Modify: `ARCHITECTURE.md`
- Modify: `AGENTS.md`
- Modify: `CLAUDE.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/README.md`
- Modify: `docs/doctoring.md`
- Modify: `docs/product-roadmap.md`
- Modify: `docs/quality-gates.md`
- Modify: `tests/test_repository_contract.py`
- Modify: `tests/test_http_governance.py`

**Interfaces:**
- Consumes: final code and measured behavior.
- Produces: binding architecture decision, APA 7th trace, operational limitations, and merge evidence.

- [ ] **Step 1: Write ADR 0007**

Include context, decision, alternatives, exact limits, public authority chain, framing decision table, timeout semantics, digest limits, MIME/disposition caveats, positive/negative consequences, non-goals, and a Mermaid sequence diagram:

```text
Origin -> destination -> TCP peer -> TLS identity -> HTTP plan
HTTP plan -> request write -> response framing -> bounded content
bounded content -> digest -> MIME/disposition -> immutable evidence
redirect evidence -> new Origin authorization chain
```

- [ ] **Step 2: Update binding documents**

State explicitly that:

- HTTP success is narrower than safe rendering or browser navigation;
- redirects are not followed;
- content is not executed or persisted;
- proxy, cookie, authentication, caching, download, HTTP/2/3, and Chromium remain separate;
- the crate is reusable by naruon and other CWL services.

- [ ] **Step 3: Add APA 7th doctoring**

Record the exact design decisions derived from RFC 9110, RFC 9112, RFC 8941, RFC 9530, RFC 6266, the reviewed WHATWG MIME snapshot, and flate2 1.1.10 primary repository documentation.

- [ ] **Step 4: Run the complete verification matrix**

```bash
python3 -m compileall -q scripts tests
python3 -m unittest discover -s tests -p 'test_*.py'
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
cargo +nightly-2026-08-01 llvm-cov --locked --workspace --all-features --branch --json --output-path coverage.json
python3 scripts/ci/verify_coverage.py coverage.json
```

Expected: every command passes and production function, line, region, and branch coverage are each exactly 100%.

- [ ] **Step 5: Verify dependency and source boundary**

```bash
cargo tree --locked -p originweave-http
rg -n 'TcpStream::connect|connect_timeout|ToSocketAddrs|lookup_host|HTTP_PROXY|HTTPS_PROXY|ALL_PROXY|reqwest|hyper::client|File::create|std::fs::write|unsafe' crates/originweave-http/src
```

Expected: only the reviewed dependency graph; the forbidden-source scan returns no matches.

- [ ] **Step 6: Commit**

```bash
git add AGENTS.md ARCHITECTURE.md CHANGELOG.md CLAUDE.md README.md docs tests

git commit -m "docs: bind HTTP semantics to authenticated streams"
```

## Plan self-review

- Every approved design requirement maps to a task: authority and limits (Tasks 1–3), strict syntax and framing (Tasks 4–5), decoding (Task 6), integrity (Task 7), MIME/disposition/redirect metadata (Task 8), deadline-bound exchange and evidence (Task 9), realistic network proof (Task 10), and documentation/release gates (Task 11).
- No task permits a second socket, DNS, proxy, redirect follow, pool, file write, browser call, or model call.
- The type names and signatures consumed by later tasks exactly match the interfaces produced by earlier tasks.
- The plan contains no placeholder, deferred implementation instruction, or unspecified test category.
