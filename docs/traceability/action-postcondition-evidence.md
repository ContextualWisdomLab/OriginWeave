# Action Post-Condition Evidence Traceability

## Text responses retain pointer safeguards — 2026-09-07

Ordinary merge `34e537b1` adopts published #267 `ebd507ae` while preserving
#268 `e567af9e` and its complete sealed text-response consumer. The text sender,
consumer, exports, all text tests and entire core crate are unchanged. Actual
RED `716fd842` first reproduced foreign-session click dispatch and replacement
success/error consumption: all three pointer regressions failed before adoption.
The inherited canonical sender and sealed click consumer now reject those cases
without consuming the original or unrelated pending work.

Text replies already require the sending connection and exact command family;
missing provenance, malformed envelopes and unknown identifiers preserve pending
work, while matched remote errors consume only their own request. No generic
response fallback or duplicated parser is introduced. The earlier text repair's
hosted success belongs to `e567af9e`, not this combined head, which requires fresh
full verification and actual visual inspection. Observed field values, browser
authentication, policy approval and causal action success remain unproven.
Status receipts and protected-foundation integration remain separate owner work.
The dated checkpoints below preserve predecessor evidence, not current acceptance.

## Text receipt-provenance repair — 2026-09-06

Actual socket RED `4632f2df` accepted a reply from a replacement connection sharing
the original listener and session. Sealed consumer `d6889c80` then exposed missing
sender provenance. Ordinary merge `e1188c86` adopts canonical #267 sender fix
`3346d8ec`; it retains the transport's private generation before sending.
The consumer now requires the existing sealed received-message type and compares
the receiving connection before consuming correlation. No caller-supplied generation,
raw-message fallback or new parser is introduced.

Seven focused tests pass at `35cb1197`: foreign success/error rejection with original
connection recovery and unrelated pending-state retention, real success and remote
error completion, missing sender provenance, wrong command family, malformed envelope
and unknown id. Extensible results and payload-free error chains remain intact.
Complete current-head verification and publication are still pending. This repair
does not prove browser-process ownership, policy approval or observed field values.
Pointer and status receipt consumers remain separate required owner repairs.

The following integration checkpoints are historical evidence, not current semantics.

## Text-response integration checkpoint — 2026-09-06

#268 adopts #267 `4435ce5f561ca069c1844a1a5bd9b603505e25f7` by ordinary merge,
preserving the original response validation and transport safeguards. Test-first
commit `a9446fbc` requires success and error replies to reject every other registered
command family without consuming its pending request. Parent adoption exposed the
removed generic correlation call; `cd93d9fc` reuses the existing `TypeText` family
guard. Five focused response tests pass, including ten wrong-family cases, the real
send/receive round trip, extensible success results, typed remote failure, malformed
envelopes and unknown identifiers. Complete combined-head verification remains pending.

The consumer still receives an assembled message without authenticated receipt
provenance. Family isolation does not prove that the reply arrived on the command's
connection, that text appeared in the field, or that an action was authorized.
Connection-bound receipt admission remains a separate required repair before runtime
acceptance. Earlier checkpoint evidence below is revision-specific, not current-head
CI, protected-main delivery or release acceptance.

## Historical parent: text sender adopts pointer safeguards — 2026-09-07

Ordinary merge `7e4bd76d` adopts #266 `e3885f69` while preserving #267
`3346d8ec`, including its unchanged text sender, public exports, text tests and
entire core crate. Actual RED `4e020e16` reproduced three inherited failures:
foreign-session click dispatch and replacement success/error replies consuming
the original request. The parent repair rejects the foreign session before any
request is reserved or sent, and rejects replacement replies without consuming
either pending command; the original reply still completes only its own request.

Text dispatch already revalidates current node and session authority, registers
the sending connection, validates deadlines before registration, and distinguishes
proven no-write rejection from ambiguous writes. Those boundaries are unchanged.
The inherited click consumer uses sealed connection evidence; this does not finish
the separate text-response consumer owned by #268. Field-value post-conditions,
browser authentication, action policy and causal browser acceptance remain open.
Full combined-head checks and actual visual inspection must be recorded separately;
older checkpoints below are historical, not current hosted or protected-main proof.

## Text sender receipt-provenance prerequisite — 2026-09-06

