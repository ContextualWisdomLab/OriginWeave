# Action Post-Condition Evidence Traceability

- **Documentation status:** Active-stack evidence dossier; protected-main truth is called out separately
- **Canonical owner:** issue #28 (`Complete the first real Chromium agent vertical slice`)
- **Protected-main baseline:** `542ca1e9c0a863595b8b6697790005d2471f5413`
- **Active stack tip at this revision:** PR #271 (`feat/network: admit and compare typed-text postconditions`)
- **Capability maturity:** **PARTIAL**
- **Governing decisions:** Accepted ADR 0003 plus Proposed ADR 0106 preserve provenance-native evidence and separation of action execution from verification.

## 1. Why this dossier exists

OriginWeave has a durable product rule: returning from a browser command is not equivalent to successful action completion. A state-changing action becomes successful only after the declared or derived post-condition is observed and verified. This dossier tracks the executable pieces that narrow the first Chromium vertical-slice gap without promoting active pull-request behavior to protected-main shipped truth.

Every exact branch head below is volatile evidence. If a contributor head, live base, dependency, review, or check state moves, its recorded evidence must be revalidated on the new exact state before it is reused.

## 2. Protected-main truth

Protected `main` at `542ca1e9c0a863595b8b6697790005d2471f5413` already contains the controlled local Agent Task fixture from merged PR #65. The checked-in fixture provides a labelled synthetic text field, submit control, deterministic `idle` → `submitted` state transition, and hidden untrusted page instruction used by hostile-content regressions. Its presence on protected main is test-infrastructure truth; it is not proof of a production browser runtime.

Protected main also retains the generic authority/evidence primitives and design requirements that keep observation, policy, execution, and verification separate. It still does not by itself establish the complete pinned-Chromium observation → policy → action → post-condition → evidence → teardown chain required by issue #28.

## 3. Current executable evidence

### PR #64 — verified action-outcome and interruption evidence

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

PR #64 remains open on exact head `5021d142583cb5a8e393248048bb824762a98056` against protected main. Its typed evidence boundary binds verified post-condition provenance to an action intent and keeps retry eligibility fail-closed around exact browser authority, cleanup/finalization state, and possible external effects.

The branch is not protected-main shipped truth. Its current outstanding failures are central review/provider evidence rather than a verified source-level vulnerability: the exact-head OpenCode path lacks an authenticated qualifying review verdict and Strix has returned provider/backend failures. Those states do not become passing evidence and do not justify a local product workaround that weakens the central gate.

### PRs #261–#264 — committed-navigation lifecycle and subscription authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_STACK`

The current issue-#28 navigation stack is dependency ordered:

- PR #261 exact `da84955d74ff12b158a8cb2e75eadf218c787f46`: canonical origin binding after an admitted committed-navigation observation and exact pre-action document epoch;
- PR #262 exact `9df2fc23abf42133beaebab4f5466fbdc942d336`: typed, context-scoped `session.subscribe` for `browsingContext.navigationCommitted` with bounded WebSocket transport and exact correlation;
- PR #263 exact `24fc763f0c4ae4e0dd2c62b9dca4b5bc0d23a94b`: typed unsubscribe consuming the validated opaque subscription receipt; and
- PR #264 exact `9c4116b23e5b35e50bb66fff9f72d52bba3adbd0`: admission of committed-navigation events only while the exact typed subscription authority remains active.

Each remains Draft and mergeable at this revision. Their exact native CI evidence is branch-local and is not transferred to descendants or protected main. Organization-required central checks that are absent from an exact stacked head remain absent evidence rather than implicit success.

### PR #265 — pointer input revalidated against admitted node authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_STACK`

PR #265 exact `ffa70ee0f499b86ff51837fb95733fd5cf57ff89` binds pointer-click serialization and transport to a registry-issued `AdmittedNodeHandle`, the exact admitted WebDriver BiDi `sharedId`, current browser session/context/origin/document epoch, and a non-cloneable validated `TypedInput` protocol-use proof immediately before correlation and network I/O. Cross-registry handles, stale nodes, changed origin authority, wrong external contexts, and caller-selected unadmitted node identifiers fail closed.

