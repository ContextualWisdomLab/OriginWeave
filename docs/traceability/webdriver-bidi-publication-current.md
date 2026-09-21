# WebDriver BiDi publication-current receipt

Status: active standards traceability
Observed: 2026-09-21
Runtime-compatible pin: `2026-09-03`
Latest published Working Draft: `2026-09-09`
Previous published Working Draft: `2026-09-03`
Editor's Draft: `https://w3c.github.io/webdriver-bidi/`

## Problem

The `originweave-bidi` presentation capability map is deliberately version-pinned, but publication provenance and runtime qualification are separate facts. A fresh 2026-09-21 read of W3C's canonical Technical Report and publication-history surfaces identifies the **9 September 2026 Working Draft** as the latest published WebDriver BiDi draft and the **3 September 2026 Working Draft** as the immediately previous publication.

The predecessor #229 generation incorrectly recorded 16 September / 14 September as published WebDriver BiDi drafts. The canonical W3C publication history contains neither entry, and the live `https://www.w3.org/TR/webdriver-bidi/` Technical Report identifies the 9 September publication with 3 September as its previous version. The prior 16/14 claim is therefore invalid standards evidence and is repaired by ordinary-forward history rather than hidden through rebase or force update.

Treating publication freshness and runtime qualification as the same datum creates two bad failure modes: documentation can become false whenever W3C publishes a new draft, or an automation can silently repin the runtime compatibility claim without re-running the browser/protocol qualification that gives the pin meaning.

## Current authoritative publication

Canonical latest publication page: https://www.w3.org/TR/webdriver-bidi/

Canonical publication history: https://www.w3.org/standards/history/webdriver-bidi/

Latest immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

Previous immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

Mutable Editor's Draft: https://w3c.github.io/webdriver-bidi/

The 9 September publication exposes the standard presentation/lifecycle surfaces used by OriginWeave's capability analysis. Their presence is standards research evidence, not proof that the existing runtime adapter has been requalified against that publication.

## Runtime compatibility decision

OriginWeave keeps `WEBDRIVER_BIDI_PRESENTATION_REVISION = "2026-09-03"` until a dedicated compatibility change proves that a newer immutable draft preserves the exact command schemas, reset semantics, capability interpretation, browser implementation behavior, and pinned-Chromium acceptance required by the adapter. The runtime-qualified 3 September pin currently happens to equal the immediately previous W3C publication; that coincidence does not collapse the two authorities.

A publication-freshness update therefore does **not** mutate the runtime pin, claim new browser capability, or promote command acknowledgement to presentation evidence. The safe sequence is:

1. read the canonical latest Technical Report and live W3C publication history;
2. retain the immediately previous immutable publication as provenance;
3. treat search indexes, cached discovery surfaces, generated summaries, and prior repository prose as non-authoritative when they disagree with live primary W3C metadata;
4. keep the mutable Editor's Draft explicitly separate from dated Technical Reports;
5. diff the relevant specification surfaces and update the versioned capability map only if needed;
6. re-run repository contracts and pinned Chromium/BiDi/CDP compatibility evidence on any proposed new runtime pin;
7. update architecture/ADR/doctoring compatibility claims together with the qualified pin and keep unsupported or unverified surfaces fail closed.

## Relationship to buyer acceptance

This receipt does not close OriginWeave #292. The buyer-visible acceptance still requires a version-pinned real Chromium path to apply the complete admitted presentation profile, observe the page-visible post-condition, survive navigation/renderer/crash cases, and prove cleanup or owned disposable-context destruction. Current #299 evidence remains pre-navigation RED, so publication freshness cannot be counted as browser GREEN.

## Traceability

- W3C live publication history observed 2026-09-21: WebDriver BiDi Working Draft, 9 September 2026.
- Immediately previous published version: WebDriver BiDi Working Draft, 3 September 2026.
- Canonical latest Technical Report independently identifies the 9 September publication and 3 September previous version.
- Superseded repository claim: 16 September / 14 September 2026; rejected because those dates do not exist in the canonical WebDriver BiDi publication history at this observation.
- Mutable Editor's Draft: https://w3c.github.io/webdriver-bidi/.
- Runtime-qualified OriginWeave adapter pin: WebDriver BiDi Working Draft, 3 September 2026.
- OriginWeave buyer acceptance owner: issue #292.
- OriginWeave profile/standard-adapter parent lineage: PR #229, which has inherited merged PR #293.
- Real pinned-Chromium evidence lane: PR #299.

## References

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* (W3C Working Draft; previous published version and runtime-qualified OriginWeave pin). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

World Wide Web Consortium. (2026). *WebDriver BiDi publication history*. Retrieved September 21, 2026, from https://www.w3.org/standards/history/webdriver-bidi/
