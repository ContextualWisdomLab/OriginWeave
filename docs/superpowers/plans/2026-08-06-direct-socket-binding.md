# Direct Socket Binding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bind an approved `ResolutionSnapshot` to the exact operating-system TCP peer through a reusable direct-only Rust crate.

**Architecture:** Add `originweave-network`, a standard-library-only synchronous transport boundary that accepts one canonical `SocketAddr`, validates it through `originweave-destination`, consumes a non-cloneable connection plan, calls `TcpStream::connect_timeout`, verifies `peer_addr`, and exposes the stream only with credential-free evidence. Deterministic private connector tests cover rare operating-system outcomes while a real loopback integration test proves the end-to-end socket path.

**Tech Stack:** Rust 1.97.1, edition 2024, `std::net::{TcpListener, TcpStream, SocketAddr}`, Python 3 repository contracts, cargo-llvm-cov 0.8.6.

## Global Constraints

- Production code must forbid `unsafe_code` and deny missing public documentation.
- `originweave-network` may depend only on `originweave-core`, `originweave-destination`, and the Rust standard library.
- Production code must not use `ToSocketAddrs`, hostname strings, `TcpStream::connect`, proxy environment variables, TLS, HTTP, Chromium, or an async runtime.
- `MAX_CONNECT_TIMEOUT` is exactly 30 seconds.
- `MAX_CONNECTION_ATTEMPTS` is exactly 4.
- Every new production function, line, region, and branch must reach exact 100% coverage.
- Every public Rust item must have beginner-readable rustdoc.
- Database naming rules remain two or more semantic words in `snake_case`; this slice creates no database object.
- Documentation references use APA 7th style and distinguish implemented behavior from follow-on work.

---

### Task 1: Establish repository and documentation contracts

**Files:**
- Modify: `tests/test_repository_contract.py`
- Create: `tests/test_network_governance.py`
- Test: `tests/test_repository_contract.py`
- Test: `tests/test_network_governance.py`

**Interfaces:**
- Consumes: existing root workspace and documentation layout.
- Produces: failing contracts requiring the new crate, ADR, design, plan, direct-only source boundary, and documentation claims.

- [ ] **Step 1: Add the new crate and document paths to the repository contract**

Update the workspace member expectation to include:

```python
"crates/originweave-network",
```

Add these required paths:

```python
"docs/adr/0005-direct-socket-binding.md",
"docs/registry-maintenance.md",
"docs/superpowers/specs/2026-08-06-direct-socket-binding-design.md",
"docs/superpowers/plans/2026-08-06-direct-socket-binding.md",
```

Add documentation assertions:

```python
self.assertIn("originweave-network", readme)
self.assertIn("originweave-network", architecture)
self.assertIn("exact operating-system peer", readme)
self.assertIn("direct-only", architecture)
```

- [ ] **Step 2: Create static network-governance tests**

```python
"""Static contracts for the direct-only TCP authority boundary."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class NetworkGovernanceTests(unittest.TestCase):
    """Prevent DNS, proxy, and protocol scope from entering the socket kernel."""

    def test_network_source_uses_one_explicit_socket_address(self) -> None:
        source = (
            ROOT / "crates/originweave-network/src/connection.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("TcpStream::connect_timeout", source)
        self.assertIn("SocketAddr", source)
        self.assertNotIn("ToSocketAddrs", source)
        self.assertNotIn("TcpStream::connect(", source)

    def test_network_source_does_not_inherit_proxy_environment(self) -> None:
        source = (
            ROOT / "crates/originweave-network/src/connection.rs"
        ).read_text(encoding="utf-8")
        self.assertNotIn("std::env", source)
        self.assertNotIn("HTTP_PROXY", source)
        self.assertNotIn("HTTPS_PROXY", source)
        self.assertNotIn("ALL_PROXY", source)

    def test_docs_keep_transport_scope_explicit(self) -> None:
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        adr = (ROOT / "docs/adr/0005-direct-socket-binding.md").read_text(
            encoding="utf-8"
        )
        for text in (architecture, adr):
            self.assertIn("direct-only", text)
            self.assertIn("TLS", text)
            self.assertIn("proxy", text.lower())
            self.assertIn("Chromium", text)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 3: Run Python contracts and verify RED**

Run:

```bash
python3 -m unittest tests.test_repository_contract tests.test_network_governance
```

Expected: FAIL because the workspace member, network crate, ADR, and binding documentation do not exist.

- [ ] **Step 4: Commit the RED contracts**

```bash
git add tests/test_repository_contract.py tests/test_network_governance.py
git commit -m "test: require direct socket authority boundary"
```

### Task 2: Write failing Rust policy-boundary tests

**Files:**
- Create: `crates/originweave-network/Cargo.toml`
- Create: `crates/originweave-network/src/lib.rs`
- Create: `crates/originweave-network/tests/policy_boundary.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

