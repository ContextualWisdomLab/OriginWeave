# Documentation Index

## Authoritative product documentation

- [Product Requirements Document (PRD)](PRD.md)
- [Technical Requirements Document (TRD)](TRD.md)
- [Architecture](../ARCHITECTURE.md)
- [Architecture Decision Record index](adr/README.md)
- [UML and control-flow diagrams](uml/README.md)
  - [Extension compatibility and Agent authority UML](uml/extension-authority.md)
- [Conceptual ERD and durable domain model](erd/README.md)
- [Data governance and privacy boundary](DATA_GOVERNANCE.md)
- [Product and decision traceability](traceability/README.md)
- [Documentation fitness assessment](DOCUMENTATION_FITNESS.md)
- [Threat model](THREAT_MODEL.md)
- [Product-wide test strategy](TEST_STRATEGY.md)
- [Operability and incident-response baseline](OPERABILITY.md)
- [OriginWeave API and protocol contract](API_CONTRACT.md)
- [Release and rollback contract](RELEASE_AND_ROLLBACK.md)
- [Product roadmap](product-roadmap.md)
- [Research and standards](doctoring.md)
  - [Browser and Agent protocol standards evidence](doctoring/browser-agent-protocols.md)
- [Current product-baseline standards addendum](doctoring/product-documentation-baseline.md)
- [Quality gates](quality-gates.md)
- [Security policy](../SECURITY.md)

The PRD/TRD/Architecture/ADR/UML/ERD/data-governance/traceability/security/operations/API/release set is the product-wide documentation graph. The documentation-fitness assessment records where that graph is current, stale, partial, or intentionally proposed. Feature-specific design specifications and plans below provide detailed implementation history but do not substitute for the product-wide baseline. Planned or conversation-derived capabilities must remain labelled Planned, Proposed, or Open until reviewed implementation evidence reaches protected `main`.

## Governance and maintenance

- [Destination registry maintenance](registry-maintenance.md)
- [Database naming](database-naming.md)
- [Agent/contributor rules](../AGENTS.md)
- [Contributing](../CONTRIBUTING.md)
- [Changelog](../CHANGELOG.md)

## Approved and historical design specifications

- [Approved safety-kernel design](superpowers/specs/2026-08-05-agent-safety-kernel-design.md)
- [Safety-kernel implementation plan](superpowers/plans/2026-08-05-agent-safety-kernel.md)
- [Resolved-destination policy design](superpowers/specs/2026-08-06-resolved-destination-policy-design.md)
- [Resolved-destination policy implementation plan](superpowers/plans/2026-08-06-resolved-destination-policy.md)
- [Direct socket binding design](superpowers/specs/2026-08-06-direct-socket-binding-design.md)
- [Direct socket binding implementation plan](superpowers/plans/2026-08-06-direct-socket-binding.md)
- [TLS service-identity design](superpowers/specs/2026-08-06-tls-server-identity-design.md)
- [TLS service-identity implementation plan](superpowers/plans/2026-08-06-tls-server-identity.md)

## Accepted protected-main architecture decisions

- [ADR 0001: Chromium compatibility kernel](adr/0001-chromium-compatibility-kernel.md)
- [ADR 0002: Agent safety kernel](adr/0002-agent-safety-kernel.md)
- [ADR 0003: Provenance-native observation](adr/0003-provenance-native-observation.md)
- [ADR 0004: Logical origin and resolved destination safety](adr/0004-resolved-destination-policy.md)
- [ADR 0005: Exact direct TCP peer binding](adr/0005-direct-socket-binding.md)
- [ADR 0006: TLS service identity over the verified peer](adr/0006-tls-server-identity.md)
- [ADR 0007: Purpose-bound sensitive-data authority](adr/0007-purpose-bound-sensitive-data-authority.md)
- [ADR 0008: Delegated-task TLS leaf-validity horizon](adr/0008-leaf-validity-horizon.md)
- [ADR 0010: Session/context-bound node authority](adr/0010-session-context-bound-node-authority.md)

## Proposed decisions retained on protected main

Proposed ADRs are reviewable architecture memory, not shipped behavior and not automatically Accepted merely because their files live on protected `main`.

- [ADR 0009: Hourly agent credential boundary](adr/0009-hourly-agent-credential-boundary.md)
- [ADR 0011: Manifest V3 compatibility and extension-to-Agent authority](adr/0011-manifest-v3-extension-authority.md)
- [ADR 0012: Architecture decision acceptance governance](adr/0012-architecture-decision-governance.md)
- [ADR 0100: Rust control-plane boundary](adr/0100-rust-control-plane-boundary.md)
- [ADR 0101: Isolated execution/profile modes](adr/0101-isolated-execution-profile-modes.md)
- [ADR 0102: Typed actions over arbitrary JavaScript](adr/0102-typed-actions-and-arbitrary-js.md)
- [ADR 0103: Semantic observation and stale-node identity](adr/0103-semantic-observation-and-stale-node-identity.md)
- [ADR 0104: Prompt-injection and secret authority separation](adr/0104-prompt-injection-and-secret-authority.md)
- [ADR 0105: Resource governor priority](adr/0105-resource-governor-priority.md)
- [ADR 0106: Provenance evidence model](adr/0106-provenance-evidence-model.md)
- [ADR 0107: Browser protocol adapter strategy](adr/0107-browser-protocol-adapter-strategy.md)
- [ADR 0108: Crawler policy](adr/0108-crawler-policy.md)
- [ADR 0109: Hourly automation secret ordering and operational closure](adr/0109-hourly-automation-operational-closure.md)

See the [ADR index](adr/README.md) for status rules, required decision structure, supersession rules, and active feature ADRs. The index and each ADR's own status metadata must agree; a PR body, chat transcript, automation prompt, or stale issue reference cannot change ADR status.