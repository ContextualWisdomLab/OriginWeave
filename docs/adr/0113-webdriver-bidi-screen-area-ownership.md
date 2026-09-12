# ADR 0113: WebDriver BiDi screen-area ownership witness

- **Status:** Proposed
- **Date:** 2026-09-10
- **Supersedes:** none
- **Superseded by:** none
- **Refines:** ADR 0107

## Context

ADR 0107 keeps WebDriver BiDi behind a versioned adapter and requires owned cleanup for presentation overrides. PR #310 exposed `emulation.setScreenSettingsOverride` as an explicit partial intent while correctly excluding it from the reusable profile-derived plan because one rectangle changes both total and available screen geometry.

A second authority problem is independent of that schema gap. WebDriver BiDi stores the screen-area override against a browsing context. Setting a non-null rectangle replaces the target entry; sending `screenArea: null` removes the target entry. The standard does not restore a predecessor override. A `WebDriverBidiBrowsingContext` therefore identifies a mutation target but cannot prove that OriginWeave owns the state being replaced or cleared.

The first ownership-witness implementation retained public explicit planner functions while intentionally exposing no Browser Session witness-mint path. Exact-head CI `34419810636` made that contradiction executable: Python repository contracts, formatting, and locked workspace tests passed, but strict Clippy rejected both planners as dead production code. Exact production coverage passed separately. A callable planner API with no legal production caller is not a deferred capability; it is unreachable surface area that obscures the lifecycle boundary.

## Decision drivers

- Preserve the useful typed WebDriver BiDi screen-area vocabulary without granting ambient mutation authority.
- Prevent a raw browsing-context identifier from authorizing replacement or removal of another owner's override.
- Keep cleanup evidence causal: ownership must exist before the destructive mutation, not be inferred from a later command acknowledgement.
- Do not suppress `dead_code` or retain unreachable public helpers merely to advertise a future capability.
- Keep the reusable profile-derived planner limited to observables represented by the profile and paired with safe cleanup semantics.
- Keep complete Screen admission fail-closed while available-screen geometry and color depth remain uncontrolled.

## Assumptions and authority boundaries

- Browser-domain and Browser Session lifecycle authority remain in OriginWeave.
- WebDriver BiDi remains an adapter; protocol addressability is not product authorization.
- The runtime-qualified 18 August 2026 Working Draft pin remains unchanged until a separate compatibility change proves a newer revision.
- `WebDriverBidiScreenArea` remains the typed width/height representation of the protocol's coupled total/available-area rectangle.
- This slice has no authoritative predecessor-state snapshot and does not invent one.
- A command acknowledgement is not page-observed application, ownership evidence, cleanup evidence, or restoration evidence.
- Screen-area mutation may become executable only after Browser Session proves an exclusive/disposable browsing context or an equivalent restoration-safe lifecycle.

## Options considered

1. **Keep context-only Set/Reset planners.** Rejected. Any caller able to supply a valid remote context identifier could replace or delete screen-settings state without proving ownership.
2. **Delete screen-area support.** Rejected. The standard capability is useful and can be represented without granting ambient mutation authority.
3. **Capture and restore an assumed predecessor value.** Rejected. This slice has no authoritative predecessor snapshot and the standard reset semantics remove the override rather than restore one.
4. **Treat a successful Set command as ownership proof.** Rejected. The Set can already have overwritten another owner's state; acknowledgement is too late to establish authorization.
5. **Keep public explicit planners that accept an opaque witness even though no production mint path exists.** Rejected by executable evidence. Exact-head strict Clippy identified both helpers as dead code; suppressing the warning would preserve an API that no legal caller can reach.
6. **Retain the typed command/witness vocabulary but expose no screen-area planner until Browser Session can mint the witness.** Selected. The protocol semantics remain represented, while executable authority appears only when the lifecycle owner supplies a reviewed mint transition and can consume the witness without reopening raw-context authority.

## Decision

`originweave-bidi` retains `WebDriverBidiScreenArea`, `WebDriverBidiScreenAreaOwnership`, and the typed `SetScreenArea` / `ResetScreenArea` command variants. Both variants carry the ownership witness rather than a raw `WebDriverBidiBrowsingContext`.

`WebDriverBidiScreenAreaOwnership` contains the exact validated browsing context and intentionally exposes no public constructor in the adapter. Its context accessor preserves the target bound to the proof. A future Browser Session integration may mint the witness only after establishing an exclusive/disposable browsing context or an equivalent lifecycle guarantee that no unrelated screen override can be replaced or removed.

Until that mint path exists, the adapter exposes no public explicit screen-area planner. This is deliberate fail-closed capability representation, not an incomplete helper API. When Browser Session adds the ownership transition, the planner/transport path must be introduced in the same reviewed slice so strict Clippy, repository contracts, runtime evidence, and lifecycle invalidation prove that the capability is actually reachable through the canonical owner.

