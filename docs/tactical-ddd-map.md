# OriginWeave Tactical DDD Map

This document classifies the tactical DDD building blocks that actually exist in OriginWeave. It is deliberately conservative: a type is not called an Entity, Aggregate, Repository, or Domain Event merely because that pattern may be useful later. Protected `main` is shipment truth; active PR #272 is identified separately; planned work is not promoted into the current model.

## Classification rules

- **Value Object**: immutable domain/security meaning is carried by validated values and equality, not by a mutable lifecycle identity.
- **Entity**: identity persists while domain state changes through an owned lifecycle.
- **Aggregate / Aggregate Root**: one root owns a consistency boundary and is the only mutation entry point for the aggregate invariant.
- **Domain Service**: protocol-independent domain logic that does not naturally belong to one Value Object or Entity.
- **Repository**: an inward-facing domain/application port for loading and persisting an Aggregate or other explicitly durable domain state. A database client, table, object-store SDK, or adapter is not itself the domain Repository contract.
- **Domain Event**: a domain-owned fact emitted after a domain state transition. External protocol events such as WebDriver BiDi notifications are integration events until an adapter translates and admits them into a domain-owned fact.
- **Invariant**: a condition that must remain true at the owning boundary and must fail closed when it cannot be established.

## Current tactical model

| Building block | Current code/API truth | Status | Required interpretation |
|---|---|---|---|
| Value Object | `originweave_core::Origin`, `BrowserSessionId`, `BrowsingContextId`, `DocumentEpoch`, `ObservedNodeHandle`, `ActionRequest`, `PolicyContext`, `ApprovalScope` | protected-main | Validated immutable authority/security values. Their identifiers do not by themselves imply mutable Entity semantics. |
| Value Object | `originweave_policy::SensitiveDataAuthority`, `SensitiveDataRequest`, `DisclosureScope`, `SensitiveValueHandleScope`, `HandleUseRequest`, `Decision`, `DisclosureDecision`, `HandleUseDecision` | protected-main | Purpose-bound authority and deterministic decision values; none stores protected field bytes or owns broker persistence. |
| Entity | No explicit mutable domain Entity is owned by protected-main core/policy or active PR #272 | current truth | `BrowserSessionId` and `BrowsingContextId` are identities, not Entities. Do not call a future browser process/session object an Entity until its lifecycle state and invariants are implemented behind an owned API. |
| Aggregate / Aggregate Root | No explicit Aggregate Root is implemented by protected-main core/policy or active PR #272 | current truth | The planned Browser Session bounded context may eventually own an Aggregate Root, but the name is not a shipment claim. Protocol adapters must not become substitute aggregate roots. |
| Domain Service | `originweave_policy::evaluate`, `evaluate_disclosure`, `evaluate_handle_use` | protected-main | Pure deterministic policy services. They perform no browser/network/storage I/O and return explicit fail-closed decisions. |
| Repository | No domain Repository contract is introduced by protected-main core/policy or active PR #272 | current truth | Durable WARC/PROV, tenant persistence, KMS/object storage, retention, legal hold, deletion, and offline replay remain issue #199 work. When persistence lands, inward repository ports must remain provider-neutral and storage adapters must depend inward on them. |
| Domain Event | No explicit OriginWeave domain-event type is introduced by protected-main core/policy or active PR #272 | current truth | WebDriver BiDi `browsingContext.navigationCommitted`, MCP requests, HTTP messages, and other wire events are adapter-owned integration events. They cannot be placed in core or treated as domain authority without translation, correlation, and invariant admission. |
| Invariant | Canonical origin admission, authority-bound observed-node reuse, deterministic action policy, purpose-bound sensitive-data scope, and fail-closed approval/secret rules | protected-main | Constructors/evaluators own these conditions. Callers cannot recover authority from a failed or ambiguous validation. |
| Invariant | MCP method/tool/action correlation and route/action equality | active PR #272 | This is adapter-owned protocol integrity. `McpRouteRejection` does not become a policy denial and the adapter delegates only an admitted typed action to protocol-independent policy. |

