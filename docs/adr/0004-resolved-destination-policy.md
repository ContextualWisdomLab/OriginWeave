# ADR 0004: Separate Logical Origin from Resolved Destination Safety

- **Status:** Accepted
- **Date:** 2026-08-06
- **Decision owners:** Contextual Wisdom Lab
- **Related issue:** #2

## Context

`originweave-core::Origin` establishes a browser-equivalent logical origin. It rejects credentials, paths, insecure remote HTTP, malformed authorities, and browser-special numeric host spellings. That contract prevents origin-identity confusion, but it cannot establish that a DNS result is safe to connect to.

A syntactically public hostname can resolve to a loopback, private, link-local, shared, metadata, documentation, benchmarking, multicast, unspecified, transition, or protocol-reserved destination. A resolver can return a safe set during policy evaluation and a different set before connection. A public first request can also redirect to a prohibited origin or address.

The IANA IPv4 and IPv6 special-purpose registries are the authoritative classification inputs. Authoritative cloud-platform documentation supplements those registries for credential and control endpoints that occupy otherwise public, shared, link-local, or unique-local space. RFC 9110 treats a redirect `Location` as a new target URI; automatic redirect handling must therefore re-evaluate authority and representation-specific state instead of inheriting the initial request's approval.

## Decision

OriginWeave will keep logical origin grants and resolved destination grants as separate typed authorities.

A new independently reusable `originweave-destination` crate will own pure Rust policy with no DNS, socket, proxy, or response-body I/O.

### Address classification

The crate classifies canonical addresses into:

- public;
- unspecified;
- loopback;
- private or unique-local;
- shared network;
- link-local;
- metadata service;
- documentation;
- benchmarking;
- multicast;
- broadcast;
- transition;
- protocol reserved.

IPv4-mapped IPv6 values are converted to canonical IPv4 before classification and comparison. The default web policy permits only the public class. Managed deployments may construct an explicit class allow-list, but no implicit local-network exception exists.

Specific metadata and platform endpoints are classified before their containing generic range so denial evidence retains the highest-risk interpretation. The initial reviewed supplement includes established instance and container metadata endpoints, Azure platform IP `168.63.129.16`, and Amazon EKS Pod Identity endpoints `169.254.170.23` and `fd00:ec2::23`.

### Resolution approval and pinning

A resolution snapshot is valid only when:

1. at least one address is present;
2. no more than 256 resolver addresses are supplied;
3. every address is permitted by the supplied destination policy;
4. every address is canonicalized;
5. the canonical address set is non-empty;
6. the set is bound to exactly one logical origin.

The resolver-answer bound is applied before the complete iterator is collected, so an adversarial or malfunctioning resolver cannot force unbounded policy-state allocation.

`localhost` can approve only loopback destinations. A literal IPv4 or IPv6 origin can approve only the exact canonical address represented in the origin. A concrete connection attempt is authorized only when its canonical address appears in the pinned snapshot.

A refreshed DNS result may contract to a non-empty subset of the pinned set. It may not introduce a new canonical address. Expansion is treated as a possible DNS-rebinding event and fails closed. The same 256-address bound applies to refreshes.

### Redirect authorization

Every redirect target requires:

- an explicit read-origin grant;
- a resolution snapshot bound to the target origin;
- no HTTPS-to-HTTP downgrade;
- a new lowercase SHA-256 digest of the complete canonical target URI;
- remaining redirect-hop capacity.

Complete target digests allow cycle detection without retaining a potentially sensitive full URI in policy state or audit evidence. Same-origin redirects remain separately evaluated because path and query changes can alter the complete action target.

The kernel accepts at most 20 redirects per chain. Adapters may configure a lower positive limit.

### Evidence

Connection evidence records the logical origin, supplied address, canonical address, and destination class. Redirect evidence records hop number, source origin, target origin, complete-target digest, and approved-address count.

Neither evidence type stores cookies, authorization values, request headers, query values, response bodies, or secret material.

## Consequences

### Positive

- A canonical origin can no longer be mistaken for an SSRF decision.
- IPv4-mapped IPv6 cannot bypass IPv4 policy or pin comparisons.
- DNS rebinding becomes a deterministic typed denial rather than an adapter convention.
- Cloud credential and platform endpoints can be denied even when their enclosing address range would otherwise appear public or managed.
- Resolver response cardinality is bounded before policy-state allocation.
- Redirects cannot inherit ambient origin or destination authority.
- Desktop, headless, MCP, BiDi, CDP, naruon, and enterprise adapters can reuse the same policy kernel.
- Security decisions remain deterministic and independently testable without network access.

### Negative

- Browser adapters must preserve and present the exact resolution snapshot used for connection.
- Managed local-network access requires explicit policy construction.
- IANA registry and cloud-platform endpoint changes require reviewed classifier updates and regression tests.
- The finite platform-endpoint supplement cannot replace egress policy, proxy enforcement, or tenant-specific deny rules.
- This decision alone does not secure proxy/PAC routing, TLS, response bodies, MIME validation, downloads, or socket races.

## Rejected alternatives

### Rely on `IpAddr` convenience predicates alone

Rejected because stable Rust predicates do not provide the complete, versioned security taxonomy needed here. In particular, IPv6 global classification and transition handling require explicit canonicalization and registry-bound policy.

### Permit every address accepted by the operating-system resolver

Rejected because resolver success is not authorization and would permit SSRF into local or special-purpose networks.

### Collect an unbounded resolver answer

Rejected because a malicious or malfunctioning adapter could consume unbounded memory before policy evaluation. A 256-address input cap is generous for browser resolution while preserving deterministic resource use.

### Approve only the hostname and resolve again immediately before connect

Rejected because it creates a DNS-rebinding gap between policy evaluation and use.

### Trust the initial request through redirects

Rejected because every redirect creates a new target URI and can cross logical origins and network boundaries.

### Store full redirect target URIs in policy state

Rejected because paths and queries may contain personal data, authorization codes, signatures, or other secrets. A canonical digest provides cycle identity without retaining those values.

## Follow-up decisions

Separate ADRs are required before implementing:

- proxy and PAC destination enforcement;
- socket and TLS adapter behavior;
- response and download resource budgets;
- MIME and observed-content validation;
- Chromium/BiDi/CDP integration;
- persistent connection and redirect provenance.

## References

Cotton, M., Vegoda, L., Bonica, R. (Ed.), & Haberman, B. (2013). *Special-purpose IP address registries* (RFC 6890). RFC Editor. https://doi.org/10.17487/RFC6890 https://www.rfc-editor.org/rfc/rfc6890

Internet Assigned Numbers Authority. (n.d.). *IANA IPv4 Special-Purpose Address Registry*. Retrieved August 25, 2026, from https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml

Internet Assigned Numbers Authority. (n.d.). *IANA IPv6 Special-Purpose Address Registry*. Retrieved August 25, 2026, from https://www.iana.org/assignments/iana-ipv6-special-registry/iana-ipv6-special-registry.xhtml

Fielding, R. (Ed.), Nottingham, M. (Ed.), & Reschke, J. (Ed.). (2022). *HTTP semantics* (RFC 9110). RFC Editor. https://doi.org/10.17487/RFC9110 https://www.rfc-editor.org/rfc/rfc9110
