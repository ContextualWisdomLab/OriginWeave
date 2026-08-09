# ADR 0103: Semantic observation precedence and stale-node identity

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

Agentic browsing needs compact, stable observations without treating one browser representation as universally authoritative. WebMCP or structured application data can expose high-level semantics, accessibility trees can expose user-visible structure, DOM/layout can fill gaps, and visual interpretation is sometimes necessary. Each layer can be incomplete, adversarial, stale, or unavailable. Node references are particularly dangerous after navigation or document mutation because a selector or backend identifier can refer to different state later.

## Decision drivers

- Prefer the highest-signal structured representation while retaining independent verification.
- Avoid blind trust in experimental WebMCP or page-authored semantics.
- Preserve accessibility and user-visible meaning.
- Prevent actions against stale nodes after document replacement or relevant mutation.
- Keep observation provenance sufficient to explain what the agent saw.

## Assumptions and authority boundaries

All page-derived observations are untrusted data. Observation sources can inform planning but cannot grant action capability or approval. A semantic node identity is meaningful only within its browser session/context and document epoch. Network or structured-data observations may corroborate semantics but do not override policy.

## Options considered

1. DOM-only observation: rejected because it is noisy and can diverge from accessibility or application semantics.
2. Visual-only observation: rejected because it is expensive, difficult to make deterministic, and loses structured provenance.
3. WebMCP-first authority: rejected because WebMCP is experimental and page/tool output can contain prompt injection.
4. Ordered multi-source observation with explicit provenance and stale identity: selected.

## Decision

Observation precedence is WebMCP/explicit structured application contracts when available, then structured data and network-derived semantics, accessibility semantics, DOM, layout, and bounded visual fallback. Higher precedence means preferred representation, not unquestioned trust. Every observation records source and document identity.

Actionable semantic nodes bind to the exact browser session, browsing context, and a monotonically increasing `document_epoch`. The browser adapter increments the epoch on cross-document navigation and on any same-document lifecycle event that can invalidate the meaning or target identity of an actionable observation, including replacement/removal of an observed DOM node, reassignment of the node's actionable role/name/state, relevant accessibility-tree invalidation, frame-document replacement, or a subtree mutation that replaces the observed target rather than merely changing unrelated descendants. The adapter may coalesce multiple mutation notifications into one epoch increment, but it must never retain the old epoch when an actionable handle's target identity or user-visible semantics could have changed.

The adapter publishes an explicit observation-invalidation lifecycle event before a stale handle can be executed. Any action request carrying a semantic-node handle must revalidate session, browsing context, epoch, and node identity at the **action linearization point**: the trusted adapter boundary immediately adjacent to the browser side effect. Where the browser integration can provide an atomic compare-and-dispatch primitive, validation and dispatch use that primitive. Otherwise the adapter performs the same session/context/origin/epoch/node check again immediately before the side effect and aborts with a stale-reference result if a **competing mutation** or invalidation event occurs between the earlier policy validation and dispatch. The adapter must never rely on a pre-dispatch check whose result can become stale before input delivery.

If the current epoch differs, or the adapter cannot prove the node identity still denotes the same current actionable target at the linearization point, the request fails without producing the side effect and must be re-observed. **Re-observation** produces a new handle bound to the new epoch; matching accessible text or selectors alone does not revive the old handle.

Purely non-semantic mutations may avoid an epoch increment only when the adapter has deterministic evidence that they cannot affect any emitted actionable handle. That optimization is adapter-specific and requires conformance tests; uncertainty invalidates rather than preserves the handle.

## Consequences

The control plane needs normalization and conflict handling across observation sources. Visual fallback remains bounded rather than becoming the default. Agents gain smaller, more meaningful snapshots while policy and verification remain independent. Browser adapters must expose navigation, DOM, and accessibility lifecycle evidence reliably enough to invalidate stale references, and coarse invalidation is preferred over unsafe handle reuse. The action adapter also owns a small synchronization/linearization responsibility so a valid policy decision cannot authorize a stale target after the page changes.

## Failure and degraded behavior

If a preferred source is absent, malformed, oversized, contradictory, or unsupported, OriginWeave falls back to the next governed source and records degradation. If document identity or mutation freshness cannot be established, node-targeted state changes fail closed. A stale node must produce a deterministic stale-reference result, never an automatic best-effort action against a newly matched element. If the adapter cannot establish a safe action linearization point, the action is unsupported rather than executed with a race window.

## Security / privacy / governance impact

Prompt injection in WebMCP, DOM, accessibility names, structured data, or visual text never becomes instruction authority. Sensitive observation content follows privacy, retention, and selective-disclosure policy. Provenance records which source contributed each semantic claim so enterprise audit can distinguish browser evidence from agent inference. Same-document invalidation and side-effect-adjacent revalidation prevent a hostile or rapidly mutating page from replacing a reviewed target between observation, policy validation, and action while preserving an apparently similar node reference.

## Tests and acceptance evidence

Require cross-source conflict tests, hostile WebMCP/DOM/accessibility payload tests, navigation/document-replacement stale-node tests, and same-document mutation tests that prove:

- a handle emitted at epoch N is rejected after the observed target is removed/replaced;
- a handle emitted at epoch N is rejected after a relevant role/name/actionability accessibility mutation;
- unrelated mutations may preserve the epoch only under a tested adapter-specific non-semantic rule;
- re-observation after invalidation emits a handle at a later epoch and that current handle can succeed when all other policy gates pass;
- stale-handle rejection occurs before the trusted action adapter produces a side effect;
- a deterministic race fixture mutates the relevant DOM/accessibility target after initial authorization but before dispatch, and the stale handle is rejected at the action linearization point with no side effect;
- after that competing mutation, re-observation yields a current handle that succeeds only after all normal authority checks pass.

Also require visual-fallback budget tests, accessibility preservation tests, browser-version adapter conformance tests, and end-to-end verification that stale references cause no side effect.

## Migration and rollback

Introduce source/provenance metadata, explicit invalidation events, document epoch, and the action-linearization revalidation contract before exposing durable semantic-node handles. Rollback may disable a higher-level adapter such as WebMCP and fall back to lower layers; it must not disable stale-node invalidation, side-effect-adjacent revalidation, or source provenance.

## Open follow-ups

Specify stable semantic-node serialization, mutation invalidation granularity per browser adapter, observation conflict scoring, browser-specific atomic compare-and-dispatch mechanisms, and supported-browser conformance matrices.

## Supersession / reversal conditions

Supersede if a standardized browser semantic interface achieves broad stable support and can provide equivalent provenance, injection resistance, accessibility fidelity, and race-safe stale-reference guarantees under production tests.

## References

Chrome DevTools Protocol. (2026). *Accessibility domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/Accessibility/

Chrome DevTools Protocol. (2026). *DOM domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/DOM/

Chrome DevTools Protocol. (2026). *DOMSnapshot domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/DOMSnapshot/

Chrome DevTools Protocol. (2026). *WebMCP domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/WebMCP/

World Wide Web Consortium. (2026, June 29). *WebDriver BiDi* [Working Draft]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260629/

## Related documents

See `docs/TRD.md`, `docs/API_CONTRACT.md`, `docs/THREAT_MODEL.md`, `docs/uml/README.md`, and the session/document work tracked by the existing OriginWeave architecture.
