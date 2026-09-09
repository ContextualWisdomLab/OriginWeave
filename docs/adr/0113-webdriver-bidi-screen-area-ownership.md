# ADR 0113: WebDriver BiDi screen-area ownership witness

- **Status:** Proposed
- **Date:** 2026-09-10
- **Supersedes:** none
- **Superseded by:** none
- **Refines:** ADR 0107

## Context

ADR 0107 keeps WebDriver BiDi behind a versioned adapter and requires owned cleanup for presentation overrides. PR #310 exposed `emulation.setScreenSettingsOverride` as an explicit partial intent while correctly excluding it from the reusable profile-derived plan because one rectangle changes both total and available screen geometry.

A second authority problem is independent of that schema gap. WebDriver BiDi stores the screen-area override against a browsing context. Setting a non-null rectangle replaces the target entry; sending `screenArea: null` removes the target entry. The standard does not restore a predecessor override. A `WebDriverBidiBrowsingContext` therefore identifies a mutation target but cannot prove that OriginWeave owns the state being replaced or cleared.

## Decision drivers

- Preserve the useful typed WebDriver BiDi screen-area capability without granting ambient mutation authority.
- Prevent a raw browsing-context identifier from authorizing replacement or removal of another owner's override.
- Keep cleanup evidence causal: ownership must exist before the destructive mutation, not be inferred from a later command acknowledgement.
- Keep the reusable profile-derived planner limited to observables represented by the profile and paired with safe cleanup semantics.
- Keep complete Screen admission fail-closed while available-screen geometry and color depth remain uncontrolled.

## Assumptions and authority boundaries

- Browser-domain and Browser Session lifecycle authority remain in OriginWeave.
- WebDriver BiDi remains an adapter; protocol addressability is not product authorization.
- The runtime-qualified 3 September 2026 Working Draft pin remains unchanged until a separate compatibility change proves a newer revision.
- `WebDriverBidiScreenArea` remains the typed width/height representation of the protocol's coupled total/available-area rectangle.
- This slice has no authoritative predecessor-state snapshot and does not invent one.
- A command acknowledgement is not page-observed application, ownership evidence, cleanup evidence, or restoration evidence.
- Screen-area mutation may become executable only after Browser Session proves an exclusive/disposable browsing context or an equivalent restoration-safe lifecycle.

## Options considered

1. **Keep context-only Set/Reset planners.** Rejected. Any caller able to supply a valid remote context identifier could replace or delete screen-settings state without proving ownership.
2. **Delete screen-area support.** Rejected. The standard capability is useful and can be represented without granting ambient mutation authority.
3. **Capture and restore an assumed predecessor value.** Rejected. This slice has no authoritative predecessor snapshot and the standard reset semantics remove the override rather than restore one.
4. **Treat a successful Set command as ownership proof.** Rejected. The Set can already have overwritten another owner's state; acknowledgement is too late to establish authorization.
5. **Require an opaque Browser Session ownership witness before planning Set or Reset.** Selected. The witness is not caller-mintable from a context identifier and can later be produced only by the lifecycle owner after exclusive/disposable-context establishment or equivalent ownership proof.

## Decision

`originweave-bidi` retains `WebDriverBidiScreenArea` and the explicit `SetScreenArea` / `ResetScreenArea` command intents, but both command variants and both explicit planner functions require `WebDriverBidiScreenAreaOwnership`.

`WebDriverBidiScreenAreaOwnership` contains the exact validated browsing context and intentionally exposes no public constructor in the adapter. Its public context accessor permits a transport integration that already possesses the witness to address the command without reopening validation. A future Browser Session integration may mint the witness only after establishing an exclusive/disposable browsing context or an equivalent lifecycle guarantee that no unrelated screen override can be replaced or removed.

This is capability representation, not runtime proof. The current adapter has no external mint path, so screen-area mutation is unavailable until Browser Session supplies the missing ownership transition. The standard reusable plan remains viewport/DPR plus timezone. Complete `PresentationSurface::Screen` remains unsupported because available-screen geometry is not represented by `ScreenMetrics` and color depth is not controlled by the standard operation.

## Consequences

