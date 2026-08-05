# ADR 0003: Make structured observation and provenance first-class outputs

- Status: Accepted
- Date: 2026-08-05

## Context

Raw HTML and screenshots are large, noisy, and insufficient for a trustworthy enterprise result. Buyers need to know which page, network value, DOM or accessibility node, artifact, model, and policy decision produced a field or action.

## Decision

OriginWeave uses a preference hierarchy of typed site tools, structured metadata, redacted network responses, accessibility/DOM/layout semantics, and visual fallback. Every extracted assertion carries a source URL, channel-specific locator, source hash, and verification state. Future persistence uses WARC for captured exchanges and W3C PROV-compatible records for derivation and responsibility.

## Consequences

- Observation adapters must preserve source identity rather than flatten everything into prose.
- Network credentials and personal data require redaction before evidence leaves the trusted boundary.
- Screenshot-only operation is a fallback, not the default architecture.
- Evidence completeness becomes a release and benchmark metric.
