# Resolved Destination Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a reusable, fail-closed Rust kernel that classifies resolved destinations, pins approved DNS answers, detects rebinding, authorizes concrete connection addresses, and re-evaluates every redirect hop.

**Architecture:** `originweave-destination` is a pure standard-library Rust crate depending only on `originweave-core`. Browser adapters provide already-resolved addresses and complete-target digests; the crate returns typed decisions and credential-free evidence without performing DNS, socket, proxy, or response-body I/O.

**Tech Stack:** Rust 1.97.1, Rust standard library networking types, Cargo workspace, cargo-llvm-cov 0.8.6, Python `unittest` repository contracts, GitHub Actions.

## Global Constraints

- Production arithmetic and policy logic remain Rust-only.
- `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` apply to the new crate.
- No new third-party dependency is permitted.
- Default web policy permits only explicitly classified public destinations.
- IPv4-mapped IPv6 values are canonicalized before policy and pin comparison.
- Every redirect rechecks target-origin authority, target resolution, scheme downgrade, exact target cycle, and hop count.
- Production function, line, region, and branch coverage must each equal exactly 100%.
- Every public production item must have useful rustdoc.
- Database naming remains two-or-more-word `snake_case`; this slice creates no persistent object.
- Documentation references use APA 7th style.
- The implementation must remain independently reusable and importable by OriginWeave, naruon, or another CWL service.

---

### Task 1: Address classification and canonicalization

**Files:**
- Create: `crates/originweave-destination/Cargo.toml`
- Create: `crates/originweave-destination/src/lib.rs`
- Create: `crates/originweave-destination/src/address.rs`
- Test: `crates/originweave-destination/tests/address_policy.rs`

**Interfaces:**
- Consumes: `std::net::{IpAddr, Ipv4Addr, Ipv6Addr}`.
- Produces: `AddressClass`, `ClassifiedAddress`, and `classify_address(IpAddr) -> ClassifiedAddress`.

- [ ] **Step 1: Write failing address-taxonomy tests**

Create table-driven tests for public, unspecified, loopback, private/unique-local, shared, link-local, metadata, documentation, benchmarking, multicast, broadcast, transition, and protocol-reserved IPv4/IPv6 ranges. Include `::ffff:127.0.0.1` and `::ffff:8.8.4.4` canonicalization cases.

- [ ] **Step 2: Run the focused test before implementation**

Run:

```bash
cargo +1.97.1 test --locked -p originweave-destination --test address_policy
```

Expected: FAIL because the crate and exported address contracts do not yet exist.

- [ ] **Step 3: Implement the minimal pure classifier**

Create an `AddressClass` enum, preserve original and canonical addresses in `ClassifiedAddress`, reduce IPv4-mapped IPv6 to canonical IPv4, and classify special-purpose blocks before the public fallback. Specific metadata endpoints must be matched before their containing generic range.

- [ ] **Step 4: Re-run focused address tests**

Run:

```bash
cargo +1.97.1 test --locked -p originweave-destination --test address_policy
```

Expected: PASS for every table row and canonicalization assertion.

- [ ] **Step 5: Commit the independently reviewable address kernel**

```bash
git add crates/originweave-destination/Cargo.toml \
  crates/originweave-destination/src/lib.rs \
  crates/originweave-destination/src/address.rs \
  crates/originweave-destination/tests/address_policy.rs
git commit -m "feat: classify resolved destination addresses"
```

### Task 2: Resolution approval, pinning, and rebinding

**Files:**
- Create: `crates/originweave-destination/src/resolution.rs`
- Modify: `crates/originweave-destination/src/lib.rs`
- Test: `crates/originweave-destination/tests/resolution_policy.rs`

**Interfaces:**
- Consumes: `Origin`, `AddressClass`, `ClassifiedAddress`, and `classify_address`.
- Produces: `DestinationPolicy`, `DestinationError`, `ResolutionSnapshot`, and `ConnectionEvidence`.

- [ ] **Step 1: Write failing policy and pinning tests**

Cover the default public-only policy, explicit managed class grants, deny-all policy, empty DNS answers, denied addresses, mapped-address deduplication, concrete connection authorization, unapproved connection denial, subset revalidation, new-address expansion, and denied/empty refreshed answers.

- [ ] **Step 2: Run the focused resolution test before implementation**

Run:

```bash
cargo +1.97.1 test --locked -p originweave-destination --test resolution_policy
```

Expected: FAIL because resolution-policy types are not defined.

- [ ] **Step 3: Implement deterministic resolution authority**

`DestinationPolicy::public_web()` must allow only `AddressClass::Public`. `ResolutionSnapshot::approve` must reject an empty answer, classify every address, canonicalize mapped values, deduplicate the canonical set, and bind it to one `Origin`. `authorize_connection` must require membership in that pinned set. `revalidate` must accept a non-empty subset and reject every newly introduced address as a possible rebinding event.

- [ ] **Step 4: Re-run focused resolution tests**

Run:

```bash
cargo +1.97.1 test --locked -p originweave-destination --test resolution_policy
```

Expected: PASS with typed failure equality assertions.

- [ ] **Step 5: Commit the resolution boundary**

```bash
git add crates/originweave-destination/src/lib.rs \
  crates/originweave-destination/src/resolution.rs \
  crates/originweave-destination/tests/resolution_policy.rs
git commit -m "feat: pin approved DNS resolution snapshots"
```

### Task 3: Redirect reauthorization and evidence

**Files:**
- Create: `crates/originweave-destination/src/redirect.rs`
- Modify: `crates/originweave-destination/src/lib.rs`
- Test: `crates/originweave-destination/tests/redirect_policy.rs`