The adapter preserves the standard screen-area value and explicit command vocabulary while making destructive mutation unavailable to ordinary context-aware callers. A later Browser Session integration has a narrow place to attach lifecycle proof instead of widening the browsing-context value object into authorization.

The trade-off is deliberate: screen-area application cannot currently be materialized outside the module. Product code must remain fail-closed until the lifecycle owner supplies a reviewed witness producer.

## Failure and degraded behavior

If Browser Session cannot prove an exclusive/disposable lifecycle or equivalent restoration-safe ownership, no ownership witness is available and screen-area Set/Reset cannot be planned by external callers. OriginWeave must not fall back to a raw context identifier, ambient browser state, an LLM decision, a command acknowledgement, or best-effort cleanup.

The reusable profile planner continues to omit screen-area mutation. Complete presentation-profile admission continues to return `MissingSurface(Screen)` because available-screen geometry is unmodelled and color depth is uncontrolled.

## Security / privacy / governance impact

A remote-issued context identifier is treated as untrusted addressing metadata rather than mutation authority. The ownership witness prevents adapters, MCP callers, LLM output, page content, or other context-aware code from acquiring screen-settings mutation merely by naming a valid browsing context.

The witness must never be synthesized from command acknowledgement, ambient browser state, mutable external metadata, or a raw context identifier. If lifecycle ownership cannot be proven, screen-area mutation remains unavailable.

No identity, egress, secret, policy, approval, or Context Fabric authority moves into the WebDriver BiDi adapter. The decision remains Proposed until policy-compliant protected-main review changes its lifecycle.

## Tests and acceptance evidence

The test-first successor to #310 initially over-constrained the repair by requiring deletion of all screen-area command intents. That was corrected before acceptance: the useful protocol capability remains, but the repository contract now requires an opaque non-caller-mintable ownership type, requires both Set and Reset variants to carry it, requires both explicit planners to accept it rather than a raw context, and continues to forbid screen-area commands in the reusable profile-derived plan.

Repository acceptance requires exact-head Python contracts, Rust formatting, locked workspace tests, strict Clippy, rustdoc/API documentation, and exact 100% owned-production function/line/region/branch coverage. Browser acceptance remains separate and requires the pinned Chromium lane to prove application, page-observed post-condition, native interaction/outcome, owned cleanup or context destruction, and post-cleanup observation. Neither this ADR nor repository GREEN is browser GREEN.

## Migration and rollback

This active branch changes only the typed planner contract. Existing callers that used context-only screen-area planners must not be mechanically migrated by manufacturing a witness; they must move behind the future Browser Session lifecycle owner or remain unable to invoke the operation.

Rollback removes ADR 0113 and the ownership-witness change together with its contract tests. It must not restore the context-only public Set/Reset authority without a separate reviewed decision, because that would reintroduce the destructive-cleanup defect.

## Open follow-ups

- Define the Browser Session aggregate transition that mints the witness only after exclusive/disposable-context establishment or equivalent ownership proof.
- Bind witness invalidation to context/session destruction and any lifecycle boundary that makes the proof stale.
- Bind runtime evidence to the exact ownership witness, Set command, page-observed post-condition, cleanup or context destruction, and post-cleanup observation.
- Decide in a separate schema change whether `PresentationProfile` should model available-screen geometry; do not infer it from total screen size.
- Continue #299/#292 real-Chromium acceptance independently of this repository-only authority contract.

## Supersession / reversal conditions

This ADR may be superseded if a later reviewed Browser Session design provides an equivalent non-forgeable capability with stronger lifetime semantics, or if a future WebDriver BiDi revision adds authoritative predecessor-state restoration that is separately compatibility-qualified. Publication of a newer draft alone is not sufficient.

It is reversed only if OriginWeave removes the screen-area capability entirely or adopts another reviewed browser protocol boundary that provides equivalent ownership and cleanup guarantees.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

Related repository evidence: ADR 0107, `docs/doctoring/webdriver-bidi-screen-area.md`, `docs/traceability/webdriver-bidi-screen-area-planning.md`, and `docs/traceability/webdriver-bidi-publication-current.md`.
