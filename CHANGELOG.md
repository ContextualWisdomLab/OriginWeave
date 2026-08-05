# Changelog

All notable changes to OriginWeave are documented in this file. The format follows Keep a Changelog, and releases use Semantic Versioning.

## [Unreleased]

### Added

- Rust workspace for independently reusable core, policy, resource, and evidence modules.
- Canonical HTTPS and loopback-origin boundary with case-normalized schemes and hosts, default-port normalization, IPv4/IPv6 handling, and explicit malformed-input errors.
- Typed browser actions, capabilities, risk classes, execution modes, approval scopes, robots decisions, and secret-delivery contracts.
- Deterministic fail-closed policy evaluation for untrusted instructions, origin grants, crawler restrictions, execution-mode and purpose consistency, approvals, and brokered secrets.
- Interactive-first RAM, VRAM, batch, local-model, and compositor-pressure directives, including fail-closed handling at exact hard and soft limits.
- Default-redacted network evidence that preserves only allowlisted protocol metadata, redacts every query value, and binds provenance to validated source URLs and lowercase SHA-256 identifiers.
- Rust 1.97.1 build contract, strict Clippy and rustdoc gates, and exact production function, line, region, and branch coverage enforcement.
- Hourly bounded OpenCode product-development workflow using `NVIDIA_NIM_API_KEY`, an unprivileged disposable workspace, loopback-only credential broker, independently verified patches, and PR-only publication.
- Architecture, agent, security, contribution, research, database naming, roadmap, and quality-gate documentation.

### Security

- Raw page content cannot become a trusted instruction.
- Raw secrets are rejected and secret-capable actions require an opaque broker handle.
- Crawler mode is read-only, must pair with the public-crawl purpose, and fails closed without an applicable robots-policy decision.
- State-changing actions are same-origin by default.
- R3 and R4 approvals are bound to the exact action and target origin; R5 legal consent is non-delegable.
- Unknown network header values and every query value are redacted by default.
- Evidence capture rejects credential-bearing source URLs, query strings, fragments, control characters, and ambiguous backslash paths.

[Unreleased]: https://github.com/ContextualWisdomLab/OriginWeave/compare/main...HEAD
