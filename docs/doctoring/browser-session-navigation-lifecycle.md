# Browser Session navigation lifecycle authority

Status: acceptance design for the active Browser Session stack; not protected-main runtime behavior.

## Standards boundary

Fresh verification on 14 September 2026 found a directly retrievable immutable W3C publication at `https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/`. The document identifies itself as *WebDriver BiDi*, W3C Working Draft, 9 September 2026, and names `https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/` as its previous published version. The 3 September dated document is also directly retrievable and identifies itself as a W3C Working Draft.

The mutable `https://www.w3.org/TR/webdriver-bidi/` publication surface and some W3C indexes were observed lagging behind those dated publications and still presenting an August draft. That index lag is not evidence that an immutable dated W3C publication is absent. OriginWeave therefore records dated standards claims from the immutable W3C URI itself and treats mutable-index freshness as separate evidence. Publication freshness remains separate from runtime qualification: this acceptance branch does not repin the adapter merely because a newer dated draft exists.

The 9 September publication defines `browsingContext.Navigation` as a per-navigation identifier and carries navigation/context correlation through `NavigationInfo` and `BaseNavigationInfo`. It defines separate lifecycle events including `navigationStarted`, `fragmentNavigated`, `domContentLoaded`, `load`, `downloadWillBegin`, `downloadEnd`, `navigationAborted`, `navigationCommitted`, and `navigationFailed`. These protocol identifiers and events are adapter evidence, not OriginWeave policy authority.

The same publication distinguishes commitment from completion. `navigationCommitted` is progress evidence; it is not a Browser Session terminal success. The navigation waiter may subsequently resolve through document readiness, fragment navigation, `"download started"`, `navigationAborted`, or `navigationFailed`. `downloadWillBegin` therefore closes the Browser Session navigation-liveness boundary without proving file-download completion, scanning, egress approval, quarantine outcome, or `downloadEnd` success. Those download-file responsibilities remain with their canonical owners.

## Acceptance decision

The active #316/#317/#318 stack remains proposed integration work. Browser Session owns deterministic mutation authority; the BiDi driver remains an adapter.

`navigationStarted` must validate the exact `(BrowserSessionIncarnation, BrowsingContextId, BrowserContextEpoch)` before mutation, revoke retained presentation mutation authority, and issue an opaque aggregate-owned pending witness. The observation itself performs no browser I/O and spends no presentation epoch.

`navigationCommitted` is non-terminal progress bound to that exact witness. It neither consumes the witness nor enables presentation-authority re-establishment. A matching `load` or complete `fragmentNavigated` may close the navigation positively; `navigationAborted` and `navigationFailed` close it through distinct typed negative outcomes; matching `downloadWillBegin` closes navigation liveness through its own typed path. Each qualified closing transition consumes the witness for authority decisions exactly once.

After a witness is consumed, delayed or duplicate `navigationCommitted`, positive settlement, negative terminal evidence, or duplicate download-start evidence must be rejected before lifecycle state, recovery evidence, retained/current authority, re-establishment eligibility, presentation epoch, or adapter I/O can change. That rule applies before and after explicit re-establishment. Rejected evidence may not temporarily mutate state and rely on a later event to compensate.

A qualified closing outcome enables at most one explicit `reestablish_presentation_authority`. The minted authority must be usable for a real authorized operation on the exact current ownership generation; matching epoch arithmetic alone is insufficient. A second re-establishment without a newer qualified navigation fails closed and leaves the current authority usable.

A newer navigation in the same live context supersedes an older pending witness and any unused re-establishment opportunity from the older generation without spending an epoch. Delayed evidence from the superseded witness must not be remapped into the newer navigation. Sibling contexts remain independent: a pending navigation in B must not erase a valid re-establishment opportunity in A, and A may re-establish while B remains fail-closed pending its own qualified closure.

All navigation authority is bound to exact context ownership generation, not raw `BrowsingContextId`. Proven destruction consumes that generation's pending witness and unused re-establishment opportunity. Destroy-then-recreate raw-id reuse receives a newer `BrowserContextEpoch`; stale witnesses and authorities from the destroyed generation must fail before adapter I/O. Aggregate trust loss (`TransportLost`, `RecoveryRequired`, ended session) also dominates any otherwise valid pending or terminal-derived opportunity.

`navigationCommitted(A)` arriving after `downloadWillBegin(A)`, ordinary positive closure, or typed negative closure is permanently stale. This remains true if a newer same-context navigation B is pending and after A has already re-established authority. Stale A evidence must not create a hidden second eligibility, advance the aggregate epoch, alter B's pending state, or revoke A's current fresh authority.

## Detailed hostile acceptance matrix

The acceptance suite deliberately separates cases that could otherwise compensate for one another. Each rejected observation is checked immediately, before a later event can restore state.

