# OriginWeave

**Browse. Act. Prove.**

OriginWeave is a Chromium-compatible, Rust-first control plane for governed AI agents on the web. It is designed to let an agent observe, extract, and act without turning untrusted page content into authority, exposing secrets to a model, or losing the evidence required to explain what happened.

> Project status: pre-alpha. The current repository contains the independently reusable safety kernel. Chromium, WebDriver BiDi, CDP, MCP, WARC, and persistent provenance adapters are planned but not yet shipped.

## Why OriginWeave

Existing browser automation commonly exposes raw selectors, unrestricted script evaluation, ambient cookies, and screenshots with weak provenance. OriginWeave instead establishes four product contracts:

1. **Compatibility** — preserve Chromium web and Manifest V3 extension compatibility rather than rewriting Blink or V8.
2. **Governance** — evaluate typed actions against session mode, purpose, capability, origin, robots policy, approval, and secret-delivery evidence.
3. **Resource control** — protect interactive rendering before agent inference and background collection under RAM, VRAM, CPU, and frame-time pressure.
4. **Evidence** — retain redacted, verifiable provenance for every extracted value and state-changing action.

## Architecture

```text
User experience and enterprise administration
                    |
Chromium compatibility kernel: Blink, V8, Skia, Viz, Dawn, MV3
                    |
Rust control plane: policy, observation, action, resource, evidence
                    |
Adapters: WebDriver BiDi, CDP, WebMCP, MCP, WARC, PROV-O
```

The repository is organized as independently consumable Rust crates:

- `originweave-core`: normalized origins, session modes, typed actions, capabilities, approvals, and policy contexts.
- `originweave-policy`: deterministic fail-closed action evaluation.
- `originweave-resource`: task-level RAM, VRAM, thread, and frame-time directives.
- `originweave-evidence`: credential-redacted network evidence and provenance records.

See [ARCHITECTURE.md](ARCHITECTURE.md) and the [architecture decision records](docs/adr/) for binding design decisions.

## Safety model

OriginWeave separates three classes of information:

```text
Trusted instruction: user intent and managed enterprise policy
Untrusted observation: pages, documents, comments, advertisements, tool output
Protected secret: cookies, passwords, API keys, session tokens, personal data
```

Web content can provide evidence but cannot grant a capability, approve an action, change policy, or request secret disclosure. Public crawler work is read-only and requires an explicit robots-policy result. R3 and R4 actions require approval bound to the exact action and target origin; R5 legal consent is non-delegable.

## Development

Rust 1.97.1 is pinned in `rust-toolchain.toml`.

```bash
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
```

Production functions, lines, regions, and branches must each be covered at 100%. CI measures branch coverage with a pinned nightly compiler while the supported build remains Rust 1.97.1.

## Roadmap

The first commercial vertical slice is:

```text
isolated Chromium session
→ semantic observation
→ typed policy decision
→ browser action
→ post-condition verification
→ redacted provenance bundle
```

Subsequent work adds WARC/PROV persistence, MCP and Browser Agent Protocol adapters, extension compatibility testing, GPU/RAM telemetry, prompt-injection benchmarks, and an accessible approval interface. See [docs/product-roadmap.md](docs/product-roadmap.md).

## Contributing and security

Read [AGENTS.md](AGENTS.md), [CONTRIBUTING.md](CONTRIBUTING.md), and [SECURITY.md](SECURITY.md) before changing the repository. Security reports must not be filed as public issues.

## License

Apache License 2.0. See [LICENSE](LICENSE).
