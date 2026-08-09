# ADR 0107: Versioned browser and agent protocol adapters

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

OriginWeave must interoperate with Chromium and external automation/agent ecosystems without allowing any one unstable protocol to define the product. WebDriver BiDi is standards-track but evolving, Chrome DevTools Protocol includes tip-of-tree surfaces without backwards-compatibility guarantees, WebMCP is experimental, and Model Context Protocol is an external tool/context protocol rather than browser authority. Directly exposing these surfaces as the OriginWeave API would couple customers to provider churn and blur policy boundaries.

## Decision drivers

- Stable OriginWeave semantics across browser/provider upgrades.
- Standards-first interoperability where mature enough.
- Ability to use Chromium-specific capabilities without making experimental CDP the sole authority.
- Explicit trust boundaries for WebMCP and MCP content/tools.
- Conformance and compatibility testing per adapter version.

## Assumptions and authority boundaries

The OriginWeave Protocol and Rust control plane own product semantics. WebDriver BiDi, Chrome DevTools Protocol, WebMCP, and Model Context Protocol are adapters or evidence/tool transports. Adapter messages are validated and cannot directly grant capabilities, approvals, secrets, origin authority, or evidence truth.

## Options considered

1. CDP as the public product API: rejected because Chromium-specific and unstable tip-of-tree surfaces create vendor/version lock-in.
2. WebDriver BiDi only: rejected because not every required Chromium/experimental capability is standardized yet.
3. Expose all upstream protocols directly: rejected because clients would inherit incompatible authority models.
4. Versioned internal protocol with explicit BiDi/CDP/WebMCP/MCP adapters: selected.

## Decision

OriginWeave exposes its own versioned protocol for session, observation, query, typed action, policy/evidence, secret-handle, resource, and lifecycle semantics. Browser and ecosystem adapters map that protocol to supported WebDriver BiDi, stable or pinned CDP, WebMCP, and MCP surfaces. Prefer standards-track BiDi when it satisfies the contract. Use Chromium-specific CDP only behind versioned adapter capability declarations. Treat WebMCP outputs as untrusted page/tool observations. Treat MCP as an external integration boundary, not a source of OriginWeave authority. Experimental/tip-of-tree surfaces are optional and must have fallback or explicit unsupported behavior.

## Consequences

OriginWeave carries adapter maintenance and version negotiation but gains a durable customer API. Multiple browser/control transports can coexist. New upstream capabilities do not silently change risk or action semantics. Compatibility matrices become release artifacts.

## Failure and degraded behavior

Adapter negotiation failure disables only affected capabilities. Unsupported or schema-incompatible messages fail closed with typed errors. OriginWeave must not bypass a failed adapter by exposing raw CDP or arbitrary JavaScript to an autonomous model. A standards adapter may fall back to a pinned vendor adapter only when the same OriginWeave semantic and security contract is proven.

## Security / privacy / governance impact

Protocol validation occurs before messages influence policy. Tool/page-provided strings remain untrusted. Secret handles never become raw protocol payloads except inside a separately authorized broker-to-browser delivery path. Adapter version/provenance is recorded for audit and incident reconstruction.

## Tests and acceptance evidence

Require version-negotiation tests, schema/property tests, malformed-message tests, BiDi/CDP semantic parity tests for shared capabilities, WebMCP prompt-injection tests, MCP authority-separation tests, browser-version compatibility matrices, and end-to-end proof that unsupported capabilities fail without side effects.

## Migration and rollback

Adapters are independently versioned and can be canaried. Clients migrate through OriginWeave Protocol compatibility rules, not upstream protocol rewrites. Rollback pins a previously supported adapter/browser pair and records that pair in provenance.

## Open follow-ups

Define internal protocol versioning rules, adapter capability descriptors, minimum supported BiDi level, CDP pin policy, and MCP/WebMCP schema isolation.

## Supersession / reversal conditions

Supersede if one mature standard gains all required capabilities, stable compatibility, explicit security semantics, and broad implementation support sufficient to replace the internal abstraction without exposing customers to upstream churn.

## References

See `docs/API_CONTRACT.md`, `docs/TRD.md`, `docs/doctoring/product-documentation-baseline.md`, and current W3C/Chromium/MCP primary documentation recorded there.
