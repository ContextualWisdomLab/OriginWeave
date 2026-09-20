# WebDriver BiDi publication-current receipt

Status: active standards traceability
Observed: 2026-09-20
Runtime-compatible pin: `2026-09-03`
Latest published Working Draft: `2026-09-03`
Previous published Working Draft: `2026-09-01`
Editor's Draft: `https://w3c.github.io/webdriver-bidi/`

## Problem

The `originweave-bidi` presentation capability map is deliberately version-pinned, but publication provenance and runtime qualification are separate facts. A fresh 2026-09-20 read of the canonical W3C latest-published page and publication-history page identifies the 3 September 2026 Working Draft as the latest published version and the 1 September 2026 Working Draft as the immediately previous published version. The previously recorded 16 September and 14 September dated Technical Report URLs are not present in W3C's canonical publication history and must not be treated as published Working Draft authority. The Editor's Draft remains a separate mutable surface. The adapter remains independently runtime-qualified against the immutable 3 September 2026 Working Draft.

Treating publication freshness and runtime qualification as the same datum creates two bad failure modes: documentation can become false whenever W3C publishes a new draft, or an automation can silently repin the runtime compatibility claim without re-running the browser/protocol qualification that gives the pin meaning. Conversely, synthesizing dated Technical Report URLs from calendar dates invents authority that W3C has not published.

## Current authoritative publication

Canonical publication page: https://www.w3.org/TR/webdriver-bidi/

Canonical publication history: https://www.w3.org/standards/history/webdriver-bidi/

Latest immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

Previous immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260901/

Mutable Editor's Draft: https://w3c.github.io/webdriver-bidi/

The 3 September publication exposes the standard presentation/lifecycle surfaces used by OriginWeave's capability analysis, including `browsingContext.setViewport`, `browser.createUserContext` / `browser.removeUserContext`, `emulation.setLocaleOverride`, `emulation.setMediaFeaturesOverride`, `emulation.setScreenSettingsOverride`, `emulation.setTimezoneOverride`, and `emulation.setUserAgentOverride`. Their presence is standards research evidence, not proof that a browser implementation or the existing adapter satisfies buyer acceptance.

## Runtime compatibility decision

OriginWeave keeps `WEBDRIVER_BIDI_PRESENTATION_REVISION = "2026-09-03"` because that is the independently qualified immutable draft. The fact that the runtime-qualified pin also remains W3C's latest published Working Draft as of the 2026-09-20 observation does not collapse the two authorities: a future publication can move publication-currentness without changing runtime compatibility.

A publication-freshness update therefore does **not** mutate the runtime pin, claim new browser capability, or promote command acknowledgement to presentation evidence. The safe sequence is:

1. read the canonical W3C latest-published page and publication history rather than deriving dated Technical Report URLs from dates;
2. record the latest authoritative W3C publication independently from the supported runtime pin;
3. retain the immediately previous immutable publication as provenance for publication-history checks;
4. keep the mutable Editor's Draft explicitly separate from dated Technical Reports;
5. diff the relevant specification surfaces and update the versioned capability map only if needed;
6. re-run repository contracts and pinned Chromium/BiDi/CDP compatibility evidence on any proposed new runtime pin;
7. update architecture/ADR/doctoring compatibility claims together with the qualified pin;
8. keep unsupported or unverified surfaces fail closed.

## Relationship to buyer acceptance

This receipt does not close OriginWeave #292. The buyer-visible acceptance still requires a version-pinned real Chromium path to apply the complete admitted presentation profile, observe the page-visible post-condition, survive navigation/renderer/crash cases, and prove cleanup or owned disposable-context destruction. Current #299 evidence remains pre-navigation RED, so publication currentness cannot be counted as browser GREEN.

## Traceability

- W3C publication history re-read on 2026-09-20: latest listed WebDriver BiDi Working Draft is 3 September 2026; the immediately previous listed Working Draft is 1 September 2026.
- W3C latest-published page observed 2026-09-20: WebDriver BiDi Working Draft, 3 September 2026.
- Previously recorded 16 September and 14 September 2026 dated WebDriver BiDi Technical Report URLs were not supported by the canonical publication history and are withdrawn as publication evidence.
- Mutable Editor's Draft: https://w3c.github.io/webdriver-bidi/.
- Runtime-qualified OriginWeave adapter pin: WebDriver BiDi Working Draft, 3 September 2026.
- OriginWeave buyer acceptance owner: issue #292.
- OriginWeave profile/standard-adapter parent lineage: PR #229, which has inherited merged PR #293.
- Real pinned-Chromium evidence lane: PR #299.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* (W3C Working Draft; latest published version observed 2026-09-20 and runtime-qualified OriginWeave pin). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

World Wide Web Consortium. (2026, September 1). *WebDriver BiDi* (W3C Working Draft; previous published version). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260901/

World Wide Web Consortium. (2026). *WebDriver BiDi publication history*. Retrieved September 20, 2026, from https://www.w3.org/standards/history/webdriver-bidi/
