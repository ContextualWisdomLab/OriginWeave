# Direct Socket Binding Design

**Status:** Approved by issue #4 and the autonomous Phase 1 product loop  
**Date:** 2026-08-06  
**Scope:** Pure-Rust policy-bound direct TCP connection from an approved `ResolutionSnapshot`

## Problem

`originweave-destination` can classify and pin a DNS answer, but it does not open a socket. A later adapter could accidentally resolve the hostname again, inherit proxy or PAC behavior, connect to another address, or expose a stream before the operating system's observed peer is checked. That gap prevents OriginWeave from proving that an approved destination became the exact transport peer.

## Goals

1. Add an independently reusable `originweave-network` crate.
2. Accept only an explicit canonical `SocketAddr`; never accept a hostname or `ToSocketAddrs` input.
3. Require the socket IP to be authorized by an existing `ResolutionSnapshot`.
4. Reject port zero, zero or excessive connect timeouts, and zero or excessive attempt counts.
5. Represent authority as a non-cloneable `ConnectionPlan` consumed by one connection attempt sequence.
6. Call `TcpStream::connect_timeout` with the exact approved `SocketAddr`.
7. Verify `TcpStream::peer_addr` equals the requested IP and port before exposing the stream.
8. Emit credential-free typed evidence with the origin, requested and observed socket addresses, destination class, successful attempt number, and timeout.
9. Preserve underlying `std::io::Error` values through `std::error::Error::source`.
10. Remain direct-only and independent of proxy environment variables, TLS, HTTP, Chromium, and LLM components.

## Non-goals

- DNS lookup, caching, or refresh
- proxy or PAC evaluation
- TLS and certificate validation
- HTTP parsing or response limits
- QUIC or UDP
- Chromium, WebDriver BiDi, CDP, or WebMCP integration
- connection pooling
- asynchronous runtime integration

These remain separate merge-gated adapters.

## Crate boundary

`originweave-network` depends only on `originweave-core`, `originweave-destination`, and the Rust standard library.

- `connection.rs` owns validation, the single-use connection plan, exact socket connection, peer verification, evidence, and errors.
- `lib.rs` exposes the minimal public API and denies missing documentation and unsafe code.
- integration tests exercise real loopback TCP behavior and policy boundaries.
- internal tests inject a private connector implementation only for operating-system outcomes that cannot be reproduced deterministically, such as a peer-address inspection failure or a successful connection whose reported peer differs from the requested socket.

## Public API

### Bounds

```rust
pub const MAX_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
pub const MAX_CONNECTION_ATTEMPTS: u8 = 4;
```

The plan accepts `1ns..=30s` and `1..=4` attempts. These bounds cap one direct call and the total number of calls. A future elapsed-time budget may be layered above this crate without changing its direct-only authority model.

### Connection plan

```rust
pub struct ConnectionPlan { /* private */ }

impl ConnectionPlan {
    pub fn new(
        resolution: &ResolutionSnapshot,
        socket_address: SocketAddr,
        connect_timeout: Duration,
        maximum_attempts: u8,
    ) -> Result<Self, NetworkError>;

    pub fn connect(self) -> Result<DirectTcpConnection, NetworkError>;
}
```

`ConnectionPlan` does not implement `Clone` or `Copy`. `connect` consumes it, making replay a compile-time error. The constructor calls `ResolutionSnapshot::authorize_connection` and rejects any non-canonical IP representation. In particular, an IPv4-mapped IPv6 socket is rejected even when its canonical IPv4 address is approved.

### Established connection

```rust
pub struct DirectTcpConnection { /* private */ }

impl DirectTcpConnection {
    pub fn stream(&self) -> &TcpStream;
    pub fn evidence(&self) -> &SocketConnectionEvidence;
    pub fn into_parts(self) -> (TcpStream, SocketConnectionEvidence);
}
```

The stream becomes observable only after `peer_addr` succeeds and exactly equals the requested socket address.

### Evidence

```rust
pub struct SocketConnectionEvidence { /* private */ }
```

Evidence includes:

- canonical logical `Origin`;
- requested `SocketAddr`;
- observed peer `SocketAddr`;
- `AddressClass`;
- one-based successful attempt number;
- per-attempt connect timeout.

