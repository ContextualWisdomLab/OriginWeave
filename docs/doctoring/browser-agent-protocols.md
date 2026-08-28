# Browser and Agent Protocol Standards Evidence

- **Reviewed:** 2026-08-18
- **Purpose:** primary-source evidence for OriginWeave browser compatibility and adapter boundaries
- **Canonical research index:** [`../doctoring.md`](../doctoring.md)

This addendum complements the main doctoring record. The main record already carries the WebDriver BiDi, WARC/ISO 28500 and W3C PROV-O evidence. This addendum records the current primary sources for Manifest V3, Chrome DevTools Protocol, WebMCP and Model Context Protocol so product documentation does not rely on uncited protocol names.

## WebDriver BiDi

The W3C publication reviewed for this baseline is the 1 June 2026 **Working Draft**, not a Recommendation. OriginWeave therefore treats BiDi as a versioned browser-automation adapter rather than product-internal authority. Raw BiDi session/context/node identifiers do not become durable OriginWeave identities.

Primary source: World Wide Web Consortium, *WebDriver BiDi*.

## Chrome Manifest V3

Chrome's current manifest documentation identifies Manifest V3 as the current extension manifest format and the supported `manifest_version` value. OriginWeave therefore tests its declared extension compatibility against a pinned real Chromium/Chrome-for-Testing build and publishes evidence by exact capability. This is a compatibility target, not a claim of universal Chrome/Web Store/Google-service/codec/DRM equivalence.

A Chrome extension permission remains separate from an OriginWeave Agent capability. Passing MV3 compatibility tests does not prove Agent-authority isolation, and a correct extension-grant kernel does not prove a real Chrome extension API works.

Primary source: Chrome for Developers, *Manifest file format* and *Manifest Version*.

## Chrome DevTools Protocol

The official CDP documentation states that tip-of-tree changes frequently and provides no backward-compatibility guarantee for capabilities it introduces. OriginWeave therefore pins the Chromium/protocol evidence used by a release and keeps CDP behind an adapter. CDP is useful for Chromium-specific Network, Accessibility, DOMSnapshot, tracing and diagnostic surfaces; it is not the durable OriginWeave authority model.

Primary source: Chrome DevTools Protocol, *Chrome DevTools Protocol—Latest (tip-of-tree)*.

## WebMCP

Chrome's 2026 WebMCP documentation describes WebMCP as an experimental/proposed structured-tool surface and its security guidance explicitly discusses indirect prompt injection and `untrustedContentHint`. The reviewed Chrome material is associated with an origin-trial / intent-to-experiment path. OriginWeave may prefer a valid structured WebMCP tool over lower-level scraping when present, but WebMCP remains optional and adapter-bound.

WebMCP tool definitions, extension-produced content and tool outputs are untrusted observations. They cannot mint OriginWeave capabilities, alter the trusted task goal, resolve secrets, or approve high-risk actions.

Primary sources: Chrome for Developers, *WebMCP*; *WebMCP tool security*; *Agent security considerations for WebMCP*.

## Model Context Protocol

The Model Context Protocol project released specification version `2026-07-28` on 28 July 2026. That release moved the protocol core toward stateless request/response operation and removed the earlier protocol-session assumptions described by previous releases. OriginWeave therefore keeps durable browser state in explicit OriginWeave application handles and exposes MCP only as a high-level adapter to the Rust runtime. MCP clients or servers do not connect models directly to Chromium/CDP authority.

The final `2026-07-28` schema requires every client request to carry `io.modelcontextprotocol/protocolVersion` and `io.modelcontextprotocol/clientCapabilities` in request `_meta`; client capabilities are request-scoped and servers must not infer them from prior requests. `io.modelcontextprotocol/clientInfo` is optional/SHOULD rather than authorization evidence. For Streamable HTTP, `MCP-Protocol-Version` must agree with the body protocol version, `Mcp-Method` is required for every request, and `Mcp-Name` is required only for named operations such as `tools/call`, `resources/read`, and `prompts/get`, not `tools/list`. OriginWeave's typed `tools/list` admission boundary therefore independently requires the transport protocol-version header and body `_meta` protocol version, rejects disagreement or an unsupported generation, requires per-request client-capabilities presence without treating its contents as OriginWeave authority, validates routing/body `tools/list` method agreement, and does not invent a name header. It rejects any supplied cursor because the current fixed catalog emits no `nextCursor`; this is a conservative local invariant against accepting pagination state OriginWeave never issued, not a claim that MCP forbids `tools/list` cursors generally.