**Interfaces:**
- Consumes: `Origin`, `ResolutionSnapshot`, and an explicit `BTreeSet<Origin>` read grant.
- Produces: `RedirectTargetDigest`, `RedirectGuard`, `RedirectError`, `RedirectEvidence`, and `MAX_REDIRECT_HOPS`.

- [ ] **Step 1: Write failing redirect tests**

Cover strict lowercase SHA-256 target digests; invalid zero and over-limit hop settings; state accessors; authorized cross-origin and same-origin hops; absent origin grants; resolution-origin mismatch; HTTPS-to-HTTP downgrade; complete-target cycles; exhausted hop budget; and an explicitly managed HTTP loopback redirect.

- [ ] **Step 2: Run the focused redirect test before implementation**

Run:

```bash
cargo +1.97.1 test --locked -p originweave-destination --test redirect_policy
```

Expected: FAIL because redirect contracts are absent.

- [ ] **Step 3: Implement per-hop fail-closed authorization**

Track the current origin, configured maximum, consumed hops, and complete-target digests. For every redirect, check hop capacity, read-origin authority, resolution ownership, secure-scheme downgrade, and cycle state before mutating the guard. Emit evidence only after all checks pass.

- [ ] **Step 4: Re-run focused redirect tests**

Run:

```bash
cargo +1.97.1 test --locked -p originweave-destination --test redirect_policy
```

Expected: PASS for each success and typed denial path.

- [ ] **Step 5: Commit the redirect boundary**

```bash
git add crates/originweave-destination/src/lib.rs \
  crates/originweave-destination/src/redirect.rs \
  crates/originweave-destination/tests/redirect_policy.rs
git commit -m "feat: reauthorize every redirect target"
```

### Task 4: Workspace and governance integration

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `tests/test_repository_contract.py`
- Create: `docs/adr/0004-resolved-destination-policy.md`
- Modify: `README.md`
- Modify: `ARCHITECTURE.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/product-roadmap.md`
- Modify: `docs/doctoring.md`
- Modify: `docs/README.md`

**Interfaces:**
- Consumes: the completed `originweave-destination` crate.
- Produces: a discoverable workspace member, binding ADR, current roadmap state, and APA 7th evidence trail.

- [ ] **Step 1: Make repository contracts fail for the absent integration**

Add `crates/originweave-destination` to the exact workspace-member expectation, require ADR 0004, and assert that the README and architecture explicitly distinguish logical origin identity from resolved destination safety.

- [ ] **Step 2: Run repository contracts before integration**

Run:

```bash
python3 -m unittest tests.test_repository_contract
```

Expected: FAIL until the workspace and required documents are synchronized.

- [ ] **Step 3: Integrate the crate and documentation**

Add the workspace member and lockfile package. Record the accepted architecture decision, current IANA registry evidence, RFC 6890/RFC 9110 implications, crate boundaries, implemented Phase 1 foundation, residual proxy/download/Chromium-adapter work, and Unreleased changes.

- [ ] **Step 4: Re-run repository contracts**

Run:

```bash
python3 -m unittest tests.test_repository_contract
```

Expected: PASS with the five-crate workspace and ADR 0004 present.

- [ ] **Step 5: Commit workspace and governance integration**

```bash
git add Cargo.toml Cargo.lock tests/test_repository_contract.py \
  README.md ARCHITECTURE.md CHANGELOG.md docs/README.md \
  docs/product-roadmap.md docs/doctoring.md \
  docs/adr/0004-resolved-destination-policy.md
git commit -m "docs: integrate resolved destination policy"
```

### Task 5: Exact verification, review, and merge

**Files:**
- Verify: all files changed by Tasks 1–4.
- Artifact: GitHub Actions coverage JSON and missing-line report.

**Interfaces:**
- Consumes: complete branch state.
- Produces: a reviewable PR with exact-head evidence and no unresolved threads.

- [ ] **Step 1: Run complete local-equivalent gates**

```bash
python3 -m compileall -q scripts tests
python3 -m unittest discover -s tests -p 'test_*.py'
cargo +1.97.1 fmt --all --check
cargo +1.97.1 check --locked --workspace --all-targets
cargo +1.97.1 test --locked --workspace --all-targets
cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo +1.97.1 doc --locked --workspace --no-deps
cargo +nightly-2026-08-01 llvm-cov --locked --workspace --all-features --branch --json --summary-only --output-path coverage.json
python3 scripts/ci/verify_coverage.py coverage.json
```

Expected: every command succeeds and production function, line, region, and branch coverage each report 100%.

- [ ] **Step 2: Open one bounded PR linked to issue #2**

The PR body must state the buyer-visible SSRF/rebinding gap, executable contracts, test evidence, standards, residual proxy/download/adapter risk, and `Closes #2`.

- [ ] **Step 3: Inspect every current-head review and check**

Confirm CI, exact coverage, Security Scan, Semgrep, and CodeRabbit/current organization reviewer results. Treat queued checks as active work, not proof of success. Address each actionable review thread in the branch and re-run exact-head checks after every mutation.

- [ ] **Step 4: Merge with head-SHA protection**

After all required checks pass, all threads are resolved, and the PR remains mergeable, perform a squash merge using the exact reviewed head SHA.

- [ ] **Step 5: Verify closure and queue state**

Confirm issue #2 is closed by the merge, no OriginWeave PR remains open, and the hourly OpenCode product-development workflow remains present on `main` for the next buyer-visible gap.
