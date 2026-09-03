# MCP 2026-07-28 authority-route traceability

- **`tools/call` capability maturity:** `IMPLEMENTED_ON_PROTECTED_MAIN`
- **`tools/list` capability maturity:** `IMPLEMENTED_ON_PROTECTED_MAIN`
- **Protected-main owning work:** merged PR #168 (`tools/call`) and PR #170 (`tools/list`)
- **Active architecture repair:** PR #272 (`originweave-mcp` adapter boundary)
- **Complete MCP adapter status:** `PLANNED`
- **Governing decision:** ADR 0107

## Scope

Protected `main@c789b802fc98a8d7fd8c09d9327f36828054d2a1` contains the bounded Rust MCP `2026-07-28` routing and discovery foundation merged through PRs #168 and #170. That protected-main implementation bounds and syntax-validates attacker-controlled routing fields, maps only the explicit reviewed `originweave.*` catalog to existing typed `ActionKind` values, derives discovery metadata from that same catalog, and rejects route/action mismatch before ordinary deterministic policy evaluation.

A successful MCP routing value proves protocol integrity only. It grants no capability, origin, approval, secret, browser, tenant, persistence, network, evidence, or ambient execution authority. Browser and policy authority remain in their OriginWeave bounded contexts.

PR #272 is an active DDD repair that moves the external MCP protocol surface into `originweave-mcp` while preserving inward dependency direction: the adapter may consume stable core contracts and the protocol-independent policy API, but core and policy must not depend outward on MCP transport types. The move is active-PR evidence, not protected-main shipment.

## Final 2026-07-28 per-request envelope

The final MCP `2026-07-28` request envelope requires `io.modelcontextprotocol/protocolVersion` and `io.modelcontextprotocol/clientCapabilities` on each request. Client capabilities are per-request state and must not be inferred from earlier requests. `io.modelcontextprotocol/clientInfo` is optional/SHOULD self-reported metadata for display, logging, or debugging; OriginWeave does not use it as browser or policy authority.

Transport binding is explicit. Streamable HTTP requires the request protocol version to agree with the `MCP-Protocol-Version` header and keeps the reviewed routing-header correlation checks. Stdio carries the JSON-RPC request body without those HTTP routing headers, so a valid stdio request must be admitted from its required body metadata rather than from fabricated HTTP evidence.

PR #272 now exposes separate adapter entry points for those two cases. `ValidatedMcpToolCall::new_with_request_metadata` retains the HTTP header↔body checks. `ValidatedMcpToolCall::new_for_stdio` and `ValidatedMcpToolsListRequest::new_for_stdio` accept only the body protocol version, per-request capabilities-presence attestation, and body routing values. The stdio constructors reuse the same bounded syntax, catalog, and cursor validators internally, but they accept, retain, and expose no HTTP header value. Missing or unsupported protocol metadata, missing capabilities, malformed or unsupported methods, malformed or unknown tools, and unissued cursors remain fail-closed.

The lower routing validator and reviewed tool-to-`ActionKind` catalog remain internal implementation details of `originweave-mcp`; core and policy receive no MCP request-envelope types. Policy evaluation still consumes only the typed action contract after the adapter has established protocol integrity.

## Executable RED and repair lineage

Test-only head `bbe6b219a33f78e3b8b1c0166a00e5c34a2ede22` introduced `crates/originweave-mcp/tests/mcp_stdio_transport.rs` before production constructors existed. Repository-native CI run `33646560232` subsequently acquired hosted runners and produced an executable RED rather than a queue-only signal:

- Production coverage job `100302670895` failed with Rust `E0599` because `ValidatedMcpToolCall::new_for_stdio` and `ValidatedMcpToolsListRequest::new_for_stdio` did not exist. Six call sites in the stdio contract failed to compile.
- Rust contracts job `100302670660` first passed 154 Python repository-contract tests, then failed `cargo fmt --all --check`. Its canonical rustfmt artifact was `9875815906`, archive SHA-256 `bb5f01d2f6a90f22bc31a7ec34337691b982f73bf56f6064cc01ffb49c024cb6`.

The causal source repair is commit `09ffcccfd91d478120642a4db9bda501655e4533`. It adds only binding-specific stdio constructors inside `originweave-mcp` and adopts the canonical rustfmt output for files identified by the failed Rust-contract job. It does not move MCP transport authority into core or policy and does not infer browser authorization from protocol success.

This predecessor RED is durable evidence, but it is not current-head GREEN. Every later commit requires fresh exact-head CI, full owned-production coverage/rustdoc, security gates, and required central review workflows before promotion.

## Product-status reconciliation

`docs/PRD.md` PRD-INT-004 and the corresponding TRD complete-adapter work remain **Planned**. Protected-main routing/discovery contracts and PR #272's architecture repair are reusable control-plane slices below the complete product adapter. They do not establish a complete MCP server/runtime.

The following remain outside the protected-main bounded contract and must not be inferred from it:

- complete Streamable HTTP transport parsing and response serialization;
- complete stdio process/runtime framing beyond the request-envelope binding proved here;
- OAuth and authenticated MCP deployment policy;
- browser-control I/O or WebDriver BiDi/CDP translation;
- secret materialization or broker transport;
- persistence, durable audit storage, or WARC/PROV export;
- general pagination/subscription runtime beyond the currently reviewed contracts; and
- an OriginWeave Protocol version transition.

## Version and authority boundary

MCP versioning is independent of the OriginWeave Protocol. A later MCP revision does not silently change OriginWeave action, risk, capability, approval, secret, origin, tenant, browser, or evidence semantics. MCP routing metadata and optional client identity remain adapter data, not domain authority.

## Executable evidence

Protected-main production/test surfaces currently include the deterministic `tools/call`/`tools/list` routing and discovery contracts and protocol-independent policy evaluator. Active PR #272 relocates the external-protocol implementation to `crates/originweave-mcp/` and adds modern HTTP metadata validation plus explicit stdio binding tests for `tools/call` and `tools/list`.

Exact-current CI/security/review evidence must be regenerated after every branch mutation. Predecessor, protected-main, skipped, status-only, model, or cancelled evidence is not current-head proof for PR #272.

## Primary sources

Model Context Protocol. (2026, July 28). *The 2026-07-28 specification*. https://modelcontextprotocol.io/specification/2026-07-28

Model Context Protocol. (2026). *Supporting protocol revision 2026-07-28* [TypeScript SDK migration guide]. https://github.com/modelcontextprotocol/typescript-sdk/blob/main/docs/migration/support-2026-07-28.md

Model Context Protocol. (2026). *2026-07-28 protocol type definitions* [TypeScript source]. https://github.com/modelcontextprotocol/typescript-sdk/blob/main/packages/core-internal/src/types/spec.types.2026-07-28.ts

The canonical broader bibliography remains `docs/doctoring.md`.

## Promotion rule

The bounded `tools/call` and `tools/list` foundations are already `IMPLEMENTED_ON_PROTECTED_MAIN`. PR #272 may change the adapter architecture only after its exact current head proves repository-native CI, full owned-production coverage/rustdoc, security gates, required central workflows, and live review governance. Neither that promotion nor the existing protected-main contracts makes the complete MCP adapter implemented; each remaining transport/runtime boundary requires its own integrated evidence.
