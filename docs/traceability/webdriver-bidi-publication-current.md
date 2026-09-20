# WebDriver BiDi publication-current receipt

Status: active standards traceability
Observed: 2026-09-21
Runtime-compatible pin: `2026-09-03`
Latest published Working Draft: `2026-09-09`
Previous published Working Draft: `2026-09-03`
Editor's Draft: `https://w3c.github.io/webdriver-bidi/`

## Problem

The `originweave-bidi` presentation capability map is deliberately version-pinned, but publication provenance and runtime qualification are separate facts. A fresh 2026-09-21 read of W3C's canonical latest-published WebDriver BiDi page identifies the **9 September 2026 Working Draft** as the current publication and links the **3 September 2026 Working Draft** as its previous version. The W3C standards index independently lists WebDriver BiDi as a Draft Standard dated 9 September 2026. The publication-history index can lag the canonical latest document; it must not be used by itself to synthesize a newer date.

The predecessor receipt asserted 16 September 2026 as latest and 14 September 2026 as previous. Fresh primary W3C surfaces do not support those dates, so they are superseded as publication-currentness evidence. This repair does **not** fall back to a search-index guess: current/previous authority comes from the canonical `/TR/webdriver-bidi/` document metadata and its `Previous Versions` link, with the standards index as an independent cross-check.

Treating publication freshness and runtime qualification as the same datum creates two bad failure modes: documentation can become false whenever W3C publishes a new draft, or an automation can silently repin the runtime compatibility claim without re-running the browser/protocol qualification that gives the pin meaning.

## Current authoritative publication

Canonical latest publication page: https://www.w3.org/TR/webdriver-bidi/

W3C standards index: https://www.w3.org/TR/

Canonical publication history: https://www.w3.org/standards/history/webdriver-bidi/

Latest immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

Previous immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

Mutable Editor's Draft: https://w3c.github.io/webdriver-bidi/

The 9 September publication continues to expose the standard presentation/lifecycle surfaces used by OriginWeave's capability analysis, including `browsingContext.setViewport`, `browser.createUserContext` / `browser.removeUserContext`, `emulation.setLocaleOverride`, `emulation.setMediaFeaturesOverride`, `emulation.setScreenSettingsOverride`, `emulation.setTimezoneOverride`, and `emulation.setUserAgentOverride`. Their presence is standards research evidence, not proof that the existing runtime adapter has been requalified against a newer publication.

## Runtime compatibility decision

OriginWeave keeps `WEBDRIVER_BIDI_PRESENTATION_REVISION = "2026-09-03"` because that immutable Working Draft is the runtime-qualified adapter pin. In the current publication generation it is also the immediately previous published Working Draft. A newer runtime pin still requires a dedicated compatibility change proving that the newer immutable draft preserves the exact command schemas, reset semantics, capability interpretation, browser implementation behavior, and pinned-Chromium acceptance required by the adapter.

A publication-freshness update therefore does **not** mutate the runtime pin, claim new browser capability, or promote command acknowledgement to presentation evidence. The safe sequence is:

1. read the canonical latest Technical Report and its explicit previous-version link;
2. cross-check the W3C standards index and record any history-index lag rather than inventing a publication;
3. keep the mutable Editor's Draft explicitly separate from dated Technical Reports;
4. diff the relevant specification surfaces and update the versioned capability map only if needed;
5. re-run repository contracts and pinned Chromium/BiDi/CDP compatibility evidence on any proposed new runtime pin;
6. update architecture/ADR/doctoring compatibility claims together with the qualified pin;
7. keep unsupported or unverified surfaces fail closed.

## Relationship to buyer acceptance

This receipt does not close OriginWeave #292. The buyer-visible acceptance still requires a version-pinned real Chromium path to apply the complete admitted presentation profile, observe the page-visible post-condition, survive navigation/renderer/crash cases, and prove cleanup or owned disposable-context destruction. Current #299 evidence remains pre-navigation RED, so publication freshness cannot be counted as browser GREEN.

## Traceability

- W3C canonical latest document observed 2026-09-21: WebDriver BiDi Working Draft, 9 September 2026.
- Previous version linked by that document: WebDriver BiDi Working Draft, 3 September 2026.
- W3C standards index cross-check: WebDriver BiDi Draft Standard, 9 September 2026.
- Publication-history index is retained as a discovery surface but does not override canonical latest-document metadata when it lags.
- Mutable Editor's Draft: https://w3c.github.io/webdriver-bidi/.
- Runtime-qualified OriginWeave adapter pin: WebDriver BiDi Working Draft, 3 September 2026.
- OriginWeave buyer acceptance owner: issue #292.
- OriginWeave profile/standard-adapter parent lineage: PR #229, which has inherited merged PR #293.
- Real pinned-Chromium evidence lane: PR #299.

## References

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

World Wide Web Consortium. (2026). *W3C standards and drafts*. Retrieved September 21, 2026, from https://www.w3.org/TR/

World Wide Web Consortium. (2026). *WebDriver BiDi publication history*. Retrieved September 21, 2026, from https://www.w3.org/standards/history/webdriver-bidi/

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* (W3C Working Draft; previous published version and runtime-qualified OriginWeave pin). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/
