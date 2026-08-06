# ADR 0003: Make structured observation and provenance first-class outputs

- Status: Accepted
- Date: 2026-08-05

## Context

Raw HTML and screenshots are large, noisy, and insufficient for a trustworthy enterprise result. Buyers need to know which page, network value, DOM or accessibility node, artifact, model, and policy decision produced a field or action.

Network metadata is hostile input. A finite deny-list cannot prove that custom authorization headers, OAuth parameters, presigned-URL fields, redirect locations, referrers, or vendor-specific token names are safe. Even conventionally benign fields such as `ETag` or `Content-Type` can carry arbitrary attacker-controlled bytes. Paths and provenance text also require explicit size and ambiguity limits before retention.

## Decision

OriginWeave uses a preference hierarchy of typed site tools, structured metadata, redacted network responses, accessibility/DOM/layout semantics, and visual fallback. Every extracted assertion carries a source URL, channel-specific locator, source hash, and verification state. Future persistence uses WARC for captured exchanges and W3C PROV-compatible records for derivation and responsibility.

The current network-evidence kernel stores only bounded request method, canonical origin, validated path, and bounded metadata field names. Every header and query value is replaced with the same redaction marker before it leaves the trusted boundary. It rejects excessive field counts, empty or oversized names, oversized values, overlong paths, controls, whitespace, query and fragment delimiters, backslashes, malformed percent escapes, encoded path separators, and literal or encoded dot segments.

Approved response bodies or typed protocol values are not smuggled through generic metadata. They require a separate schema-specific capture contract with its own MIME, size, retention, encryption, authorization, and provenance rules.

## Consequences

- Observation adapters must preserve source identity rather than flatten everything into prose.
- Generic evidence records deliberately sacrifice metadata values to obtain a fail-closed credential boundary.
- A future typed metadata API may retain a value only after strict field-specific grammar and size validation.
- Network credentials and personal data require redaction before evidence leaves the trusted boundary.
- Screenshot-only operation is a fallback, not the default architecture.
- Evidence completeness becomes a release and benchmark metric.
- WARC or object-storage adapters must not weaken the limits established by the in-memory evidence contract.