The standard reusable plan remains viewport/DPR plus timezone. Complete `PresentationSurface::Screen` remains unsupported because available-screen geometry is not represented by `ScreenMetrics` and color depth is not controlled by the standard operation.

## Consequences

The adapter preserves the protocol vocabulary needed for a future owned integration while ordinary context-aware callers cannot plan destructive screen-area mutation. The Browser Session owner now has a narrow future integration point instead of a context-only authorization escape hatch or dead public planner.

The trade-off is deliberate: screen-area application cannot currently be materialized outside the module. Product code remains fail-closed until the lifecycle owner supplies a reviewed witness producer and a live consumer path.

## Failure and degraded behavior

If Browser Session cannot prove an exclusive/disposable lifecycle or equivalent restoration-safe ownership, no ownership witness is available and no screen-area Set/Reset plan is exposed to external callers. OriginWeave must not fall back to a raw context identifier, ambient browser state, an LLM decision, a command acknowledgement, best-effort cleanup, or a `dead_code` suppression.

The reusable profile planner continues to omit screen-area mutation. Complete presentation-profile admission continues to return `MissingSurface(Screen)` because available-screen geometry is unmodelled and color depth is uncontrolled.

## Security / privacy / governance impact

A remote-issued context identifier is treated as untrusted addressing metadata rather than mutation authority. The ownership witness prevents adapters, MCP callers, LLM output, page content, or other context-aware code from acquiring screen-settings mutation merely by naming a valid browsing context.

The witness must never be synthesized from command acknowledgement, ambient browser state, mutable external metadata, or a raw context identifier. If lifecycle ownership cannot be proven, screen-area mutation remains unavailable.

No identity, egress, secret, policy, approval, or Context Fabric authority moves into the WebDriver BiDi adapter. The decision remains Proposed until policy-compliant protected-main review changes its lifecycle.

## Tests and acceptance evidence

The test-first successor to #310 initially over-constrained the repair by requiring deletion of all screen-area command intents. That was corrected: the useful protocol vocabulary remains, but the repository contract requires an opaque non-caller-mintable ownership type and requires both Set and Reset variants to carry it. After executable CI exposed the dead-helper contradiction, the contract was tightened to require that no public explicit screen-area planner exists before a Browser Session mint path does.

Repository acceptance requires exact-head Python contracts, Rust formatting, locked workspace tests, strict Clippy, rustdoc/API documentation, and exact 100% owned-production function/line/region/branch coverage. The failing `34419810636` run is RED evidence, not acceptance. Browser acceptance remains separate and requires the pinned Chromium lane to prove application, page-observed post-condition, native interaction/outcome, owned cleanup or context destruction, and post-cleanup observation. Neither this ADR nor repository GREEN is browser GREEN.

## Migration and rollback

This active branch changes only the typed authority boundary. Existing callers must not be mechanically migrated by manufacturing a witness. There is intentionally no explicit public planner to call until the future Browser Session lifecycle owner creates the witness and the consuming path together.

Rollback removes ADR 0113 and the ownership-witness change together with its contract tests. It must not restore context-only public Set/Reset authority or dead planner helpers without a separate reviewed decision, because either would reintroduce the authority or reachability defect.

## Open follow-ups

- Define the Browser Session aggregate transition that mints the witness only after exclusive/disposable-context establishment or equivalent ownership proof.
- Add the screen-area planner/transport consumer only in the same slice that makes the ownership witness legitimately mintable and reachable.
- Bind witness invalidation to context/session destruction and any lifecycle boundary that makes the proof stale.
- Bind runtime evidence to the exact ownership witness, Set command, page-observed post-condition, cleanup or context destruction, and post-cleanup observation.
- Decide in a separate schema change whether `PresentationProfile` should model available-screen geometry; do not infer it from total screen size.
- Continue #299/#292 real-Chromium acceptance independently of this repository-only authority contract.

## Supersession / reversal conditions

This ADR may be superseded if a later reviewed Browser Session design provides an equivalent non-forgeable capability with stronger lifetime semantics, or if a future WebDriver BiDi revision adds authoritative predecessor-state restoration that is separately compatibility-qualified. Publication of a newer draft alone is not sufficient.

It is reversed only if OriginWeave removes the screen-area capability entirely or adopts another reviewed browser protocol boundary that provides equivalent ownership and cleanup guarantees.

## References

World Wide Web Consortium. (2026, August 18). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/

Related repository evidence: ADR 0107, `docs/doctoring/webdriver-bidi-screen-area.md`, `docs/traceability/webdriver-bidi-screen-area-planning.md`, and `docs/traceability/webdriver-bidi-publication-current.md`.
