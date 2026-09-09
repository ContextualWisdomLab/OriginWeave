# WebDriver BiDi screen-area planning traceability

## Problem

The runtime-qualified WebDriver BiDi adapter plans reversible viewport/device-pixel-ratio and time-zone overrides, while the 3 September 2026 Working Draft also defines `emulation.setScreenSettingsOverride`. The screen operation is wider and more destructive than its width/height payload initially suggests.

WebDriver BiDi applies one `screenArea` rectangle to both the web-exposed total screen area and the web-exposed available screen area. OriginWeave `ScreenMetrics` currently models width, height, and color depth, but not `screen.availWidth` or `screen.availHeight`. Automatically deriving the command from `ScreenMetrics` inside the reusable profile plan would therefore mutate a page-observable fingerprint surface that the profile neither selected nor digest-bound. Color depth remains independently uncontrolled.

A second authority defect remains even when the operation is separated from the profile-derived plan. The standard stores one override per target browsing context. Setting a rectangle replaces that target's current override; `screenArea: null` removes the target from the override map. The standard does not restore a predecessor value. A validated browsing-context identifier therefore identifies where a mutation would occur but does not prove that OriginWeave owns the state being replaced or cleared.

## Constraints

- Keep browser-domain truth in OriginWeave; WebDriver BiDi remains an adapter, not policy authority.
- Preserve the runtime-qualified 3 September 2026 Working Draft pin. Publication freshness is owned separately by `webdriver-bidi-publication-current.md`.
- Reuse validated presentation value objects rather than reopen raw width/height validation in the adapter.
- Do not treat a browsing-context identifier as mutation authority.
- A reusable browsing context may automatically plan only observables represented by the explicit presentation contract and paired with non-destructive cleanup.
- Screen-area mutation requires an exclusive/disposable Browser Session context or equivalent ownership proof before the command can be materialized.
- Do not add media-feature cleanup, ambient-host fallback, live protocol I/O, command-ACK success semantics, or Chromium-specific authority here.

## Alternatives

1. **Insert screen settings into the reusable profile-derived plan.** Rejected. The apply operation changes the currently unmodelled available-screen rectangle, and the nullable reset does not restore a predecessor override.
2. **Mark `PresentationSurface::Screen` supported after planning width/height.** Rejected because color depth remains page-observable and uncontrolled, and available-screen geometry is absent from the profile.
3. **Carry full `ScreenMetrics` in the command payload.** Rejected because the command would contain color depth, which the protocol operation does not apply, while still failing to name the available-screen side effect.
4. **Expose context-only explicit Set/Reset commands.** Rejected after review. A context identifier does not establish ownership; setting can replace another owner's override and resetting can erase it without restoration.
5. **Remove the standard capability entirely.** Rejected. The protocol operation is useful and can be represented safely without making it ambient authority.
6. **Keep the typed screen-area value and gate explicit mutation on an opaque Browser Session ownership witness.** Selected. The adapter retains protocol semantics while making lifecycle authority non-caller-mintable until a Browser Session owner proves an exclusive/disposable context or equivalent safe ownership transition.
7. **Expand `PresentationProfile` immediately with available-screen dimensions.** Deferred. That changes the canonical fingerprint schema, replay digest, consistency rules, fixtures, and buyer evidence and needs its own test-first change.

## Decision

`originweave-bidi` retains `WebDriverBidiScreenArea` as the validated width/height projection and retains explicit `SetScreenArea` / `ResetScreenArea` command intent. Both command variants and both explicit planner functions require `WebDriverBidiScreenAreaOwnership` rather than a raw `WebDriverBidiBrowsingContext`.

`WebDriverBidiScreenAreaOwnership` contains the exact validated browsing context but intentionally has no public constructor. The adapter therefore cannot mint its own proof from a context identifier. A future Browser Session integration may create the witness only after proving an exclusive/disposable lifecycle or an equivalent ownership transition. Possession of the witness is the authority to plan both the apply and matching cleanup for that owned lifecycle; it is not transport acknowledgement or page-observed evidence.

The ordinary `plan_standard_presentation_commands` and `plan_standard_presentation_cleanup` remain limited to viewport/DPR and time zone. The complete capability map continues to omit `PresentationSurface::Screen`, so `require_complete_presentation_profile()` still returns `MissingSurface(Screen)` until a reviewed owner models available-screen geometry, controls color depth, and proves the runtime application/cleanup lifecycle.

## Evidence and acceptance

PR #310 review identified two distinct findings. The first was the unmodelled available-screen side effect, repaired by keeping screen-area mutation out of the profile-derived reusable plan. The later exact-head review identified the ownership gap: a context-only `ResetScreenArea` could remove another owner's active override because `screenArea: null` deletes the target's override-map entry rather than restoring a prior value.

The successor contract requires:

- `WebDriverBidiScreenArea` to remain the typed width/height representation derived from validated screen metrics;
- an opaque `WebDriverBidiScreenAreaOwnership` carrying the exact context with no public mint constructor in the adapter;
- `SetScreenArea`, `ResetScreenArea`, and both explicit planners to require that ownership witness rather than a raw context identifier;
- no screen-area mutation in the reusable profile-derived plan while available-screen geometry is unmodelled;
- no media-feature reset;
- no color-depth field in the screen-area value object; and
- continued fail-closed complete Screen admission.

The initial successor RED briefly over-constrained the repair by requiring removal of all screen-area command intents. That was corrected before acceptance: deleting a useful standard capability is not necessary when its mutation authority can instead be represented explicitly and made non-caller-mintable.

Hosted exact-head repository checks, 100% owned-production coverage, security checks, central required workflows, and realistic pinned-Chromium acceptance remain separate evidence. A command intent or acknowledgement is never substituted for apply → page-observed post-condition → interaction/outcome → owned cleanup/destruction → post-cleanup observation.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/
