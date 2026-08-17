# MCP 2026-07-28 authority-route traceability

- **Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`
- **Owning work:** PR #168 `feat/mcp): bind stateless tool routing to typed actions`
- **Protected-main status:** non-shipped active-PR evidence
- **Complete MCP adapter status:** `PLANNED`
- **Governing decision:** ADR 0107

## Scope

PR #168 implements a bounded Rust control-plane foundation for MCP `2026-07-28` `tools/call` routing. It validates the represented stateless routing envelope, bounds and syntax-validates both attacker-controlled tool-name fields before correlation, maps only an explicit reviewed `originweave.*` catalog to existing typed `ActionKind` values, derives discovery metadata from the same catalog, and rejects route/action mismatch before ordinary deterministic policy evaluation.

A successful `ValidatedMcpToolCall` proves routing integrity only. It grants no capability, origin, approval, secret, browser, tenant, persistence, network, or evidence authority. `originweave_policy::evaluate_mcp` still delegates to the ordinary policy evaluator after the route/action match.

## Product-status reconciliation

`docs/PRD.md` PRD-INT-004 and `docs/TRD.md` Section 12 intentionally remain **Planned** at the complete-adapter level. That status is not contradicted by this active PR: the PR implements only a reusable routing/action-policy foundation below the product adapter. `README.md` and `CHANGELOG.md` therefore distinguish the active foundation from shipped protected-main capability, and ADR 0107 records the same version and authority boundary.

The following remain outside PR #168 and must not be inferred from it:

- Streamable HTTP transport parsing and header materialization;
- complete request `_meta` validation, including per-request client capabilities;
- `tools/list` serialization, pagination, cache semantics, and subscription handling;
- OAuth and authenticated MCP deployment policy;
- browser-control I/O or BiDi/CDP/WebMCP translation;
- secret materialization or broker transport;
- persistence, durable audit storage, or WARC/PROV export; and
- an OriginWeave Protocol version transition.

## Version boundary

The active routing foundation accepts only protocol generation `2026-07-28`. MCP versioning is independent of the OriginWeave Protocol. A later MCP revision does not silently change OriginWeave action, risk, capability, approval, secret, origin, tenant, browser, or evidence semantics.

The reviewed primary source is:

Model Context Protocol. (2026, July 28). *Specification: 2026-07-28*. https://modelcontextprotocol.io/specification/2026-07-28

The canonical bibliography remains `docs/doctoring.md`.

## Executable evidence

Current PR #168 production/test surfaces include:

- `crates/originweave-core/src/mcp.rs` — bounded deterministic catalog and `ValidatedMcpToolCall` routing primitive;
- `crates/originweave-core/tests/mcp_authority_route.rs` — mapping, bounds, malformed-input, version/method/header-body, and error-contract evidence;
- `crates/originweave-policy/src/lib.rs` — `evaluate_mcp` route/action guard before normal policy evaluation; and
- `crates/originweave-policy/tests/mcp_route_binding.rs` — confused-deputy and policy-preservation evidence.

Exact current-head CI/security/review evidence must be regenerated after every branch mutation. Predecessor-head success is historical only.

## Promotion rule

This dossier may change to `IMPLEMENTED_ON_PROTECTED_MAIN` for the bounded routing foundation only after PR #168 reaches protected `main` under live governance and exact-head acceptance. That promotion still does **not** promote the complete MCP adapter from `PLANNED`; each remaining transport/runtime boundary requires its own integrated evidence.