The same specification requires every Result to carry `resultType`, using `complete` for a terminal result, and adds explicit cache hints for cacheable result families including `tools/list`: `ttlMs` expresses freshness lifetime and `cacheScope` expresses whether reuse is private or shareable. OriginWeave's first typed `tools/list` result therefore binds `resultType = complete`, chooses the conservative boundary `ttlMs = 0` and private scope, derives the page directly from the reviewed tool catalog, and emits no continuation cursor for the current fixed single-page catalog. These metadata choices do not grant tool authority and do not claim JSON-RPC serialization, transport caching, OAuth, or a general pagination implementation.

Primary sources: Model Context Protocol, *2026-07-28 Specification* and the maintainers' official release announcement.

## Provenance standards

The main [`docs/doctoring.md`](../doctoring.md) records the stable W3C PROV-O Recommendation and ISO 28500:2017 WARC format. OriginWeave treats both as interoperability/persistence adapters around its typed evidence identities. A WARC record, PROV statement, model judgement, check result or action log is evidence of its own class; none becomes authorization merely because it is captured in a provenance format.

## Product consequences

1. Version adapter contracts independently from OriginWeave session/context/action/evidence types.
2. Pin exact Chromium/CDP compatibility evidence at release time.
3. Keep WebDriver BiDi's Working Draft status visible in compatibility claims.
4. Keep WebMCP experimental/optional and propagate untrusted-content semantics.
5. Keep MCP browser state application-level rather than equating protocol transport/session metadata with browser authority.
6. Require modern MCP per-request protocol version and client capabilities from request `_meta`; on Streamable HTTP require the matching protocol-version header and exact method routing, while treating optional client identity metadata as non-authoritative.
7. Bind mandatory MCP result disposition and cacheable-list metadata to reviewed typed results; use a complete terminal result with zero freshness and private scope unless a separate reviewed policy proves broader semantics safe. Reject a `tools/list` cursor while the current fixed page has never issued one.
8. Test Manifest V3 compatibility and extension-to-Agent authority isolation as separate evidence classes.
9. Treat WARC/PROV as provenance representations, not policy or truth escalation.

## References — APA 7th

Chrome DevTools Protocol. (n.d.). *Chrome DevTools Protocol—Latest (tip-of-tree)*. Retrieved August 10, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/

Google Chrome Developers. (n.d.). *Manifest file format*. Chrome for Developers. Retrieved August 10, 2026, from https://developer.chrome.com/docs/extensions/reference/manifest

Google Chrome Developers. (n.d.). *Manifest Version*. Chrome for Developers. Retrieved August 10, 2026, from https://developer.chrome.com/docs/extensions/reference/manifest/manifest-version

Google Chrome Developers. (2026). *WebMCP*. Chrome for Developers. https://developer.chrome.com/docs/ai/webmcp

Pagnucco, J., & Klepper, A. (2026, June 9). *Agent security considerations for WebMCP*. Chrome for Developers. https://developer.chrome.com/docs/agents/security

Pagnucco, J., & Klepper, A. (2026, June 9). *WebMCP tool security*. Chrome for Developers. https://developer.chrome.com/docs/ai/webmcp/secure-tools

Soria Parra, D., & Delimarsky, D. (2026, July 28). *The 2026-07-28 specification*. Model Context Protocol. https://blog.modelcontextprotocol.io/posts/2026-07-28/

Model Context Protocol. (2026). *Model Context Protocol specification (2026-07-28)*. https://modelcontextprotocol.io/specification/2026-07-28

World Wide Web Consortium. (2013). *PROV-O: The PROV ontology*. https://www.w3.org/TR/prov-o/

World Wide Web Consortium. (2026, June 1). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260601/

International Organization for Standardization. (2017). *Information and documentation—WARC file format* (ISO Standard No. 28500:2017). https://www.iso.org/standard/68004.html