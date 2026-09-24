# MCP 2026-07-28 authority-route traceability

- **`tools/call` capability maturity:** `IMPLEMENTED_ON_PROTECTED_MAIN`
- **`tools/list` capability maturity:** `IMPLEMENTED_ON_PROTECTED_MAIN`
- **Protected-main owning work:** merged PR #168 `feat(mcp): bind stateless tool routing to typed actions`
- **Discovery refinement:** merged PR #170 `feat(mcp): expose conservative tools list cache contract`
- **Complete MCP adapter status:** `PLANNED`
- **Governing decision:** ADR 0107

## Scope

Protected main at `542ca1e9c0a863595b8b6697790005d2471f5413` contains the bounded Rust control-plane foundation for MCP `2026-07-28` `tools/call` routing that merged through PR #168. It validates the represented stateless routing envelope, bounds and syntax-validates both attacker-controlled method fields and both attacker-controlled tool-name fields before correlation, maps only an explicit reviewed `originweave.*` catalog to existing typed `ActionKind` values, derives discovery metadata from the same catalog, and rejects route/action mismatch before ordinary deterministic policy evaluation. Methods are nonempty reviewed-ASCII routing tokens of at most 64 bytes; tool names are nonempty reviewed-ASCII identifiers of at most 128 bytes. Invalid method metadata is rejected distinctly from a bounded but unsupported MCP method.

A successful `ValidatedMcpToolCall` proves routing integrity only. It grants no capability, origin, approval, secret, browser, tenant, persistence, network, or evidence authority. `originweave_policy::evaluate_mcp` still delegates to the ordinary policy evaluator after the route/action match.

The merged PR #170 adds a conservative typed `tools/list` request/result boundary to that protected-main catalog. It requires matching MCP protocol metadata, required client-capability presence, bounded and syntax-validated routing/body methods, exact `tools/list` routing, and no caller-supplied cursor because the fixed catalog issues none. Its result is one complete page with zero freshness, private cache scope, and no continuation cursor. It does not grant any OriginWeave action authority.

## Product-status reconciliation

`docs/PRD.md` PRD-INT-004 and `docs/TRD.md` Section 12 intentionally remain **Planned** at the complete-adapter level. That status is not contradicted by the bounded `tools/call` and `tools/list` foundations now on protected main: both are reusable control-plane contracts below the complete product adapter. `README.md` and `CHANGELOG.md` distinguish protected-main routing/discovery from the still-planned complete adapter, and ADR 0107 records the protocol/version and authority boundary.

The following remain outside protected main and PR #170 and must not be inferred from either foundation:

- Streamable HTTP transport parsing and header materialization;
- JSON-RPC/HTTP response serialization of the typed discovery page;
- OAuth and authenticated MCP deployment policy;
- browser-control I/O or BiDi/CDP/WebMCP translation;
- secret materialization or broker transport;
- persistence, durable audit storage, or WARC/PROV export;
- general pagination/subscription state beyond the fixed no-cursor catalog; and
- an OriginWeave Protocol version transition.

## Version boundary

The protected-main routing foundation and merged discovery refinement accept only protocol generation `2026-07-28`. MCP versioning is independent of the OriginWeave Protocol. A later MCP revision does not silently change OriginWeave action, risk, capability, approval, secret, origin, tenant, browser, or evidence semantics.

The reviewed primary source is:

Model Context Protocol. (2026, July 28). *Specification: 2026-07-28*. https://modelcontextprotocol.io/specification/2026-07-28

The canonical bibliography remains `docs/doctoring.md`.

## Executable evidence

Protected-main PR #168 production/test surfaces include:

- `crates/originweave-core/src/mcp.rs` — bounded deterministic catalog plus method/tool routing validation in the `ValidatedMcpToolCall` primitive;
- `crates/originweave-core/tests/mcp_authority_route.rs` — mapping, exact method/tool bounds, empty/oversized/malformed inputs, version/method/header-body correlation, and error-contract evidence;
- `crates/originweave-policy/src/lib.rs` — `evaluate_mcp` route/action guard before normal policy evaluation; and
- `crates/originweave-policy/tests/mcp_route_binding.rs` — confused-deputy and policy-preservation evidence.

PR #170's merged discovery contract is exercised in `crates/originweave-core/tests/mcp_tools_list_cache.rs`, including result/cache semantics, required protocol/client metadata, bounded protocol and method validation, routing correlation, cursor rejection, and public error contracts.

Exact current-head CI/security/review evidence must be regenerated after every branch mutation. Protected-main evidence proves only the merged typed routing/discovery foundations, not the complete adapter.

## Promotion rule

The bounded `tools/call` routing foundation and `tools/list` discovery refinement are `IMPLEMENTED_ON_PROTECTED_MAIN` through PRs #168 and #170. Neither makes the complete MCP adapter implemented; each remaining transport/runtime boundary requires its own integrated evidence.
