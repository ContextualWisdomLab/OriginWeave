# ADR 0102: Typed actions instead of arbitrary JavaScript authority

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

A browser agent can express nearly any behavior by evaluating arbitrary JavaScript, but that flexibility destroys the product's ability to reason about capability, risk, target origin, secret use, approval, resource cost, and provenance before execution. OriginWeave needs auditable action semantics that remain stable even when page content is adversarial and browser adapters change.

## Decision drivers

- State-changing behavior must be classifiable before execution.
- Policy must reason about exact capability, target, intent, and approval scope.
- Evidence should describe meaningful business actions rather than opaque script blobs.
- Web content is untrusted data and cannot become instruction authority.
- Compatibility escape hatches must not become routine privileged paths.

## Assumptions and authority boundaries

The policy engine receives typed requests from trusted control-plane code. Page-derived selectors, values, and observations remain untrusted parameters until validated. Browser adapters may translate a typed action into lower-level protocol calls, but the adapter cannot grant a new action kind or expand its declared authority. WebDriver BiDi and CDP provide browser operations; they do not define OriginWeave's business-risk, approval, tenant, or secret authority.

## Options considered

1. Permit arbitrary JavaScript for all automation: rejected because risk and provenance become opaque.
2. Maintain a deny-list of dangerous script operations: rejected because JavaScript is too expressive for a complete negative policy.
3. Expose typed actions with an exceptional, separately governed diagnostic script facility: selected.

## Decision

Product automation uses a versioned typed action API for observation, extraction, navigation, download, drafting, submission, upload, brokered secret fill, purchase, deletion, permission management, and future explicitly modeled actions. Arbitrary JavaScript is not an implicit fallback for failed typed actions. If a diagnostic or compatibility script facility is introduced, it is disabled for ordinary autonomous execution, separately capability-gated, origin-scoped, evidence-recorded, and prohibited from bypassing secret, approval, network, or tenant policy.

Adapters prefer browser mechanisms that preserve native user-facing semantics and post-condition observability. A low-level CDP `Runtime.evaluate` or BiDi script operation remains a protocol primitive, not permission for a planner/model to supply executable code. If a future typed action needs script-backed implementation internally, the script is product-owned, versioned, reviewed, parameterized through validated data, and bound to the same action/risk/post-condition contract.

## Consequences

New web behaviors sometimes require a new typed action or adapter capability instead of a one-line script. In return, risk classes, approvals, test coverage, compatibility, and evidence become deterministic. SDKs can provide stable contracts across browser backends.

## Failure and degraded behavior

When no typed action can express a requested effect, OriginWeave reports an unsupported capability or requires explicit human handling. It must not silently evaluate arbitrary JavaScript, inject a privileged extension script, or mark the task successful without post-condition evidence.

## Security / privacy / governance impact

Typed actions reduce prompt-injection and confused-deputy blast radius by constraining what model output can request. Secret delivery remains opaque-handle based. High-risk actions preserve governed approval, and evidence can record parameters after credential-safe redaction. Protocol script evaluation is treated as code execution within the browser context and remains outside ordinary autonomous model authority.

## Tests and acceptance evidence

Require a table-driven mapping from action kind to capability, mutability, risk, secret usage, and post-condition requirements; tests that web content cannot create trusted instructions; adapter tests proving no arbitrary-script fallback; hostile selector/value tests; tests that model/page strings cannot reach CDP/BiDi script-evaluation code paths as executable source; and end-to-end evidence that denied or unsupported actions cause no side effect.

## Migration and rollback

Migrate existing scripted flows action-by-action. During migration, legacy script paths remain explicitly marked experimental and cannot receive broader authority than the typed equivalent. Rollback may disable a newly introduced action but may not reactivate an unrestricted script fallback.

## Open follow-ups

Define extension/SDK APIs for registering future typed actions and determine whether a non-production diagnostic scripting surface is needed at all.

## Supersession / reversal conditions

Supersede only if a more expressive action representation preserves equivalent pre-execution risk classification, capability enforcement, secret isolation, approval semantics, provenance, and deterministic denial under adversarial tests.

## References

Chrome DevTools Protocol. (2026). *Runtime domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/Runtime/

World Wide Web Consortium. (2026, June 29). *WebDriver BiDi* [Working Draft]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260629/

## Related documents

See ADR 0002, `crates/originweave-core`, `crates/originweave-policy`, `docs/API_CONTRACT.md`, and `docs/THREAT_MODEL.md`.
