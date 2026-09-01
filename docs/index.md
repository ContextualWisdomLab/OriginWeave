# OriginWeave

OriginWeave is a Chromium-compatible, Rust-first control plane for governed AI agents on the web. It separates trusted instruction, untrusted web observation, protected secrets, destination authority, browser action authority, and evidence so an agent can browse and act without turning page content into ambient control.

> Status: pre-alpha. This page describes protected-default-branch product truth and intentionally does not promote active pull requests, queued checks, planned adapters, or unpublished release work to shipped capability.

## Start here

- [Repository overview](https://github.com/ContextualWisdomLab/OriginWeave#readme)
- [Architecture and trust boundaries](https://github.com/ContextualWisdomLab/OriginWeave/blob/main/ARCHITECTURE.md)
- [Product roadmap](product-roadmap.md)
- [Architecture decisions](adr/)
- [Product and technical gap baseline](product-technical-gap-baseline.md)
- [Repository releases](https://github.com/ContextualWisdomLab/OriginWeave/releases)
- [Ask DeepWiki](https://deepwiki.com/ContextualWisdomLab/OriginWeave)
- [Security policy](https://github.com/ContextualWisdomLab/OriginWeave/blob/main/SECURITY.md)
- [Contributing](https://github.com/ContextualWisdomLab/OriginWeave/blob/main/CONTRIBUTING.md)

## Product responsibility

OriginWeave owns governed browser-agent control contracts: browser-equivalent origin identity, fail-closed typed action policy, resolved-destination authorization, exact direct TCP peer binding, authenticated TLS service identity, bounded resource governance, and credential-safe evidence/provenance primitives. These foundations are independently reusable while the complete Chromium/BiDi/CDP/HTTP/proxy/persistence adapter surface remains subject to protected-main integration evidence.

The product does not treat successful parsing, DNS resolution, TCP connection, TLS authentication, browser protocol acknowledgement, or model output as interchangeable proof. Each boundary must preserve its own authority and evidence before a later layer can consume it.

## Safety model

OriginWeave treats page content and tool output as untrusted observations. They can contribute evidence, but they cannot grant capabilities, approve actions, rewrite policy, or request protected values. Destination admission is separate from name resolution; exact TCP peer evidence is separate from TLS identity; transport identity is separate from HTTP resource policy; and browser action acknowledgement is separate from an observed post-condition.

See the root architecture document and accepted ADRs for the binding contracts and reversal paths.

## Development and verification

The repository pins its supported Rust toolchain and verifies formatting, locked workspace checks, tests, strict Clippy, rustdoc, and exact owned-production coverage through protected CI. Current-head checks and counted reviews are integration evidence; predecessor-head, skipped, queued, model-only, or active-PR results are not treated as shipped product proof.

## Publication boundary

GitHub Pages availability is a repository-facing deployment state, not a property of this source file alone. This landing becomes a published product surface only after protected integration, Pages configuration/deployment, and live HTTPS content verification succeed.

## License

OriginWeave source is licensed under the [Apache License 2.0](https://github.com/ContextualWisdomLab/OriginWeave/blob/main/LICENSE). Third-party dependencies retain their own license obligations.
