# Documentation Index

## Authoritative product documentation

- [Product Requirements Document (PRD)](PRD.md)
- [Technical Requirements Document (TRD)](TRD.md)
- [Architecture](../ARCHITECTURE.md)
- [Architecture Decision Record index](adr/README.md)
- [UML and control-flow diagrams](uml/README.md)
- [Conceptual ERD and durable domain model](erd/README.md)
- [Product and decision traceability](traceability/README.md)
- [Threat model](THREAT_MODEL.md)
- [Product-wide test strategy](TEST_STRATEGY.md)
- [Operability and incident-response baseline](OPERABILITY.md)
- [OriginWeave API and protocol contract](API_CONTRACT.md)
- [Release and rollback contract](RELEASE_AND_ROLLBACK.md)
- [Product roadmap](product-roadmap.md)
- [Research and standards](doctoring.md)
- [Current product-baseline standards addendum](doctoring/product-documentation-baseline.md)
- [Quality gates](quality-gates.md)
- [Security policy](../SECURITY.md)

The PRD/TRD/Architecture/ADR/UML/ERD/traceability/security/operations/API/release set is the product-wide documentation graph. Feature-specific design specifications and plans below provide detailed implementation history but do not substitute for the product-wide baseline. Planned or conversation-derived capabilities must remain labelled Planned, Proposed, or Open until reviewed implementation evidence reaches protected `main`.

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

## Protected-main architecture decisions

- [ADR 0001: Chromium compatibility kernel](adr/0001-chromium-compatibility-kernel.md)
- [ADR 0002: Agent safety kernel](adr/0002-agent-safety-kernel.md)
- [ADR 0003: Provenance-native observation](adr/0003-provenance-native-observation.md)
- [ADR 0004: Logical origin and resolved destination safety](adr/0004-resolved-destination-policy.md)
- [ADR 0005: Exact direct TCP peer binding](adr/0005-direct-socket-binding.md)
- [ADR 0006: TLS service identity over the verified peer](adr/0006-tls-server-identity.md)

See the [ADR index](adr/README.md) for status rules, required decision structure, and the rule that active-PR ADRs do not become Accepted merely because they exist on an unmerged branch.