**Interfaces:**
- Consumes: `originweave_core::Origin`, `originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot}`.
- Produces: wished-for `ConnectionPlan`, `NetworkError`, `MAX_CONNECT_TIMEOUT`, and `MAX_CONNECTION_ATTEMPTS` API.

- [ ] **Step 1: Add only the crate manifest and empty public module shell**

`crates/originweave-network/Cargo.toml`:

```toml
[package]
name = "originweave-network"
description = "Direct-only policy-bound TCP connection authority for OriginWeave."
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
homepage.workspace = true
publish = false

[dependencies]
originweave-core = { path = "../originweave-core" }
originweave-destination = { path = "../originweave-destination" }

[lints]
workspace = true
```

`src/lib.rs`:

```rust
//! Direct-only policy-bound TCP connection authority for OriginWeave.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connection;

pub use connection::{
    ConnectionPlan, DirectTcpConnection, MAX_CONNECTION_ATTEMPTS, MAX_CONNECT_TIMEOUT,
    NetworkError, SocketConnectionEvidence,
};
```

Add `crates/originweave-network` to the workspace and add a lock entry depending on core and destination.

- [ ] **Step 2: Write policy-boundary tests before `connection.rs` exists**

```rust
#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_network::{
    ConnectionPlan, MAX_CONNECTION_ATTEMPTS, MAX_CONNECT_TIMEOUT, NetworkError,
};

fn loopback_snapshot() -> ResolutionSnapshot {
    ResolutionSnapshot::approve(
        Origin::parse("http://localhost").expect("loopback origin"),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("managed loopback resolution")
}

#[test]
fn plan_rejects_port_zero_before_network_io() {
    let error = ConnectionPlan::new(
        &loopback_snapshot(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        Duration::from_secs(1),
        1,
    )
    .expect_err("port zero must fail");
    assert!(matches!(error, NetworkError::InvalidPort));
}

#[test]
fn plan_rejects_zero_and_excessive_timeouts() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80);
    for timeout in [Duration::ZERO, MAX_CONNECT_TIMEOUT + Duration::from_nanos(1)] {
        assert!(matches!(
            ConnectionPlan::new(&loopback_snapshot(), socket, timeout, 1),
            Err(NetworkError::InvalidConnectTimeout { .. })
        ));
    }
}

#[test]
fn plan_rejects_zero_and_excessive_attempt_counts() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80);
    for attempts in [0, MAX_CONNECTION_ATTEMPTS + 1] {
        assert!(matches!(
            ConnectionPlan::new(
                &loopback_snapshot(),
                socket,
                Duration::from_secs(1),
                attempts,
            ),
            Err(NetworkError::InvalidAttemptCount { .. })
        ));
    }
}

#[test]
fn public_policy_rejects_loopback_before_plan_creation() {
    let origin = Origin::parse("https://example.com").expect("public origin");
    let resolution = ResolutionSnapshot::approve(
        origin,
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::public_web(),
    );
    assert!(resolution.is_err());
}

#[test]
fn mapped_loopback_is_not_accepted_as_a_canonical_socket() {
    let mapped = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 1);
    let error = ConnectionPlan::new(
        &loopback_snapshot(),
        SocketAddr::new(IpAddr::V6(mapped), 80),
        Duration::from_secs(1),
        1,
    )
    .expect_err("mapped form must be rejected");
    assert!(matches!(
        error,
        NetworkError::NonCanonicalSocketAddress { .. }
    ));
}
```

