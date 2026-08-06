# TLS Server Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans task-by-task. Follow test-driven development and verification-before-completion.

**Goal:** Authenticate one canonical HTTPS origin over an already verified `DirectTcpConnection` without DNS, reconnect, proxy inheritance, verifier bypass, or ambiguous evidence.

**Architecture:** Add `originweave-tls`, a synchronous Rust crate that builds a fixed-time rustls 0.23.42 client with explicit roots and TLS 1.2/1.3, performs a monotonic deadline-bound handshake over the existing TCP stream, validates ALPN and bounded peer certificates, and exposes the authenticated stream only with credential-free evidence.

**Production dependencies:** Rust 1.97.1; rustls 0.23.42 with `default-features = false` and `ring,std,tls12`; sha2 0.10.9; x509-parser 0.18.1 without default features; path dependencies on core and network.  
**Test dependencies:** rcgen 0.14.8 and rustls server APIs.

## Global merge constraints

- No production `ClientConfig::dangerous`, custom verifier, key-log file, early data, secret extraction, session resumption, client authentication, hostname connection API, DNS, proxy environment, HTTP, QUIC, or Chromium integration.
- Production functions, lines, regions, and branches exactly 100% covered.
- Every public item documented.
- Current-head CI, Security Scan, Semgrep, review threads, and standards doctoring pass before merge.

## Task 1 — RED repository and static-governance contracts

**Files:** modify `tests/test_repository_contract.py`; create `tests/test_tls_governance.py`.

1. Require workspace member `crates/originweave-tls` and paths for ADR 0006, design, and plan.
2. Require README and architecture terms `originweave-tls`, `TLS service identity`, and separation from TCP peer proof.
3. Add static production-source assertions requiring `builder_with_details`, fixed `TimeProvider`, explicit `TLS13` and `TLS12`, root certificates, disabled resumption/early-data/secret extraction, `NoKeyLog`, peer revalidation, ALPN, and certificate bounds.
4. Forbid `dangerous()`, `set_certificate_verifier`, `TcpStream::connect`, `ToSocketAddrs`, `std::env`, proxy strings, `KeyLogFile`, and client-certificate configuration.
5. Run Python tests and observe failure before crate implementation.

## Task 2 — RED public policy contracts

**Files:** add crate manifest and lib shell; create `tests/policy_contract.rs` initially.

Write failing tests for:

- trust bundle identifier and root count/byte bounds;
- malformed root rejection;
- zero/excessive timeout;
- ALPN empty value, duplicate, count, value-length, and total-byte bounds;
- HTTP origin rejection;
- DNS, IPv4, and IPv6 reference identity extraction;
- direct connection peer-evidence consistency;
- one-shot plan ownership.

Run focused tests and preserve RED evidence.

## Task 3 — Implement trust, identity, and policy

**Files:** `src/trust.rs`, `src/identity.rs`, `src/policy.rs`, `src/error.rs`, `src/lib.rs`.

- canonical root sorting/deduplication and domain-separated bundle hash;
- rustls `RootCertStore` construction;
- bounded ASCII `TrustBundleIdentifier`;
- HTTPS canonical origin parsing to `TlsReferenceIdentity` and owned rustls `ServerName`;
- bounded ALPN and timeout policy;
- explicit `RevocationStatus::NotConfigured`;
- deterministic public errors and sources.

Run focused tests to GREEN and commit.

## Task 4 — RED real TLS integration

**Files:** initially create integration tests for a loopback rustls server.

Use rcgen to create:

- a test CA;
- valid `localhost` DNS SAN certificate;
- wrong-name certificate with CN `localhost` but different SAN;
- IPv4 SAN certificate;
- expired and not-yet-valid certificates;
- trusted and untrusted roots;
- ALPN-present and ALPN-absent servers.

Each client obtains a real managed-loopback `DirectTcpConnection` before TLS. Observe RED because authentication is not implemented.

## Task 5 — Implement fixed-time bounded handshake

**Files:** `src/handshake.rs` and `src/evidence.rs`.

- fixed `TimeProvider`;
- ring provider and only TLS 1.3/TLS 1.2;
- explicit roots and no client authentication;
- resumption disabled, early data disabled, secret extraction disabled, no key log, no certificate compression;
- DNS-only SNI, IP reference identity without SNI;
- remaining-deadline socket timeouts before every `write_tls` and `read_tls`;
- `process_new_packets` after each read;
- no-progress and elapsed-deadline rejection;
- peer equality before, during, and after handshake;
- clear socket timeouts before exposing the authenticated stream;
- typed rustls certificate-error classification.

Run integration tests to GREEN and commit.

## Task 6 — Certificate and protocol evidence

**Files:** `src/evidence.rs` and tests.

- require TLS 1.2 or TLS 1.3 and a negotiated suite;
- enforce ALPN policy;
- enforce 1–16 presented certificates and at most 1 MiB DER;
- parse complete leaf DER with x509-parser;
- SHA-256 of leaf, leaf SPKI, and ordered presented certificates;
- leaf validity timestamps;
- trust-bundle identifier/hash/count/bytes;
- fixed verification time, revocation status, handshake duration/deadline;
- exact inherited/requested/observed peer evidence;
- no certificate bytes, subject values, serials, keys, or plaintext in evidence.

Add deterministic malformed/excessive evidence tests and reach GREEN.

## Task 7 — Consolidate coverage and documentation

If cargo-llvm-cov instruments separate library copies for integration binaries, consolidate real and deterministic tests inside one `#[cfg(test)]` source module while retaining the public rustdoc compile-fail replay test. Do not weaken the exact 100% gate.

Create ADR 0006 and update README, architecture, roadmap, quality gates, docs index, CHANGELOG, and APA 7th doctoring with RFC 5280, RFC 8446, RFC 9525, rustls 0.23.42, rustls-pki-types, x509-parser, and rcgen test-only evidence. Clearly label `peer_certificates` as the server-presented chain accepted by verification, not the exact internally constructed path.

## Task 8 — Exact-head verification, review, and merge

Run:

```bash
python3 -m compileall -q scripts tests
python3 -m unittest discover -s tests -p 'test_*.py'
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
cargo +nightly-2026-08-01 llvm-cov --locked --workspace --all-features --branch --json --summary-only --output-path coverage.json
python3 scripts/ci/verify_coverage.py coverage.json
```

Open a non-draft PR titled `feat: authenticate verified TCP peers with TLS`, closing issue #6. Process every review thread and failed check on the exact head. Squash-merge only after all current-head gates pass; then re-query the PR and issue queues and continue to the next buyer-visible HTTP resource-budget gap.