## Aggregate and Entity admission rule

The first real Chromium Agent Task vertical must not manufacture an Aggregate merely to satisfy a diagram. When browser-session lifecycle code reaches its owning bounded context, classify it as an Entity or Aggregate Root only if all of the following are true in code and tests:

1. a stable domain identity survives meaningful state transitions;
2. one owning API controls those transitions;
3. the consistency boundary is explicit, including session/context/origin/document-epoch relationships;
4. teardown and crash recovery close the lifecycle rather than leaving adapter-owned ambient state;
5. external BiDi/CDP/MCP identifiers remain ACL/integration values and cannot mutate the domain object directly;
6. tests prove stale epoch, wrong context, wrong origin, replay, teardown, and crash paths fail closed.

Until then, current identifiers and immutable authority values remain Value Objects.

## Domain-service boundary

`originweave-policy` is the current clearest Domain Service boundary. Its evaluators accept complete typed domain values and return deterministic decisions. They do not open sockets, call Chromium, parse MCP wire DTOs, resolve secrets, persist records, or invoke an LLM. Browser/network/secret/evidence adapters may consume a decision, but successful adapter I/O cannot reverse or widen that decision.

MCP remains an external protocol ACL. Active PR #272 may translate MCP method/tool vocabulary into an existing typed `ActionRequest` and call the published policy API. Core and policy must not import MCP, WebDriver BiDi, CDP, Chromium SDK, HTTP-client, provider, persistence, or UI DTOs.

## Repository and database truth

There is no OriginWeave-owned production database or domain Repository introduced by PR #272/#273. That absence is intentional and machine-visible; documentation must not draw repository boxes or ERD tables as if they were shipped.

Issue #199 owns the durable extraction/evidence direction. When that work introduces persistence:

- the domain/application Repository port is defined independently of PostgreSQL, object-storage, KMS, WARC transport, or cloud-provider SDK types;
- adapters implement the port and preserve tenant, retention, legal-hold, deletion, replay, and cryptographic-evidence invariants;
- database objects use multi-word `snake_case` names and represent only data owned by the bounded context;
- cross-context authoritative tables are not copied into OriginWeave and cross-service SQL is not used as an integration contract;
- `context-graph-contracts` and `enterprise-architecture-core` remain the owners of shared provenance/context/identity/architecture contracts when those shared contracts are required.

## Domain-event boundary

A protocol notification is not a Domain Event. In particular, WebDriver BiDi `browsingContext.navigationCommitted` is external evidence from a browser adapter. It becomes usable domain state only after the adapter correlates the event to the pinned session/context, establishes the canonical origin/document epoch transition, applies bounds, and invokes the owning domain/application contract. A command ACK is likewise not a post-condition event.

If OriginWeave later publishes domain events, each event must have an owning bounded context, typed payload, version, causation/correlation identity, explicit invariant that was established before emission, and tests proving that wire/protocol DTOs do not leak into the event contract. Shared event/provenance schemas should reuse released `context-graph-contracts` contracts rather than duplicate them locally.

## Fitness and traceability

`tests/test_ddd_documentation_contract.py` keeps this tactical classification explicit alongside `docs/context-map.md` and `docs/ubiquitous-language.md`. `tests/test_repository_contract.py` enforces the MCP dependency direction introduced by active PR #272. Rust unit/integration tests remain the authority for the constructors and deterministic evaluators named above; documentation tests do not substitute for production behavior, exact-head CI, coverage, security, or a real Chromium E2E.

Any future code change that introduces an Entity, Aggregate Root, Repository, or Domain Event must update this file, the Context Map, relevant ADR/API/ERD material, and executable fitness tests in the same change. Planned work must remain marked planned until it reaches protected `main`.