- [ ] **Step 3: Run focused Rust tests and verify RED**

Run:

```bash
cargo test --locked -p originweave-network --test policy_boundary
```

Expected: FAIL because `connection.rs` and its public API are missing.

- [ ] **Step 4: Commit the RED crate shell and tests**

```bash
git add Cargo.toml Cargo.lock crates/originweave-network
git commit -m "test: specify direct connection policy bounds"
```

### Task 3: Implement plan validation minimally

**Files:**
- Create: `crates/originweave-network/src/connection.rs`
- Modify: `crates/originweave-destination/src/resolution.rs`
- Test: `crates/originweave-network/tests/policy_boundary.rs`

**Interfaces:**
- Consumes: `ResolutionSnapshot::authorize_connection(IpAddr)` and `ConnectionEvidence` accessors.
- Produces: validated `ConnectionPlan` with private origin, requested socket, address class, timeout, and attempt bound.

- [ ] **Step 1: Implement `Display` and `Error` for `DestinationError`**

Add deterministic formatting for every existing variant and:

```rust
impl std::error::Error for DestinationError {}
```

- [ ] **Step 2: Implement constants, validation, and errors without socket I/O**

Define:

```rust
pub const MAX_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
pub const MAX_CONNECTION_ATTEMPTS: u8 = 4;
```

`ConnectionPlan::new` must validate in the design order, call `authorize_connection`, reject a non-canonical requested IP, and retain only credential-free values.

- [ ] **Step 3: Run the policy tests and verify GREEN**

Run:

```bash
cargo test --locked -p originweave-network --test policy_boundary
```

Expected: PASS.

- [ ] **Step 4: Run destination tests for regression safety**

Run:

```bash
cargo test --locked -p originweave-destination
```

Expected: PASS.

- [ ] **Step 5: Commit plan validation**

```bash
git add crates/originweave-destination/src/resolution.rs crates/originweave-network/src/connection.rs
git commit -m "feat: validate single-use direct connection plans"
```

### Task 4: Write the real loopback integration test

**Files:**
- Create: `crates/originweave-network/tests/direct_connection.rs`

**Interfaces:**
- Consumes: `ConnectionPlan::new` and the wished-for `connect`, `stream`, `evidence`, and `into_parts` methods.
- Produces: end-to-end proof that the exact approved socket is the observed operating-system peer.

- [ ] **Step 1: Write the failing loopback test**

```rust
#![allow(clippy::expect_used)]

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::thread;
use std::time::Duration;

use originweave_core::Origin;
use originweave_destination::{AddressClass, DestinationPolicy, ResolutionSnapshot};
use originweave_network::ConnectionPlan;

#[test]
fn approved_loopback_socket_becomes_the_exact_peer() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind listener");
    let socket = listener.local_addr().expect("listener address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept connection");
        stream.write_all(b"ok").expect("write response");
    });

    let origin = Origin::parse("http://localhost").expect("loopback origin");
    let snapshot = ResolutionSnapshot::approve(
        origin.clone(),
        [IpAddr::V4(Ipv4Addr::LOCALHOST)],
        &DestinationPolicy::from_allowed_classes([AddressClass::Loopback]),
    )
    .expect("approve loopback");
    let connection = ConnectionPlan::new(&snapshot, socket, Duration::from_secs(1), 1)
        .expect("build plan")
        .connect()
        .expect("connect exact peer");

    assert_eq!(connection.stream().peer_addr().expect("peer"), socket);
    assert_eq!(connection.evidence().origin(), &origin);
    assert_eq!(connection.evidence().requested_socket(), socket);
    assert_eq!(connection.evidence().observed_peer(), socket);
    assert_eq!(connection.evidence().address_class(), AddressClass::Loopback);
    assert_eq!(connection.evidence().attempt_number(), 1);
    assert_eq!(connection.evidence().connect_timeout(), Duration::from_secs(1));

    let (mut stream, evidence) = connection.into_parts();
    let mut body = [0_u8; 2];
    stream.read_exact(&mut body).expect("read response");
    assert_eq!(&body, b"ok");
    assert_eq!(evidence.observed_peer(), socket);
    server.join().expect("server thread");
}
```

