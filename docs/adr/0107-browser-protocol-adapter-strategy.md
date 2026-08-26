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

MCP version negotiation is independent of the OriginWeave Protocol version. As of this review, MCP `2026-07-28` is the current released protocol generation; a future MCP change does not silently alter OriginWeave task, approval, secret, tenant, or browser semantics. MCP tool/resource content remains untrusted input and any server-to-client/user interaction capability is mediated by the same OriginWeave policy/approval boundaries as other adapter traffic.

### Current implementation boundary

The complete MCP adapter remains **Planned**. Active PR #168 is narrower **IMPLEMENTED_ON_ACTIVE_PR** evidence inside the Rust control plane: it validates the `2026-07-28` stateless `tools/call` routing envelope presented to this boundary, bounds and syntax-checks both untrusted method fields and both untrusted tool-name fields before cross-field correlation, derives one of the existing typed `ActionKind` values from a deterministic reviewed registry, exposes discovery metadata from that same registry, and requires the resulting action to pass the ordinary OriginWeave policy evaluator. The method boundary accepts only nonempty ASCII method names up to 64 bytes using the reviewed routing alphabet, while the tool-name boundary accepts only nonempty ASCII names up to 128 bytes using its narrower reviewed alphabet. The catalog and validated route grant no capability, approval, origin, secret, browser, persistence, or evidence authority by themselves.

PR #168 does not implement Streamable HTTP transport parsing, complete request `_meta` validation, `tools/list` serialization/caching/pagination, OAuth, browser I/O, WebMCP/BiDi/CDP translation, secret delivery, persistence, or a complete OriginWeave Protocol adapter. Those remain separate adapter/runtime work. Protected `main` therefore must continue to describe MCP as planned until this active-PR evidence is integrated, and even after integration only the merged bounded routing foundation may be called implemented; the full adapter remains planned until its remaining acceptance boundaries ship.

The version boundary is explicit: the routing foundation accepts only MCP `2026-07-28`; it does not infer compatibility with later protocol generations. OriginWeave Protocol versioning remains independent and cannot be changed by MCP metadata.

## Consequences

OriginWeave carries adapter maintenance and version negotiation but gains a durable customer API. Multiple browser/control transports can coexist. New upstream capabilities do not silently change risk or action semantics. Compatibility matrices become release artifacts.

## Failure and degraded behavior

Adapter negotiation failure disables only affected capabilities. Unsupported or schema-incompatible messages fail closed with typed errors. OriginWeave must not bypass a failed adapter by exposing raw CDP or arbitrary JavaScript to an autonomous model. A standards adapter may fall back to a pinned vendor adapter only when the same OriginWeave semantic and security contract is proven.

## Security / privacy / governance impact

Protocol validation occurs before messages influence policy. Tool/page-provided strings remain untrusted. Method and tool routing metadata is shape-bounded before correlation, preventing malformed or oversized untrusted routing strings from being reinterpreted through mismatch handling. Secret handles never become raw secret protocol payloads; only the separately authorized trusted broker-to-browser delivery path may materialize the value, and that value does not pass through MCP, WebMCP, BiDi observation, or model-visible CDP output. Adapter version/provenance is recorded for audit and incident reconstruction.

## Tests and acceptance evidence

Require version-negotiation tests, schema/property tests, malformed-message tests, BiDi/CDP semantic parity tests for shared capabilities, WebMCP prompt-injection tests, MCP authority-separation and version-change tests, browser-version compatibility matrices, and end-to-end proof that unsupported capabilities fail without side effects.

For active PR #168 specifically, acceptance additionally requires deterministic method and tool-name bounds/syntax, exact header/body method and tool-name correlation only after both sides are bounded, explicit invalid-method/invalid-tool-name/unknown-tool rejection, one unambiguous tool-to-action registry, independent capability/risk expectations, route/action mismatch denial before ordinary policy evaluation, exact 100% owned-production coverage, and unchanged-head CI/security/review evidence. These checks do not substitute for complete transport or adapter conformance.

## Migration and rollback

Adapters are independently versioned and can be canaried. Clients migrate through OriginWeave Protocol compatibility rules, not upstream protocol rewrites. Rollback pins a previously supported adapter/browser/protocol pair and records that pair in provenance.

## Open follow-ups

Define internal protocol versioning rules, adapter capability descriptors, minimum supported BiDi level, CDP pin policy, complete MCP Streamable HTTP/request-metadata validation, MCP discovery/serialization/cache behavior, and MCP/WebMCP schema isolation.

## Supersession / reversal conditions

Supersede if one mature standard gains all required capabilities, stable compatibility, explicit security semantics, and broad implementation support sufficient to replace the internal abstraction without exposing customers to upstream churn.

## References

Chrome DevTools Protocol. (2026). *Chrome DevTools Protocol — latest (tip-of-tree)*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/

Chrome DevTools Protocol. (2026). *WebMCP domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/WebMCP/

Model Context Protocol. (2026, July 28). *Specification: 2026-07-28*. https://modelcontextprotocol.io/specification/2026-07-28

Parra, D. S., & Delimarsky, D. (2026, July 28). *The 2026-07-28 specification*. Model Context Protocol Blog. https://blog.modelcontextprotocol.io/posts/2026-07-28/

World Wide Web Consortium. (2026, June 29). *WebDriver BiDi* [Working Draft]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260629/

## Related documents

See `docs/API_CONTRACT.md`, `docs/TRD.md`, `docs/doctoring.md`, `docs/doctoring/product-documentation-baseline.md`, `docs/traceability/README.md`, and `docs/DATA_GOVERNANCE.md`.
