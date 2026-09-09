# WebDriver BiDi screen-area doctoring

The runtime-qualified protocol identity remains the W3C WebDriver BiDi Working Draft published 3 September 2026. The current 9 September 2026 publication retains the same relevant `emulation.setScreenSettingsOverride` shape, but publication freshness does not itself change OriginWeave's runtime pin.

For one exact browsing context, `emulation.setScreenSettingsOverride` accepts `screenArea` as width/height or `null`. The W3C operation uses the same non-null rectangle for both the web-exposed total screen area and the web-exposed available screen area; `screenArea: null` removes that context-scoped override. The reset is symmetric, but the mutation is wider than `ScreenMetrics(width, height, color_depth)` because the current presentation identity does not model `screen.availWidth` or `screen.availHeight`.

OriginWeave therefore exposes this as an explicit partial `WebDriverBidiScreenArea` intent rather than inserting it into the reusable profile-derived presentation plan. The value object can only project width and height from validated `ScreenMetrics`, and its rustdoc makes the total/available-area coupling explicit. The ordinary reusable planner remains limited to viewport/DPR and time zone until the presentation schema deliberately models and digest-binds the available-screen observable.

The standard operation also does **not** control color depth. `PresentationSurface::Screen` continues to fail closed with `MissingSurface(Screen)`: neither an explicit screen-area command nor its command acknowledgement proves the complete Screen fingerprint surface.

This evidence changes only typed command planning. It is not live WebDriver BiDi transport, command acknowledgement, page-observed state, browser cleanup proof, or complete Chromium presentation acceptance. Those remain separate Browser Session/runtime evidence, including post-reset re-observation before a reusable context can be trusted again.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* [Working Draft; latest publication tracked separately]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