- [ ] **Step 2: Run the integration test and verify RED**

Run:

```bash
cargo test --locked -p originweave-network --test direct_connection
```

Expected: FAIL because direct connection and evidence methods are missing.

- [ ] **Step 3: Commit the RED integration test**

```bash
git add crates/originweave-network/tests/direct_connection.rs
git commit -m "test: prove exact loopback peer binding"
```

### Task 5: Implement exact connection and evidence

**Files:**
- Modify: `crates/originweave-network/src/connection.rs`
- Test: `crates/originweave-network/tests/direct_connection.rs`

**Interfaces:**
- Produces: `DirectTcpConnection`, `SocketConnectionEvidence`, and `ConnectionPlan::connect(self)`.

- [ ] **Step 1: Add the private connector abstraction**

```rust
trait SocketConnector {
    type Stream;

    fn connect_timeout(
        &self,
        socket_address: &SocketAddr,
        timeout: Duration,
    ) -> io::Result<Self::Stream>;

    fn peer_addr(&self, stream: &Self::Stream) -> io::Result<SocketAddr>;
}
```

`SystemConnector` must call only `TcpStream::connect_timeout` and `TcpStream::peer_addr`.

- [ ] **Step 2: Implement retry, exact peer verification, evidence, and stream exposure**

The private generic helper returns `(Stream, SocketConnectionEvidence)`. The public method wraps the system stream in `DirectTcpConnection` only after exact equality.

- [ ] **Step 3: Run the real integration test and verify GREEN**

Run:

```bash
cargo test --locked -p originweave-network --test direct_connection
```

Expected: PASS.

- [ ] **Step 4: Commit the real connection path**

```bash
git add crates/originweave-network/src/connection.rs
git commit -m "feat: bind approved sockets to observed TCP peers"
```

### Task 6: Cover deterministic failures and standard error chains

**Files:**
- Modify: `crates/originweave-network/src/connection.rs`
- Create: `crates/originweave-network/tests/error_contract.rs`

**Interfaces:**
- Produces: complete `Display`, `Error::source`, retry, timeout, peer-inspection, peer-mismatch, and attempt evidence behavior.

- [ ] **Step 1: Add private fake-connector tests inside `connection.rs`**

Create deterministic cases for:

```text
timeout on all configured attempts
ordinary failure on all configured attempts
ordinary failure followed by successful exact peer
peer_addr returning an I/O error
peer_addr returning a different socket
```

Each test must assert the exact number of calls and the one-based attempt number.

- [ ] **Step 2: Add public error-contract tests**

The integration test must match every public variant, call `to_string`, and verify `source()` for destination and I/O variants. It must also prove non-source validation errors return `None`.

- [ ] **Step 3: Add a compile-fail doctest for single use**

Document `ConnectionPlan::connect` with a `compile_fail` example that tries to call `connect` twice on the same value and fails because the first call moved the plan.

- [ ] **Step 4: Run focused and full crate tests**

Run:

```bash
cargo test --locked -p originweave-network --all-targets
cargo test --locked --doc -p originweave-network
```

Expected: PASS.

- [ ] **Step 5: Commit error and replay contracts**

```bash
git add crates/originweave-network/src/connection.rs crates/originweave-network/tests/error_contract.rs
git commit -m "test: enforce bounded network error contracts"
```

### Task 7: Synchronize architecture, ADR, roadmap, doctoring, and changelog

**Files:**
- Create: `docs/adr/0005-direct-socket-binding.md`
- Modify: `README.md`
- Modify: `ARCHITECTURE.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/README.md`
- Modify: `docs/product-roadmap.md`
- Modify: `docs/doctoring.md`

