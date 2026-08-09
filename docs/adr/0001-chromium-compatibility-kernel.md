# ADR 0001: Retain Chromium as the compatibility kernel

- Status: Accepted
- Date: 2026-08-05
- Supersedes: none
- Superseded by: none

## Context

OriginWeave needs modern web compatibility, JavaScript execution, graphics acceleration, process isolation, extension compatibility, accessibility semantics, and security updates at browser cadence. Reimplementing Blink, V8, the compositor, networking integration, and the extension runtime as a browser-engine rewrite would consume the product while producing materially worse compatibility and slower vulnerability response. The product differentiation is the governed Rust control plane, agent/runtime policy, evidence, resource governance, and protocol surface rather than a new rendering engine.

## Decision drivers

- Chromium-compatible rendering and JavaScript behavior must remain credible for enterprise web applications.
- Security updates must be consumable on upstream browser cadence.
- OriginWeave-owned authority boundaries should be memory-safe where practical and independently testable.
- Experimental browser surfaces must not become the sole product authority.
- The architecture must permit stock-Chromium validation before a maintained Chromium distribution is justified.

## Assumptions and authority boundaries

Chromium is an untrusted-complexity compatibility kernel, not the policy authority. Rust-owned OriginWeave components govern session mode, capabilities, navigation authority, typed actions, secrets, resource budgets, and evidence. Browser-provided DOM, accessibility, network, WebMCP, CDP, or extension observations are inputs to those policies rather than instructions that can grant authority.

## Options considered

1. **Full browser-engine rewrite.** Rejected because compatibility, staffing, security maintenance, and time-to-market costs are disproportionate to product differentiation.
2. **Unmodified browser driven only by arbitrary automation scripts.** Rejected because it does not create durable typed authority boundaries and makes product semantics depend on unstable external automation behavior.
3. **Chromium compatibility kernel plus Rust control plane.** Selected because it preserves web compatibility while keeping OriginWeave policy and evidence semantics independently owned.

## Decision

Chromium remains the compatibility kernel. New OriginWeave product behavior is implemented in Rust control-plane modules connected through narrow, validated, versioned adapters. Chromium patches are limited to integration points that cannot be supplied safely through WebDriver BiDi, stable or pinned CDP surfaces, Mojo, extension APIs, or an external process boundary. Any tip-of-tree or experimental browser interface remains behind an OriginWeave-owned adapter and cannot become the sole source of product authority.

Chromium's sandbox, multi-process model, and Site Isolation remain security boundaries that OriginWeave preserves rather than replacing. OriginWeave nevertheless assumes renderer compromise is possible and keeps privileged policy/secret/evidence decisions outside renderer authority. Chromium's Rust support and supported C++/Rust interoperability mechanisms are implementation inputs, not permission to move unvalidated browser types directly across the control-plane boundary.

## Consequences

- OriginWeave inherits Chromium's large upstream attack and maintenance surface and must track security releases.
- The project can validate product demand with stock Chromium before maintaining a downstream distribution.
- Manifest V3 and mainstream web compatibility remain achievable.
- Rust ownership and memory-safety guarantees apply to OriginWeave-owned control-plane modules, not every line of Chromium.
- Integration adapters become explicit compatibility contracts that require versioning and conformance tests.

## Failure and degraded behavior

If a Chromium upgrade breaks an adapter, OriginWeave must fail closed for the affected governed capability rather than bypass policy through arbitrary JavaScript or silently downgrade evidence. Read-only Human Mode may remain available when its own safety contract is unaffected. A browser-version rollback is acceptable only to a still-supported, security-reviewed artifact and must preserve evidence of the downgrade.

## Security / privacy / governance impact

Chromium compromise is modeled as a renderer/kernel threat and must not grant policy, secret-broker, tenant, or approval authority. Secrets are delivered through opaque handles and trusted broker paths rather than model-visible or page-visible raw values. Browser observations are untrusted data. Enterprise logging must distinguish browser evidence from OriginWeave policy decisions.

## Tests and acceptance evidence

Acceptance requires compatibility smoke tests against supported Chromium versions, adapter conformance tests, hostile renderer/input tests, session-isolation tests, security update rehearsal, and exact evidence that a broken/unsupported adapter fails closed. A proposal to replace Blink or V8 requires measured compatibility, security-update, staffing, migration, and benchmark evidence rather than architectural preference.

## Migration and rollback

Adapters are versioned so a Chromium update can be canaried and rolled back independently of OriginWeave protocol consumers. Rollback must pin an immutable known-good browser artifact, preserve the same Rust policy boundary, and record the artifact/version in provenance. No rollback may re-enable a deprecated arbitrary-script authority path.

## Open follow-ups

- Define the supported Chromium version window and update SLO.
- Complete conformance matrices for BiDi/CDP/WebMCP/extension adapters.
- Decide when commercial demand justifies maintaining a signed OriginWeave Chromium distribution.

## Supersession / reversal conditions

Supersede this ADR only if another engine or an OriginWeave-owned engine demonstrates equivalent target-web compatibility, materially better security/update economics, sustainable staffing, migration tooling, and buyer-visible benefit. A narrow embedded-engine experiment does not by itself reverse this decision.

## References

Chromium. (n.d.-a). *Getting started with Rust in Chromium*. Chromium source documentation. Retrieved August 9, 2026, from https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/rust/README.md

Chromium. (n.d.-b). *Process model and Site Isolation*. Chromium source documentation. Retrieved August 9, 2026, from https://chromium.googlesource.com/chromium/src.git/+/refs/heads/main/docs/process_model_and_site_isolation.md

Chromium. (n.d.-c). *Sandbox*. Chromium source documentation. Retrieved August 9, 2026, from https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md

Chromium. (n.d.-d). *Threat model and defenses against compromised renderers*. Chromium source documentation. Retrieved August 9, 2026, from https://chromium.googlesource.com/chromium/src.git/+/main/docs/security/compromised-renderers.md

Chrome DevTools Protocol. (2026). *Chrome DevTools Protocol — latest (tip-of-tree)*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/

World Wide Web Consortium. (2026, June 29). *WebDriver BiDi* [Working Draft]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260629/

## Related documents

See `docs/doctoring.md`, `docs/doctoring/product-documentation-baseline.md`, and ADR 0100 for current Rust/protocol evidence.
