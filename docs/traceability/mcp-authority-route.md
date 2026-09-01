# MCP 2026-07-28 authority-route traceability

- **`tools/call` capability maturity:** `IMPLEMENTED_ON_PROTECTED_MAIN`
- **`tools/list` capability maturity:** `IMPLEMENTED_ON_PROTECTED_MAIN`
- **Protected-main owning work:** merged PR #168 (`tools/call`) and merged PR #170 (`tools/list`)
- **Bounded-context correction:** active PR #272 moves MCP protocol contracts from `originweave-core` to `originweave-mcp` and makes route-integrity rejection adapter-owned rather than policy-shaped
- **Complete MCP adapter status:** `PLANNED`
- **Governing decision:** ADR 0107

## Scope

Protected main at `542ca1e9c0a863595b8b6697790005d2471f5413` contains the bounded Rust control-plane foundations for MCP `2026-07-28` `tools/call` routing and conservative `tools/list` discovery. PR #168 established stateless route validation and mapping to existing typed `ActionKind` values. PR #170, merged on 2026-08-27 as `c4e127036e75cd3c5682ef15b69fb9ec29ff1dd2`, added the fixed discovery page and request metadata validation.

The protected-main implementation validates represented stateless routing envelopes, bounds and syntax-validates attacker-controlled method and tool-name fields before correlation, maps only the reviewed `originweave.*` catalog to existing typed actions, derives discovery metadata from the same catalog, and rejects route/action mismatch before ordinary deterministic policy evaluation. Methods are nonempty reviewed-ASCII routing tokens of at most 64 bytes; tool names are nonempty reviewed-ASCII identifiers of at most 128 bytes. Invalid method metadata remains distinct from a bounded but unsupported MCP method.

A successful `ValidatedMcpToolCall` proves routing integrity only. It grants no capability, origin, approval, secret, browser, tenant, persistence, network, or evidence authority. On protected main the route/action bridge still resides in `originweave-policy`. Active PR #272 moves that protocol-specific bridge to `originweave-mcp`: `originweave_mcp::evaluate_mcp` returns `McpRouteRejection::ActionMismatch` for a route/request disagreement and calls the protocol-independent `originweave_policy::evaluate` only after route/action equality. The adapter rejection is not a policy `DenialReason` and cannot be reinterpreted as authorization.

The protected-main `tools/list` contract requires matching MCP protocol metadata, required client-capability presence, bounded and syntax-validated routing/body methods, exact `tools/list` routing, and no caller-supplied cursor because the fixed catalog issues none. Its result is one complete page with zero freshness, private cache scope, and no continuation cursor. Discovery metadata grants no OriginWeave action authority.

## Bounded-context correction

Protected main still places the MCP protocol implementation under `crates/originweave-core/src/mcp.rs`. That placement conflicts with the architecture rule that stable shared domain/security contracts remain free of external protocol DTOs and adapters. Active PR #272 is the canonical DDD repair for that ownership drift:

- introduces `crates/originweave-mcp` as the MCP protocol-adapter bounded context;
- moves the existing routing/discovery implementation while preserving fail-closed routing behavior;
- moves protocol-boundary tests and the MCP-to-policy binding tests with the adapter;
- keeps `originweave-core` as the stable shared action/authority vocabulary;
- keeps `originweave-policy` protocol-independent and dependent only on `originweave-core`;
- keeps route/action mismatch in the MCP adapter as `McpRouteRejection::ActionMismatch` rather than an MCP-specific policy denial;
- makes `originweave-mcp` depend inward on the stable core vocabulary and the published policy evaluator; and
- adds a machine-checkable repository fitness test preventing MCP protocol or adapter vocabulary from returning to core or policy.

PR #272 is active-PR evidence, not protected-main behavior. Until it merges, the protected-main source paths below remain the shipped ownership truth even though the architectural defect is known.

## Product-status reconciliation

`docs/PRD.md` PRD-INT-004 and `docs/TRD.md` Section 12 intentionally remain **Planned** at the complete-adapter level. That status is not contradicted by the protected-main `tools/call` and `tools/list` foundations: both are reusable protocol contracts below the complete product adapter.

The following remain outside the complete protected-main adapter and must not be inferred from the routing/discovery foundations or PR #272:

- Streamable HTTP transport parsing and header materialization;
- JSON-RPC/HTTP response serialization of the typed discovery page;
- OAuth and authenticated MCP deployment policy;
- browser-control I/O or BiDi/CDP/WebMCP translation;
- secret materialization or broker transport;
- persistence, durable audit storage, or WARC/PROV export;
- general pagination/subscription state beyond the fixed no-cursor catalog; and
- an OriginWeave Protocol version transition.

## Version boundary

The routing and discovery foundations accept only protocol generation `2026-07-28`. MCP versioning is independent of the OriginWeave Protocol. A later MCP revision does not silently change OriginWeave action, risk, capability, approval, secret, origin, tenant, browser, or evidence semantics.

The reviewed primary source is:

Model Context Protocol. (2026, July 28). *Specification: 2026-07-28*. https://modelcontextprotocol.io/specification/2026-07-28

The canonical bibliography remains `docs/doctoring.md`.

## Executable evidence

Protected-main production/test surfaces before PR #272 are:

- `crates/originweave-core/src/mcp.rs` — bounded deterministic catalog plus `tools/call` and `tools/list` protocol validation;
- `crates/originweave-core/tests/mcp_authority_route.rs` — action mapping, method/tool bounds, malformed inputs, version/method/header-body correlation, and error contracts;
- `crates/originweave-core/tests/mcp_tools_list_cache.rs` — fixed discovery result/cache semantics, protocol/client metadata, method validation, routing correlation, cursor rejection, and public errors;
- `crates/originweave-policy/src/lib.rs` — protected-main `evaluate_mcp` route/action guard before normal policy evaluation; and
- `crates/originweave-policy/tests/mcp_route_binding.rs` — protected-main confused-deputy and policy-preservation evidence.

On active PR #272 the protocol implementation and protocol-owned evidence move to:

- `crates/originweave-mcp/src/routing.rs`;
- `crates/originweave-mcp/src/lib.rs` — adapter-owned `McpRouteRejection` plus `evaluate_mcp`, which delegates to `originweave_policy::evaluate` only after route/action equality;
- `crates/originweave-mcp/tests/mcp_authority_route.rs`;
- `crates/originweave-mcp/tests/mcp_tools_list_cache.rs`; and
- `crates/originweave-mcp/tests/policy_route_binding.rs`.

The repository fitness contract in `tests/test_repository_contract.py` verifies the new package boundary, prevents `originweave-core/src/mcp.rs` from reappearing, requires `originweave-mcp` to depend inward on `originweave-core` and `originweave-policy`, requires `originweave-policy` to depend only on core, and rejects MCP adapter vocabulary such as `McpRouteRejection` or other `Mcp`-specific concepts from the policy source. `tests/test_ddd_documentation_contract.py` binds the Context Map, Ubiquitous Language, and this traceability record to the same ownership split.

Exact current-head CI/security/review evidence must be regenerated after every branch mutation. Protected-main evidence proves only the integrated foundations; predecessor or protected-main results are not current-head proof for PR #272.

## Promotion rule

The bounded `tools/call` and `tools/list` foundations are already `IMPLEMENTED_ON_PROTECTED_MAIN`. The ownership correction may be described as protected-main architecture only after PR #272 reaches protected `main` under live governance and exact-head acceptance. That promotion still does not make the complete MCP server implemented; each remaining transport/runtime boundary requires its own integrated evidence.
