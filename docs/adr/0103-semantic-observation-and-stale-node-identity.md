# ADR 0103: Semantic observation precedence and stale-node identity

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

Agentic browsing needs compact, stable observations without treating one browser representation as universally authoritative. WebMCP or structured application data can expose high-level semantics, accessibility trees can expose user-visible structure, DOM/layout can fill gaps, and visual interpretation is sometimes necessary. Each layer can be incomplete, adversarial, stale, or unavailable. Node references are particularly dangerous after navigation or document mutation because a selector or backend identifier can refer to different state later.

## Decision drivers

- Prefer the highest-signal structured representation while retaining independent verification.
- Avoid blind trust in experimental WebMCP or page-authored semantics.
- Preserve accessibility and user-visible meaning.
- Prevent actions against stale nodes after document replacement or relevant mutation.
- Keep observation provenance sufficient to explain what the agent saw.

## Assumptions and authority boundaries

All page-derived observations are untrusted data. Observation sources can inform planning but cannot grant action capability or approval. A semantic node identity is meaningful only within its browser session/context and document epoch. Network or structured-data observations may corroborate semantics but do not override policy.

## Options considered

1. DOM-only observation: rejected because it is noisy and can diverge from accessibility or application semantics.
2. Visual-only observation: rejected because it is expensive, difficult to make deterministic, and loses structured provenance.
3. WebMCP-first authority: rejected because WebMCP is experimental and page/tool output can contain prompt injection.
4. Ordered multi-source observation with explicit provenance and stale identity: selected.

## Decision

Observation precedence is WebMCP/explicit structured application contracts when available, then structured data and network-derived semantics, accessibility semantics, DOM, layout, and bounded visual fallback. Higher precedence means preferred representation, not unquestioned trust. Every observation records source and document identity. Actionable semantic nodes bind to session/context identity plus a monotonically changing document epoch or equivalent navigation generation. A node from an earlier epoch is stale and must be re-observed before action.

## Consequences

The control plane needs normalization and conflict handling across observation sources. Visual fallback remains bounded rather than becoming the default. Agents gain smaller, more meaningful snapshots while policy and verification remain independent. Browser adapters must expose lifecycle events reliably enough to invalidate stale references.

## Failure and degraded behavior

If a preferred source is absent, malformed, oversized, contradictory, or unsupported, OriginWeave falls back to the next governed source and records degradation. If document identity cannot be established, node-targeted state changes fail closed. A stale node must produce a deterministic stale-reference result, never an automatic best-effort action against a newly matched element.

## Security / privacy / governance impact

Prompt injection in WebMCP, DOM, accessibility names, structured data, or visual text never becomes instruction authority. Sensitive observation content follows privacy, retention, and selective-disclosure policy. Provenance records which source contributed each semantic claim so enterprise audit can distinguish browser evidence from agent inference.

## Tests and acceptance evidence

Require cross-source conflict tests, hostile WebMCP/DOM/accessibility payload tests, navigation/document-replacement stale-node tests, history and same-document mutation tests where identity semantics differ, visual-fallback budget tests, accessibility preservation tests, and end-to-end verification that stale references cause no side effect.

## Migration and rollback

Introduce source/provenance metadata and document epoch before exposing durable semantic-node handles. Rollback may disable a higher-level adapter such as WebMCP and fall back to lower layers; it must not disable stale-node invalidation or collapse source provenance.

## Open follow-ups

Specify stable semantic-node serialization, mutation invalidation granularity, observation conflict scoring, and browser-version conformance requirements.

## Supersession / reversal conditions

Supersede if a standardized browser semantic interface achieves broad stable support and can provide equivalent provenance, injection resistance, accessibility fidelity, and stale-reference guarantees under production tests.

## References

See `docs/TRD.md`, `docs/API_CONTRACT.md`, `docs/THREAT_MODEL.md`, `docs/uml/README.md`, and the session/document work tracked by the existing OriginWeave architecture.
