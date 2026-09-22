# WebDriver BiDi publication-current receipt

Status: active standards traceability
Observed: 2026-09-22
Runtime-compatible pin: `2026-09-03`
Latest published Working Draft: `2026-09-16`
Previous published Working Draft: `2026-09-14`
Earlier published Working Draft: `2026-09-09`
Editor's Draft: `https://w3c.github.io/webdriver-bidi/`

## Problem

The `originweave-bidi` presentation capability map is deliberately version-pinned, while publication provenance can move independently. A fresh 2026-09-22 read of W3C's live WebDriver BiDi publication-history surface lists Working Drafts for **16 September 2026**, **14 September 2026**, **9 September 2026**, and **3 September 2026** in descending order. The immutable 14 September Technical Report is also live and identifies the 9 September Working Draft as its previous version.

The predecessor #229 generation said the 16 September / 14 September entries did not exist and therefore treated 9 September / 3 September as the current publication pair. That statement is no longer true against live primary W3C metadata. This repair advances the publication receipt without changing the separately qualified runtime pin.

Publication freshness and runtime qualification are intentionally different authorities. Treating them as one datum creates two bad failure modes: documentation can become false whenever W3C publishes a new draft, or an automation can silently repin runtime compatibility without re-running the browser/protocol qualification that gives the pin meaning.

## Current authoritative publication

Canonical latest publication alias: https://www.w3.org/TR/webdriver-bidi/

Canonical publication history: https://www.w3.org/standards/history/webdriver-bidi/

Latest immutable published Working Draft listed by the live history: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260916/

Previous immutable published Working Draft listed by the live history: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260914/

Earlier immutable published Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

Runtime-qualified immutable Working Draft: https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

Mutable Editor's Draft: https://w3c.github.io/webdriver-bidi/

The 14 September publication labels itself a W3C Working Draft and names the 9 September draft as its previous version. W3C's live publication-history page now places a 16 September Working Draft above that 14 September entry. These are publication-provenance facts only; they do not establish that OriginWeave's adapter has been requalified against either newer draft.

## Runtime compatibility decision

OriginWeave keeps `WEBDRIVER_BIDI_PRESENTATION_REVISION = "2026-09-03"` until a dedicated compatibility change proves that a newer immutable draft preserves the exact command schemas, reset semantics, capability interpretation, browser implementation behavior, and pinned-Chromium acceptance required by the adapter.

A publication-freshness update therefore does **not** mutate the runtime pin, claim new browser capability, or promote command acknowledgement to presentation evidence. The safe sequence is:

1. read W3C's live publication history and the immutable dated Technical Reports;
2. retain publication provenance separately from the runtime-qualified revision;
3. treat cached search/discovery surfaces, generated summaries, and prior repository prose as non-authoritative when they disagree with fresher primary W3C metadata;
4. keep the mutable Editor's Draft explicitly separate from dated Technical Reports;
5. diff the relevant specification surfaces and update the versioned capability map only if needed;
6. re-run repository contracts and pinned Chromium/BiDi/CDP compatibility evidence on any proposed new runtime pin;
7. update architecture/ADR/doctoring compatibility claims together with the qualified pin and keep unsupported or unverified surfaces fail closed.

## Relationship to buyer acceptance

This receipt does not close OriginWeave #292. Buyer-visible acceptance still requires a version-pinned real Chromium path to apply the complete admitted presentation profile, observe the page-visible post-condition, survive navigation/renderer/crash cases, and prove cleanup or owned disposable-context destruction. Current #299 evidence remains pre-navigation RED, so publication freshness cannot be counted as browser GREEN.

## Traceability

- W3C live publication history observed 2026-09-22: WebDriver BiDi Working Draft entries for 16 September, 14 September, 9 September, and 3 September 2026 in descending order.
- The immutable 14 September Working Draft identifies 9 September as its previous version.
- Superseded repository claim: 16 September / 14 September 2026 did not exist in canonical publication history. That claim is contradicted by the current primary history surface.
- Mutable Editor's Draft: https://w3c.github.io/webdriver-bidi/.
- Runtime-qualified OriginWeave adapter pin: WebDriver BiDi Working Draft, 3 September 2026.
- OriginWeave buyer acceptance owner: issue #292.
- OriginWeave profile/standard-adapter parent lineage: PR #229, which has inherited merged PR #293.
- Real pinned-Chromium evidence lane: PR #299.

## References

World Wide Web Consortium. (2026, September 16). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260916/

World Wide Web Consortium. (2026, September 14). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260914/

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* (W3C Working Draft; runtime-qualified OriginWeave pin). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

World Wide Web Consortium. (2026). *WebDriver BiDi publication history*. Retrieved September 22, 2026, from https://www.w3.org/standards/history/webdriver-bidi/
