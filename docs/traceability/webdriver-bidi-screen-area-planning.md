# WebDriver BiDi screen-area planning traceability

## Problem

The runtime-qualified WebDriver BiDi adapter already plans reversible viewport/device-pixel-ratio and time-zone overrides, while the 3 September 2026 Working Draft also defines `emulation.setScreenSettingsOverride`. OriginWeave did not expose that standard operation in its typed planning boundary.

The operation is not merely a narrower version of `PresentationSurface::Screen`. WebDriver BiDi applies one `screenArea` rectangle to both the web-exposed total screen area and the web-exposed available screen area. OriginWeave `ScreenMetrics` currently models width, height, and color depth, but not `screen.availWidth` or `screen.availHeight`. Automatically deriving the command from `ScreenMetrics` inside the reusable profile plan would therefore mutate a page-observable fingerprint surface that the profile neither selected nor digest-bound. Color depth remains independently uncontrolled as well.

## Constraints

- Keep browser-domain truth in OriginWeave; WebDriver BiDi remains an adapter, not policy authority.
- Preserve the runtime-qualified 3 September 2026 Working Draft pin. Publication freshness is owned separately by `webdriver-bidi-publication-current.md`.
- Reuse validated presentation value objects rather than reopen raw width/height validation in the adapter.
- A reusable browsing context may automatically plan only observables represented by the explicit presentation contract and paired with a context-scoped, non-destructive reset.
- Do not add media-feature cleanup, ambient-host fallback, live protocol I/O, command-ACK success semantics, or Chromium-specific authority here.

## Alternatives

1. **Insert screen settings into the reusable profile-derived plan.** Rejected. Although `screenArea: null` provides a symmetric reset, the apply operation also changes the currently unmodelled available-screen rectangle. Reversibility alone does not authorize an additional page observable.
2. **Mark `PresentationSurface::Screen` supported after planning width/height.** Rejected because color depth remains page-observable and uncontrolled, and available-screen geometry is absent from the profile.
3. **Carry full `ScreenMetrics` in the command payload.** Rejected because the command would contain color depth, which the protocol operation does not apply, while still failing to name the available-screen side effect.
4. **Expose an explicit coupled screen-area partial intent and keep it out of the reusable profile-derived plan.** Selected. `WebDriverBidiScreenArea` projects validated width/height, documents that the same rectangle becomes both total and available screen area, and has a separate context-scoped reset. This preserves the protocol capability without silently broadening the presentation profile.
5. **Expand `PresentationProfile` immediately with available-screen dimensions.** Deferred. That changes the canonical fingerprint schema, replay digest, consistency rules, fixtures, and buyer evidence. It requires its own test-first bounded change rather than being hidden inside an adapter slice.

## Decision

`originweave-bidi` exposes `plan_explicit_screen_area_override` and `plan_explicit_screen_area_cleanup` as a separately explicit partial capability. The ordinary `plan_standard_presentation_commands` and `plan_standard_presentation_cleanup` remain limited to viewport/DPR and time zone because those are the currently modelled, reusable-plan observables with symmetric resets.

`WebDriverBidiScreenArea` can only be derived from validated `ScreenMetrics`; its documentation records that WebDriver BiDi couples total and available screen areas to the same rectangle. The complete capability map intentionally continues to omit `PresentationSurface::Screen`, so `require_complete_presentation_profile()` still returns `MissingSurface(Screen)` until a reviewed owner models the available-screen observable and controls color depth as well.

The planner produces typed intent only. Transport execution, page-observed post-conditions, browser/session cleanup evidence, crash recovery, and the remaining Chromium-only presentation surfaces stay with the existing #292/#299 acceptance path and its canonical runtime owners.

## Evidence and acceptance

The review finding on PR #310 exact `e3b2b412d8ad880c87354fb3ffd5f5b4ff6cde0d` identified the unmodelled available-screen side effect. Test-first successor `8f74471e1a5414e8781531f968b46807e2d7e3d8` adds a contract that fails whenever the profile-derived reusable planner schedules `SetScreenArea` without available width/height being represented by `ScreenMetrics`. The minimal source repair separates the explicit screen-area operation from the reusable profile-derived plan.

Acceptance requires:

- a typed screen-area intent derived from validated screen metrics;
- explicit documentation that one WebDriver BiDi rectangle controls both total and available screen areas;
- a separately explicit context-scoped screen-area reset;
- no screen-area mutation in the reusable profile-derived plan while available-screen geometry is unmodelled;
- no media-feature reset;
- no color-depth field in the screen-area command value object; and
- continued fail-closed complete Screen admission.

Hosted exact-head repository checks, 100% owned-production coverage, security checks, central required workflows, and realistic pinned-Chromium acceptance remain separate evidence and must not be transferred from predecessor heads.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* [Working Draft; latest publication tracked separately]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
