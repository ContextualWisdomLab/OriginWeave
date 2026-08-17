# Browser and Agent Protocol Standards Evidence

- **Reviewed:** 2026-08-18
- **Purpose:** primary-source evidence for OriginWeave browser compatibility and adapter boundaries
- **Canonical research index:** [`../doctoring.md`](../doctoring.md)

This addendum complements the main doctoring record. The main record already carries the WebDriver BiDi, WARC/ISO 28500 and W3C PROV-O evidence. This addendum records the current primary sources for Manifest V3, Chrome DevTools Protocol, WebMCP and Model Context Protocol so product documentation does not rely on uncited protocol names.

## WebDriver BiDi

The latest published W3C technical-report baseline reviewed here remains the 1 June 2026 **Working Draft**, not a Recommendation. The current Editor’s Draft reviewed on 18 August 2026 identifies itself as the 20 July 2026 draft. OriginWeave therefore treats BiDi as a versioned browser-automation adapter rather than product-internal authority. Raw BiDi session/context/node identifiers do not become durable OriginWeave identities.

For the bounded `browsingContext.locateNodes` command-serialization boundary, the reviewed Editor’s Draft defines a command envelope with `id: js-uint`, defines `js-uint` as `0..9007199254740991`, and defines `browsingContext.locateNodes` parameters containing a browsing context, locator, optional positive `maxNodeCount`, optional `serializationOptions`, and optional `startNodes`. OriginWeave serializes only its separately reviewed accessibility-locator subset and fixed minimal serialization options; this deterministic JSON value is not transport authentication or browser/Agent authority.

WebDriver BiDi commands may execute concurrently and finish out of order. The Editor’s Draft defines the command id as the local end’s correlation identifier and sets a successful `CommandResponse.id` to that exact command id; an `ErrorResponse.id` may be `null` when no valid command id can be recovered. OriginWeave therefore fails closed unless a non-null protocol-range response id exactly matches the consumed command before later payload admission. Parsing success/error envelopes, handling nullable malformed-command errors, and authenticating the browser transport remain separate adapter boundaries.

Primary sources: World Wide Web Consortium, *WebDriver BiDi* (published Working Draft and current Editor’s Draft).

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

Primary sources: Model Context Protocol, *2026-07-28 Specification* and the maintainers' official release announcement.

## Provenance standards

The main [`docs/doctoring.md`](../doctoring.md) records the stable W3C PROV-O Recommendation and ISO 28500:2017 WARC format. OriginWeave treats both as interoperability/persistence adapters around its typed evidence identities. A WARC record, PROV statement, model judgement, check result or action log is evidence of its own class; none becomes authorization merely because it is captured in a provenance format.

## Product consequences

1. Version adapter contracts independently from OriginWeave session/context/action/evidence types. Admit a BiDi `script.NodeRemoteValue` only as an untrusted transport handle when its type is exactly `node` and a usable control-free `sharedId` is present; do not treat a realm-local `handle`, a missing shared identifier, or control/whitespace-bearing protocol text as OriginWeave node authority. Treat an accessibility-query role as one exact WAI-ARIA token, not a whitespace-separated fallback list.
2. Serialize reviewed BiDi commands from already validated bounded values only, then correlate each non-null response id to the exact consumed command before payload admission; a protocol-shaped JSON envelope or matching id never substitutes for authenticated browser transport, current session/context/origin/document authority, policy authorization, or post-condition evidence.
3. Pin exact Chromium/CDP compatibility evidence at release time.
4. Keep WebDriver BiDi's Working Draft status visible in compatibility claims.
5. Keep WebMCP experimental/optional and propagate untrusted-content semantics.
6. Keep MCP browser state application-level rather than equating protocol transport/session metadata with browser authority.
7. Test Manifest V3 compatibility and extension-to-Agent authority isolation as separate evidence classes.
8. Treat WARC/PROV as provenance representations, not policy or truth escalation.

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

World Wide Web Consortium. (2023, June 6). *Accessible Rich Internet Applications (WAI-ARIA) 1.2*. https://www.w3.org/TR/2023/REC-wai-aria-1.2-20230606/

World Wide Web Consortium. (2026, June 1). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260601/

World Wide Web Consortium. (2026, July 20). *WebDriver BiDi* (Editor’s Draft). https://w3c.github.io/webdriver-bidi/

World Wide Web Consortium. (2026, August 5). *Accessible name and description computation 1.2* (W3C Working Draft). https://www.w3.org/TR/2026/WD-accname-1.2-20260805/

International Organization for Standardization. (2017). *Information and documentation—WARC file format* (ISO Standard No. 28500:2017). https://www.iso.org/standard/68004.html
