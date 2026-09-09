# WebDriver BiDi screen-area planning traceability

## Problem

The runtime-qualified WebDriver BiDi adapter already plans reversible viewport/device-pixel-ratio and time-zone overrides, while the 3 September 2026 Working Draft also defines `emulation.setScreenSettingsOverride`. OriginWeave did not expose that standard screen-area operation in its typed planning boundary.

This is a narrower gap than the complete `PresentationSurface::Screen` requirement. `ScreenMetrics` includes width, height, and color depth, but the WebDriver BiDi `screenArea` payload controls only width and height. Advertising the complete Screen surface after adding this command would therefore create a false-green admission path.

## Constraints

- Keep browser-domain truth in OriginWeave; WebDriver BiDi remains an adapter, not policy authority.
- Preserve the runtime-qualified 3 September 2026 Working Draft pin. Publication freshness is owned separately by `webdriver-bidi-publication-current.md`.
- Reuse validated presentation value objects rather than reopen raw width/height validation in the adapter.
- A reusable browsing context may plan only overrides with a context-scoped, non-destructive reset.
- Do not add media-feature cleanup, ambient-host fallback, live protocol I/O, command-ACK success semantics, or Chromium-specific authority here.

## Alternatives

1. **Keep screen area unplanned.** Rejected because the qualified standard already provides an independently resettable screen-area operation and omitting it leaves a useful standard capability unused.
2. **Mark `PresentationSurface::Screen` supported after planning width/height.** Rejected because color depth remains page-observable and uncontrolled.
3. **Carry full `ScreenMetrics` in the command payload.** Rejected because the command would then contain a field the protocol operation does not apply, making evidence and later serialization authority ambiguous.
4. **Project a dedicated `WebDriverBidiScreenArea` from validated `ScreenMetrics`.** Selected. The adapter carries exactly the standard-owned width/height payload while retaining the complete Screen fail-closed invariant.

## Decision

`originweave-bidi` plans `SetScreenArea` before viewport/DPR and time-zone operations and plans the matching `ResetScreenArea` during reusable-context cleanup. `WebDriverBidiScreenArea` can only be derived from validated `ScreenMetrics`; it contains width and height only. The complete capability map intentionally continues to omit `PresentationSurface::Screen`, so `require_complete_presentation_profile()` still returns `MissingSurface(Screen)` until another reviewed owner controls color depth as well.

The planner produces typed intent only. Transport execution, page-observed post-conditions, browser/session cleanup evidence, crash recovery, and the remaining Chromium-only presentation surfaces stay with the existing #292/#299 acceptance path and its canonical runtime owners.

## Evidence and acceptance

The test-first lineage begins at PR #310 test-only commits and requires:

- a typed screen-area intent derived from validated screen metrics;
- a context-scoped screen-area reset;
- no media-feature reset;
- no color-depth field in the screen-area command value object; and
- continued fail-closed complete Screen admission.

Hosted exact-head repository checks, 100% owned-production coverage, security checks, central required workflows, and realistic pinned-Chromium acceptance remain separate evidence and must not be transferred from predecessor heads.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* [Working Draft; latest publication tracked separately]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
