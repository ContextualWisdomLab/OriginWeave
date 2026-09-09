# WebDriver BiDi screen-area doctoring

The runtime-qualified protocol identity remains the W3C WebDriver BiDi Working Draft published 3 September 2026. The current 9 September 2026 publication retains the same relevant `emulation.setScreenSettingsOverride` shape, but publication freshness does not itself change OriginWeave's runtime pin.

For one exact browsing context, `emulation.setScreenSettingsOverride` accepts `screenArea` as width/height or `null`. A non-null screen area changes the web-exposed screen dimensions for the target context; `screenArea: null` removes that override. This gives OriginWeave a symmetric apply/reset path suitable for reusable-context planning.

The standard operation does **not** control color depth. OriginWeave's `ScreenMetrics` and `PresentationSurface::Screen` contract include color depth as well as dimensions. The adapter therefore projects a dedicated `WebDriverBidiScreenArea` containing only validated width and height and continues to reject complete-profile admission with `MissingSurface(Screen)`. Treating the screen-area command as proof of the complete Screen surface would overstate protocol authority.

This evidence changes only typed command planning. It is not live WebDriver BiDi transport, command acknowledgement, page-observed state, browser cleanup proof, or complete Chromium presentation acceptance. Those remain separate Browser Session/runtime evidence.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* [Working Draft; latest publication tracked separately]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/
