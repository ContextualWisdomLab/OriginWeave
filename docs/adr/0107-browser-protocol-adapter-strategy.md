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

The complete MCP adapter remains **Planned**. Protected main now contains the narrower bounded Rust `tools/call` routing/action-policy foundation merged through PR #168. That protected-main foundation validates the `2026-07-28` stateless `tools/call` routing envelope presented to this boundary, bounds and syntax-checks both untrusted method fields and both untrusted tool-name fields before cross-field correlation, derives one of the existing typed `ActionKind` values from a deterministic reviewed registry, exposes discovery metadata from that same registry, and requires the resulting action to pass the ordinary OriginWeave policy evaluator. The method boundary accepts only nonempty ASCII method names up to 64 bytes using the reviewed routing alphabet, while the tool-name boundary accepts only nonempty ASCII names up to 128 bytes using its narrower reviewed alphabet. The catalog and validated route grant no capability, approval, origin, secret, browser, persistence, or evidence authority by themselves.

Active PR #170 is a separate non-shipped refinement on top of that protected-main catalog. It adds one conservative typed `tools/list` request/result contract: both protocol-version fields are required and bounded before comparison, client-capability metadata must be present without becoming authority, both routing/body methods are syntax-bounded before correlation, only exact `tools/list` is admitted, and every caller-supplied cursor is rejected because the current fixed catalog issues none. The result is one complete page with zero freshness, private cache scope, and no continuation cursor.

Neither protected main nor PR #170 implements Streamable HTTP transport parsing, JSON-RPC/HTTP serialization, OAuth, browser I/O, WebMCP/BiDi/CDP translation, secret delivery, persistence, general pagination/subscription state, or a complete OriginWeave Protocol adapter. Those remain separate adapter/runtime work. Protected `main` may therefore describe only the bounded merged `tools/call` foundation as implemented; the full MCP adapter remains planned, and the `tools/list` refinement remains active-PR evidence until separately integrated.

The version boundary is explicit: the protected-main routing foundation and active discovery refinement accept only MCP `2026-07-28`; neither infers compatibility with later protocol generations. OriginWeave Protocol versioning remains independent and cannot be changed by MCP metadata.

PR #293 was merged into PR #229 on 2026-09-09, so its `originweave-bidi` capability boundary is inherited by this parent rather than remaining a separate active stacked slice. The adapter is runtime-qualified 18 August 2026 against the immutable WebDriver BiDi Working Draft URI `https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/`. W3C's latest published 18 August 2026 Working Draft is recorded in `docs/traceability/webdriver-bidi-publication-current.md`. A newer runtime pin requires a dedicated compatibility/conformance change and pinned-browser evidence.

The inherited capability map delegates complete-profile admission to `originweave-fingerprint` and intentionally excludes `Screen`, `Languages`, `HardwareConcurrency`, and `Platform`. The standard screen-settings command omits color depth and, importantly, applies one rectangle to both the web-exposed total screen area and available screen area, while the current OriginWeave presentation profile does not model the available-screen rectangle. The locale command likewise cannot prove ordered language preferences. Standard BiDi alone must therefore return the kernel's first `MissingSurface(Screen)` result rather than accept ambient host values.

PR #310 exposes the standard `emulation.setScreenSettingsOverride` operation as a separately explicit partial intent instead of inserting it into the reusable profile-derived plan. `WebDriverBidiScreenArea` projects validated width and height from `ScreenMetrics` and documents the protocol's total/available-area coupling; its matching reset is also explicit. The ordinary reusable-context plan remains viewport/DPR plus timezone while available-screen geometry is unmodelled. Reduced motion remains an expressible protocol capability but is excluded from the reusable plan because the standard cannot selectively restore prior media state; no caller-mintable exclusive-reset type substitutes for Browser Session lifecycle evidence. Planning does not send a command, create an acknowledgement, apply or prove cleanup of a profile, or produce page-observed evidence. Those remain #292 follow-up work and require exact-head verification plus a version-pinned Chromium/CDP adapter for the remainder. The detailed decision and acceptance boundary are recorded in `docs/traceability/webdriver-bidi-screen-area-planning.md`.

## Consequences

OriginWeave carries adapter maintenance and version negotiation but gains a durable customer API. Multiple browser/control transports can coexist. New upstream capabilities do not silently change risk or action semantics. Compatibility matrices become release artifacts.

## Failure and degraded behavior

Adapter negotiation failure disables only affected capabilities. Unsupported or schema-incompatible messages fail closed with typed errors. OriginWeave must not bypass a failed adapter by exposing raw CDP or arbitrary JavaScript to an autonomous model. A standards adapter may fall back to a pinned vendor adapter only when the same OriginWeave semantic and security contract is proven. A partial presentation-emulation capability set is unsupported for complete-profile admission; it cannot be completed with ambient browser values.

## Security / privacy / governance impact

Protocol validation occurs before messages influence policy. Tool/page-provided strings remain untrusted. Method and tool routing metadata is shape-bounded before correlation, preventing malformed or oversized untrusted routing strings from being reinterpreted through mismatch handling. Secret handles never become raw secret protocol payloads; only the separately authorized trusted broker-to-browser delivery path may materialize the value, and that value does not pass through MCP, WebMCP, BiDi observation, or model-visible CDP output. Adapter version/provenance is recorded for audit and incident reconstruction.

