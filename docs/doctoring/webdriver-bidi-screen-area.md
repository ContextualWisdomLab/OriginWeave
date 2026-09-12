# WebDriver BiDi screen-area doctoring

The runtime-qualified protocol identity remains the W3C WebDriver BiDi Working Draft published 18 August 2026. Publication freshness is tracked separately in `docs/traceability/webdriver-bidi-publication-current.md` and does not by itself change OriginWeave's runtime pin.

For one exact browsing context, `emulation.setScreenSettingsOverride` accepts `screenArea` as width/height or `null`. The W3C operation uses the same non-null rectangle for both the web-exposed total screen area and the web-exposed available screen area. When `screenArea` is `null`, the remote end removes that context from the screen-settings override map; the command does not restore any predecessor override value.

That lifecycle matters independently of the profile schema. `ScreenMetrics(width, height, color_depth)` still does not model `screen.availWidth` or `screen.availHeight`, so the reusable profile-derived plan cannot silently apply the operation. A raw `WebDriverBidiBrowsingContext` also cannot authorize the separate explicit operation: replacing or removing the current override could mutate state installed by another owner.

OriginWeave therefore keeps `WebDriverBidiScreenArea` and the `SetScreenArea` / `ResetScreenArea` command vocabulary behind an opaque `WebDriverBidiScreenAreaOwnership` witness. That witness has no public constructor in the adapter. A Browser Session integration may create it only after establishing an exclusive/disposable browsing context or an equivalent lifecycle proof that prevents replacement or removal of unrelated screen-settings state. Possession of a remote context identifier alone is not ownership evidence.

The first witness implementation also retained two public explicit screen-area planner helpers even though no legal production path could mint the witness. Exact-head CI `34419810636` rejected both helpers under strict Clippy as dead code while repository contracts, formatting, workspace tests, and exact production coverage otherwise passed. OriginWeave does not suppress that finding. Until Browser Session introduces the reviewed witness-mint transition and a real consuming path, the adapter exposes no public explicit screen-area planner; the typed command vocabulary remains dormant and fail-closed.

The standard operation also does **not** control color depth. `PresentationSurface::Screen` continues to fail closed with `MissingSurface(Screen)`: neither an owned screen-area command nor its command acknowledgement proves the complete Screen fingerprint surface.

This evidence changes typed command authority only. It is not live WebDriver BiDi transport, command acknowledgement, page-observed state, browser cleanup proof, or complete Chromium presentation acceptance. Those remain separate Browser Session/runtime evidence, including post-reset observation and actual disposable-context destruction or equivalent restoration proof before a reusable boundary can be trusted again.

## References

World Wide Web Consortium. (2026, August 18). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/