**Interfaces:**
- Consumes: executable API and test evidence from Tasks 3–6.
- Produces: buyer-readable scope, trust-boundary diagrams, release notes, and APA 7th traceability.

- [ ] **Step 1: Write ADR 0005**

Record:

```text
Context: destination authorization did not bind the OS peer.
Decision: direct-only SocketAddr + connect_timeout + exact peer_addr verification.
Consequences: no hostname resolution, proxy/PAC, TLS, HTTP, or Chromium integration in this crate.
Rejected: hostname-based TcpStream::connect, environment proxy inheritance, exposing streams before peer verification.
```

Include a Mermaid sequence diagram from `ResolutionSnapshot` through `ConnectionPlan`, OS connect, peer verification, and evidence.

- [ ] **Step 2: Update product documents**

README and architecture must state that `originweave-network` proves the exact operating-system peer but does not yet prove TLS identity or Chromium socket use. The roadmap must mark direct socket binding complete and make TLS identity the next bounded slice.

- [ ] **Step 3: Update APA 7th doctoring**

Add:

```text
Eddy, W. (Ed.). (2022). Transmission Control Protocol (TCP) (RFC 9293). Internet Engineering Task Force. https://doi.org/10.17487/RFC9293

The Rust Project Developers. (2026). TcpStream in std::net (Rust 1.97.1) [Software documentation]. https://doc.rust-lang.org/stable/std/net/struct.TcpStream.html
```

Explain that RFC 9293 defines the connection by local and remote sockets and Rust's single-`SocketAddr` timeout API prevents adapter-level re-resolution.

- [ ] **Step 4: Update CHANGELOG**

Under `Unreleased`, describe the direct-only crate, exact peer verification, single-use plan, bounded attempts and timeout, standard error chains, real loopback proof, and remaining TLS/proxy scope.

- [ ] **Step 5: Run Python contracts and verify GREEN**

Run:

```bash
python3 -m compileall -q scripts tests
python3 -m unittest discover -s tests -p 'test_*.py'
```

Expected: PASS.

- [ ] **Step 6: Commit documentation and governance**

```bash
git add README.md ARCHITECTURE.md CHANGELOG.md docs tests/test_repository_contract.py tests/test_network_governance.py
git commit -m "docs: govern direct socket binding boundary"
```

### Task 8: Exact-head verification and PR publication

**Files:**
- No product changes unless a verification failure receives a failing regression test first.

**Interfaces:**
- Produces: one reviewable PR closing issue #4.

- [ ] **Step 1: Run all deterministic quality gates**

```bash
python3 -m compileall -q scripts tests
python3 -m unittest discover -s tests -p 'test_*.py'
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
```

Expected: every command succeeds without warnings.

- [ ] **Step 2: Run exact production coverage**

```bash
cargo +nightly-2026-08-01 llvm-cov \
  --locked \
  --workspace \
  --all-features \
  --branch \
  --json \
  --summary-only \
  --output-path coverage.json
python3 scripts/ci/verify_coverage.py coverage.json
```

Expected: function, line, region, and branch coverage are each exactly 100%.

- [ ] **Step 3: Self-review the diff**

Check for:

```text
hostname or ToSocketAddrs input
TcpStream::connect use
std::env or proxy strings
stream exposure before peer verification
unbounded timeout or attempts
Clone/Copy on ConnectionPlan
credential-bearing evidence
undocumented public items
mutable or non-APA source references
```

Expected: none present.

- [ ] **Step 4: Open a non-draft PR**

Title:

```text
feat: bind approved destinations to TCP peers
```

Body must summarize buyer-visible risk, API, real loopback proof, coverage gates, standards, out-of-scope adapters, and `Closes #4`.

- [ ] **Step 5: Process review and CI in a bounded loop**

For the exact current head:

```text
review threads → failing checks → regression test → minimal fix → full verification → resolved thread
```

Merge only when current-head CI, Security Scan, Semgrep, coverage, rustdoc, and review requirements pass. Re-query the open PR list after merge.