Child #268 regression `4632f2df` opens two connections to the same listener and
session, sends text input on the first and observes a successful acknowledgment
from the second being accepted. Sealed-consumer candidate `d6889c80` rejects both
connections because the sender has not retained connection provenance. This owner
repair reuses `register_command_for_connection` before frame I/O and records the
established transport's private generation. It preserves session/current-node
validation, deadlines, no-write retirement and ambiguous-write retention.

This prerequisite alone does not make generic consumers connection-sensitive.
#268 must adopt it and finish sealed-reader migration, foreign success/error
rejection, original-connection recovery and complete verification. The regression
and consumer commits are local integration evidence until their publication is
verified. Browser authentication, policy approval and observed text-value success
remain unproven. Earlier checkpoint evidence below remains revision-specific.

## Text-transport integration checkpoint — 2026-09-06

#267 adopts #266 `eb6c236ff2f4a58b807a2f2c914bd1ddb6079fb3` through ordinary merge
`d903cf6b`, preserving original transport `46a05d7f`. Text dispatch reconstructs the
command from current node authority and now uses the parent's sealed typed-write lane
with a distinct `TypeText` correlation family. The shared frame-deadline validator
runs before registration. Actual RED `2b960002` exposed a zero deadline retaining an
unsent command; RED `e2b49e68` exposed the same retention after reused-mask preflight.
The repair retires only malformed-frame rejections that prove no write began and
retains correlation after ambiguous socket failure. Independent review then exposed a
pre-existing cross-session gap: RED `10131eb7` showed that a node admitted for session A
could be sent on session B's transport. Dispatch now uses the parent's canonical
read-only registry-to-transport session check before correlation or action bytes.
The socket regression requires the exact typed mismatch, zero pending commands and
wire silence; valid fixtures name the same session at both boundaries. Complete
exact-head acceptance is still pending.

The transport is not policy approval, browser authentication, a typed response consumer,
an observed text-value post-condition, protected-main delivery, or release evidence.
Those boundaries remain separate #28 work. The predecessor dossier below is historical;
its counts, source heads and hosted results are not current combined-head acceptance.

## Historical parent pointer session and reply adoption — 2026-09-07

Ordinary merge `d9503b30` adopts #265 `e94a2372` without changing this child's
text-command source, exports or eight existing text/privacy tests. Three real socket
regressions at `009f9a41` first reproduced foreign-session pointer dispatch and
replacement success/error replies consuming the original request (0/3 passing).
The integrated sender keeps current-node, outbound-session and monotonic typed
dispatch checks; its replies require the exact sending connection. Foreign replies
leave both pending commands intact, and the original reply completes only its own.

All 21 focused pointer, navigation-postcondition and text-command tests pass. The
child's revised issue-#28 dossier remains intact alongside the parent's dated receipt
history. This is still bounded non-secret text construction, not typed text dispatch
or observed browser success. The existing #267 transport owns the next integration;
this change does not duplicate it. Full combined-head verification, hosted checks
and actual visual inspection are separate gates, and none of the historical heads
below supplies protected-main, policy, browser-authentication or release acceptance.

## Historical parent-adoption checkpoint — 2026-09-06

Ordinary integration `5a722867` preserves the text-input and diagnostic-privacy delta
from `cc9980c0` while adopting pointer parent `7147893c96ca95c9b5b275d8011c5bfe99aab065`.
The inherited foreign-session socket regression first failed at `18a64573`; the
parent's canonical session guard, current-node pointer revalidation, typed dispatch,
deadline safeguards and four restored navigation-postcondition tests are retained.
Text-command source and its existing eight regression bodies are unchanged.

This text slice still constructs a bounded command; it does not implement a typed
text transport, revalidate text authority at dispatch, authorize actions, or prove a
browser state change. Parent pointer/subscription safeguards do not supply those
missing text boundaries. Current protected main is `87c4daa1830bac5a5228b6036752ad5633232085`.
The dossier below retains its earlier revision-specific observations; old exact-head
CI and screenshots are not combined-head acceptance. Keep Draft pending fresh full
checks, visual inspection, parent-first protected integration and runtime evidence.

- **Documentation status:** Active-stack evidence dossier; protected-main truth is called out separately
- **Canonical owner:** issue #28 (`Complete the first real Chromium agent vertical slice`)
- **Protected-main baseline:** `542ca1e9c0a863595b8b6697790005d2471f5413`
- **Active stack tip at this revision:** PR #266 (`feat/core: add node-bound WebDriver BiDi text input`)
- **Capability maturity:** **PARTIAL**
- **Governing decisions:** Accepted ADR 0003 plus Proposed ADR 0106 preserve provenance-native evidence and separation of action execution from verification.

