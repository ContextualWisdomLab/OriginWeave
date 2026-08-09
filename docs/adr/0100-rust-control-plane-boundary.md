# ADR 0100: Rust control-plane boundary

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

OriginWeave is intended to be a Chromium-compatible enterprise agentic web runtime, not a thin automation script collection. The Chromium compatibility kernel owns rendering and browser mechanics; OriginWeave must own the authority-bearing semantics that buyers rely on: execution mode, capabilities, origin and destination policy, typed actions, approvals, secret brokering, resource governance, provenance, and the versioned OriginWeave Protocol. Letting browser scripts or experimental DevTools domains become the policy authority would make those semantics unstable and difficult to audit.

## Decision drivers

- Memory-safe ownership for new authority-bearing product code.
- Stable semantics across Chromium and adapter upgrades.
- Deterministic, testable policy before side effects.
- Clear separation between browser evidence and product authority.
- A module model that can operate independently or as part of a larger enterprise control plane.

## Assumptions and authority boundaries

The Chromium compatibility kernel may be compromised or return malformed observations. Rust control-plane components are trusted only to the extent established by their own tests, provenance, and runtime isolation. Browser-originated DOM, accessibility, network, CDP, WebMCP, and visual data are evidence inputs. They do not grant capabilities or approvals.

## Options considered

1. Browser-extension or JavaScript-first control plane: rejected because arbitrary page-adjacent code is too close to untrusted content and unstable browser APIs.
2. Chromium-fork-only implementation: rejected because authority semantics would become tightly coupled to a large C++ codebase and downstream patch maintenance.
3. Rust control plane around Chromium compatibility kernel: selected.

## Decision

OriginWeave-owned authority-bearing behavior is implemented in a Rust control plane with narrow adapters to Chromium and external protocols. Adapters translate evidence and commands but cannot silently widen capability, origin, approval, secret, resource, or provenance authority. Arbitrary JavaScript is not a control-plane escape hatch. Experimental interfaces remain replaceable adapters behind versioned internal contracts.

## Consequences

The Rust crates become the durable product boundary and must maintain strict APIs, rustdoc, compatibility tests, and exact owned-code coverage. Some browser operations require additional adapter work instead of direct scripting. Chromium integration can evolve without redefining policy semantics.

## Failure and degraded behavior

If a required adapter is unavailable or incompatible, the governed capability fails closed. Human browsing may degrade independently where safe. The runtime must not switch to an ungoverned script path, discard provenance, or weaken approval requirements merely to preserve feature availability.

## Security / privacy / governance impact

Secret material, approval authority, tenant policy, and high-risk action decisions remain outside renderer/page authority. Rust boundaries reduce accidental memory-unsafety in OriginWeave-owned code but do not imply Chromium itself is memory-safe. Evidence must identify which component made each decision.

## Tests and acceptance evidence

Require crate-level API and property tests, adapter conformance tests, hostile-input tests, renderer-compromise simulations, exact 100% owned production coverage, rustdoc, Clippy, and end-to-end tests proving that adapter failure cannot bypass policy. Architecture documentation must map each authority to an owning module.

## Migration and rollback

Migrate browser-facing functionality behind Rust adapter traits before deprecating legacy paths. Rollback may restore a previous compatible adapter version, but cannot restore an arbitrary-script authority path or cross the secret/policy boundary.

## Open follow-ups

Define the stable internal adapter protocol and compatibility matrix; identify the minimum Chromium integration points that require downstream patches rather than external APIs.

## Supersession / reversal conditions

Supersede only if another implementation boundary demonstrates equal or better memory safety, auditability, browser compatibility, migration economics, and fail-closed authority semantics with production evidence.

## References

See ADR 0001, `ARCHITECTURE.md`, `docs/TRD.md`, and `docs/doctoring/product-documentation-baseline.md`.
