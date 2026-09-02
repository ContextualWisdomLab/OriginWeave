# MCP 2026-07-28 authority-route traceability

- **`tools/call` capability maturity:** `IMPLEMENTED_ON_PROTECTED_MAIN`
- **`tools/list` capability maturity:** `IMPLEMENTED_ON_PROTECTED_MAIN`
- **Protected-main owning work:** merged PR #168 (`tools/call`) and PR #170 (`tools/list`)
- **Active architecture repair:** PR #272 (`originweave-mcp` adapter boundary)
- **Complete MCP adapter status:** `PLANNED`
- **Governing decision:** ADR 0107

## Scope

Protected `main@542ca1e9c0a863595b8b6697790005d2471f5413` contains the bounded Rust MCP `2026-07-28` routing and discovery foundation merged through PRs #168 and #170. The current protected-main implementation lives in `originweave-core`: it bounds and syntax-validates attacker-controlled routing fields, maps only the explicit reviewed `originweave.*` catalog to existing typed `ActionKind` values, derives discovery metadata from that same catalog, and rejects route/action mismatch before ordinary deterministic policy evaluation. The `tools/list` boundary requires matching transport/request protocol-version metadata and per-request client-capabilities presence, and returns one complete, private, zero-TTL page with no continuation cursor.

A successful MCP routing value proves protocol integrity only. It grants no capability, origin, approval, secret, browser, tenant, persistence, network, evidence, or ambient execution authority. Browser and policy authority remain in their OriginWeave bounded contexts.

PR #272 is an active DDD repair that moves the external MCP protocol surface into `originweave-mcp` while preserving the inward dependency direction: the adapter may consume stable core contracts and the protocol-independent policy API, but core and policy must not depend outward on MCP transport types. The move is active-PR evidence, not protected-main shipment.

## Final 2026-07-28 per-request envelope

The final MCP `2026-07-28` request envelope requires `io.modelcontextprotocol/protocolVersion` and `io.modelcontextprotocol/clientCapabilities` on each request. For HTTP, the per-request protocol version must match the `MCP-Protocol-Version` header. Client capabilities are per-request state and must not be inferred from earlier requests.

`io.modelcontextprotocol/clientInfo` is different: the final revision demoted it to **SHOULD**, not MUST. Requests without `clientInfo` remain valid; when present it is self-reported metadata intended for display, logging, and debugging rather than authorization or security decisions. OriginWeave therefore must not reject an otherwise valid `tools/call` solely because client identity is absent, and must never turn `clientInfo` into browser or policy authority.

PR #272 now implements the adapter-local repair for this distinction. The exported `ValidatedMcpToolCall` requires both the HTTP protocol-version surface and structured request protocol version, bounds both before comparison, rejects disagreement or unsupported versions, and requires an attestation that the per-request client-capabilities object was present. It retains neither capabilities contents nor optional client identity. The pre-modern constructor shape remains as a fail-closed compatibility surface: malformed legacy routes still receive deterministic routing diagnostics, while a syntactically valid legacy route cannot become a validated modern tool call because that signature cannot prove required per-request metadata.

The lower routing validator and reviewed tool-to-`ActionKind` catalog remain internal implementation details of `originweave-mcp`; core and policy receive no MCP request-envelope types. Policy evaluation still consumes only the typed action contract after the adapter has established protocol integrity.

## Product-status reconciliation

`docs/PRD.md` PRD-INT-004 and the corresponding TRD complete-adapter work remain **Planned**. Protected-main routing/discovery contracts and PR #272's architecture repair are reusable control-plane slices below the complete product adapter. They do not establish a complete MCP server/runtime.

The following remain outside the protected-main bounded contract and must not be inferred from it:

- complete Streamable HTTP transport parsing and response serialization;
- OAuth and authenticated MCP deployment policy;
- browser-control I/O or WebDriver BiDi/CDP translation;
- secret materialization or broker transport;
- persistence, durable audit storage, or WARC/PROV export;
- general pagination/subscription runtime beyond the currently reviewed contracts; and
- an OriginWeave Protocol version transition.

## Version and authority boundary

MCP versioning is independent of the OriginWeave Protocol. A later MCP revision does not silently change OriginWeave action, risk, capability, approval, secret, origin, tenant, browser, or evidence semantics. MCP routing metadata and optional client identity remain adapter data, not domain authority.

## Executable evidence

Protected-main production/test surfaces currently include:

- `crates/originweave-core/src/mcp.rs` — deterministic catalog, `tools/call` routing validation, and `tools/list` request/result contracts;
- `crates/originweave-core/tests/mcp_authority_route.rs` — explicit mapping, bounds, malformed inputs, version/method correlation, and public error contracts;
- `crates/originweave-core/tests/mcp_tools_list_cache.rs` — required protocol/client-capabilities metadata, conservative cache/result semantics, method correlation, and cursor rejection; and
- the protocol-independent policy evaluator and route/action preservation tests.

Active PR #272 relocates the external-protocol implementation to `crates/originweave-mcp/`. Its adapter tests now cover the required modern `tools/call` request metadata, fail-closed legacy constructor, header↔request-version mismatch, missing per-request capabilities, bounded method/tool routing, explicit catalog mapping, and preservation of ordinary policy denials. Exact-current CI/security/review evidence must be regenerated after every branch mutation. Predecessor, protected-main, skipped, status-only, or model evidence is not current-head proof for PR #272.

## Primary sources

Model Context Protocol. (2026, July 28). *The 2026-07-28 specification*. https://modelcontextprotocol.io/specification/2026-07-28

Model Context Protocol. (2026). *Supporting protocol revision 2026-07-28* [TypeScript SDK migration guide]. https://github.com/modelcontextprotocol/typescript-sdk/blob/main/docs/migration/support-2026-07-28.md

Model Context Protocol. (2026). *2026-07-28 protocol type definitions* [TypeScript source]. https://github.com/modelcontextprotocol/typescript-sdk/blob/main/packages/core-internal/src/types/spec.types.2026-07-28.ts

The canonical broader bibliography remains `docs/doctoring.md`.

## Promotion rule

The bounded `tools/call` and `tools/list` foundations are already `IMPLEMENTED_ON_PROTECTED_MAIN`. PR #272 may change the adapter architecture only after its exact current head proves repository-native CI, full owned-production coverage/rustdoc, security gates, required central workflows, and live review governance. Neither that promotion nor the existing protected-main contracts makes the complete MCP adapter implemented; each remaining transport/runtime boundary requires its own integrated evidence.
