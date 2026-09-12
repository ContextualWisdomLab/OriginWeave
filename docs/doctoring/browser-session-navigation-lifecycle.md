# Browser Session navigation lifecycle authority

Status: acceptance design for the active Browser Session stack; not protected-main runtime behavior.

## Standards boundary

The canonical W3C Technical Report page identifies *WebDriver BiDi* as a Working Draft published on 9 September 2026 and links the immutable dated publication `https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/`. It separately identifies the 3 September 2026 Working Draft as the previous published version and the `w3c.github.io` document as the Editor's Draft.

Publication freshness is not runtime compatibility. OriginWeave's current standard-BiDi presentation adapter remains qualified against the immutable 3 September 2026 publication, as recorded in `docs/traceability/webdriver-bidi-publication-current.md`. Advancing that runtime pin requires dedicated schema, semantics, pinned-Chromium, and cleanup/post-condition requalification; this navigation acceptance work does not repin it.

The 9 September publication defines `browsingContext.Navigation` as a per-navigation identifier and carries it through `NavigationInfo` with the browsing context, timestamp, URL, and user context. It defines separate navigation lifecycle events including `navigationStarted`, `fragmentNavigated`, `navigationCommitted`, `navigationAborted`, and `navigationFailed`. Those protocol identifiers and events are evidence available to a BiDi adapter. They are not OriginWeave policy authority.

## Acceptance decision

The active #316/#317/#318 stack is still proposed integration work. The intended boundary is:

- #316 must use the protocol navigation identifier to distinguish delivery replay from a genuinely later navigation and correlate terminal evidence. It must not expose that identifier as Browser Session capability authority.
- #317 must validate non-authorizing Browser Session incarnation, browsing-context, and context-epoch provenance before invalidating presentation mutation authority and issuing an opaque aggregate-owned pending witness.
- Positive settlement is an OriginWeave mapping from qualified matching commit or same-document fragment evidence. `navigationAborted` and `navigationFailed` map to typed negative terminal outcomes. Neither path implicitly restores presentation authority.
- A later qualified navigation start for the same live context supersedes the earlier pending witness without spending a presentation epoch. Delayed positive or negative terminal evidence for the superseded navigation must fail closed while the newer navigation remains pending.
- Pending witnesses must die on aggregate trust loss or proven loss of the exact context ownership, and terminal assignment must be exactly once. Re-establishment of presentation authority is explicit and is permitted only after the current pending navigation reaches a valid terminal outcome.

This design deliberately keeps WebDriver BiDi as an adapter contract. Browser Session remains the deterministic authority boundary, and command acknowledgement or protocol event receipt is not a browser-observed success post-condition.

## Evidence still required

The current #318 branch supplies hostile contract tests, not production implementation or browser acceptance. A releasable path still requires #317 production state-machine repair, exact-head repository contracts, canonical rustfmt, locked tests, strict Clippy, warning-denying rustdoc, exact function/line/region/branch coverage, non-destructive #316 adoption, and real pinned-Chromium evidence for start → invalidation → current terminal → explicit re-establishment, including replay, overlapping-navigation, crash/transport-loss, and cleanup cases.

## References

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* (W3C Working Draft; current OriginWeave runtime-qualified standard-BiDi pin). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/