## 1. Why this dossier exists

OriginWeave has a durable product rule: returning from a browser command is not equivalent to successful action completion. A state-changing action becomes successful only after the declared or derived post-condition is observed and verified. This dossier tracks the executable pieces that narrow the first Chromium vertical-slice gap without promoting active pull-request behavior to protected-main shipped truth.

Every exact branch head below is volatile evidence. If a contributor head, live base, dependency, review, or check state moves, its recorded evidence must be revalidated on the new exact state before it is reused.

## 2. Protected-main truth

Protected `main` at `542ca1e9c0a863595b8b6697790005d2471f5413` already contains the controlled local Agent Task fixture from merged PR #65. The checked-in fixture provides a labelled synthetic text field, submit control, deterministic `idle` → `submitted` state transition, and hidden untrusted page instruction used by hostile-content regressions. Its presence on protected main is test-infrastructure truth; it is not proof of a production browser runtime.

Protected main also retains the generic authority/evidence primitives and design requirements that keep observation, policy, execution, and verification separate. It still does not by itself establish the complete pinned-Chromium observation → policy → action → post-condition → evidence → teardown chain required by issue #28.

## 3. Current executable evidence

### Historical pointer-click connection-bound receipt checkpoint

Ordinary merge `0234b587d1bca9286eb5b597f9dab33be47ff518` integrates #257
`9451fd8a23dec95b31749376bc78c2eaca977fe8` with #258's sealed received-message
consumer. Test commit `d9396f05` strengthened the published `8193fcd5` regression:
two sockets share one listener and session; foreign success and error replies
must produce the exact connection-mismatch error, retain both pending commands,
and permit the original reply to complete only its click. Both cases failed
before repair. Consumer-only `588fe731` also failed original-response acceptance
because the sender lacked connection provenance; this prevents a partial repair
from appearing complete.

All six focused response and replacement-connection tests pass on the integrated
tree. Existing malformed-envelope, unknown-id, extensible-success and matched
remote-error behavior is retained. No public receipt constructor, caller-supplied
generation, generic fallback, dependency, or new authority boundary was added.
This supersedes the receipt prerequisite below, not its dated evidence or the
remaining outbound session-authority and real-browser postcondition gaps.
Full exact-head verification, hosted acceptance and protected integration remain
separate requirements; focused loopback success does not establish them.

### Historical pointer-click originating-connection prerequisite

PR #258 test-only head `8193fcd50125d9e9a43b4755e0f7626801b74374`, on
PR #257 `8f1507346f65798a6bf4eaf370d65a2d406a6f44`, reproduced a replacement
connection consuming the original connection's pointer command. The Rust 1.97.1
loopback regression failed at its rejection assertion (zero passed, one failed);
the four predecessor response tests passed separately.

The #257 sender now registers the existing private transport generation before
writing, reusing the shared correlation owner without changing local deadline
rejection, preflight retirement, or ambiguous-write retention. This prerequisite
alone does not reject foreign responses: #258 must consume the existing sealed
received-message type and require connection-bound correlation. Its regression
must also retain unrelated requests and allow the original connection's response.
Outbound session authority, browser authentication, observed click effects,
protected-main acceptance, and release evidence remain separate and unproven.

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
- PR #266 node-bound non-secret text input with privacy-safe diagnostics.

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

## 8. Pointer descendant reply integration

The #265 integration at `d847b530` preserves current-node and outbound-session checks
from `ddce7248`, then adopts parent #264 `43395711` reply provenance. The real socket
regression `e7fb1527` first reproduced replacement success and error consuming the
original click request. Both now reject the foreign connection while retaining the
original request and unrelated work; the genuine original reply still completes.

All 14 focused tests pass, including stale-node rejection, foreign-session rejection
before pending state or command bytes, and the preserved navigation-postcondition
cases. Full exact-head local/hosted gates and visual inspection remain independently
required. No inherited checkpoint establishes acceptance for this combined tree.

A matching response is still only protocol acknowledgment. Policy approval, browser
authentication, trusted event provenance and causal page effects remain separate.
This active Draft does not close issue #28 or establish protected-main delivery.
