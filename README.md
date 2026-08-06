# OriginWeave

**Browse. Act. Prove.**

OriginWeave is a Chromium-compatible, Rust-first control plane for governed AI agents on the web. It is designed to let an agent observe, extract, and act without turning untrusted page content into authority, exposing secrets to a model, connecting to an unapproved network destination, or losing the evidence required to explain what happened.

> Project status: pre-alpha. The current repository contains independently reusable safety, resolved-destination, and direct TCP peer-binding kernels. Chromium, WebDriver BiDi, CDP, MCP, TLS, HTTP, proxy, WARC, and persistent provenance adapters are planned but not yet shipped.

## Why OriginWeave

Existing browser automation commonly exposes raw selectors, unrestricted script evaluation, ambient cookies, implicit resolver authority, and screenshots with weak provenance. OriginWeave instead establishes six product contracts:

1. **Compatibility** — preserve Chromium web and Manifest V3 extension compatibility rather than rewriting Blink or V8.
2. **Governance** — evaluate typed actions against session mode, purpose, capability, browser-equivalent origin, robots policy, secret-delivery evidence, and approval bound to the complete action intent.
3. **Destination safety** — classify resolved addresses, bind a non-empty and bounded approved address set to each origin, preserve `localhost` and literal-IP host semantics, reject DNS-rebinding through set expansion, and reauthorize every redirect target.
4. **Transport proof** — consume a single-use direct connection plan, submit one exact canonical socket address to the operating system, and verify the observed TCP peer before exposing the stream.
5. **Resource control** — protect interactive rendering before agent inference and background collection with cumulative RAM, VRAM, admission, model-offload, batch, and frame-pressure mitigations.
6. **Evidence** — retain bounded, universally value-redacted network metadata and verifiable provenance for extracted values and state-changing actions.

## Architecture

```text
User experience and enterprise administration
                    |
Chromium compatibility kernel: Blink, V8, Skia, Viz, Dawn, MV3
                    |
Rust control plane: policy, destination, network, observation, action, resource, evidence
                    |
Adapters: TLS, HTTP, proxy/PAC, WebDriver BiDi, CDP, WebMCP, MCP, WARC, PROV-O
```

The repository is organized as independently consumable Rust crates:

- `originweave-core`: normalized origins, immutable action-intent digests, session modes, typed actions, capabilities, approvals, and policy contexts.
- `originweave-policy`: deterministic fail-closed action evaluation.
- `originweave-destination`: address classification, explicit destination policy, origin-bound DNS snapshots, connection pinning, rebinding detection, and redirect reauthorization.
- `originweave-network`: direct-only, single-use TCP connection plans that bind an approved canonical address to the exact operating-system peer and emit credential-free evidence.
- `originweave-resource`: task-level RAM, VRAM, thread, and frame-time budgets with cumulative mitigation plans.
- `originweave-evidence`: universally value-redacted network evidence and source-bound provenance records.

See [ARCHITECTURE.md](ARCHITECTURE.md) and the [architecture decision records](docs/adr/) for binding design decisions.

## Safety model

OriginWeave separates three classes of information:

```text
Trusted instruction: user intent and managed enterprise policy
Untrusted observation: pages, documents, comments, advertisements, tool output
Protected secret: cookies, passwords, API keys, session tokens, personal data
```

Web content can provide evidence but cannot grant a capability, approve an action, change policy, or request secret disclosure. Public crawler work is read-only and requires an explicit robots-policy result. R3 and R4 actions require approval bound to the exact action kind, target origin, and immutable lowercase SHA-256 digest of the complete canonical action intent; R5 legal consent is non-delegable.

The origin type rejects shortened, integer, hexadecimal, and legacy octal-looking IPv4 spellings that a browser could reinterpret differently from a DNS validator. This protects logical origin identity. The destination kernel separately canonicalizes IPv4-mapped IPv6, classifies IPv4 and IPv6 special-purpose ranges and reviewed cloud platform endpoints, permits only public destinations by default, accepts at most 256 resolver addresses per snapshot, pins approved DNS address sets, rejects set expansion, and re-evaluates origin, resolution, downgrade, cycle, and hop authority for each redirect. The special `localhost` name may resolve only to loopback, and a literal IPv4 or IPv6 origin may approve only its exact canonical address.

The direct network kernel then accepts one explicit canonical `SocketAddr`, rejects port zero and unbounded timeout or retry requests, consumes a non-cloneable plan, calls the standard-library single-address timeout API, and checks the stream's reported peer. It releases the stream only when the requested and observed IP and port match exactly. This proves the exact operating-system peer without resolving a hostname again or inheriting proxy configuration.

Safe Chromium navigation still requires adapters that route the browser's real socket through these contracts, validate TLS server identity, define proxy and PAC authority, apply HTTP connection and response budgets, validate downloads and observed MIME, and record complete transport evidence. A syntactically valid origin is never treated as an SSRF decision, and TCP peer equality is never presented as TLS identity.

Generic network evidence retains bounded field names but no header or query values. Every value is replaced before the record leaves the trusted boundary, and malformed, ambiguous, or excessive paths and metadata fail closed. Any future typed response value or body requires a separate schema-specific capture policy.

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
→ origin grant and resolved-destination authorization
→ exact direct TCP peer binding
→ TLS and HTTP authority
→ semantic observation
→ typed policy decision
→ browser action
→ post-condition verification
→ redacted provenance bundle
```

Subsequent work connects the live Chromium network service, adds explicit proxy and download policy, WARC/PROV persistence, MCP and Browser Agent Protocol adapters, extension compatibility testing, GPU/RAM telemetry, prompt-injection benchmarks, and an accessible approval interface. See [docs/product-roadmap.md](docs/product-roadmap.md).

## Hourly product-development loop

The repository defines an hourly bounded OpenCode workflow. It runs only when no PR or release blocker is open, calls models through a loopback-only broker backed by `NVIDIA_NIM_API_KEY`, gives the agent no Git or GitHub authority, restricts the unprivileged agent user to loopback network egress, and keeps generated build outputs outside the proposed source tree. It seals the permitted source edits in a deterministic change bundle, independently reapplies and verifies that exact bundle, and uses a dedicated `OPENCODE_PR_TOKEN` only to publish one verified PR. It never uses `COPILOT_GITHUB_TOKEN`, review credentials, merge credentials, or self-approval. Organization-level PR maintenance remains the independent review and merge authority.

## Contributing and security

Read [AGENTS.md](AGENTS.md), [CONTRIBUTING.md](CONTRIBUTING.md), and [SECURITY.md](SECURITY.md) before changing the repository. Security reports must not be filed as public issues.

## License

Apache License 2.0. See [LICENSE](LICENSE).
