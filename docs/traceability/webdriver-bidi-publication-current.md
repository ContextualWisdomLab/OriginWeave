# WebDriver BiDi publication-current receipt

Status: active standards traceability
Observed: 2026-09-10
Runtime-compatible pin: `2026-09-03`
Latest published Working Draft: `2026-09-09`

## Problem

The `originweave-bidi` presentation capability map is deliberately version-pinned, but its repository contract had conflated that qualified runtime pin with the latest W3C publication. On 2026-09-10 the canonical W3C Technical Report page identifies the 9 September 2026 Working Draft as the latest published version, while the adapter remains qualified against the immutable 3 September 2026 Working Draft.

Treating those as the same datum creates two bad failure modes: documentation can become false whenever W3C publishes a new draft, or an automation can silently repin the runtime compatibility claim without re-running the browser/protocol qualification that gives the pin meaning.

## Current authoritative publication

Canonical publication page: https://www.w3.org/TR/webdriver-bidi/

Latest immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

The 9 September publication still exposes the standard presentation/lifecycle surfaces used by OriginWeave's capability analysis, including `browsingContext.setViewport`, `browser.createUserContext` / `browser.removeUserContext`, `emulation.setLocaleOverride`, `emulation.setMediaFeaturesOverride`, `emulation.setScreenSettingsOverride`, `emulation.setTimezoneOverride`, and `emulation.setUserAgentOverride`. Their presence is standards research evidence, not proof that the existing runtime adapter has been requalified against the new publication.

## Runtime compatibility decision

OriginWeave keeps `WEBDRIVER_BIDI_PRESENTATION_REVISION = "2026-09-03"` until a dedicated compatibility change proves that the newer immutable draft preserves the exact command schemas, reset semantics, capability interpretation, browser implementation behavior, and pinned-Chromium acceptance required by the adapter.

A publication-freshness update therefore does **not** mutate the runtime pin, claim new browser capability, or promote command acknowledgement to presentation evidence. The safe sequence is:

1. record the latest authoritative W3C publication independently from the supported runtime pin;
2. diff the relevant specification surfaces and update the versioned capability map only if needed;
3. re-run repository contracts and pinned Chromium/BiDi/CDP compatibility evidence on the proposed new pin;
4. update architecture/ADR/doctoring compatibility claims together with the qualified pin;
5. keep unsupported or unverified surfaces fail closed.

## Relationship to buyer acceptance

This receipt does not close OriginWeave #292. The buyer-visible acceptance still requires a version-pinned real Chromium path to apply the complete admitted presentation profile, observe the page-visible post-condition, survive navigation/renderer/crash cases, and prove cleanup or owned disposable-context destruction. Current #299 evidence remains pre-navigation RED, so publication freshness cannot be counted as browser GREEN.

## Traceability

- W3C latest published version observed 2026-09-10: WebDriver BiDi Working Draft, 9 September 2026.
- Runtime-qualified OriginWeave adapter pin: WebDriver BiDi Working Draft, 3 September 2026.
- OriginWeave buyer acceptance owner: issue #292.
- OriginWeave profile/standard-adapter parent lineage: PR #229, which has inherited merged PR #293.
- Real pinned-Chromium evidence lane: PR #299.

## References

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* (W3C Working Draft; runtime-qualified OriginWeave pin). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/
