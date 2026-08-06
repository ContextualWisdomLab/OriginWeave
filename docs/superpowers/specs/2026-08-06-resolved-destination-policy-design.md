# Resolved Destination Policy Design

**Status:** Approved by the Phase 1 roadmap and issue #2  
**Date:** 2026-08-06  
**Scope:** Pure Rust destination classification, resolution pinning, rebinding detection, connection authorization, and redirect reauthorization

## Problem

A normalized web origin identifies the logical security principal, but it does not prove that a DNS answer is safe to connect to. A public-looking hostname can resolve to loopback, private, link-local, shared, metadata, or other special-purpose address space. The answer can change after approval, and a safe first request can redirect to a prohibited destination.

OriginWeave cannot claim safe real navigation until the browser-network adapter has a deterministic policy object that evaluates every resolved address and every redirect hop before connection.

## Goals

1. Classify IPv4 and IPv6 destinations using the current IANA special-purpose registries.
2. Canonicalize IPv4-mapped IPv6 values before policy and pin comparison.
3. Deny every non-public address class by default.
4. Bind a non-empty approved resolution set to one canonical `Origin`.
5. Authorize only a concrete address that appeared in the pinned set.
6. Treat any newly introduced DNS address as a possible rebinding event.
7. Re-evaluate origin grant, resolution ownership, scheme downgrade, exact target cycle, and hop limit for every redirect.
8. Emit credential-free typed evidence suitable for a later audit adapter.
9. Remain usable independently, inside OriginWeave, or as a naruon/CWL module.

## Non-goals

- DNS lookup or caching
- socket creation
- certificate validation
- proxy or PAC execution
- response body, MIME, or download limits
- Chromium, WebDriver BiDi, or CDP integration
- organization policy persistence

These remain separate Phase 1 adapters and kernels.

## Crate boundary

`originweave-destination` depends only on `originweave-core` and the Rust standard library. It performs no I/O. The crate is split by responsibility:

- `address.rs`: canonical address classification
- `resolution.rs`: policy, approved snapshots, rebinding checks, and connection evidence
- `redirect.rs`: complete-target digests, hop state, redirect decisions, and redirect evidence

## Address model

`ClassifiedAddress` preserves both the resolver-supplied address and the canonical comparison address. IPv4-mapped IPv6 values become canonical IPv4 addresses so `::ffff:127.0.0.1` cannot bypass a loopback rule or a pin set containing `127.0.0.1`.

The policy taxonomy is:

- `Public`
- `Unspecified`
- `Loopback`
- `PrivateNetwork`
- `SharedNetwork`
- `LinkLocal`
- `MetadataService`
- `Documentation`
- `Benchmarking`
- `Multicast`
- `Broadcast`
- `Transition`
- `ProtocolReserved`

The default `DestinationPolicy` permits only `Public`. Managed deployments can construct an explicit class allow-list, but there is no implicit local-network exception.

Specific metadata endpoints are classified before their containing link-local, shared, or unique-local blocks so audit evidence states the highest-risk interpretation. The initial set includes `169.254.169.254`, `169.254.170.2`, `100.100.100.200`, and `fd00:ec2::254`.

IPv4-compatible, IPv4-mapped, NAT64, Teredo, and 6to4 forms are not treated as ordinary public IPv6. Mapped values are canonicalized to IPv4; the remaining transition forms are denied by the public-web policy.

## Resolution lifecycle

`ResolutionSnapshot::approve` accepts one `Origin`, an iterator of resolved `IpAddr` values, and a `DestinationPolicy`.

It:

1. rejects an empty answer;
2. classifies every address;
3. rejects the first class not explicitly permitted;
4. canonicalizes IPv4-mapped values;
5. deduplicates the canonical addresses;
6. binds the non-empty set to the origin.

`authorize_connection` reclassifies the concrete candidate immediately before connection and requires its canonical form to appear in the pinned set. The returned `ConnectionEvidence` contains the origin, supplied address, canonical address, and class, but no hostname credentials, headers, cookies, or body data.

`revalidate` applies the policy again to a fresh DNS answer. A non-empty subset of the original set is accepted. Any address outside the original pinned set is rejected as `ResolutionSetExpanded`. This gives adapters a deterministic DNS-rebinding boundary while tolerating normal answer contraction.

## Redirect lifecycle

A redirect target is represented by:

- its separately parsed canonical `Origin`;
- a lowercase `sha256:` digest of the complete canonical target URI;
- a separately approved `ResolutionSnapshot`;
- the caller's explicit read-origin grant.

The digest allows path- and query-sensitive cycle detection without retaining a potentially sensitive URI in policy state or evidence.

`RedirectGuard` validates a maximum of `1..=20` hops. For each hop it checks, in order:

1. the configured hop limit;
2. explicit target-origin authority;
3. equality between target origin and resolution origin;
4. HTTPS-to-HTTP downgrade prohibition;
5. exact complete-target digest cycle detection.

Only then does it update the current origin and emit `RedirectEvidence`. Same-origin redirects remain valid when their complete target digest changes. Cross-origin redirects require a separate read-origin grant and resolution approval.

## Error model

Failures are typed enums and contain only the minimum values needed for deterministic remediation:

- empty resolution;
- denied address and class;
- unapproved connection address;
- expanded DNS set;
- invalid redirect maximum;
- exceeded redirect limit;
- missing origin grant;
- resolution-origin mismatch;
- insecure scheme downgrade;
- repeated target digest.

The crate does not log or recover automatically. The browser-network adapter decides whether to stop, request managed authorization, refresh policy, or hand control to a person.

## Testing and release gates

Tests cover every IPv4 and IPv6 class, exact range boundaries, IPv4-mapped canonicalization, default and managed policy, empty and denied resolution answers, deduplication, connection evidence, subset revalidation, rebinding expansion, strict digest parsing, redirect grants, mismatched resolutions, downgrade, same-origin redirect, cycles, and hop exhaustion.

Merge gates remain:

- Rust 1.97.1 formatting, locked check, tests, strict Clippy, and rustdoc;
- production function, line, region, and branch coverage exactly 100%;
- public rustdoc exactly complete;
- Security Scan and Semgrep success;
- ADR, architecture, roadmap, CHANGELOG, README, and APA 7th doctoring synchronized with executable behavior.

## Follow-on slices

This design intentionally leaves the following independently reviewable work for later PRs:

1. proxy and PAC destination enforcement;
2. connection, header, body, redirect, and elapsed-time budgets;
3. download MIME, observed-content, and persistence policy;
4. Chromium/BiDi resolver and socket adapter;
5. connection and redirect provenance serialization.
