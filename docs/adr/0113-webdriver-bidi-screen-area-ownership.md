# ADR 0113: WebDriver BiDi screen-area ownership witness

- Status: Proposed
- Date: 2026-09-10
- Supersedes: none
- Superseded by: none
- Refines: ADR 0107

## Problem

ADR 0107 keeps WebDriver BiDi behind a versioned adapter and requires owned cleanup for presentation overrides. PR #310 then exposed `emulation.setScreenSettingsOverride` as an explicit partial intent while correctly excluding it from the reusable profile-derived plan because one rectangle changes both total and available screen geometry.

The remaining authority problem is independent of that schema gap. WebDriver BiDi stores the screen-area override against a browsing context. Setting a non-null rectangle replaces the target entry; sending `screenArea: null` removes the target entry. The standard does not restore a predecessor override. A `WebDriverBidiBrowsingContext` therefore identifies a mutation target but cannot prove that OriginWeave owns the state being replaced or cleared.

## Constraints

- Keep browser-domain and Browser Session lifecycle authority in OriginWeave.
- Keep WebDriver BiDi as an adapter; protocol addressability is not product authorization.
- Preserve the runtime-qualified 3 September 2026 Working Draft pin until a separate compatibility change proves a newer revision.
- Preserve the typed `WebDriverBidiScreenArea` width/height representation and the protocol's total/available-area coupling.
- Do not invent a snapshot/restore facility that WebDriver BiDi does not provide.
- Do not let a command acknowledgement substitute for page-observed application or cleanup evidence.
- Keep the reusable profile-derived planner free of screen-area mutation while available-screen geometry remains unmodelled and color depth remains uncontrolled.

## Alternatives

1. **Keep context-only Set/Reset planners.** Rejected. Any caller able to supply a valid remote context identifier could replace or delete screen-settings state without proving ownership.
2. **Delete screen-area support.** Rejected. The standard capability is useful and can be represented without granting ambient mutation authority.
3. **Capture and restore an assumed predecessor value.** Rejected. This slice has no authoritative predecessor snapshot and the standard reset semantics remove the override rather than restore one.
4. **Treat a successful Set command as ownership proof.** Rejected. It can already have overwritten another owner's state; acknowledgement is too late to establish authorization.
5. **Require an opaque Browser Session ownership witness before planning Set or Reset.** Selected. The witness is not caller-mintable from a context identifier and can later be produced only by the lifecycle owner after exclusive/disposable-context establishment or equivalent ownership proof.

## Decision

`originweave-bidi` retains `WebDriverBidiScreenArea` and the explicit `SetScreenArea` / `ResetScreenArea` command intents, but both command variants and both explicit planner functions require `WebDriverBidiScreenAreaOwnership`.

`WebDriverBidiScreenAreaOwnership` contains the exact validated browsing context and intentionally exposes no public constructor in the adapter. Its public context accessor permits a transport integration that already possesses the witness to address the command without reopening validation. A future Browser Session integration may mint the witness only after establishing an exclusive/disposable browsing context or an equivalent lifecycle guarantee that no unrelated screen override can be replaced or removed.

This is capability representation, not runtime proof. The current adapter has no external mint path, so screen-area mutation is unavailable until Browser Session supplies the missing ownership transition. The standard reusable plan remains viewport/DPR plus timezone. Complete `PresentationSurface::Screen` remains unsupported because available-screen geometry is not represented by `ScreenMetrics` and color depth is not controlled by the standard operation.

## Security and governance effects

A remote-issued context identifier is treated as untrusted addressing metadata rather than mutation authority. The ownership witness prevents adapters, MCP callers, LLM output, page content, or other context-aware code from acquiring screen-settings mutation merely by naming a valid browsing context.

The witness must never be synthesized from command acknowledgement, ambient browser state, mutable external metadata, or a raw context identifier. If the Browser Session owner cannot prove an exclusive/disposable lifecycle or equivalent restoration-safe ownership, screen-area mutation remains unavailable and the complete presentation profile continues to fail closed.

## Acceptance evidence

The test-first successor to #310 initially over-constrained the repair by requiring deletion of all screen-area command intents. That was corrected before acceptance: the useful protocol capability remains, but the tests now require an opaque non-caller-mintable ownership type, require both Set and Reset variants to carry it, require both explicit planners to accept it rather than a raw context, and continue to forbid screen-area commands in the reusable profile-derived plan.

Repository acceptance requires exact-head Python contracts, Rust formatting, locked workspace tests, strict Clippy, rustdoc/API documentation, and exact 100% owned-production function/line/region/branch coverage. Browser acceptance remains separate and requires the pinned Chromium lane to prove application, page-observed post-condition, native interaction/outcome, owned cleanup or context destruction, and post-cleanup observation. Neither this ADR nor repository GREEN is browser GREEN.

## Risks and follow-up

The opaque witness deliberately makes screen-area application unusable until Browser Session integration exists. That is preferred to exposing destructive context-only cleanup. The next browser-runtime slice must define where the witness is minted, how exclusivity/disposability is proven, how it is invalidated on context destruction/navigation boundaries where applicable, and how runtime evidence binds the witness to the exact command and cleanup lifecycle.

If a future WebDriver BiDi revision adds authoritative predecessor-state restoration, the ownership model may be revisited through a separate versioned compatibility decision; publication alone is not sufficient.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

## Related documents

See ADR 0107, `docs/doctoring/webdriver-bidi-screen-area.md`, `docs/traceability/webdriver-bidi-screen-area-planning.md`, and `docs/traceability/webdriver-bidi-publication-current.md`.