The branch remains Draft and mergeable. Native CI and Manifest V3 compatibility were successful on that exact head, but neither automation nor author activity counts as independent approval.

### PR #266 — node-bound non-secret text input and diagnostic redaction

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_STACK`

PR #266 is stacked directly on PR #265. The current implementation adds a deterministic WebDriver BiDi `input.performActions` text-input command that:

1. revalidates the exact registry-issued browser session, external browsing-context identifier, canonical origin, current document epoch, node provenance, and admitted `sharedId` before serialization;
2. focuses the exact admitted element with an element-origin primary-button sequence before keyboard input;
3. accepts only non-empty, bounded, protocol-safe non-secret text; and
4. keeps secrets on the separately governed broker/fill path rather than this public text-input surface.

A current-source privacy defect was found and repaired test-first on this same canonical branch. The original derived `Debug` implementation exposed the complete serialized command, including buyer-provided typed text. Exact RED head `11ada2a54fc3f9fc3225e654670319bc5fa6f0b2` added a diagnostic regression that failed because the private marker was present. The production repair replaced derived `Debug` with a metadata-only representation that retains command id, method, and text byte count while omitting typed text, browsing-context identifiers, admitted node identifiers, and the serialized wire payload.

The pre-documentation exact repaired head `1958720a2f0f7e33e40bcea0073c486f37ad278d` passed CI run `33451284736` and Manifest V3 Compatibility run `33451284820`; Rust contracts included formatting, workspace checks, full tests, strict Clippy, and public API documentation, while Production coverage passed exact owned-production function/line/region/branch enforcement. A later documentation-only head must obtain its own fresh exact-head evidence before these results can be treated as current for the PR.

### PR #269 — fixed text-value observation for an admitted current node

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_STACK`

PR #269 adds a fixed sandboxed text-value observation command on top of the node-bound text-input stack. Construction revalidates the exact registered session, external browsing context, canonical origin, current document epoch, registry-issued node provenance, and admitted `sharedId`. The caller cannot replace the `script.callFunction` method, `node => node.value` function declaration, isolated sandbox, argument shape, or result-ownership policy.

The command only serializes an observation request. It performs no browser I/O, accepts no page/model-supplied script, grants no policy or action authority, and does not prove browser execution or post-condition success. Descendant transport and response slices must correlate the exact response and compare the returned non-secret text with the intended value before any action outcome can become verified.

