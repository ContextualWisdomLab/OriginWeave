# Changelog

All notable changes to OriginWeave are documented in this file. The format follows Keep a Changelog, and releases use Semantic Versioning.

## [Unreleased]

### Added

- Rust workspace for independently reusable core, policy, resource, and evidence modules.
- Normalized HTTPS and loopback-origin boundary with explicit malformed-input errors.
- Typed browser actions, capabilities, risk classes, execution modes, approval scopes, robots decisions, and secret-delivery contracts.
- Deterministic fail-closed policy evaluation for untrusted instructions, origin grants, crawler restrictions, approvals, and brokered secrets.
- Interactive-first RAM, VRAM, batch, local-model, and compositor-pressure directives.
- Credential-redacted network evidence and SHA-256-bound provenance records.
- Rust 1.97.1 build contract, strict Clippy and rustdoc gates, and exact production function, line, region, and branch coverage enforcement.
- Architecture, agent, security, contribution, research, database naming, roadmap, and quality-gate documentation.

### Security

- Raw page content cannot become a trusted instruction.
- Raw secrets are rejected and secret-capable actions require an opaque broker handle.
- Crawler mode is read-only and public crawling fails closed without an applicable robots-policy decision.
- R3 and R4 approvals are bound to the exact action and target origin; R5 legal consent is non-delegable.

[Unreleased]: https://github.com/ContextualWisdomLab/OriginWeave/compare/main...HEAD