It contains no hostname credentials, URL path, query, fragment, cookie, header, body, proxy credential, or secret.

## Validation order

`ConnectionPlan::new` fails before any I/O in this order:

1. port is nonzero;
2. timeout is nonzero and no greater than `MAX_CONNECT_TIMEOUT`;
3. attempt count is within `1..=MAX_CONNECTION_ATTEMPTS`;
4. the IP is approved by the supplied snapshot;
5. the supplied IP is already canonical.

This produces stable, testable remediation and avoids leaking network state for malformed authority.

## Connection lifecycle

For each one-based attempt:

1. call `TcpStream::connect_timeout(&requested_socket, connect_timeout)`;
2. on a timeout error, retry until the configured bound, then return `ConnectionTimedOut` with the final `io::Error` source;
3. on another connect error, retry until the configured bound, then return `ConnectionFailed` with the final `io::Error` source;
4. on success, call `peer_addr`;
5. if peer inspection fails, return `PeerInspectionFailed` with the `io::Error` source;
6. if the observed peer differs in either IP or port, drop the stream and return `PeerMismatch`;
7. otherwise construct evidence and expose `DirectTcpConnection`.

A verification failure is not retried because the established stream failed the authority check rather than the transport-open operation.

## Error model

`NetworkError` distinguishes:

- `InvalidPort`;
- `InvalidConnectTimeout`;
- `InvalidAttemptCount`;
- `DestinationNotApproved`, retaining `DestinationError` as its source;
- `NonCanonicalSocketAddress`;
- `ConnectionTimedOut`, retaining the final `io::Error`;
- `ConnectionFailed`, retaining the final `io::Error`;
- `PeerInspectionFailed`, retaining the `io::Error`;
- `PeerMismatch`.

Messages are deterministic and credential-free. `DestinationError` receives `Display` and `std::error::Error` implementations so it can participate in the network error chain.

## Testing

### Real integration tests

- bind an ephemeral loopback `TcpListener`;
- approve loopback through an explicit managed destination policy;
- connect with a direct plan;
- prove requested and observed peer addresses match exactly;
- prove evidence fields match the real connection;
- prove a public-only destination policy rejects loopback and IPv4-mapped loopback before I/O;
- prove a dropped ephemeral listener yields a bounded connection-failure result.

### Deterministic internal tests

A private connector trait simulates:

- timeout on every attempt;
- ordinary failure followed by success;
- peer-address inspection failure;
- mismatched observed peer;
- exact attempt-number evidence.

The public production path always uses the standard-library system connector.

### Static governance tests

Python repository contracts verify that production source:

- uses `TcpStream::connect_timeout` with `SocketAddr`;
- does not use `ToSocketAddrs`;
- does not call `TcpStream::connect`;
- does not read proxy environment variables;
- documents direct-only routing and the separation from TLS, HTTP, proxy/PAC, and Chromium.

A compile-fail doctest proves a consumed `ConnectionPlan` cannot be reused.

## Quality gates

- Rust 1.97.1 formatting, locked check, all tests, strict Clippy, and rustdoc warnings denied;
- production function, line, region, and branch coverage exactly 100%;
- public Rust API documentation exactly complete;
- Python governance contracts pass;
- Security Scan and Semgrep pass on the exact reviewed head;
- README, architecture, roadmap, ADR, CHANGELOG, documentation index, and APA 7th doctoring match executable behavior.

## Standards basis

RFC 9293 defines a TCP connection by its local and remote socket pair and consolidates the current Standards Track TCP specification. Rust 1.97.1 documents that `TcpStream::connect_timeout` takes one `SocketAddr`, applies the timeout to that individual address, and uses an operating-system-specific nonblocking connection mechanism; `peer_addr` returns the remote peer socket address. These properties make the API appropriate for a no-reresolution, exact-peer adapter.

## Follow-on slices

1. TLS server-name, certificate, and ALPN validation bound to the same origin and peer evidence;
2. proxy/PAC authority with separately approved proxy and final target evidence;
3. HTTP request, header, body, redirect, and elapsed-time budgets;
4. MIME and download persistence policy;
5. Chromium/BiDi/CDP integration and complete transport provenance.