- **Commit progress:** `navigationStarted → navigationCommitted` keeps the same pending witness alive, keeps retained pre-navigation authority revoked, performs zero adapter I/O, spends no presentation epoch, and does not permit re-establishment. The same witness may later close through complete-positive, `navigationAborted`, `navigationFailed`, or `downloadWillBegin`.
- **Exactly-one terminal assignment:** once one positive/negative/download-start closure consumes a witness, every competing closing outcome for that witness is stale. A stale outcome cannot overwrite the recorded terminal/liveness fact or manufacture another authority transition.
- **Duplicate download start:** duplicate `downloadWillBegin` is non-mutating both before and after re-establishment. Before re-establishment it preserves the one legitimate download-derived opportunity; after re-establishment it leaves the fresh authority usable and does not consume another epoch.
- **Late commit after closure:** delayed `navigationCommitted` after positive, negative, or download-start closure is non-mutating before and after re-establishment. The stale event cannot recreate eligibility, revoke the current fresh authority, or perturb the next qualified navigation's epoch.
- **Same-context supersession:** `navigationStarted(B)` in the same ownership generation supersedes pending A and any unused A-derived re-establishment opportunity. Delayed A progress/terminal/download evidence fails while B remains pending and cannot be reinterpreted as B evidence.
- **Download-close then newer start:** if `downloadWillBegin(A)` closes A and B starts before A re-establishes, B start discards A's unused opportunity but keeps A's witness consumed. Late A `navigationCommitted`, positive settlement, `navigationAborted`, `navigationFailed`, or duplicate download-start remains non-mutating while B is pending.
- **Sibling-context independence:** terminal/download eligibility in context A may coexist with pending navigation in sibling context B. Starting or closing B does not erase A's opportunity. A may re-establish and execute a current operation while B remains fail-closed pending its own closure.
- **Context-local destruction:** proven destruction consumes only the destroyed context's ownership, pending witness, and unused re-establishment opportunity. It must not clear or settle a sibling pending navigation, synthesize recovery evidence for the sibling, or spend a presentation epoch.
- **Destroy while pending:** delayed evidence for a proven-destroyed pending context is rejected as ownership-invalid and must not alter a surviving sibling's pending state or current authority.
- **Destroy/recreate ABA:** reuse of the same raw `BrowsingContextId` after proven destruction receives a newer `BrowserContextEpoch`. Prior-generation navigation witnesses, re-establishment opportunities, and presentation authority remain stale even when external ids are equal.
- **Terminal-then-destroy-then-recreate:** an unused terminal-derived opportunity belongs to the destroyed ownership generation and cannot be inherited by the recreated generation. Only a qualified closure produced by the recreated generation may enable its next re-establishment.
- **Invalid later observation:** cross-context, cross-incarnation, wrong-epoch, stale-generation, or otherwise invalid navigation evidence is rejected before mutation and cannot erase an already valid pending witness, terminal-derived opportunity, or current authority.
- **Aggregate trust loss:** pending witnesses and unused opportunities die when the session enters `TransportLost`, `RecoveryRequired`, or `Ended`. Earlier navigation evidence cannot be used to resurrect mutation authority after aggregate trust is lost.
- **Executable authority:** every successful re-establishment is checked through an actual authorized operation routed to the exact bound adapter/current context. A token carrying only the expected epoch but unable to execute the intended operation is not accepted.
- **Epoch conservation:** navigation observation, commit progress, download-start observation, rejected replay, destruction proof, and sibling-context events do not silently spend a presentation epoch. Each successful explicit re-establishment advances the aggregate-issued presentation epoch exactly once.
- **Download ownership boundary:** `downloadWillBegin` is only navigation-liveness evidence. `downloadEnd`, file persistence, scanning, egress control, quarantine, and malware decisions remain outside Browser Session and do not mint presentation authority.

## Evidence still required

#318 supplies hostile acceptance contracts, not the production navigation state machine. #317 remains the production owner and must implement the same validation-before-mutation invariants while preserving the existing lifecycle/recovery boundaries. #316 remains the protocol-correlation owner and must map exact BiDi navigation/context evidence without turning driver identifiers into Browser Session capability authority.

Before this stack can be adoption-ready, one exact successor head must pass repository contracts, canonical rustfmt, locked workspace tests, strict Clippy, warning-denying rustdoc/API documentation, and production function/line/region/branch coverage at 100%. Real acceptance must then exercise the corresponding paths against an explicitly qualified Chromium/WebDriver BiDi runtime and verify browser-observed post-conditions rather than command acknowledgement alone.

Required real-browser scenarios include commit-progress followed by positive, negative, and download-start closure; replay before and after re-establishment; overlapping same-context navigation; sibling-context independence; destruction while another context is pending; raw-id ABA reuse; trust loss; crash/cleanup; and the ability of each newly minted authority to perform its intended current-context browser mutation.

## Traceability

- Production Browser Session owner: PR #317.
- WebDriver BiDi correlation owner: PR #316.
- Test-first navigation acceptance successor: PR #318.
- Navigation-to-download liveness acceptance: issue #320.
- WebDriver BiDi publication/runtime receipt: `docs/traceability/webdriver-bidi-publication-current.md`.
- Publication-current repository contract: `tests/test_webdriver_bidi_docs_currentness_contract.py`.

## References

World Wide Web Consortium. (2026, September 9). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260909/

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/

World Wide Web Consortium. (2026). *WebDriver BiDi: latest published version*. https://www.w3.org/TR/webdriver-bidi/
