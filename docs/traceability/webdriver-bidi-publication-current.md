# WebDriver BiDi publication-current receipt

Status: active standards traceability
Observed: 2026-09-12
Runtime-compatible pin: `2026-08-18`
Latest published Working Draft: `2026-08-18`

## Current authoritative publication

Canonical publication page: https://www.w3.org/TR/webdriver-bidi/

Latest immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/

The canonical W3C Technical Report page and Browser Tools and Testing Working Group publication index identify the 18 August 2026 Working Draft. The earlier September dated-TR claims are unsupported and therefore cannot be used as runtime qualification or publication-freshness evidence.

## Runtime compatibility decision

OriginWeave pins `WEBDRIVER_BIDI_PRESENTATION_REVISION = "2026-08-18"`. A later W3C publication requires a dedicated compatibility change with exact command-schema, browser-behaviour, and pinned-Chromium evidence before the adapter pin changes.

## Relationship to buyer acceptance

This receipt does not close OriginWeave #292. Buyer-visible acceptance still requires a real Chromium path, page-visible post-condition evidence, navigation/renderer/crash cases, and cleanup or owned disposable-context destruction. Current #299 remains pre-navigation RED.

## Traceability

- W3C latest published version observed 2026-09-12: WebDriver BiDi Working Draft, 18 August 2026.
- Runtime-qualified OriginWeave adapter pin: WebDriver BiDi Working Draft, 18 August 2026.
- OriginWeave buyer acceptance owner: issue #292.
- OriginWeave profile/standard-adapter parent lineage: PR #229, which has inherited merged PR #293.
- Real pinned-Chromium evidence lane: PR #299.

## References

World Wide Web Consortium. (2026, August 18). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260818/