### PR #271 — correlated typed-text post-condition comparison

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_STACK`

PR #271 admits only a `script.callFunction` response correlated to the exact outstanding text-value observation command family. A successful string result becomes positive evidence only when it exactly equals the already-authorized expected text; a different value returns `PostconditionMismatch`, so protocol acknowledgement and parser success cannot certify browser state.

The page-controlled text is discarded at the comparison boundary. The public result retains only the command identifier and UTF-8 byte count, while errors expose neither observed nor expected text. This evidence does not prove the preceding action was authorized, transported, or executed through the intended browser process, and it remains active-stack evidence until the full vertical slice is integrated and accepted on protected main.

### PR #95 — deterministic semantic-node policy authorization

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

PR #95 consumes the exact semantic-node/business-action binding from #93, and only `Decision::Allow` produces a policy-authorized value. `Decision::Deny` and `ApprovalRequired` remain typed non-authorizing outcomes; neither can be converted into a browser action token.

The retained action must still revalidate registry-owned current authority immediately before dispatch. Policy authorization does not grant destination, secret, approval, or adapter authority, and does not execute browser I/O or prove a post-condition. The branch remains non-shipped evidence until its dependency stack is integrated and accepted on protected main.

### PR #96 — dispatch-time semantic-node authority revalidation

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

PR #96 revalidates the retained registry-issued node authority before invoking one adapter callback. The callback is never invoked when the node is stale, retired, forged, or owned by another registry; after document advance removes the admission, dispatch fails closed as `NotAdmitted`.

The callback result remains adapter-local. Adapter failure stays distinct from authority failure, and adapter completion is not post-condition proof. This boundary grants no destination, secret, approval, or network authority and remains non-shipped until its dependency stack is integrated and accepted on protected main.

### PR #101 — reject known-disabled semantic-node actions

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

PR #101 rejects an advertised interactive action when the retained semantic observation already marks the node disabled, returning typed `NodeNotEnabled`. `ScrollIntoView` remains selectable because scrolling does not require enabled state.

This is a descriptive target filter. It does not observe current Chromium state or authenticate the observation source, and it does not grant dispatch authority. The trusted adapter must still obtain fresh state and the later dispatch boundary must revalidate registry-owned browser authority before input.

## 4. Non-transitive success semantics

The intended first-slice chain remains:

```text
typed action intent
-> policy-authorized dispatch
-> real browser input/event
-> observed bounded post-condition
-> independently verified provenance
-> temporally ordered verified action outcome
```

The following implications remain invalid:

```text
command return -/> successful action completion
protocol acknowledgement -/> successful action completion
subscription receipt -/> event occurrence
admitted node handle -/> policy authorization
successful pointer/text serialization -/> successful browser state change
Unverified or Rejected provenance -/> successful action completion
caller-supplied timestamp ordering -/> trusted clock provenance
typed evidence object existence -/> proof of real Chromium execution
controlled fixture success -/> proof of real Chromium execution
```

The active navigation/input stack narrows browser transport and node-lifetime authority, but it does not prove that a dispatched input caused the declared post-condition. The verified outcome boundary remains separate, and the final runtime must compose real browser execution with post-dispatch observation and credential-safe provenance without inheriting ambient browser, policy, destination, secret, or model authority.

## 5. Current issue #28 dependency shape

The first real Chromium vertical slice remains distributed rather than shipped as one protected-main runtime. Current relevant boundaries include:

- protected-main controlled hostile workflow fixture from merged PR #65;
- browser protocol/session/context/origin/document/node authority primitives already represented in the repository;
- PR #64 verified post-condition and interruption evidence;
- PRs #261–#264 committed-navigation origin/subscription/admission lifecycle;
- PR #265 send-time pointer input revalidation against admitted node authority; and
- PR #266 node-bound non-secret text input with privacy-safe diagnostics; and
- PR #269 fixed text-value observation construction, still without browser I/O or outcome verification;
- PR #270 exact transport of that fixed observation, still without response success; and
- PR #271 correlated response admission and exact text-value comparison with value-free evidence.

These pieces do not transfer evidence across heads. A descendant must be revalidated after any parent movement, and protected-main shipment requires fresh integrated acceptance after dependency-ordered merge by an authorized integrator.

## 6. Remaining issue #28 boundary

This dossier does **not** close issue #28. Material remaining work includes:

- one reproducible pinned stock-Chromium Agent Task path that composes the current authority kernels rather than proving them only in isolated protocol fixtures;
- isolated task profile/context lifecycle and deterministic cleanup in that production vertical path;
- real semantic observation feeding typed query and policy-authorized typed action;
- real pointer/text dispatch followed by an independently observed declared post-condition and verified credential-safe evidence;
- hostile stale/cross-session/cross-context/cross-origin/prompt-injection/secret-leak/crash/oversize regressions across the integrated runtime;
- deterministic renderer/tab/process failure and recovery evidence;
- Chromium process-set discovery/attribution composed into resource telemetry; and
- fresh protected-main security, coverage, rustdoc, browser compatibility, provenance, review, rollback, and operational acceptance before release claims.

## 7. Documentation fitness consequence

The documentation graph remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**. The active stack materially narrows WebDriver BiDi navigation and typed-input authority, but it introduces no new OriginWeave-owned durable database schema or persistence owner. A physical ERD entity would therefore overstate the implementation. Detailed as-built sequence diagrams should be reconciled when the executable pinned-Chromium composition is stable enough that they describe measured runtime behavior rather than anticipated integration.