For presentation emulation, protocol availability is not presentation evidence. The adapter must bind its capability claim to an explicit protocol/browser revision, fail closed on missing required surfaces, and must not silently mutate a page-observable surface absent from the selected and digest-bound presentation identity. Every override actually applied must have owned cleanup before reuse is treated as clean, followed by page-visible post-cleanup observation. Neither a protocol command acknowledgement nor an unobserved browser setting is sufficient evidence.

## Tests and acceptance evidence

Require version-negotiation tests, schema/property tests, malformed-message tests, BiDi/CDP semantic parity tests for shared capabilities, WebMCP prompt-injection tests, MCP authority-separation and version-change tests, browser-version compatibility matrices, and end-to-end proof that unsupported capabilities fail without side effects.

For the protected-main `tools/call` foundation, acceptance includes deterministic method and tool-name bounds/syntax, exact header/body method and tool-name correlation only after both sides are bounded, explicit invalid-method/invalid-tool-name/unknown-tool rejection, one unambiguous tool-to-action registry, independent capability/risk expectations, route/action mismatch denial before ordinary policy evaluation, exact 100% owned-production coverage, and integrated review evidence from PR #168. For active PR #170, exact-current acceptance additionally requires bounded protocol metadata before cross-field comparison, required client-capabilities presence, bounded `tools/list` method correlation, rejection of unissued cursors, deterministic result/cache semantics, exact 100% owned-production coverage, and unchanged-head CI/security/review evidence. These checks do not substitute for complete transport or adapter conformance.

For the inherited PR #293 capability-boundary delta now carried by PR #229, acceptance requires the original regression proving the absence of an `originweave-bidi` bounded context on its predecessor, cleanup regressions that refuse to leave adapter-owned overrides behind, and exact-head Rust/Python/rustdoc/Clippy/coverage verification that the minimal adapter compiles and the runtime-qualified standard set fails with the canonical fingerprint-kernel missing-surface error. PR #310 additionally requires an explicit screen-area intent derived from validated `ScreenMetrics`, explicit total/available-area coupling semantics, a matching context-scoped reset, absence of color depth from the standard payload object, no automatic screen-area mutation in the reusable profile-derived plan while available-screen geometry is unmodelled, and continued `MissingSurface(Screen)` admission. This is not acceptance of #292 as a whole. Real pinned-Chromium application, page-observed post-condition evidence, navigation/renderer/crash/cleanup behavior, and the Chromium-only CDP remainder still require realistic browser E2E. Publication of a newer Working Draft is not compatibility evidence and cannot by itself change this acceptance basis.

## Migration and rollback

Adapters are independently versioned and can be canaried. Clients migrate through OriginWeave Protocol compatibility rules, not upstream protocol rewrites. Rollback pins a previously supported adapter/browser/protocol pair and records that pair in provenance.

## Open follow-ups

Define internal protocol versioning rules, complete MCP Streamable HTTP/request-metadata validation, MCP transport serialization, authenticated deployment, and MCP/WebMCP schema isolation. For presentation identity, decide and test the canonical available-screen-area model before any profile-derived `setScreenSettingsOverride` application, implement the exact pinned Chromium/BiDi command path, add a narrow version-pinned `originweave-cdp` capability owner for required non-BiDi surfaces, require post-application and post-cleanup page observation, navigation/renderer invalidation, crash/cleanup behavior, and release compatibility evidence.

## Supersession / reversal conditions

Supersede if one mature standard gains all required capabilities, stable compatibility, explicit security semantics, and broad implementation support sufficient to replace the internal abstraction without exposing customers to upstream churn.

## References

Chrome DevTools Protocol. (2026). *Chrome DevTools Protocol — latest (tip-of-tree)*. Chromium. Retrieved September 7, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/

Chrome DevTools Protocol. (2026). *Emulation domain*. Chromium. Retrieved September 7, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/Emulation/

Chrome DevTools Protocol. (2026). *WebMCP domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/WebMCP/

Model Context Protocol. (2026, July 28). *Specification: 2026-07-28*. https://modelcontextprotocol.io/specification/2026-07-28

Parra, D. S., & Delimarsky, D. (2026, July 28). *The 2026-07-28 specification*. Model Context Protocol Blog. https://blog.modelcontextprotocol.io/posts/2026-07-28/

World Wide Web Consortium. (2026, August 18). *WebDriver BiDi* [Working Draft; latest publication observed 2026-09-12]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/

World Wide Web Consortium. (2026, August 18). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/

## Related documents

See `docs/API_CONTRACT.md`, `docs/TRD.md`, `docs/doctoring.md`, `docs/doctoring/product-documentation-baseline.md`, `docs/traceability/README.md`, `docs/traceability/webdriver-bidi-publication-current.md`, `docs/traceability/webdriver-bidi-screen-area-planning.md`, and `docs/DATA_GOVERNANCE.md`.
