# Product and Technical Gap Baseline

This is a dated delivery baseline, not a substitute for the PRD, TRD, roadmap, architecture decisions, or live GitHub state. It keeps buyer-visible gaps, current issues, active pull-request evidence, and commercial completion tracks in one discoverable place. Protected `main` is the implementation boundary: code in an open pull request is not shipped behavior.

## Current live delivery state

This volatile section is refreshed from live GitHub state and is authoritative only for the exact observations recorded here. The dated snapshot below remains historical evidence and is not promoted to current acceptance evidence. Live GitHub PR/base/head/check APIs are authoritative over PR bodies and prior maintenance prose; a body that still names an older head is stale evidence, not merge evidence.

### Latest verified cut: 2026-09-06

#### Connection-bound text responses: 14:54 UTC

This checkpoint supersedes the source status and executable actions in the earlier
timed checkpoints below, which remain historical evidence. The fresh complete
five-page inventory still contains 125 open PRs (12 Ready, 113 Draft) and 13 open
non-PR issues. All Ready candidates remain BLOCKED; only #147 retains unresolved
thread `PRRT_kwDOTulPlM6coZwc`. Protected main is unchanged at
`87c4daa1830bac5a5228b6036752ad5633232085`; release and tag inventories are empty.

Ready roots: #37, #50, #166, #219, #220, #229, #238, #240, #272, #274, #285, #287.

Published sender #267 `3346d8ecc72932b98ec495d9cc52d6e5727c3064` remains on #266
`eb6c236ff2f4a58b807a2f2c914bd1ddb6079fb3`. Published response owner
#268 `e567af9e678fd4791776df795e89ed666975e6c2` adopts it by ordinary merge
`e1188c86`. Actual socket RED `4632f2df` accepted a reply from a replacement
connection using the same listener/session. Sealed consumer `d6889c80` exposed
missing sender provenance; the parent now records its private connection generation
before I/O and the consumer checks the sealed receipt before consuming pending work.
Foreign success/error replies retain both pending entries; the original reply still
completes its request and leaves unrelated work pending. Seven focused response tests
preserve real remote-error consumption, extensible success, malformed/unknown
retention, family isolation and missing-provenance rejection.

Both exact source heads pass full local Rust1.97.1 gates, 145 Python contracts,
compileall, CodeGraph and diff checks. Their function/line/region/branch coverage is
1317/13835/17598/1456 for #267 and 1324/13890/17670/1456 for #268, each 100%.
Coverage SHA-256 values are respectively
`8d7a3687557d067ee99fd512936077c0c9662d40d9d6544232bb5a377dc88e23` and
`5865d6e69dc5584bad8ef4e866970eb6aa845ad692aa5421a0ffb6f6f01e1540`.
CI `34040202356` and `34040306372` remain queued; formal reviews are empty.
Actual Edge screenshots verified the published source PRs' exact heads, Draft state
and readable evidence without observed clipping or overlap. This is GitHub
presentation evidence, not product-browser acceptance or protected-main delivery.
Writers `5559965531` and `5559948553` are released. This documentation revision
requires its own publication, visual inspection and exact-head checks.

Next source work must address pointer and status receipt provenance in their existing
owners, plus the separate pointer outbound session-binding gap, before promoting
their evidence. Reuse the sealed reader and canonical connection checks; status
projection must still validate before correlation consumption. Browser ownership,
policy approval and observed field/DOM postconditions remain unfinished. Current
foundation #195 `63997bcf555e2c5c8e91ba287734ffba3837a1b7` remains on
`6922dd98779e8f8aad132a3b1f563d7ba6e6d070`; #279 workflow generation and #212
sandbox boundaries remain prerequisites. Counted approval and central workflow
requirements must not be bypassed. Source and documentation writers stay separate.

#### Session repair and child adoption: 13:35 UTC

This update supersedes the 12:28 source/adoption and visual-boundary claims below.
The complete five-page live inventory read at 13:29 UTC still found **125 open pull requests: 12 Ready/non-draft and 113 Draft; 13 open non-PR issues**.
All 12 Ready PRs were BLOCKED, with only #147's unresolved thread
`PRRT_kwDOTulPlM6coZwc`. Subsequent REST reads confirmed 13 non-PR issues, no releases
or tags, and unchanged protected main `87c4daa1830bac5a5228b6036752ad5633232085`.

Parent #264 is now `6f331a5b220349a1aaa1b1841d5e8ec027b9ad49`, Draft on #263
`3f22de94b63da83eaa8b5b1270912b21a3ecd006`. Its original actual-socket regression
was executed and reproduced foreign-session dispatch before the canonical session
mapping guard repaired it. Unknown, retired and mismatched mappings reject without
creating state, registering correlation or emitting command bytes. The fixture now
waits for seed-Pong setup before simulating disconnection; the diagnostic unit-copy
coverage case is retained. Full Rust 1.97.1 gates, 145 Python contracts and exact
1282/13455/17136/1440 production functions/lines/regions/branches each pass at 100%.
Artifact SHA-256 is `b85acd79c3e3a3a49561b9148b6aafadc26cd292a542ec74ef2b523e88c42eaa`.
Current CI `34035628391` and MV3 `34035628337` remain queued, not hosted GREEN.
The source writer released in comment `5559331330`; actual Edge visual inspection
of its published body and Checks page was completed on this head.

Child #265 now publishes `7147893c96ca95c9b5b275d8011c5bfe99aab065` and actually
adopts that parent. Original parent regression replay `98621adf` first reproduced
the same socket failure on the child (0 passed, 1 failed). An ordinary merge then
preserved both histories, admitted-node/current-document authority, typed dispatch,
deadline and ambiguous-write safeguards, all four previously restored postcondition
tests, and both Proposed ADR refinements. All 16 focused regressions, full Rust
1.97.1 gates, 145 Python contracts, compileall, CodeGraph and diff checks pass.
Exact pinned-nightly coverage is 1296/13601/17303/1442, each 100%; artifact SHA-256
is `480d6d3ca492565e8a04bfce4b0c135dffbff276532d129c410674838e8fec2f`.
CI `34036335342` and MV3 `34036335505` remain queued. Fresh actual Edge visual inspection
verified the published body and Checks page at this head: readable wrapping, no
observed clipping or overlap, and exact job IDs matching the API. This is GitHub
presentation evidence, not product-browser acceptance. Writer `5559497861` released
after normal publication and readback. Independent preservation review found no
actionable finding; it is not counted approval.

These are local parent/child acceptance results, not protected-main delivery.
The branch-instrumentation warning, #195/#279 foundation prerequisite, counted-review
rule, browser authentication, action authorization, pointer-reply connection binding,
navigation causality and release gates remain separate. Earlier revision measurements
and screenshots are historical. This new documentation revision still requires its
own post-publication visual inspection; prior #238 `112ba13a` inspection is not transferred.

#### Text-input adoption and review correction: 13:52 UTC

Published #266 `eb6c236ff2f4a58b807a2f2c914bd1ddb6079fb3` adopts #265 `7147893c`
through ordinary merge `5a722867`, preserving original child `cc9980c0` and its unchanged
text-input source/eight tests. Actual inherited socket RED `18a64573` failed before
adoption. All 24 focused regressions, complete Rust 1.97.1 gates, 145 Python contracts,
compileall, CodeGraph and diff checks pass. Exact pinned-nightly production coverage is
1309/13749/17512/1452, each 100%; SHA-256
`596097f5545490f6a4bd29a7b718a61aeb43f0ef738e02a61f372232ecd64543`.
CI `34037184023` and MV3 `34037183844` remain queued. Actual Edge inspection of the
published body and Checks page confirms the exact head, Draft state and queued jobs,
with readable wrapping and no observed clipping or overlap. Writer `5559594727` is
released. This is not product-browser acceptance or protected delivery. #267 already
owns typed text transport and needs separate current-parent adoption and dispatch review;
#266 itself still provides construction, not text dispatch or observed action success.

The later complete queue read confirmed the canonical inventory above and all Ready
roots remained BLOCKED, but found two new #238 review threads in addition to #147: `PRRT_kwDOTulPlM6fsIep` and
`PRRT_kwDOTulPlM6fsIet`. Both findings were verified against `ca8f8475`: the supplement
presented predecessor source claims as current, and the inventory assertion selected
historical counts by formatting. Regression `6fe2af69` observed both failures. The
repair explicitly labels the supplement historical and bounds current inventory checks
to this latest cut, paired with the dedicated current CHANGELOG entry. A mutation test
proves that a changed latest count cannot be hidden by unchanged historical counts.
Thread resolution requires published repair evidence; this documentation revision still
needs its own post-publication visual inspection and exact-head hosted checks.

#### Current executable queue and lineage: 14:18 UTC

Historical Ready roots: #37, #50, #166, #219, #220, #229, #238, #240, #272, #274, #285, #287.

The fresh complete inventory confirms the canonical counts above. These are Ready
review candidates, not merge-ready approvals: all remain BLOCKED. Re-fetch each
head, base, checks, findings and counted approval before acting; do not substitute
the historical root list below. #238's moving head remains live metadata rather than
a self-referential SHA in this document.

Current foundation #195 `63997bcf555e2c5c8e91ba287734ffba3837a1b7` remains Draft on
`6922dd98779e8f8aad132a3b1f563d7ba6e6d070`. The earlier `48eb2d` observations are
historical, not current lineage. #279's workflow-generation prerequisite and #212's
sandbox-helper boundary still require their own evidence and must not be bypassed.

Published transport #267 `4435ce5f561ca069c1844a1a5bd9b603505e25f7` adopts #266
`eb6c236ff2f4a58b807a2f2c914bd1ddb6079fb3` through ordinary merge `d903cf6b`, preserving
both histories. Actual REDs `2b960002`, `e2b49e68` and `10131eb7` preceded repairs for
invalid deadlines, no-write preflight retention and foreign-session dispatch. The
implementation reuses canonical timeout/session checks and sealed typed dispatch;
ambiguous writes remain pending. Ten focused socket tests, full Rust 1.97.1 gates,
145 Python contracts and exact production coverage 1317/13832/17596/1456 each at 100%
pass. Coverage SHA-256 is
`065e535fd49c34699b35d53b4cc6b09f22b3dff506f07eb5549c21b73074bdc8`.
Actual Edge inspection verified its published body and Checks page at that head,
with readable wrapping and no observed clipping or overlap. CI `34038399969` remains
queued; this is not hosted acceptance, counted approval or product-browser evidence.
Source writer `5559716889` is released.

Execute the verified #238 root/lineage review repairs first, then adopt #267 into
existing response owner #268 `8d4027e40b790d28d866051ba741db12927ec22c`. That consumer
still uses removed generic correlation; migrate to the parent's distinct text family
without restoring generic routing, weakening malformed-response retention, or treating
response admission as observed action success. Received-message provenance remains a
separate unproven boundary. Keep source and documentation writers separate. This
documentation revision needs its own publication, visual inspection and hosted checks.

#### Prior observation: 12:28 UTC

All current-state wording in this prior observation is scoped to its recorded time.

Observed through (UTC): `2026-09-06T12:28:16Z`. This cut supersedes volatile claims in the prior observation section below; those earlier exact-head measurements remain historical rather than transferable acceptance evidence. The complete queue read finished at `12:28:16 UTC`, after #265 publication and writer release. The new parent #264 regression is independently owned and is not yet adopted by #265.

The inventory remains **125 open pull requests: 12 Ready/non-draft and 113 Draft; 13 open non-PR issues**. Protected main remains `87c4daa1830bac5a5228b6036752ad5633232085`; release and tag inventories are both empty. Ready-root source heads other than this document's moving branch are unchanged. GraphQL access recovered after the earlier rate-limit failure. GraphQL thread resolution was refreshed for all 125 open PRs alongside exact heads, bases, check rollups and formal reviews in five pages, finishing at `12:28:16 UTC`; all nested histories fit their 100-item pages, with no remaining pagination flags. #147 retains the sole unresolved thread `PRRT_kwDOTulPlM6coZwc`. #166/#220 retain `CHANGES_REQUESTED`, and the other Ready roots retain `REVIEW_REQUIRED`; all twelve are `BLOCKED`. The earlier successful GraphQL cut at `10:00:47 UTC`, subsequent failed inventory/minimal probe and complete 375-read REST fallback at `10:12:37 UTC` remain historical diagnostics, not the current freshness boundary. Active ruleset `18156473` still requires a counted approval and seven central workflows, while the collaborator inventory still contains only the author. #219 has a same-head formal bot approval, but its freshly verified GitHub decision remains `REVIEW_REQUIRED`; an uncounted review does not satisfy the gate. None of this authorizes a bypass or substitutes for an eligible counted approval.

The restored foundation changes the next executable queue. #195 is Draft at `63997bcf555e2c5c8e91ba287734ffba3837a1b7`, on `6922dd98779e8f8aad132a3b1f563d7ba6e6d070`. Its owner restored product/evidence assets lost by historical whole-tree repair `5c111d0db6c363f9d1786c21cc01c5c7398007bd`; this is a content-recovery boundary, not permission to copy an old tree over later work. Fresh hosted CI `34013251657` fails in repository contracts while exact production coverage and MV3 `34013251651` succeed. The exercised failures compare inherited workflow concurrency and `nightly-2026-08-01` against the protected repository-scoped identity and `nightly-2026-08-18`. #279 owns current-workflow reconstruction; weakening the restored tests does not repair the generation mismatch. #242 remains at `2d0e9f69df9ade21d8e8e3d807c3ff644d83b310`, with a stale pre-recovery #195 base `48eb2d23009c1c804520dd5efcd0d4d072aacef1`. Its old CI success is not recovered-foundation acceptance. After the owner prerequisite, adopt the verified foundation content-aware and non-destructively before regenerating descendant checks.

The owner has already moved #255 directly onto #252 so the valid #253/#254 deltas and connection-bound repair travel together. #253/#254 remain open, not silently discarded. Current native Rust and coverage checks succeed on each of these exact heads; earlier local coverage counts do not validate these newer heads:

| PR | Current head | Exact native CI |
| --- | --- | --- |
| #255 | `6865faa8185c5fe6a8b6aaba9535d774575b0061` | `34003175242` success |
| #256 | `881c7f09ee9161ce8664dd75226938ecf60b85e5` | `34003259914` success |
| #257 | `8f1507346f65798a6bf4eaf370d65a2d406a6f44` | `34003269343` success |
| #258 | `b8eaa97f7a417c79250b6b760ff214ac03d39b8a` | `34003285350` success |
| #259 | `91d95423cf31947f691db5ebbd3072c481d86542` | `34003299669` success |
| #260 | `e5228396be8d9faade44a30aed704cacbeb91b46` | `34003322381` success |
| #261 | `572fc4224ddc09c010bd9ccf100764076899495a` | `34003334868` success |
| #277 | `973e34bc24ae9bdd96a50764f2be6c8603eff66c` | `34003345253` success |
| #263 | `3f22de94b63da83eaa8b5b1270912b21a3ecd006` | `34009256997` success |

#263 now targets actual #277. Its current Rust and exact-coverage jobs `101422055630` / `101422055538` succeed after the owner repaired formatting and exercised the unsubscribe frame-error branch. This is native exact-tree evidence, not protected integration, authenticated browser post-condition, or central review acceptance. The deeper #195 foundation prerequisite still applies to this whole stack.

#288 is now Draft at `fd589cd693946ef1ce2c9270c2dfb6a1087bdfb9`. The independently inspected two-file diff from `0f434bc...` contains documentation-consistency RED `e74acd7...` and the doctoring correction `fd589cd...`, with no production or workflow change. Its sandbox-source and historical mock-fixture boundaries are explicit. Current Strix and Noema succeed, CodeQL fails, and native Rust/coverage/MV3 are skipped; predecessor local 173-test/coverage proof is not current browser execution. The earlier planned documentation repair is done and must not be duplicated.

#148 is now Draft at `0135984f1bc1f68d89d7777f49c4999474105a12`. Current native CI `33990522263` succeeds, but pinned-Chromium MV3 `33990522248` fails. The later bounded diagnostic artifact locates browser-crash failure at session creation; #212 records 0/3 for every real-browser lane after all sandbox-disabling overrides were removed. #43's separate helper-generation MV3 `33866932365` succeeds only on its own exact source. #212 owns sandbox-helper adoption against the current protected workflow, followed by fresh consumer execution; cleanup bookkeeping and leaf success are not transferable runtime evidence.

#150 is Draft at `6ab7fc9166f397f49442243492ae88d6ccad56cf`, ordinarily adopting #148 `0135984f1bc1f68d89d7777f49c4999474105a12` while preserving its combined ordinary-pass teardown waiter. Reused parent regressions first exposed three unsafe browser-launch paths and missing crash-stage diagnostics on the child predecessor. The repair preserves all sandboxed parent launch/cleanup behavior and the child waiter delta. All 253 Python contracts pass on Linux without skips; the corresponding macOS run explicitly skips three Linux pidfd cases. Host/guest tracked manifests match `985411ef6a7d3b812be086b43b6341a96f6b8acace63c0161fa8453aae065bd3`. Complete local Rust gates and 415/3555/4444/476 exact 100% coverage pass. Native CI `34018588292` now succeeds, but MV3 `34018588298` / job `101446865332` fails on that same head. Artifact `9985228944`, digest `sha256:05b6d1cd6092a9e2e18a5003a5a1faeb73b8a8d9b4fd7533f7f24bc938833436`, records all four browser lanes at 0/3, with each browser-crash trial failing at `session_create` and all profiles cleaned. This identifies the failed stage, not the underlying Chrome startup cause. The bounded diagnostic and sandboxed runner/helper follow-up is attached to [#212 comment 5557994915](https://github.com/ContextualWisdomLab/OriginWeave/issues/212#issuecomment-5557994915); no sandbox bypass or unchanged rerun was attempted. #147's ordinary-deadline review thread remains unresolved until this child is actually integrated.

#### Subscription provenance predecessors

The following paragraphs preserve exact predecessor observations. Their remaining-boundary statements and queued checks describe those revisions, not the newer original-registry repair below.

#264 remains Draft on actual #263 `3f22de94b63da83eaa8b5b1270912b21a3ecd006`. The preceding published head `43d3b5a3a2b5ce4f51a93d1152a0ee82620f4f3e` retained private original-command identity after test-first `73f11de2232060ac7680e188db88ef7609123296` and the two-registry collision regression. Its five admission tests, 144 Python contracts and 1273/13294/16963/1430 exact 100% local coverage are predecessor evidence; native CI `34020055803` is now cancelled, not current acceptance.

The inbound-provenance repair was first preserved as unpublished checkpoint `bd29f405a6c927b2f7d1c437dccbfb8bcec424f1`, following real crossed-connection RED `918c4ebe`. It reused the existing connection-owned reader and private connection generation to reject foreign success, protocol-error and navigation-event messages before pending-command or document mutation, then proved original-connection recovery. Seven admission tests, 23 focused tests, all 144 Python contracts, full Rust gates and 1273/13311/16973/1432 exact 100% coverage passed on that private tree. Independent read-only review found no actionable defect; it is not counted approval. Those measurements remain attached to that predecessor, not transferred to the later integration below.

A separately active writer advanced the remote through `bc2e69d5`, `a3dfd190`, `c46642e2`, `ed434fc7`, `d8983d50`, `ced211a7`, `7ae9657c`, `7c20907b` and then-current head `8ebcc6131a5dd6bf0b4720c2f1ff8d40d1cc39f8`. The inspected intervening commits contain valid regression and connection-provenance work. [Writer coordination comment 5557964460](https://github.com/ContextualWisdomLab/OriginWeave/pull/264#issuecomment-5557964460) requested task identification, writer acknowledgement and a final exact-head handoff. The private writer claim was released without a competing push; the subsequent integration and release are recorded below. The current caller-supplied registry binding, actual-resend freshness on the same connection, unsubscribe transport/lifetime provenance and real-browser causality remain distinct unproven boundaries. The #195/#279 prerequisite, branch-instrumentation warning and Draft status remain; no workflow, dependency, deadline or gate changed.

A fresh unmodified detached checkout at the earlier remote `8ebcc613` diagnostic cut passed Rust 1.97.1 network/all-targets checking, strict workspace/all-targets/all-features Clippy and all 145 Python contracts. At that cut, the Rust 1.97.1 formatting check fails in five test/support files. The pinned nightly coverage test run completed, but the unchanged numerical verifier failed at lines 13327/13330 and regions 17004/17008; functions 1274/1274 and branches 1434/1434 are exactly covered. Coverage artifact SHA-256 is `f352e3f04bdc3ed1912be7c452c5c426d9caaec7dd1a613b7d3ab9c6486ab69e`. The uncovered regions were the duplicated missing-generation propagation in correlation completion at line 240 and the admission diagnostic at lines 396–397. Independent source review confirmed that the former follows an intent check whose retained constructor always sets the generation; shared validation and a real rejected-event diagnostic/recovery test addressed these causes without exclusions or fabricated production state. The comparison found no reachable authority bypass and identified remote typed-send malformed/unknown/error fixtures, stronger rustdoc and the source contract as deltas to preserve together with private original-connection recovery, unchanged document state, authentic receipt redaction assertions and bounded evidence. Neither private nor remote results establish combined-source acceptance. Earlier native CI `34021674375` was queued at that diagnostic cut; formatting, numerical coverage and hosted execution remain separate gates. The coordination comment recorded the diagnostic and preservation map before shared-source integration or writer acknowledgement occurred.

After the other writer's explicit [release at 52cff969, comment 5558261278](https://github.com/ContextualWisdomLab/OriginWeave/pull/264#issuecomment-5558261278), ordinary content-aware merges preserved both complete contributor histories. Published #264 head `2a9fdc5418b9af10353cdad0f6f6470655bf457d` contains private `bd29f405a6c927b2f7d1c437dccbfb8bcec424f1` and remote `52cff96954c3fec7b8cda5409e4560e970bfcbdf`: both complete lineages are now ancestors. Independent read-only review found no actionable preservation defect. Final-tree Rust 1.97.1 formatting, locked workspace/all-targets/all-features check/test/strict Clippy, strict rustdoc and all 145 Python contracts pass; the workspace run retains all 26 focused socket tests. The unchanged pinned-nightly verifier confirms 1273/13317/16984/1432 functions/lines/regions/branches at exactly 100%, with coverage artifact SHA-256 `f7e71e6fd67723688535053c389b4ff718c4563b403924af462848d3d51b541f`. No test exclusion or quality-gate change was introduced. The normal push and exact remote head were verified, and the integration writer released. Fresh exact-head native CI `34024499232` remains queued; this is local combined-source acceptance, not hosted acceptance, counted approval, protected-main delivery, browser causality or release evidence. The unstable branch-instrumentation warning and all unproven boundaries above remain.

#### Original-registry ownership predecessor

The preceding published #264 head is `10f138f8787d596e8b556fe50c9e4e52bc1295b7`, still Draft on the same #263 base at that cut. Four real-socket REDs at `b3ffeac9` proved original-command send, original-receipt admission, original-event admission and document mutation could use a replacement registry with colliding local identifiers. The document-mutation case did not require matching external context text. The repair retains one opaque core-owned allocation witness through the existing command, binding and subscribed observation, rejecting foreign ownership before correlation/I/O, replay insertion or the common document-mutation sink. Origin binding uses that same sink. Original registry moves and cloned witnesses remain valid; owner destruction does not let a replacement inherit the old identity. The socket receipt regression drops the original registry before creating its replacement, and the existing core diagnostic contract exercises the real dropped-owner mismatch.

The final exact head passes all four new socket regressions, all 145 Python contracts without skips, complete Rust 1.97.1 formatting/check/tests/strict Clippy/rustdoc, compileall, graph and diff checks. Unchanged pinned-nightly coverage is exactly 100% at **1278/13371/17056/1434** functions/lines/regions/branches, artifact SHA-256 `9f5b4942d6a040a83f09593a2b81406c44bc9d2b402fa995a95294ce8e9cbb7e`. Initial `e686b3a0` coverage missed one diagnostic line and three regions in the core unit copy; the genuine error path now exercises that copy. Intermediate `33787617` reached 100% but failed the strict test-lint rule; the final test uses the established error-collection/cardinality assertion style. No exclusion, lint allowance, dependency or production authority relaxation was introduced. Independent read-only review found no actionable authority, lifetime, construction, preservation or documentation finding. Both complete contributor lineages remain ancestors; normal publication, the current PR body and explicit writer release were verified.

Native CI `34026519860`, Rust `101468348712` and coverage `101468348775`, and real MV3 `34026519878` / job `101468347302` now all completed successfully on `10f138f8`. These are terminal-success predecessor evidence, correcting the earlier queued observation without validating a later head. The repair pins the original registry but does not authenticate the registry-to-transport association. Actual-resend freshness was still unfinished on this predecessor; the next section records its separate repair. Unsubscribe lifetime/transport provenance, action causality and end-to-end Chromium product acceptance remain unfinished. ADR 0107 stays Proposed, #195/#279 remain prerequisites, and the unstable branch-instrumentation warning remains. No workflow, gate, merge, tag or release mutation occurred.

#### Command-response freshness predecessor

The preceding published #264 head `805051527cf95e14ba126c9dd3159db86d190224` was Draft on unchanged #263 `3f22de94b63da83eaa8b5b1270912b21a3ecd006`. Actual success-first RED `92fd0b07` and error-first RED `15aea15e` each exposed four repeated-send lifecycles: completed, retired, replacement correlation and unread buffered response. Three additional actual-wire REDs exposed raw/typed mixing and repeated identifiers; a compile-fail contract exposed the pre-upgrade socket alias. The shared established owner enforces strictly increasing typed identifiers and exclusive raw-text/typed mode across all five typed senders. Zero is valid first, out-of-order replies for distinct outstanding commands remain valid, and reader reconstruction/Pong cannot reset history. The consuming socket handoff remains; its nonconsuming alias is removed. This is a stricter local dispatch policy, not a W3C requirement or authenticated browser-session association. The next section records its later teardown refinement; this predecessor's unfinished boundaries and screenshots are not current-head acceptance.

All 22 focused admission/frame tests and the added real public opening-deadline test pass within the complete Rust 1.97.1 workspace verification. Formatting, locked all-target/all-feature check, strict Clippy, warning-denying rustdoc, all workspace tests, all 145 Python contracts, compileall, CodeGraph and diff checks pass. Unchanged pinned-nightly coverage is exactly **1279/13409/17087/1436** functions/lines/regions/branches, artifact SHA-256 `64a62b1e03ee3ed3d62654a3227495e82b7ff93357038ae146d8584c581ac060`. Earlier buffered-fixture BrokenPipe/reset failures and `e6c02cf`'s 17086/17087 region result remain recorded. The latter combined complementary ordinary-library/unit-test gaps rather than unioning source regions; the real public deadline case covers the ordinary copy, retaining the private real revoked-socket test without production or verifier changes. That one-nanosecond case has local passing evidence; portability beyond executed targets remains unproven. Independent read-only review found no actionable finding and does not count as GitHub approval.

Fresh exact-head CI `34029687813`, Rust `101476824185` and coverage `101476824305`, and MV3 `34029687816` / `101476824298` are queued. Post-publication actual Edge visual inspection covered the new PR body and Checks page: exact head/base, Draft state and three queued checks matched the API; desktop screenshots showed readable wrapping with no observed clipping or overlap. GitHub presentation evidence is not OriginWeave product-browser acceptance. Both contributor histories remain ancestors, ordinary publication was verified, and the #264 writer was explicitly released before this documentation-only slice. Registry-to-transport authentication, unsubscribe lifetime/transport provenance, action causality, real browser acceptance, #195/#279 integration, protected delivery and release gates remain open.

#### Subscription teardown ownership and provenance predecessor

At the preceding cut, published #264 was `7fb93e77b800f27187a5c02333298cc31a025bd6`, Draft on unchanged #263
`3f22de94b63da83eaa8b5b1270912b21a3ecd006`. Lifetime RED `2c45cea8` reproduced event admission
after borrowed teardown construction. Transport RED `ce6f6fd4` reproduced 91 actual masked bytes
on a foreign connection and foreign success/error consuming the original pending command. Genuine
original replies then failed as no longer outstanding; real servers were joined before assertions.

The repair consumes the existing non-cloneable subscription receipt and retains its original
connection identity through teardown dispatch and acknowledgment. It reuses connection-aware
correlation and the existing connection-bound reader; no revocation registry, dependency, workflow
or quality-gate change is added. Receipt reuse is now rejected by the compiler with `E0382`, not
merely by a fixture setup error. Construction failure also ends local admission availability.
Previously admitted observations are not retroactively revoked, and acknowledgment does not prove
event drainage or browser cleanup. Proposed ADR 0107 records these costs and rejected alternatives.

All three transport regressions, eight preserved failure cases, fourteen admission tests and the
escaped-identifier round trip pass. Full Rust 1.97.1 workspace/all-feature formatting, locked check,
strict Clippy, warning-denying rustdoc and tests pass, including three network doctests. All 145
Python contracts, compileall, CodeGraph and diff checks pass. Unchanged pinned coverage is exactly
**1279/13421/17106/1438** functions/lines/regions/branches; artifact SHA-256
`277c5bc0a17f16966a8eb388d54e4e172be36356d7dead5bbead9ed53ad8cff6`. The unstable branch warning
remains. Independent read-only source review found no actionable issue, not counted GitHub approval.

New exact-head CI `34031281825`, Rust `101481131938` and coverage `101481131907`, and MV3
`34031281812` / `101481131930` are queued, not GREEN. Both contributor histories are preserved,
normal publication was verified, and source writer `5557964460` was explicitly released before
this documentation writer began. The required new-head visual inspection remains incomplete:
actual browser control reports that the Mac is locked and automatic unlock is unavailable. Manual
unlock was requested; neither the previous #264 screenshots nor the previous #238 Preview validates
these new heads. Registry-to-browser authentication, action causality, real Chromium acceptance,
#195/#279 integration, protected delivery and release gates remain open.

At this read, #238 predecessor `81e334aff063290bde3298671cfd20e77c7c1fff` still has queued native
CI `34030165220` plus central review/security work. This documentation commit needs its own checks;
neither its earlier 172 local Python passes nor #264's source coverage transfers to the new head.

#### Current pointer integration and independently owned parent repair

Published #265 is `35555d0f8491d4ee95c2e61d1a2aaa2c02a0635c`, still Draft. Ordinary two-parent
integration `7535d8af` preserves original child `ffa70ee0f499b86ff51837fb95733fd5cf57ff89` and
parent `7fb93e77b800f27187a5c02333298cc31a025bd6`. Real-socket RED `3f9cdc1a` observed no command
bytes but one outstanding command after a zero deadline. The repair retains the child's immediate
admitted-node revalidation and the parent's typed dispatch, local no-write retirement and ambiguous
write retention. Both use the existing registry identity allocation. Independent review identified
four silently deleted parent postcondition tests and two obsolete sender calls; all were restored
or migrated without dropping assertions. The final review found no remaining actionable finding,
which is not a counted GitHub approval.

All 19 focused tests, complete Rust 1.97.1 formatting/check/tests/strict Clippy/rustdoc, all 145
Python contracts, compileall, CodeGraph and diff checks pass. The exact-head pinned-nightly rerun
passes unchanged production coverage at **1293/13567/17273/1440** functions/lines/regions/branches,
each 100%; artifact SHA-256 is
`b1374f65538fb6557b4e5d877aa067e183202ed1f56c75be047b3ff2dfdc92a0`. This is local source evidence.
CI `34032661775`, Rust `101484986934` and coverage `101484987071`, and MV3 `34032661781` /
`101484986948` remain queued. Normal publication was verified; writer `5559058158` released.

The live parent #264 subsequently advanced to test-only `b4702cd503fa3f721e0d1f44b355563753dac0a2`.
Its 129-line socket regression targets registry-session A dispatch over transport-session B;
writer [5559076006](https://github.com/ContextualWisdomLab/OriginWeave/pull/264#issuecomment-5559076006)
released in [5559140277](https://github.com/ContextualWisdomLab/OriginWeave/pull/264#issuecomment-5559140277)
without executing the regression or implementing a production repair. Parent CI `34031977586`
and MV3 `34031977597` remain queued. The available local backend can now execute that regression
under a new source lease. Adoption waits for verified repair: #265's local GREEN is **not latest-parent acceptance**.
The PR base snapshot still names `7fb93e77`, so it must not replace the fetched live parent as
dependency evidence. No parent source was changed during this documentation slice.

Browser access recovered after the earlier locked-screen failures. Fresh actual Edge visual
inspection of #265's published body and Checks page verified head `35555d0f`, Draft state and the
three queued checks. The inspected desktop screenshots show readable wrapping with no observed
clipping or overlap. This replaces the earlier lock blocker only for that inspected presentation;
the new documentation head still requires its own visual inspection. GitHub presentation is not
OriginWeave product-browser acceptance, browser-process authentication or navigation causality.

The #238 predecessor `88447b7d43e4a87b48fb8fde33f93b2e3ce8cbc5` still has pending native CI
`34031539064` and central review/security work. Its 172 local Python passes and #265's source
coverage do not transfer to this documentation commit. The next executable items are this dated
evidence update, then an executed parent regression and verified repair under a new writer lease; #195/#279,
#212 and protected-main/release gates remain prerequisite work.

#### Scheduling and central-owner follow-up

This section retains owner-reported scheduling and central-repository evidence. Those reports are
not refreshed owner-source or released-consumer acceptance in this bounded documentation slice.

The existing hourly-task owner attempted a prompt-only coordination/evidence improvement, but the scheduler rejected the update with `too_many_active_automations` at the ten-active-task limit. The owner reports that a subsequent read confirmed the old prompt and update timestamp, cadence, enabled state and notification preferences were unchanged. This is a failed update, not an applied scheduling improvement; no duplicate schedule or unrelated-task pause was created.

On #238's earlier `a2600ee8...` head, native CI `34018002926` succeeded. The later `38e5eb44...` cut recorded queued CI `34021771606`; subsequent published `7ca86a2f546adf3a0af3fec41d97675115db384d` still had queued native CI `34022635038` before this refresh, together with central review/security work. At the now-predecessor `8e6284f5f89965983d025ce302d7c99e2f5c38d5`, native CI `34027019757`, Rust `101469694749` and coverage `101469694911` have completed successfully; nine central security/review jobs remain queued. This document's next commit requires new exact-head checks; it cannot inherit those results. Existing central CodeQL dispatch/verdict work remains with `.github#712`. Central Strix #1563 remains unmerged at `eaf9594f7fe8d8e1994349289183d6cbad056579`, reported behind its base: the earlier `13fbb48e...` source repair and owner-reported harness results do not prove consumer recovery. The unrelated #37 local 900-second deadline remains distinct from the #166 completed-report classifier defect and #219/#240 actual gateway/hosted-limit incidents. The active central Noema owner additionally reports source REDs under `.github#1641`; owner reports and comments are not released transport or SDK-compatibility evidence, and no consumer workaround is introduced here.

### Prior observation cut: 2026-09-05

The following cut preserves prior source measurements and diagnostics. Present-tense wording within this dated cut describes its observation time only; use the latest verified cut above and live APIs for the current queue.

Observed at (UTC): `2026-09-05T13:50:14Z` (full REST inventory/review/check refresh and targeted source verification; subsequent bounded diagnostics are recorded below).

GraphQL rejected both the inventory query and a minimal probe with its rate-limit error while the REST endpoints remained available. REST reviews, check runs, and combined statuses were read for all 125 open PRs; truncated output was recovered by querying the exact missing records. New source heads were refreshed separately after pushes. Review-thread resolution cannot be inferred from these REST endpoints: earlier thread-resolution statements below retain their prior observation date and are not refreshed merge evidence. No query failure, queued check, or metadata update relaxes a quality or review gate.

- #37 Strix run `33946242342` / job `101285198427` failed at 13:26 UTC. Artifact `9970303662` (SHA256 `93ce81a9c3060f52a23d0910b5a995a2b491aeb3889a54432d954747a97010ca`) records `Strix run timed out after 900s`, 903 seconds elapsed and exit 124; its current-attempt record is interrupted, not completed. The provider-unavailable wrapper label does not establish a provider outage. Canonical Strix deadline/classification ownership received the exact artifact; no consumer timeout change or blind rerun was made.
- #148 hosted compatibility run `33962062608` / job `101295542349` failed at 13:33 UTC on `bded4fc9d32e5e047cea99d182aa05ae2cd03bf6`: browser-crash recovery fails 0/3 trials, while ordinary Agent Task and forced-close each pass 3/3 and profiles are cleaned. Artifact `9970408851` (SHA256 `daefc93ff2792cbc82a8937107e7992e91447f56692c4ea7071bc2b39c0faa7b`) initially exposed only `RuntimeError`; the stage was not reconstructible from it. Subsequent owner diagnostics and newer all-sandboxed results are recorded in the latest cut, not backdated into this earlier artifact.

- Protected `main` is `87c4daa1830bac5a5228b6036752ad5633232085` through #286. PR #286 skips repository-native CI jobs for draft pull requests; a skipped job is not passing evidence.
- The September 5 search also returned 125 PRs (12 Ready, 113 Draft) and 13 non-PR issues. PR #290 was converted back to Draft after current-protected workflow review found that its #245-relative patch would drop protected #286 lifecycle controls; queue movement is not protected-main delivery. PR #248 also returned to Draft for current-parent adoption and complete revalidation. Temporary synthesis #291 has since been externally merged and closed as recorded below; its closure does not establish product acceptance.
- Ready roots are #37 `1e2f41072854edcdbaf0f9ecf14697a3bfd62195`, #50 `ad87cfea59db711cb29ef90559790ba77e22029f`, #166 `e84a1a2cc82b1c666218efd441da97849f47b8c2`, #219 `65e4315d80137badc0b55e1b9617015beb1db568`, #220 `e545b94e1de499b96b867694f80ac04ad247becd`, #229 `024f63690cf05cfe6f0d4a430f0e18ea8fd2c4d6`, #240 `24930a3a9ee79c0b712ee3df6589b0592eb6e18f`, #272 `b1cae8ad1cbd8eb6992037c830aea30b9aa436b3`, #274 `802d0bdff7536d9ac253305d3e0237b4e4a1789e`, #285 `f455c2cd64b3dd3f027c91d396103792a205ddd0`, and #287 `af83c40dd2990a03064a92ca75430a9cc400f098`. PR #290 is Draft at exact head `ebeefcd534db4324498fdb18046ebc6255ddcdf2`, remains a #245-dependent workflow-owner candidate, and is not a root. PR #238's moving exact head is intentionally omitted from its self-referential document; live PR metadata is authoritative. Their hosted exact-head checks remain non-terminal and none has an eligible exact-head approval, so none is merge-ready.
- PR #166 exact head `e84a1a2cc82b1c666218efd441da97849f47b8c2` and PR #220 exact head `e545b94e1de499b96b867694f80ac04ad247becd` retain formal `CHANGES_REQUESTED` decisions. REST review records were refreshed; thread-resolution state could not be refreshed because GraphQL rejected the current read. The last successful thread-resolution snapshot was the prior 12:36 UTC sweep, not this REST refresh. Thread resolution alone is not approval or grounds to dismiss a review, and the active counted-approval gate remains unmet.
- Targeted CI RCA at `2026-09-05T06:45Z` confirmed that #219 Noema run `33925442322` / job `101226941175` and #240 run `33925596923` / job `101227537529` had terminated on gateway HTTP 502 without review verdicts. Neither unchanged exact head had a successor Noema run. One supported failed-job rerun per head created attempt 2, with queued jobs `101264704581` and `101264705575`; Noema attempt 2 is queue-admission evidence, not provider recovery or a review verdict. Successful sampled Noema wrapper runs skipped their review jobs and cannot establish recovery. The CodeQL failures independently record dispatch handoff: #219 central runs `33947047322`, `33947036734`, `33947037072` and #240 runs `33947189162`, `33947189634`, `33947187643` match those exact heads; these central CodeQL dispatches remain queued and were not duplicated. Active Strix runs were left untouched. Revisit these exact attempts after a terminal result; do not create repeated retries while they remain queued.
- A fresh repository Actions query at `2026-09-05T06:12:27Z` returned **115 queued workflow runs**; the newest sampled run was #238 CI `33949183271` on predecessor head `bb2f18570f319a763acc24d7c4206ec9d0a11a29`. This is runner-admission/backlog evidence, not a code failure or passing check; rerunning unchanged heads would only add duplicate queue load. The moving PR's exact current head remains live-metadata-only to avoid a non-convergent self-reference.
- Active organization ruleset `18156473` (`CWL Central required workflows`) applies to the default branch. It requires one approving review, dismissal of stale approvals after pushes, resolved review threads, extra approval for unattributed changes, and **7 central required workflows**: `opencode-review`, `pr-review-merge-scheduler`, `security-scan`, `strix`, `sast-semgrep`, `noema-review`, and `codeql-pr`. Current `ContextualWisdomLab/.github` consolidates OSV and Scorecard PR scanning into the required `security-scan.yml`; standalone `osv-scanner-pr.yml` and `scorecard-pr.yml` are no longer ruleset entries. Administrative bypass capability is not authorization to use it.
- PR #284 head `61bcf88c960c6c437ccd29b3fbb73cd4325f9e5a` reached `main` through rule-suite `3948421709`, whose live result is **`result: bypass`** with actor `seonghobae`. The required non-author approval and workflows were not proven before integration. The earlier pending post-merge state is historical: fresh native Rust and production-coverage checks now succeed on `87c4daa1830bac5a5228b6036752ad5633232085`. This is a governance incident tracked by #215, not a policy-compliant merge or evidence that later checks can retroactively authorize it.
- PR #285 is Ready at exact head `f455c2cd64b3dd3f027c91d396103792a205ddd0` on current protected main. It addresses #284's three post-merge review findings without changing `.github/**` and adds a dedicated regression contract. All 154 repository contracts, the full Rust gates, and exact 100% local coverage pass. Its Ready transition materialized repository-native CI `33930234387`: Rust contracts `101207153048` and Production coverage `101207153244` succeeded by `2026-09-05T03:16:29Z`; native CI success does not replace the seven required central workflows. The Ready event did not materialize fresh central required-workflow runs. Targeted RCA at `2026-09-05T06:53Z` verified that the previously cancelled Security Scan `33924016851`, SAST Semgrep `33924016903`, and CodeQL PR `33924016883` still bind the current head and protected base, with no duplicate central CodeQL dispatch. The convenience CLI failed on a cross-repository workflow lookup before rerunning anything; the supported run-ID REST route then created attempt 2 for all three, with queued scope/language jobs `101265706025`, `101265708882`, and `101265710202`. These are verified replay admissions, not terminal scan results or proof that Ready-event materialization is repaired. Remaining required review evidence and the absent eligible approval still block merge; toggling Draft or creating a no-op commit is not an acceptable substitute.
- Issue #279 owns the remaining documentation-CI partitioning gap. PR #287 is Ready at exact head `af83c40dd2990a03064a92ca75430a9cc400f098` on current protected main; it is the workflow-independent classifier foundation, changes no `.github/**` path, and is the prerequisite for an authorized workflow owner. Its predecessor failed its own release-record contract; the current head binds Git rename/copy similarity to blob identity, and its focused and full local suites plus exact 100% Rust coverage pass. Exact native CI, Semgrep and scoped Security Scan have now completed as described next; remaining required review/CodeQL evidence and eligible approval are not complete. PR #282 is Draft at exact head `b64e0708584beff3fb54acf226cb3e667773e473` on current protected main. Its latest workflow/test delta removes `converted_to_draft` and `closed` from the CI event list, removes #286's closed-event guard, and makes the repository contract assert those protections are absent. Exact review `5120043505` therefore requires the authorized #279 owner to reconstruct the valid trusted-base classifier/contract partition while preserving #286's lifecycle and closed+Draft fail-closed controls; Draft checks do not transfer as GREEN.
- #287 CodeQL handoff RCA at `2026-09-05T08:34Z` confirms that exact native CI `33931806137` passed, Semgrep `33931806165` executed its scan successfully, and Security Scan `33931806226` executed Scorecard/Trivy while scope-skipping gitleaks/OSV/dependency-review. CodeQL wrapper `33931806139` failed intentionally after its three language jobs recorded successful dispatch and a pending verdict, not a failed terminal scan. The cross-repository convenience CLI returned 404; direct REST job logs identify the handoff. Central `.github` dispatches `33954721186`, `33955024697` and `33955164029` have display titles identifying the exact OriginWeave target head, but their immutable run heads are central-owner revisions (`71dd84d...` / `27d7331...`), not the product source head. All three remain queued and were not duplicated; acceptance still requires validated target checkout, an authenticated terminal verdict and the original-job replay. OpenCode is queued, Noema/Strix remain in progress, and no eligible formal approval exists.
- PR #290 is Draft at exact head `ebeefcd534db4324498fdb18046ebc6255ddcdf2` on #245 exact `a769f484e2c110e0523b3b28cd21573f43867562`. Its workflow/repository/PR-number concurrency isolation and manual-dispatch non-cancellation intent are valid, but its predecessor-relative MV3 patch would remove protected #286 `converted_to_draft`/`closed` lifecycle events and the closed-event job guard. Exact review `5120039692` and canonical-owner issue #212 comment `5549868655` require reconstruction on the current protected MV3 workflow; the scheduled product writer did not mutate `.github/**`.
- PR #283 is Draft at exact head `c904300a6a1bda83af24f84d586f1c5f6a6491aa`, stacked on predecessor #282 head `b54a5856d8201911f05d69622f0d5594a371adf0` rather than current #282 exact `b64e0708584beff3fb54acf226cb3e667773e473`. It must adopt the corrected current parent before its compare can become current evidence. The child remains one prose-only doctoring canary; Draft-hosted jobs are skipped, so no trigger-shape GREEN is claimed for #279. Runner admission is a separate condition.
- PR #288 is Draft at exact head `0f434bc29d468127412366b6864b733b40b83c4d` on current protected main. Predecessor `39e36256651f62940ec3ca6149067f0cfcb2285a` had 172 passing Python contracts; that evidence is historical. The subsequent test-first sandbox contract and doctoring change led to the ordinary launch override removal, followed by restoration of the original bounded ChromeDriver diagnostic. Direct cumulative comparison from `99fea8985b4649c444ecc72c11e72c0693b17f5b` to current `0f434bc...` confirms one net production line removed: `--no-sandbox`. Fresh isolated verification confirms all 173 Python contracts pass on that unchanged head, including the source contract for both ordinary and Agent Task launch paths. Complete Rust gates and numeric 100% coverage also pass: 521 functions, 4420 lines, 5349 regions, 654 branches; the unstable branch-option warning remains. The doctoring problem paragraph still describes the historical override as current, so its existing author owns a bounded documentation/test correction after this writer releases. Unit-fixture output is not executed browser evidence. #212 still owns authorized workflow activation and fresh sandboxed pinned-Chromium compatibility/Agent Task trials; no workflow, sandbox-helper configuration, or hosted replay was changed here.
- Issue #28 remains the P0 governed-browser integration target. PR #261 is Draft at exact head `934eb7d37568b439c442ffe1d1f6a9c8f8ed58a0`; #261 now includes actual #260 `2c5049aff97a90958e8262b1d403bdcbd64a1e8b` by normal merge. Its origin-binding implementation and tests remain byte-identical to predecessor `323ac9e147691e9f6572711f5a748e13f1036624`. Native release-contract discovery reproduced zero tests before parent adoption and one passing test after it. Fresh verification passed 18 focused loopback tests, 142 Python contracts, full Rust gates, and numeric 100% coverage: 1173 functions, 12104 lines, 15511 regions, 1334 branches. Exact CI `33967462582` is queued; no previous check is transferred.
- Repair PR #277 is Draft at exact head `117f6414e8a6db46eb2b32f4ebae85cf2a208371`, based on actual #261 `934eb7d37568b439c442ffe1d1f6a9c8f8ed58a0`. Parent adoption `3c0484174eeda0703492ba76b530be125e3e99dd` preserved its two subscription production files, three test files, and Proposed ADR, with 16 focused tests, 142 Python contracts, full Rust gates, and numeric 100% coverage at 1220 functions, 12774 lines, 16397 regions, 1418 branches. The subsequent deadline repair reused the existing frame validator before registration: its RED observed no subscription bytes but two outstanding commands where only one pre-existing command should remain. Zero and over-limit deadlines now preserve that command and leave the rejected identifier reusable. Post-registration frame-failure retention remains conservative, including the separately tested repeated-mask rejection. Fresh current-tree verification passed 11 focused tests, 142 Python contracts, complete Rust gates, and numeric 100% coverage: 1221 functions, 12781 lines, 16404 regions, 1418 branches. The actual typed family is `NavigationCommittedSubscription`, not the historical proposed `SessionSubscribe` name. The response parser and authority boundaries are unchanged; exact CI `33968278171` is queued, and `warning: --branch option is unstable` remains. Stale conflicting #262 is not superseded, and #263+ are not retargeted or promoted until current-parent hosted acceptance and complete successor evidence exist.
- Stacked PRs #178 and #85 were repaired without force pushes at exact heads `640e594dbc0a64f251a0f28b8e80943f53337e40` and `c9adea680d6186f1842a280e248ce55da4f1305b`. Parent-relative deltas now preserve the current MV3 diagnostic and extension-authority contracts; local full suites and exact 100% coverage passed, while hosted exact-head acceptance remains independently required.
- PR #70 is Draft at exact head `77eb0f2ee71783e06171784b7173c0b4cd530e61`; it targets protected main but has not adopted current main `87c4daa1830bac5a5228b6036752ad5633232085`. Its focused Agent Task contract proves the pre-repair `--no-sandbox` launch as source RED and the branch removes that argument only from the Agent Task lane. Sandbox-enabled pinned-Chromium E2E and repository-wide GREEN remain unproven while its exact-head browser/CI/security runs are non-passing.
- DDD/MCP repair #272 is Ready at exact head `b1cae8ad1cbd8eb6992037c830aea30b9aa436b3` on current protected main. Its exact-head hosted acceptance remains incomplete and no eligible approval exists. Documentation child PR #273 is Draft at exact head `e5c8fcb66bf644dfa750bb1b40ba3d600cb7805a` on predecessor #272 head `fe124e447cad3f679e22337fb6fbdfd135ab3652`, so it must adopt the parent only after #272's current head completes its gates. Both remain active-PR evidence.
- PR #229 is Ready at exact head `024f63690cf05cfe6f0d4a430f0e18ea8fd2c4d6` on current protected main. Its 157 Python contracts, full Rust gates, and exact 100% local coverage pass, while hosted exact-head acceptance remains incomplete and no eligible approval exists. The presentation-identity work retains the boundary that Chromium application and page-observed effectiveness remain separate adapter/browser-E2E work.
- PR #281 remains Draft at exact head `adaca6427d68f550b39293a69b7c733430d1c385` on canonical MV3 parent #43 `6f3134d18d3118aab33d28048671dc71a5f47b77`. Its child delta is diagnostic-evidence contract/doctoring only; inherited #43 implementation redacts browser/page-derived HTTP and WebDriver protocol data, startup exception details, returned capability values, DOM-dataset values, and click post-condition text before CI/audit evidence. Exact-head CI and Manifest V3 Compatibility succeeded and review threads are resolved, but parent-first workflow-ownership verification remains open, so the child stays Draft.
- PR #43 is Draft at exact head `6f3134d18d3118aab33d28048671dc71a5f47b77`; it targets protected main but has not adopted current main `87c4daa1830bac5a5228b6036752ad5633232085`. The branch now installs and configures the pinned `chrome_sandbox`; exact-head hosted verification remains required, and restoring `--no-sandbox` is not acceptable.
- PR #274 remains the bounded GitHub Pages source/README/CHANGELOG public-surface delta at exact head `802d0bdff7536d9ac253305d3e0237b4e4a1789e`, with a repository regression contract for the exact badge target, pre-alpha status, active-PR non-promotion statement, and publication boundary. Source presence is not publication evidence; live HTTPS publication and navigation remain required after the authorized Pages configuration/deployment path.
- PR #37 is Ready at exact head `1e2f41072854edcdbaf0f9ecf14697a3bfd62195`. The head repairs the Rust 1.97.1 formatting failure in its segmented Content-Length regression; formatting, locked workspace check/test, strict Clippy, rustdoc, all 167 Python contracts, and pinned-nightly production function/line/region/branch coverage pass locally at 100%. The reader returns after the exact declared bytes, already-buffered surplus remains fail-closed, and valid self-delimited content does not require TLS EOF. Native Rust contracts and coverage have succeeded on this head, but central workflow evidence remains incomplete and no eligible approval exists.
- PR #50 is Ready at exact head `ad87cfea59db711cb29ef90559790ba77e22029f` on protected main. At predecessor `e981ac45d0bfcd3906fc64dae5f4490edf39f9e5`, seven focused tests passed but pinned Rust formatting failed on the default-port regression. The formatter-only repair preserves all default HTTP/HTTPS and explicit-port assertions and every production source. Fresh independent verification passed all seven fresh-resolution tests and 152 Python contracts, compileall and complete Rust 1.97.1 format/check/workspace-test/strict-Clippy/rustdoc gates. Numerical coverage is 530 functions, 4509 lines, 5441 regions and 660 branches at 100%, with the unstable branch-option warning retained. Current CI `33964793228` and compatibility `33964793282` remain non-passing queue evidence. Earlier `2bd85188...` documentation and `ddbefc91...` origin-port repairs remain historical: an origin-approved IP could be paired with a different service port; the planner now binds the socket port to the effective scheme-host-port origin before I/O. ADR 0005 and doctoring retain trusted monotonic time and revalidation before socket I/O. Local admission tests do not prove an HTTP/TLS exchange, browser navigation, hosted acceptance or required approval.
- PR #269 is Draft at exact head `7854394266d3f292e779193c01413a34f6798d7c`, stacked on #268 exact `8d4027e40b790d28d866051ba741db12927ec22c`. It adds only a fixed, product-owned `script.callFunction` text-value observation for the exact admitted current node; it performs no browser I/O and proves no post-condition. Its new documentation boundary contract and all 140 Python contracts pass locally, while hosted exact-head CI is queued and the parent-first dependency keeps it Draft.
- PR #270 is Draft at exact head `191a14535219ea8033777fa4c970efb281b62418`, now non-force synchronized onto #269 exact `7854394266d3f292e779193c01413a34f6798d7c`. Its parent-relative transport delta is unchanged; 140 Python contracts, full Rust gates, and exact 100% local production function/line/region/branch coverage pass. Dispatch still proves neither the correlated result nor text-entry success, and fresh hosted exact-head checks plus parent-first integration remain required.
- PR #271 is Draft at exact head `802ec806cdd4560eab48c484f435766ecabda353`, non-force synchronized onto #270 exact `191a14535219ea8033777fa4c970efb281b62418`. It admits only the exact typed observation response, treats unequal text as `PostconditionMismatch`, and retains no page-controlled or expected text in public evidence or diagnostics. Its test-first documentation contract, all 141 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered ancestor integration remain required.
- PR #195 is Draft at exact head `48eb2d23009c1c804520dd5efcd0d4d072aacef1`. Its loopback regressions hold the accepted peer through opening-write timeout cleanup and locally revoked-stream classification, removing macOS close races without weakening production failures. The original affected test passed 50 consecutive focused regression passes, followed by 139 Python contracts, full Rust gates, and exact 100% local production coverage; fresh hosted exact-head checks and prerequisite integration remain required.
- Opening-exchange fixture repair at #242 is Draft at exact head `2d0e9f69df9ade21d8e8e3d807c3ff644d83b310`, stacked on #195 exact `48eb2d23009c1c804520dd5efcd0d4d072aacef1`. A complete descendant Rust run reproduced a premature peer-close race in the mismatched-accept fixture after the existing invalid-deadline repair had been adopted. One shared test-only server now reads the complete request before replying and retains the peer through assertions in all three exchange fixtures. Fifty suite runs passed all 250 cases, followed by 139 Python contracts, complete Rust gates and exact 100% local coverage (809 functions, 7802 lines, 9959 regions, 942 branches). Production cleanup errors remain fail-closed. Native CI `33952463254` is queued; repeated local passes do not establish a zero flake rate or hosted acceptance.
- PR #243 adopts that corrected parent by ordinary merge at `97fab641ed9d76e6c515eadcef0629edfc8064a3`, with exact base `2d0e9f69df9ade21d8e8e3d807c3ff644d83b310`. The parent-relative delta preserves the existing bounded frame transport and adds only its integration evidence; the shared fixture is identical to the owner version. All 139 Python contracts, full Rust gates and exact 100% local coverage pass (884 functions, 8733 lines, 11176 regions, 1008 branches). Native CI `33952588687` is queued, and the PR remains Draft. This repairs the previously stale #242-to-#243 dependency edge; later descendants still need ordered parent adoption and their own exact-head verification.
- PR #246 adopts #243 at `585791f3641fbe757c3bd9fd36d5316adcc78d63`, with exact parent `97fab641ed9d76e6c515eadcef0629edfc8064a3`. Its ordinary merge preserves the message/JSON source and test blobs while carrying the owner fixture repairs. All 141 Python contracts, complete Rust gates and exact 100% pinned-nightly local coverage pass (962 functions, 9701 lines, 12427 regions, 1084 branches). Native CI `33953247053` is queued and this PR remains Draft. Fifty pre-integration fixture-suite runs passed, so no newly reproduced failure or measured failure-rate improvement is claimed. Raw protocol messages still carry no received-connection provenance or browser authority; child integration and hosted acceptance remain pending.
- PR #247 exact head `6407895f4db4bee640074cb9c9d3cbe8b0e9e13a` merged as `b87191bcb6a95dfd7e0ed234e600639a1093c43a` into unprotected parent `feat/webdriver-bidi-text-message-assembly`; this is stack integration, not protected-main delivery. PR #248 is Draft at `b386f17c4826adabebda084bff2fba35aee94dd0`, now based on #246 `585791f3641fbe757c3bd9fd36d5316adcc78d63` by ordinary merge and explicit retargeting away from the merged #247 branch. Correlation production and Rust test blobs remain unchanged. Review reproduced that native unittest discovery collected zero checks from the free-function release contract; the same assertions now use the repository's TestCase format, collect one check and reject a missing release record. All 142 Python contracts, full Rust gates and exact 100% pinned-nightly local coverage pass (975 functions, 9848 lines, 12563 regions, 1092 branches). Native CI `33953719566` is queued. Previous Ready status and local totals did not prove enforcement of the uncollected check; current hosted verification, protected parents and received-connection authority remain unproven.
- PR #249 is Draft at exact head `84b9407978ae0f6c115f01170b6069c601b21104`, ordinarily merged onto current #248 `b386f17c4826adabebda084bff2fba35aee94dd0`. The old `017d6e816f5a86544a63821b3ceaba94d5f17f44` tree lacked that parent and its newly discoverable release check; the parent adoption also reproduced a CHANGELOG-only merge conflict. The integration retains both release records, the parent's canonical opening fixture and native TestCase, and all four child-owned production/Rust-test blobs unchanged. All 142 Python contracts, the full Rust 1.97.1 gates and pinned-nightly 100% coverage pass locally: 989 functions, 9978 lines, 12717 regions, 1098 branches. Fresh exact-head CI `33954334610` is pending, not acceptance. The current #250 adoption is recorded next.
- PR #250 now adopts #249 at `ec433b844a121f8554c062f92267991af9cacb6f`, with exact parent `84b9407978ae0f6c115f01170b6069c601b21104`. The conflict-free ordinary merge preserves all five child-owned production/Rust-test blobs and propagates the canonical fixture and native release TestCase. The pre-integration zero-test discovery RED becomes one executed check; all 142 Python contracts, full Rust 1.97.1 gates and pinned-nightly 100% coverage pass: 1023 functions, 10518 lines, 13525 regions, 1186 branches. Fresh exact-head CI `33955410724` is pending. Bounded status parsing still grants no authority and does not claim the later received-connection capability; current #251 adoption is recorded next.
- PR #251 now adopts #250 at `f02af6d0dd01708d495cc08dec785675f3d58898`, ordinarily retaining both exact parents `86e8ad76838f2a64aa7e0cd56ba1f931c8d0c3dc` and `ec433b844a121f8554c062f92267991af9cacb6f`. Only the changelog required content reconciliation; both release records remain. The sender, public exports and both child-owned Rust integration tests are byte-identical to the predecessor. Native release-contract discovery first failed at zero and now executes one test; all 142 Python contracts, compileall, complete Rust 1.97.1 gates and enforced 100% coverage pass: 1037 functions, 10647 lines, 13681 regions, 1192 branches. The pinned branch-instrumentation warning below was independently reproduced on this tree too. Exact CI `33959168801` is queued; Draft, current-parent protection and hosted acceptance remain separate. A successful session-end frame write is not remote completion or process/profile teardown proof. Current #252 adoption is recorded next.
- PR #252 now adopts #251 at `6569bf40b6595ac74c2f0a997d202137f07ba1db`, retaining predecessor `2015259529ada99af836989079cc85a15779a2d8` and current parent `f02af6d0...` by ordinary merge. The response implementation, public exports and four loopback response tests are unchanged; both conflicting release records remain. The inherited release-contract loader moves from zero RED to one executed GREEN, and all 142 Python contracts, compileall and complete Rust 1.97.1 gates pass. Enforced numerical coverage is 100%: 1044 functions, 10699 lines, 13751 regions, 1192 branches; the branch-instrumentation warning remains. Exact CI `33959789688` is queued. This response layer does not authenticate the received connection or prove resource teardown. #253's current-parent adoption is recorded next; the later #255 provenance repair remains separately owned.
- PR #253 now adopts #252 at `afb623e4449b7cbf926fdcef7225ceaca822cfcf`, retaining predecessor `0d72082e595c0e1fcc03d609ba337896ed14e2fc` and current parent `6569bf40...` by ordinary merge. The assessment implementation, public exports and two child tests are unchanged; both conflicting release records remain. Native inherited release-contract discovery moves from zero RED to one executed GREEN. All 142 Python contracts, compileall and complete Rust 1.97.1 gates pass, with enforced numerical 100% coverage: 1054 functions, 10747 lines, 13787 regions, 1194 branches. The branch-instrumentation warning remains, and exact CI `33960670119` is queued. Review `5120013340` remains unresolved: caller-supplied teardown claims still cannot authenticate operational completion, regardless of local coverage. The #254 adoption below preserves #255's separately owned removal/provenance repair as a remaining prerequisite. This intermediate Draft is not security or operational acceptance.
- PR #254 now adopts #253 at `b11b6c9bccd8335b58a4fb599f8ad29ac419637f`, retaining predecessor `cbaf50dcc97753cc73135497ea8225e8b18de190` by ordinary merge. The closure implementation, public exports and all six real-loopback child tests are byte-identical to that predecessor; both release records survive the changelog conflict. The inherited correlation release test moves from zero collected RED to one executed GREEN. All 142 Python contracts, compileall and complete Rust 1.97.1 gates pass, with numerical 100% coverage: 1061 functions, 10802 lines, 13843 regions, 1200 branches. Review `5120077272` remains actionable: closure kind/status still cannot establish that the acknowledged connection generation closed. Exact CI `33961562189` is queued; #255's current adoption below preserves its connection-provenance repair. Neither coverage nor loopback tests authenticate the missing connection binding, and the instrumentation warning remains.
- PR #255 now adopts #254 at `3e7057443d7c9532ff526acb5eefe8cd4778c767`, retaining predecessor `ebac126d1632c94775c2454423575275eec45def` and parent `b11b6c9bccd8335b58a4fb599f8ad29ac419637f` by ordinary merge. All 15 owner production/test blobs are byte-identical to the predecessor. Native release-contract discovery moves from zero RED to one executed GREEN; 13 focused real-loopback tests, 142 Python contracts, compileall and the complete Rust 1.97.1 gates pass. Numerical 100% coverage is 1082 functions, 11032 lines, 14086 regions, 1202 branches. CI `33964381844` was queued at new-head observation. The previous unused-accessor repair is correctly attributed to `ebac126d...`, not its failing `63cbca0...` predecessor. Process/profile cleanup and hosted acceptance remain unproven.
- PR #256 now adopts #255 at `ced4a851ca66c08d895a098725c7f0ad3ecf0c38`, preserving predecessor `9f2e6f29be46371762e3031a97c1cac04720694f` and current parent `3e705744...`. The core click implementation, exports and four tests are byte-identical; the correlation child delta remains only its typed click kind. Four click tests, 13 provenance/teardown tests, 142 Python contracts and complete Rust gates pass after inherited release discovery changes from zero RED to one GREEN. Numerical 100% coverage is 1088 functions, 11077 lines, 14146 regions, 1210 branches. CI `33965138401` and compatibility `33965138418` were queued; inert serialization remains distinct from authorized input or an observed browser click.
- PR #257 now adopts #256 at `f4a8f2cbf515bea348f500b59615a9581c1b96a2`, preserving predecessor `ea2b5b78868917219c46f1304558b92490a7f6fe` and parent `ced4a851...`. The sender and two child-test files are byte-identical; the composition conflict retains both the sender and connection-bound reader. Ten focused tests, 142 Python contracts and full Rust gates pass after the inherited zero-to-one release-contract repair. Numerical 100% coverage is 1093 functions, 11131 lines, 14202 regions, 1214 branches. CI `33965440517` was queued. No focused retry was needed in this run; that does not repair the unchanged fixture's previously observed platform-level socket race. Frame-write completion is not browser-action completion.
- PR #258 is Draft at exact head `5f830324f6d5a47ac213a57528ef95649bdfd0df`, retaining predecessor `f2ceabb3ea50b1959e936503c50cae12f3e6e480` and actual #257 `f4a8f2cb...` by ordinary merge. Its response implementation, four loopback tests and child-owned bounded socket-observation adjustment are byte-identical. Fourteen focused tests, 142 Python contracts and full Rust gates pass; numerical 100% coverage is 1100 functions, 11185 lines, 14272 regions, 1214 branches. CI `33965695405` was queued. The inherited release contract now executes. This click-response API remains protocol correlation only; the separate received-connection proof for session teardown does not automatically authenticate it or establish a browser post-condition.
- PR #259 is Draft at exact head `66731982ed51ad62a04fcfbc759b0a33721a0254`, retaining predecessor `e1105ddf86f6c79443af8b4d306b9d34cb703c17` and current #258 `5f830324...`. Seven child production/test blobs remain byte-identical, including context admission, navigation projection and all three navigation suites. Fifteen focused tests, 142 Python contracts and full Rust gates pass after native release-contract discovery is restored. Numerical 100% coverage is 1140 functions, 11841 lines, 15129 regions, 1332 branches. CI `33965966736` and compatibility `33965966679` were queued. Exact context/URL matching is observation evidence, not transport authentication, click causation, origin rebinding or document advancement.
- PR #260 is Draft at exact head `2c5049aff97a90958e8262b1d403bdcbd64a1e8b`, retaining predecessor `3a651967c421f77088fe25e86a63faae295390b3` and current #259 `66731982...`. The document-advance implementation, two loopback tests and Proposed ADR 0103 are byte-identical. Seventeen focused tests, 142 Python contracts and full Rust gates pass after the inherited release contract becomes executable. Numerical 100% coverage is 1154 functions, 11969 lines, 15314 regions, 1334 branches. CI `33966229736` was queued. Stale-epoch and retired-context rejection remain intact; registry advancement cannot authenticate the observation, bind the new origin or prove action causation. All six current integrations above retain the unstable branch-option warning, remain Draft and unmerged, and require their own hosted checks and policy-satisfied promotion.
- PRs #250 through #257 remain Draft. The recorded fully locally verified chain used exact heads #250 `0eab23d5e388c5c8b984c0021a58316680c9ba8b`, #251 `86e8ad76838f2a64aa7e0cd56ba1f931c8d0c3dc`, #252 `2015259529ada99af836989079cc85a15779a2d8`, #253 `0d72082e595c0e1fcc03d609ba337896ed14e2fc`, #254 `cbaf50dcc97753cc73135497ea8225e8b18de190`, #255 `a13de5f9321e72c1867974eb7a43230f031e58df`, #256 `9f2e6f29be46371762e3031a97c1cac04720694f`, and #257 `ea2b5b78868917219c46f1304558b92490a7f6fe`. Each updated tree passed 141 Python contracts, the full Rust gates, and CI-equivalent pinned-nightly 100% production coverage. #250 and #257 each had one known macOS socket-observation race; their exact focused retries and complete coverage reruns passed. Exact-head review found that #253 still lets three unauthenticated caller booleans mint `OperationallyComplete`; raw teardown claims must remain explicitly unverified until session-bound transport, process-exit, and profile-removal evidence types are issued by their owning runtime boundaries. Hosted exact-head checks and ordered parent integration remain required. The current #255–#260 rows above supersede this older lineage only at their explicitly verified heads; no predecessor GREEN transfers to later hosted checks or protected-main delivery.
- Reviews [#254](https://github.com/ContextualWisdomLab/OriginWeave/pull/254#pullrequestreview-5120077272) at `cbaf50dcc97753cc73135497ea8225e8b18de190` and [#255](https://github.com/ContextualWisdomLab/OriginWeave/pull/255#pullrequestreview-5120077213) at `a13de5f9321e72c1867974eb7a43230f031e58df` found a remaining connection-provenance gap. #254 passed six focused closure tests, formatting, strict network Clippy and network rustdoc locally. #255 removes the parent process/profile booleans and keeps overall teardown pending, but a new loopback regression fails when an acknowledgment from connection A is combined with closure from connection B: the assessment reports the acknowledged transport closed. Both connections can use command id 7; #254 retains only closure kind/status and the acknowledgment retains only its command id. Repair the established-stream, frame/message and command-correlation owners to preserve a non-forgeable connection-generation binding, carry it into acknowledgment and closure evidence, then reject mismatches in #255. Different-session and same-session reconnect regressions are required; endpoint equality, command-id equality and caller-supplied markers are insufficient. This is unresolved evidence integrity, not operational-completion authority or shipped behavior. Both PRs remain Draft with hosted checks queued.
- Follow-up review `5120203295` at #255 head `92392b2da33bcd5446c6d2eb1b5504c39e60e3d6` established that received-message provenance must be checked before correlation is consumed. At that reviewed head, sender/closure generations rejected independently parsed evidence, but the two-real-socket response-substitution regression still failed: response B combined with outstanding-command state A became acknowledgment A because its generation came from the registry rather than the received text. The [exact-head review and runnable regression](https://github.com/ContextualWisdomLab/OriginWeave/pull/255#pullrequestreview-5120203295) preserve the RED lineage; the later repair evidence below supersedes this failure only for its explicitly tested snapshot.
- Received-connection verification at #255 head `e7bfec4488b7cb4776df7b546cacb46c8c9eb13e` found that all 12 focused tests passed: the trusted reader retains one non-cloneable stream and assembler through fragments/control messages, the parser rejects a foreign generation before consuming the original command, and cross-connection closure cannot satisfy the acknowledgment. All 141 Python contracts and strict rustdoc passed. Full quality remained incomplete: formatting and Clippy failed, and pinned-nightly coverage rejected lines=11046/11049 and regions=14067/14071 even though functions and branches reached 100%. The first complete stable Rust run also exposed the inherited fixture race; its unchanged retry passed, which was not treated as repair. The fixture root cause is now repaired at #242 and integrated into #243 as recorded above. [Verification and exact uncovered paths](https://github.com/ContextualWisdomLab/OriginWeave/pull/255#issuecomment-5550208364) remain snapshot-specific; later #255 changes, hosted checks and descendant adoption need fresh verification. Overall teardown remains pending and none of this is protected-main delivery.
- Follow-up quality verification at #255 head `63cbca0a98cf9496af981819d98029e656fc4342` completed locally at `2026-09-05T08:20Z`: 141 Python contracts, compileall and the complete Rust 1.97.1 tests passed. Formatting still failed, and strict Clippy rejected an unused private accessor left after the impossible provenance fallback was removed. Workspace check and strict rustdoc completed with that compiler warning. Pinned-nightly coverage enforcement failed at functions=1082/1083, lines=11046/11051, regions=14071/14074; branches=1202/1202. The missing-lines report identifies only the obsolete accessor's five lines; prior event/null-id and generation-exhaustion gaps are covered. The minimal owner correction is removal of that uncalled accessor plus actual pinned rustfmt, preserving received-connection validation and rejecting warning/coverage exclusions. All read-only validation handles are terminal and the existing #255 source owner has the diagnostics; no competing source change was made. Exact CI `33953476003` remains pending. These results supersede the earlier local snapshot only at this tested head, not at later repairs or protected main.
- PR #139 at `b7ea5bfe336456fb263dd479a60b1cd0193d8a47`, based on #136 `1cffb2e23d4002f12e8462c4c8c24f404a4eeee7`, preserves all inherited documentation and limits its unique delta to the intended cleanup wrapper, regression and changelog. Both indexes retain ADRs 0007–0010 and ADR 0009 remains Proposed. All 179 Python contracts passed; [review evidence](https://github.com/ContextualWisdomLab/OriginWeave/pull/139#issuecomment-5550147096) records all three resolved findings. Current hosted Rust, coverage and pinned-Chromium runs were cancelled, so this Draft has no current hosted or browser-runtime GREEN from that verification.
- PR #141 at `fbdf64f5818ce0c53b475196d5bcfa2ac9900846`, based on #139 `b7ea5bfe336456fb263dd479a60b1cd0193d8a47`, retains the reviewed failure/surface/timeout cleanup contracts. All 183 Python contracts passed; controlled probes confirmed that unexpected exceptions propagate after temporary-profile removal and cleanup-exit failures cannot return false cleanup evidence. The remaining two informational threads were resolved without source changes, leaving all four threads resolved; [review evidence](https://github.com/ContextualWisdomLab/OriginWeave/pull/141#issuecomment-5550364179) binds these results to the unchanged head. Hosted CI `33916583623` and pinned-Chromium run `33916583506` were cancelled. This is not current browser-runtime GREEN, adversarial erasure proof, counted approval or protected-main delivery; the PR remains Draft.
- PR #142 at `015025f2539e4fb1dbd7d259ec22dad50f944396` passed fresh read-only failure-boundary review: 187 Python contracts passed, and controlled probes preserve unknown identity on observation failure, reject a still-live identity at the existing deadline, and accept absence or a different start-time as exit of the original identity. Both informational threads were resolved without production changes. Its exact-head CI `33916627099` and compatibility `33916627091` remain cancelled, not Linux/browser runtime GREEN.
- PR #143 at `44fd9a450f864feff5cf2ba2883425a71ba10b9b` is a documentation and real-path regression repair on exact #142. The release-record check first failed for the missing failure-cleanup entry, then all five focused and 192 Python contracts, full Rust 1.97.1 gates and pinned-nightly 100% coverage passed: 415 functions, 3555 lines, 4444 regions, 476 branches. The production runner is unchanged. Observed exit and a surviving identity both retain failed-task status; process-observation errors leave termination unproven and record only the bounded observation-error type, losing the original browser-failure type in that fallback record. Private exception messages remain absent, and profile cleanup cannot turn a failed task into a pass. Both informational threads are resolved with this limitation documented. CI `33954715539` and compatibility `33954715568` are queued; controlled local probes do not prove real Linux/pinned-Chromium acceptance. Current #144 adoption is recorded below.
- PR #144 at `09f2e087d0c20fe81386c18099e739a9e611a8ad` completed read-only informational review: 194 Python contracts passed, and controlled real-helper probes confirmed conservative failure when a descendant PID is reused before start-time capture, separate root and process-set deadline budgets, and the defensive exit-count invariant across all eight absent/live combinations of three descendants. Three informational threads are resolved with those limits recorded; no runtime code or timeout changed. CI `33916685294` and compatibility `33916685137` remain cancelled, not current browser acceptance. This is predecessor evidence superseded by the current adoption below; process identities appearing after sampling, cgroup ownership and OS-wide orphan absence remain outside this evidence.
- Earlier #258 `f2ceabb3ea50b1959e936503c50cae12f3e6e480` and #259 `e1105ddf86f6c79443af8b4d306b9d34cb703c17` each passed 141 Python contracts and full Rust/numerical-coverage checks on the older #257 lineage. Their new current-parent integrations and independent 142-contract verification are recorded above; the historical measurements do not transfer.
- PR #93 is Draft at exact head `0664f0452cb329cd692cce7f61f9001652abfda2`, based on #271 exact `802ec806cdd4560eab48c484f435766ecabda353` and synchronized with #242's complete opening-fixture cleanup. `SemanticNodeActionBinding` preserves semantic-node and business-origin identity, but does not authorize policy or execute input. Its 141 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered ancestor integration remain required.
- PR #95 is Draft at exact head `97aa0f2e340ee6fd920d0418f97af276b190554f`, stacked on #93 exact `0664f0452cb329cd692cce7f61f9001652abfda2`. Only `Decision::Allow` creates policy-authorized value; other decisions fail closed, and registry-owned current authority returns `NotAdmitted` after document advance removes the node. The slice performs no browser I/O or postcondition proof. Its 142 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #96 is Draft at exact head `b7ba8dd1433410cee43084a73e31816da841b2a2`, stacked on #95 exact `97aa0f2e340ee6fd920d0418f97af276b190554f`. Registry-owned node authority is revalidated before one adapter callback, and the callback is never invoked after admission is removed. Adapter completion remains separate from postcondition proof. Its 143 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #101 is Draft at exact head `bc810b121bb0303f55afa8777a23cc0f9748c1db`, stacked on #96 exact `b7ba8dd1433410cee43084a73e31816da841b2a2`. A known-disabled interactive action fails as `NodeNotEnabled`, while `ScrollIntoView` remains selectable; retained observation state is neither current Chromium proof nor dispatch authority. Its 144 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #102 is Draft at exact head `a123c55d4839dae1db7e6671f7d4d158c7cfd9db`, stacked on #101 exact `bc810b121bb0303f55afa8777a23cc0f9748c1db`. Fresh semantic comparison rejects another node as `ObservationAuthorityMismatch`, removed action support, and newly disabled interactive state, but does not obtain or authenticate the observation or dispatch input. Its 145 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #103 is Draft at exact head `8b3416169346fa04b53c915b813d55ccf47d1876`, stacked on #102 exact `a123c55d4839dae1db7e6671f7d4d158c7cfd9db`. Same-call dispatch checks registry-owned browser authority first and fresh semantic state second; the callback is never invoked on either failure, and completion is not postcondition proof. Its 146 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- Issue #201's release/SBOM lane remains dependency ordered, and GitHub Releases is empty. OriginWeave remains pre-GA until exact protected-head versioning, signed artifacts, SBOM/provenance, reproducibility, rollback, compatibility, security, and commercial-acceptance evidence exist.
- The principal commercial gaps remain #27 Manifest V3/native-host isolation, #9 bounded HTTP/browser-network consumption, #10 purpose-bound protected-data runtime, #28 the first complete Chromium Agent Task vertical slice, #199 durable WARC/PROV and retention/replay, #200 stable BAP/MCP runtime API, #201 signed cross-platform distribution/update/rollback/SBOM/SLSA, #202 enterprise identity/tenant/policy/approval/audit/SLO operations, #203 exact-artifact commercial acceptance, #276 contextual-orchestrator migration, and #279 exact-head documentation verification without unnecessary Rust queue load.
- Current evidence procedure remains fail-closed: queued reviewer evidence is non-passing; paginate the full PR inventory, bind checks/reviews/threads/workflow runs to each unchanged exact head and independently resolved live base, consult the active ruleset, and discard queued, skipped, cancelled, absent, predecessor, synthetic, status-only, and model-only evidence as passing proof.

- Current PR #144 parent adoption is `3c0c5c363c2da0b4e8a621226e2f7766c735b743`, ordinarily integrating #143 `44fd9a450f864feff5cf2ba2883425a71ba10b9b`. The previous child collected three inherited failure-path contracts instead of five. Both release records and the parent's real failure-path regression now coexist with unchanged child production and process-set tests. All five focused and all 196 Python contracts, complete Rust 1.97.1 gates and pinned-nightly 100% coverage pass locally: 415 functions, 3555 lines, 4444 regions, 476 branches. Exact CI `33956094018` and compatibility `33956094011` are queued, not acceptance; the prior informational review limits remain unchanged.
- PR #145 now adopts #144 at `bb5e8f834c37f9ce35f84db8ed1146da3659d6aa`. The ordinary merge preserves the child runner and both process-set/forced-close regressions while moving the missing inherited-contract count from three to five. All five focused and all 202 Python contracts, full Rust gates and the same four-dimension 100% coverage pass. Ten controlled actual browser-pass/trial probes distinguish observed root/set survival, observation errors and interrupted capture; session/driver cleanup alone does not prove process exit. Both informational threads are resolved with that corrected limit. Exact CI `33956596014` and compatibility `33956596013` remain queued.
- PR #146 review repair is `812c0020bd2ecdb3eda39d999ed3327647550bdf`. Five new tests first reproduced 10 assertion failures and 7 uncaught protocol-error cases despite the predecessor's 216 passing contracts. The repair restores the non-boolean pre-shutdown count range, preserves only validated known ordinary cleanup fields, records typed terminal protocol errors without remote messages, and removes the obsolete HTTP-body close recognizer. Startup retry policy, request redaction and owned-process deadlines remain unchanged. Six new behavioral tests and all 224 Python contracts, complete Rust gates and the same 100% production coverage pass. The concurrent parent-adoption `11944410450684809ee1a71a35c77abafc5358db` is retained through an ordinary merge whose tree is identical to verified repair `d636a1829332610ada458df7b5b9d267f038a2f8`. All four review threads are resolved; exact CI `33957036553` and compatibility `33957036650` are queued, not browser acceptance. #147 must adopt this parent while retaining its own shared-deadline behavior. These three PRs remain Draft and no protected delivery, eligible approval or release is claimed.
- Follow-up #287 review evidence at `2026-09-05T09:07Z`: Strix scan job `101243872506` executed successfully, including its quick scan and report upload. Noema job `101245089068` failed with HTTP 502 after 2739.2 seconds and one caller attempt. A fresh exact-head inventory showed no duplicate Noema run, so the same failed job was queued for a supported rerun; attempt 2 now has successful exact-head admission and queued review job `101282290732`. Queue admission is not provider recovery, executed review, a verdict or eligible approval. No caller timeout, provider, model, workflow or quality gate changed. The three central CodeQL dispatches remain queued and were not duplicated. This supersedes the earlier in-progress Noema/Strix snapshot above without transferring any old result to a new source head.

- Prior #255 source-owner repair is `ebac126d1632c94775c2454423575275eec45def`, verified at that earlier owner snapshot before current-parent adoption. Its 11-file delta removes the obsolete private accessor, applies pinned formatting and repairs the Rust documentation link without removing received-connection validation. The source owner reports 141 Python contracts, full Rust 1.97.1 verification and coverage functions=1082/1082, lines=11025/11025, regions=14071/14071, branches=1202/1202. This is the owner's historical exact-tree local verification, not an independent rerun by this documentation lane or a transfer of earlier GREEN. At that observation, hosted CI `33957423557` was queued; the earlier `63cbca0...` quality RED is retained as historical RCA, and the new independent parent-adoption results are recorded above.
- The pinned cargo-llvm-cov 0.8.6 branch report emits `warning: --branch option is unstable`. The warning was independently reproduced when reporting the existing #144/#145/#146 measurements, whose numerical verifiers still pass. Thus numerical coverage enforcement is distinct from warning-free instrumentation. The #255 source owner likewise leaves warning-free measurement acceptance outstanding. No warning suppression, coverage exclusion, denominator manipulation or alternate unreviewed measurement is used; compiler/Clippy/rustdoc results, instrumentation maturity, hosted acceptance and release readiness remain separate.
- Central Strix #1563 advanced to `13fbb48e0b3eeca4ce7d9678add934f9bd87ad3f` from `1221b1604e1a6cfde8ca5ab7fd3e93e0fe9faf69`. The independently inspected four-file source diff adds a completed-artifact regression using the exact #166 denied-report prose and restricts console denial matching to explicit diagnostic lines while retaining warning, fatal, and workflow-error signals. This establishes a source repair, not an executed Origin acceptance result. The focused and full shell harness results plus 2890 passed, 1 skipped, and 21 subtests are owner-reported; this consumer lane did not execute them, and the skipped case is not passing evidence. At 13:46 UTC, ordinary merge `eaf9594f7fe8d8e1994349289183d6cbad056579` retained `13fbb48e...` and adopted protected central main `6f8c51d7389c22ebaf294fe8fe9ef495257883c0`. That fresh head is unmerged and hosted checks are queued; its cancelled predecessor checks and earlier local results do not validate the new merge. No Origin #166 consumer rerun was performed, so its historical executed failure remains unreplaced. Actual #219/#240 gateway and hosted execution-limit failures remain distinct.
- Historical #147 `46e75859ef5aff2054fd6ca1720d518438e97d0f` included #146 `812c0020...` in ancestry but dropped the parent's six review-evidence regressions and reversed their associated production repairs. Native discovery collected zero tests; read-only replay of the exact parent module against that child reproduced 10 assertion failures and 8 uncaught protocol-error cases. The merge title, ancestor presence and earlier 222 passing child tests did not establish successful inheritance.
- At predecessor #147 `af1b98ba377c73b88baa9633e2232e7f76e76f36`, the external synthesis restored parent runtime repairs and all six methods while preserving the combined forced-close observer. Its recorded RED is that all 228 native tests execute but two forced-close protocol-fault subcases fail. The inherited test injected the individual root waiter although the forced-close lane called the combined observer; a fresh focused reproduction took 10.417 seconds. The [review evidence](https://github.com/ContextualWisdomLab/OriginWeave/pull/147#issuecomment-5550841022) retains this failure and its subsequent repair rather than treating recovered ancestry as acceptance.
- Current #147 `3dff28d9bf2dd27b72507e39979d51b8bf140fb4` repairs only that lane-specific test injection and its documentation. Ordinary cases retain the individual False result; forced-close cases inject the combined (False, False) result and assert the exact root-only identity tuple. Every existing assertion and all six methods remain; all six focused and all 228 native tests now pass, along with compileall, complete Rust 1.97.1 gates and the same 415/3555/4444/476 numerical 100% coverage. Production, deadlines, retry policy and failure denominators are unchanged; the branch warning remains explicit. Writer reassessment occurred after the temporary route closed and contacted tasks confirmed read-only status; the current repair is a normal forward push. CI `33959982049` is pending and compatibility `33959982033` is queued. The ordinary-pass two-deadline finding stays unresolved in the separate #150 scope; no hosted, protected-main or real-browser acceptance is claimed.
- Current #148 `bded4fc9d32e5e047cea99d182aa05ae2cd03bf6` adopts that exact #147 by ordinary merge and repairs review `5120692604`: a real crash-trial startup probe first failed because the emitted browser arguments disabled Chromium's sandbox. Removing that single argument preserves the startup-failure path, driver termination and temporary-profile removal, with no unsandboxed fallback. The inherited six review regressions and both child crash-cleanup/exit-detection test blobs remain intact. The 247-test macOS run has 244 passes and three explicitly skipped Linux pidfd cases; all 247 Python contracts pass on Linux with zero skips, including a killed unreaped child, non-terminating signal and stale process identity. Complete Rust 1.97.1 gates and numerical 100% coverage pass at 415/3555/4444/476, with the branch warning retained. CI `33962062616` and compatibility `33962062608` are queued. Three other launch lanes still disable the sandbox in this branch and require their separate owner repairs; this is not runner-wide sandbox completion, pinned-Chromium acceptance, review dismissal or protected delivery.
- Current #150 `d3c29359fd10e540ca9b3b2723bde6f94866cdb6` adopts the same #147 while preserving its existing combined ordinary-pass observer and both child test blobs. The inherited six review methods were absent before adoption. Their restoration exposed two ordinary protocol-fault failures, and full replay then exposed three sibling failure-observation cases still injecting the superseded individual waiter. Test-only reconciliation now injects paired observer outcomes and asserts the exact root-only identity tuple; incomplete process capture cannot claim full-set termination. No production timeout, retry, polling rule, success condition or denominator changed. All inherited and child checks execute, and all 229 Python contracts pass on macOS and Linux. Complete Rust gates and the same four-dimension numerical 100% coverage pass with the instrumentation warning retained. CI `33962429156` and compatibility `33962429172` are queued. The ordinary-pass deadline finding remains valid on #147 until this separate child is actually integrated; it was not resolved early.
- Supplemental Linux verification used the existing Colima guest (Python 3.12.3, kernel 6.8.0-117-generic), without installing dependencies or changing VM configuration. A first macOS archive introduced binary AppleDouble files and caused two unrelated TLS source-decoding failures; exporting the same tracked files without filesystem metadata repaired the transport, not the tests. Final host/guest tracked-file SHA-256 manifests match exactly: #148 `c808905cbea16fd85b9519fa0a7c431d032f6692bea172950e5ba23613e7e98b`, #150 `0f5f93ed8789843ca4f228d1117328d029e3fab461f0450c75964f2ef6b98324`. These are actual Linux contract runs, not actual pinned-Chromium browser trials, hosted CI or release evidence.
- #166 Strix run `33929688857` / job `101237371800` failed at `2026-09-05T10:53:36Z` on unchanged `e84a1a2...`. Its executed trusted central source was `a9aeee8fc94ad6002a059b380b268590ce496ef0`. The final scanner attempt exited zero, recorded completion from `10:25:17Z` to `10:53:29Z` and emitted zero SARIF results, but the console classifier also matches ordinary denied-policy prose. Replaying its exact first predicate against the sanitized final console log matches two policy-description lines and independently explains the final provider-unavailable classification. The final scanner report log has no corresponding failure-token match; the initial report-directory hypothesis was rejected. Artifact `9968177796` has SHA-256 `1ed63eb46e83d8acb61e281a6563400269a45c0924ae989ee30a4addc041048b`. Earlier attempts did have errors, so neither all-attempt success nor authoritative security acceptance is inferred. The reproduced classifier defect and exact evidence were sent to the existing central owner; no central source, provider, timeout, gate or required status was changed here, and repeating the same classifier was not used as a remedy.
- #219 Strix run `33925442296` / job `101226800608` on `65e4315...` is a distinct failure: a real gateway HTTP 500 occurred at `08:08Z`, and the job was cancelled at `10:51:35Z` with the six-hour GitHub job execution limit explicitly recorded in its check annotation. This is neither a passing scan nor evidence that the #166 console defect caused that cancellation. Its already admitted Noema attempt 2 remains a separate queued item. Canonical provider/orchestration and runner feasibility must be reassessed before another attempt; no caller timeout or model fallback was expanded.
- #240 Strix run `33925596890` / job `101227893828` on unchanged `24930a3...` was also cancelled at `11:20:05Z`; its annotation independently confirms the same six-hour hosted execution limit. Its actual console records gateway HTTP 500 at `07:01Z` and `10:34Z`, followed by cancellation at `11:19:59Z`. These are observed failures, not a generic inference from a red status. No successor Strix run existed in the refreshed exact-head inventory, but an unchanged retry would not repair the canonical provider or classification defects; this source/documentation lane did not start one or alter execution limits.

- Temporary integration PR #291 is now closed after an external merge into `automation/147-merge-base-20260905`, whose base was #146 `812c0020...`. Live API metadata records head `52575632de07bcb90791e491b9a64b6875532ebe` and merge `24fad0498a49267982805d35d2dd57484efb4671`; Git confirms that merge is an ancestor of current #147 through forward integration `07369a37b54c11c32f810c3d5c107ad6d4b9feb4`. The temporary route neither replaced #147 nor reached protected main. The recovered runtime and test content, subsequent test-reconciliation repair and ordered acceptance requirements are evaluated separately from its merged/closed label. This lane performed no merge or closure. The fresh full inventory is 125 open / 12 Ready / 113 Draft.

## Observed snapshot: 2026-08-29

### Protected-main truth

- Protected `main` is at `542ca1e9c0a863595b8b6697790005d2471f5413` for this snapshot. Since the 2026-08-26 observation (`b05d5acca82b9d916ada2c8e82f59f92a89817e1`), protected `main` absorbed #161 (TLS trust-bundle identifier shape). PR #170's conservative `tools/list` discovery contract is also merged into protected `main`; the complete MCP adapter remains planned.
- Phase 0 remains complete as a reusable safety-kernel foundation: typed policy contracts, destination classification, direct TCP peer verification, TLS service identity, evidence bounds, resource mitigation, document-node authority, and protected-main tests.
- Phase 1 is **in progress**, not shipped. The first real Chromium vertical slice still needs the active WebDriver BiDi transport stack to reach protected `main`, then compose isolated Chromium launch, session/context identity, semantic observation, typed action authorization, native browser input, post-condition proof, evidence, cancellation, crash recovery, and profile/process teardown.
- HTTP/1.1 bounds, downloads/MIME, proxy/PAC consumption, full browser-network integration, the sensitive-data broker runtime, durable WARC/PROV capture, persistent task/API surfaces, signed cross-platform distribution, enterprise administration, and release-grade buyer acceptance remain open.
- Active pull requests remain evidence, not shipped behavior. Successful checks on a feature or stacked branch do not prove that protected `main` contains the capability or that a child can merge before its prerequisite.

### Open pull requests

The live repository contained **108 open pull requests: 24 non-draft and 84 draft** when this snapshot re-paginated the complete open inventory. Compared with the prior **2026-08-28 111-PR snapshot**, stacked evidence PR #53 was merged into its unprotected feature parent; the preceding **2026-08-28 116-PR snapshot** had already recorded #71, #154, #233, #234, and #235 merging into their unprotected feature parents. PR #217 was squash-merged into the unprotected #210 feature parent, followed by PR #67 into the unprotected #64 feature parent; the current queue therefore contains no protected-main shipment. These counts are queue evidence, not protected-main delivery; protected `main` remains `542ca1e9c0a863595b8b6697790005d2471f5413`, with 11 open issues and no releases or tags. The volume and stack depth remain themselves a product-delivery risk: review, exact-head checks, dependency order, and integration truth can drift faster than a buyer-visible vertical slice reaches protected `main`.

#### 2026-08-29 maintenance-loop record

This snapshot re-fetched the complete open-PR inventory, the protected `main` commit, the active required-workflow ruleset, collaborator permissions, and the exact base/head pair for each representative PR. Before PR #238 was published, the same maintenance loop observed 115 open pull requests (31 ready, 84 draft); a later 116-PR recheck preceded five stack merges, leaving 111 open pull requests (27 ready, 84 draft), and subsequent child-stack merges left 108 open pull requests (24 ready, 84 draft). The active ruleset still requires one counted approving review and resolved threads; the only repository collaborator is `seonghobae`, so review provisioning remains the merge blocker for main-targeting PRs. PR #170 is merged and is not active-PR evidence.

During this recheck, #71, #154, #233, #234, and #235 were merged only into their unprotected feature-parent branches after exact current-head checks and review-thread resolution. Their successful stack checks are not protected-main delivery, and their child branches retain independent evidence requirements.

The sensitive-data child slice #53 was subsequently squash-merged into the unprotected #46 feature branch at merge commit `93c713a107df05385f745db4dca20091f21c4a3a`; #53 was exact head `4ecc81e59ae7bc3a640e65e2442bf30c079bd94c`. PR #46 remains an active main-targeting parent. Its current exact base/head pair is protected `main` `542ca1e9c0a863595b8b6697790005d2471f5413` to `373113119446d99f578febd39efc19366e7736b1`; the head adds the ADR 0007 predicate-boundary clarification and regression contract, with local Python/Rust verification green. Current hosted evidence remains incomplete: automatic OpenCode run `33189822385` / job `98913006386` failed closed without a current-head verdict, and central Strix run `33190794267` / job `98915422837` failed closed after three provider HTTP 500 attempts without a vulnerability report. A direct central `opencode-review` dispatch run `33192478312` / job `98921183278` was rejected because repository_dispatch actor `seonghobae` did not match configured scheduler identity `github-actions[bot]`; no qualifying non-author approval is present.

The WARC/PROV child slice #217 was squash-merged into the unprotected #210 feature branch at merge commit `66f360ccac5cec60c72222cc79d58e39f6f00088`; #217 was exact head `6b8a3fdeae52ad94b90086bbc9b42863b90c9614`. PR #210 subsequently advanced to current exact head `7946dce9a3dd074047d93fca299d48c7aef40e47` after its merged-child attribution repair, recursively encoded-control repair, and exact coverage repair; this stack remains active-PR evidence and not protected-main delivery or approval evidence.

The browser-task interruption child slice #67 was squash-merged into the unprotected #64 feature branch at merge commit `5021d142583cb5a8e393248048bb824762a98056` from exact PR head `25ab76e8279d4a904d04afeb264bac3e89f46b45`. PR #64 consequently advanced from `debc761aa59aee1509b7a260474fa33216453511` to current exact head `5021d142583cb5a8e393248048bb824762a98056`; its exact-head hosted checks were regenerating at this snapshot, with no unresolved inline review threads. This stack remains active-PR evidence and not protected-main delivery or approval evidence.

The later exact-head recheck also recorded the WARC/PROV retention-lifecycle slice: #239 is Draft at `e840ca299d29a15223c8b9bb1397002c4f41b4a3` on #227 head `e45cd6cdcdee73b5c16dc942e6c98cb7e745fae0`, and #237 is Draft at `2459af602e72fbfe1ce816919473a1075ec0c41f` on protected `main`; their current exact checks and reviews remain independently actionable evidence. Neither active branch is protected-main behavior.

The same exact-head recheck corrected workflow provenance: the active ruleset's seven required workflow entries (`close-empty-pr`, `opencode-review`, `pr-review-merge-scheduler`, `security-scan`, `strix`, `sast-semgrep`, and `noema-review`) point to the central `.github` repository, repository ID `1274066402`, where all seven files exist on `main`; their absence from the OriginWeave tree did not prove missing workflow identities. On PR #210 head `0341079331f9cea669eb9a5cc21842fd6027431e`, run `33177641855` failed closed because no OpenCode current-head verdict existed, while run `33177641888` failed closed after three bounded attempts because the Strix provider/backend returned an internal server error and produced no vulnerability report. A `gh run view` workflow-endpoint 404 for the external workflow IDs is a lookup mismatch, not passing evidence or proof that the identities are absent. A bounded central scheduler dispatch was accepted as run `33178984025` with auto-merge and branch updates disabled; no result is transferred until the exact head is revalidated. These failures block affected PRs until current-head review and security evidence complete. This does not authorize bypass, self-approval, stale checks, or weaker gates.

The WARC resource-record slice #210 is now Ready; PR #210 current exact head is `7946dce9a3dd074047d93fca299d48c7aef40e47` based directly on protected `main` `542ca1e9c0a863595b8b6697790005d2471f5413`, after incorporating #217, correcting the merged-child attribution, and closing the recursively encoded-query-control coverage gap. Its predecessor head `5f59947f5e4b0d3bc0aa5b2d4c6722d3b7c43047`, prior stack merge head `66f360ccac5cec60c72222cc79d58e39f6f00088`, earlier exact head `bea65643109449d63d367a35b8d9bf327ee7cb2c`, and their OpenCode/Strix provider failures remain historical evidence only. At the current head, `Rust contracts` job `98942518975` and `Production coverage` job `98942518680` succeeded, while `noema-review` job `98942513421` and `strix` job `98942803402` remain in progress; the current-head OpenCode verdict is still absent and no counted approval exists. Central repair PR #1391 was opened at historical head `e4ba6b599cd1e50d0139762885682607b731655d` and is now open at exact head `36ac3aa71b2580685f84d416a81e42c39dee927c` on current central `main` `e1b03eebc6dc5c85aed393e5928927c96376cf46`. Its branch-update merge and follow-up prompt hardening are not approval or coverage evidence.

The documentation refresh PR #238 remains Ready on protected `main` `542ca1e9c0a863595b8b6697790005d2471f5413`; its moving current head is intentionally authoritative in live GitHub metadata and the PR body rather than repeated in this self-referential baseline, while local full-suite evidence is green. The immediately preceding PR #238 head `d0b0d1ed92f891f14646fc673b8e1c0d912586fd` remains historical: automatic OpenCode run `33193822920` / job `98926243116` failed closed without a current-head verdict, current Strix run `33193822929` / job `98925769697` succeeded, and central dispatch run `33194506918` / job `98928580387` also failed closed at OpenCode after its validator, bootstrap, and coverage jobs succeeded. The preceding exact-head OpenCode run `33184553025` / job `98894986761` and earlier targeted run `33182749298` remain historical; targeted run `33182749298` dispatched OpenCode run `33182772296`, which failed closed as `MODEL_OUTPUT_UNAVAILABLE` with `model pool exhausted`, and optional cross-repository status publication was denied with HTTP 403. The current Devin documentation-evidence finding requiring an enforcing `tools/list` traceability contract has been implemented and its review thread resolved. No non-author counted approval exists, so these are review-tool/documentation findings, not approval or protected-main shipping evidence.

The controlled Chromium prerequisite stack was also re-fetched after its exact-head repairs: #70 is Ready at `441a8ce1d09c329c5c1168f4906d9a38fd0abc01` on protected `main` `542ca1e9c0a863595b8b6697790005d2471f5413`; #71 is merged into the unprotected #70 feature branch; #72 is Draft at `600d3975c02b68da1974a4c73069b966b39dce7b` on the retained #71 branch; and #73 is Draft at `ce1b138509ab4f52cb0f80290f104358473c6ed3` on #72. #70's Rust, coverage, pinned-Chrome, and ordinary security checks are successful, while exact-head OpenCode failed closed without a current verdict and exact-head Strix failed closed after three provider HTTP 500 attempts; all current Devin informational threads are resolved. #72 and #73 retain independent exact-head evidence requirements. #82 is Ready at `f5776f5f233ac0a7c05e3f4a2846436c23438043` on protected `main`; its Rust, coverage, Chrome, and ordinary security checks pass, exact-head OpenCode failed closed without a current verdict, and its current Devin informational thread is resolved. #152 is Ready at `81407a0e5189a413d1be0963fea90a0c2f254ce1` on protected `main`; its source, coverage, and security checks are successful except exact-head `opencode-review`, which failed closed without a current-head verdict. These are active-stack evidence only, not protected-main behavior or merge authorization.

#### Current exact-head active PR evidence

The following representative slices were re-fetched from GitHub for this snapshot. Their exact base/head pairs are recorded so later checks, reviews, and restacks cannot be confused with predecessor evidence:

| PR | State | Exact base head | Exact head |
|---|---|---|---|
| #73 | Draft | `600d3975c02b68da1974a4c73069b966b39dce7b` | `ce1b138509ab4f52cb0f80290f104358473c6ed3` |
| #72 | Draft | `f86ce504138e79d6e95141a441f60b40920e1fa6` | `600d3975c02b68da1974a4c73069b966b39dce7b` |
| #46 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `373113119446d99f578febd39efc19366e7736b1` |
| #70 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `441a8ce1d09c329c5c1168f4906d9a38fd0abc01` |
| #82 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `f5776f5f233ac0a7c05e3f4a2846436c23438043` |
| #210 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `7946dce9a3dd074047d93fca299d48c7aef40e47` |
| #64 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `5021d142583cb5a8e393248048bb824762a98056` |
| #237 | Draft | `542ca1e9c0a863595b8b6697790005d2471f5413` | `2459af602e72fbfe1ce816919473a1075ec0c41f` |
| #239 | Draft | `e45cd6cdcdee73b5c16dc942e6c98cb7e745fae0` | `e840ca299d29a15223c8b9bb1397002c4f41b4a3` |
| #229 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `0145ccba5901e301b41d4be674ca1ed23483ad37` |
| #220 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `b11db2be68f9b6d71aa4c4290b97a8b22097b353` |
| #211 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `52a918577958a5701e1146c7eb8b62fe8f8ccd44` |
| #152 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `81407a0e5189a413d1be0963fea90a0c2f254ce1` |
| #195 | Draft | `6922dd98779e8f8aad132a3b1f563d7ba6e6d070` | `48eb2d23009c1c804520dd5efcd0d4d072aacef1` |
| #242 | Draft | `48eb2d23009c1c804520dd5efcd0d4d072aacef1` | `55fef0c3fae1724eddada53e52c4a0311f509aa3` |
| #248 | Ready | `6407895f4db4bee640074cb9c9d3cbe8b0e9e13a` | `7d6db16b2ead201fcec320854923f90d3ad0d8bc` |
| #249 | Draft | `de7754aaeb97ccb0fd47bcbe1c4d99c10eaf84eb` | `2279d18189fcd6cdb2b38aca53b877434d41c913` |
| #250 | Draft | `2279d18189fcd6cdb2b38aca53b877434d41c913` | `cbddf507ac41080ee65230a8d6047dd8d06fd719` |
| #251 | Draft | `cbddf507ac41080ee65230a8d6047dd8d06fd719` | `99e2eb946e7ebbffa68f65a00d25243a2cf4242a` |
| #252 | Draft | `99e2eb946e7ebbffa68f65a00d25243a2cf4242a` | `881a599fb3a920b6bfd4f0a276f3cf24a61d8194` |
| #253 | Draft | `881a599fb3a920b6bfd4f0a276f3cf24a61d8194` | `5e2b17c7a8d1953bb06a41e0296f801f0d015a9a` |
| #254 | Draft | `5e2b17c7a8d1953bb06a41e0296f801f0d015a9a` | `a4841016b94cb18e917d250a9ca9149af54ceef0` |
| #255 | Draft | `a4841016b94cb18e917d250a9ca9149af54ceef0` | `f8edec38cf8ab7fde22b8d1de9305728c1a2f25b` |
| #256 | Draft | `f8edec38cf8ab7fde22b8d1de9305728c1a2f25b` | `bd1f5ac60a76d2edb35e63095d406e53bc43931f` |
| #257 | Draft | `bd1f5ac60a76d2edb35e63095d406e53bc43931f` | `ac73abfe7edd5786a7eb3eaab1a8c773093be7d3` |
| #93 | Draft | `802ec806cdd4560eab48c484f435766ecabda353` | `0664f0452cb329cd692cce7f61f9001652abfda2` |
| #95 | Draft | `0664f0452cb329cd692cce7f61f9001652abfda2` | `97aa0f2e340ee6fd920d0418f97af276b190554f` |
| #96 | Draft | `97aa0f2e340ee6fd920d0418f97af276b190554f` | `b7ba8dd1433410cee43084a73e31816da841b2a2` |
| #101 | Draft | `b7ba8dd1433410cee43084a73e31816da841b2a2` | `bc810b121bb0303f55afa8777a23cc0f9748c1db` |
| #102 | Draft | `bc810b121bb0303f55afa8777a23cc0f9748c1db` | `a123c55d4839dae1db7e6671f7d4d158c7cfd9db` |
| #103 | Draft | `a123c55d4839dae1db7e6671f7d4d158c7cfd9db` | `8b3416169346fa04b53c915b813d55ccf47d1876` |
| #124 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `fdb88698ca20626a6643bc2ad7944fb968835700` |
| #37 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | `5e3dfcbd7a4daea297782cb99635990368589232` |

These rows are delivery evidence only. None has counted independent approval in the current collaborator inventory.

The current documentation branch is PR #238 itself, so its self-referential exact-head row is intentionally omitted; GitHub PR metadata and the PR body are the authoritative current-head record for this change.

#### Historical 2026-08-26 maintenance-loop record

The interactive maintenance loop performed the following verified state changes on exact heads; none of them is protected-main behavior until merged:

| Action | Exact evidence |
|---|---|
| Supersession closure | #153 closed with replacement evidence: base-stack tip (`4da223ac`) already implements `_terminate_owned_process_bounded` exit-race tolerance that supersedes the branch delta |
| Conflict reconciliation | Merge commits pushed to #37 (`27f6acd6`, ci.yml aligned to reviewed `nightly-2026-08-18` pin), #149 (`7852a540` + rustfmt fix `54f96008`), #152 (`65b0c705`), #173 (`ecc9574a`), #175 (`765c88f6`, keeps `crate_root.rs` naming) |
| Governance remediation (#212) | #43 reconciled with main in `04e262d5`; the `chrome_sandbox` workflow mutation was first removed, then restored under recorded independent authorization (issue #212 option (b)) because the PR's own contract test fails closed without it; fresh exact-head checks re-ran on the restored head |
| Security finding fix (#124) | Strix vuln-0001 (Unicode homoglyph path confusion, MEDIUM) remediated in `30cc458b`: audited workflow paths now restricted to a canonical ASCII alphabet with homoglyph/fraction-slash/fullwidth regression contract tests; CHANGELOG updated |
| Fail-closed provider re-dispatch | ~21 failed Strix required-check runs re-dispatched on unchanged exact heads; completed reruns returned success on #46, #48, #156, #157, #159, #218, and #219 heads at snapshot time; cancellations only where newer heads superseded the run |
| Current-head review re-dispatch | Central merge-scheduler dispatches sent for #47, #62, #63, #65, #74, #166, #173, #175, and #220 because their stale `CHANGES_REQUESTED` verdicts cited coverage-evidence results that are green on the same heads today |

#### Organization review-pipeline congestion record

Between 2026-08-26T02:44Z and 2026-08-26T03:35Z the organization-wide Actions queue exhibited a systemic backlog: scheduler, OpenCode-review-dispatch, Noema, and Strix runs across `.github`, `naruon`, `pg-erd-cloud`, and OriginWeave sat `queued`/`pending` while only single-digit runs were `in_progress`. This delays every current-head AI review and therefore every ruleset-gated merge. It is an infrastructure-capacity signal, not a code defect, and it does not authorize merging without current-head review evidence.

Representative active workstreams at this snapshot were:

| Workstream | Representative active PR evidence | Delivery boundary |
|---|---|---|
| Product baseline | (merged: #196 on 2026-08-24) | Baseline publication reached protected `main`; this document is its successor snapshot |
| Presentation identity | #229 at `fb868589d065c2cea0b9c8c0f5e655a89f42bee6` onto `542ca1e9c0a863595b8b6697790005d2471f5413` | Ready/non-draft local privacy kernel; current required checks include a failed Strix run, and the PR remains blocked without counted approval; no protected-main shipment is claimed |
| Enterprise approval authority | #220 at `b11db2be68f9b6d71aa4c4290b97a8b22097b353` onto protected `main` `542ca1e9c0a863595b8b6697790005d2471f5413` | Ready/non-draft bounded maker-checker approval lifecycle on the exact `ApprovalScope`; current checks and review state require a fresh exact-head audit, with no counted approval |
| Release artifact identity | #218 and #219 | Ready/non-draft fail-closed benchmark release decision and canonical release manifest binding; Strix provider-failure reruns completed green on both heads |
| Schema-bound extraction and BAP lifecycle | #209 and #208 | Ready/non-draft schema-bound extraction contract and resumable task-lifecycle kernel; #209 Strix rerun green, #208 rerun re-dispatched after a further provider failure |
| WebDriver BiDi transport | #188 through #205 | Active stack whose top #205 merged into its prerequisite branch, not protected `main`; it exercises framed `locateNodes` exchange over a bounded WebSocket opening path, but authenticated browser-process provenance, semantic task execution, and protected-main shipment remain unproven |
| MCP adapter | (#168 and #170 merged) | Typed MCP routing and conservative `tools/list` cache metadata are protected-main behavior; the complete MCP transport, OAuth, browser I/O, and persistence adapter remains planned |
| Workflow-registry audit | #124 | Real Strix finding vuln-0001 (Unicode homoglyph path confusion, MEDIUM) remediated on head `30cc458b` with regression contract tests; fresh exact-head checks and review re-running |
| Controlled Chromium and recovery | #65, #70, #72-#73, #100, #105, #142-#152 and descendants | Real pinned-browser fixture, semantic location, resource, crash, and teardown evidence exists on active stacks; #71 is merged only into the #70 feature branch, and evidence does not transfer across heads or prerequisites |
| Durable WARC/PROV evidence | #210, #217, #239 | Bounded WARC resource records, PROV JSON-LD binding, and retention-lifecycle boundaries are active-PR foundations; durable ownership, replay, retention/deletion, and browser side-effect reconciliation remain open |
| Manifest V3 and native messaging | #27, #43 governance remediation, and the extension/native-host stack including merged #154 and active #169 | Compatibility and Agent-authority isolation remain incomplete until exact release artifacts and platform matrices are proven; #43's sandbox workflow mutation is now owner-authorized under issue #212 option (b) |
| Sensitive-data and model route policy | #10 and its active policy stacks | Deterministic policy values exist, but trusted broker execution, retention/deletion, runtime isolation, and auditable product workflows remain open |
| VPN/profile intent | #149 | Bounded WireGuard/IKEv2 profile authority reconciled with main (`54f96008`); it does not create a tunnel, route, DNS state, authenticated gateway, or connectivity proof |

PR #205 head `f427aa69151987d7e3369bd96d5739ea38d0f7ad` merged as `6c5ef5e2079d54c617183ecfa757e406f48f0aea` into stacked prerequisite branch `feat/webdriver-bidi-websocket-frame-transport` at base `c1bc7e78f3a9debf4f517fb6b5f11dd67be4ad92`. Its successful exact-head checks are stacked-branch integration evidence only; the current protected `main` is `542ca1e9c0a863595b8b6697790005d2471f5413`.

#### Historical exact-head active PR evidence: 2026-08-26

The following newest slices were re-fetched from GitHub for this snapshot. Their exact base/head pairs are recorded so later checks, reviews, and restacks cannot be confused with predecessor evidence:

| PR | State | Exact base head | Exact head |
|---|---|---|---|
| #220 | Ready | `b05d5acca82b9d916ada2c8e82f59f92a89817e1` | `e0740a6f3a41067a4460249378e0266815018a74` |
| #219 | Ready | `b05d5acca82b9d916ada2c8e82f59f92a89817e1` | `3e34a54ae279686a28309d59b8b3b9bfbd283a80` |
| #218 | Ready | `b05d5acca82b9d916ada2c8e82f59f92a89817e1` | `911ea33d8a5aca7673307bb6fdcad4b450f5c111` |
| #209 | Ready | `b05d5acca82b9d916ada2c8e82f59f92a89817e1` | `b35d739017aa5d361b605be48045be50b5a35f6f` |
| #208 | Ready | `b05d5acca82b9d916ada2c8e82f59f92a89817e1` | `e41d3be4c290c4e434aac33d777e511dfb94e03d` |
| #124 | Ready | `b05d5acca82b9d916ada2c8e82f59f92a89817e1` | `296ad25bb541023dbc869ae07ae1d853820f83a4` |

These rows are delivery evidence only. None has counted independent approval in the current collaborator inventory, and predecessor rows from earlier snapshots are retained below as regression anchors that must never be promoted to current-head evidence.

#### Historical regression-anchor exact-head evidence: superseded 2026-08-24 rows

The following rows were current on 2026-08-24 and are retained only as regression anchors; every listed head has since been superseded or merged and must never be promoted to current-head evidence:

| PR | State | Exact base head | Exact head |
|---|---|---|---|
| #222 | Draft | `56fcfa56525e4f2e980e0ee05b6776d621bcddc5` | `1e2ce3d4071a1a75ee891bdcd71c506b3b50d4bc` |
| #221 | Draft | `8145d40f1b028a8f4dc7e7da47ac89bb9e5bb2c7` | `6f339df1e5b3ddb265f4ddd7b262d4de1e0b5e1f` |
| #220 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `ed4cab16cf88c76ce1c145a22d0a274ef2d57263` |
| #219 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `8145d40f1b028a8f4dc7e7da47ac89bb9e5bb2c7` |
| #218 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `49e98fba6974219b3bb0336c822b12667f1e1c03` |
| #216 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `75130851a0f7ce528a7a36382eb026ac7942a0aa` |
| #214 | Draft | `40d642d5470a7753b8211907c190367f742f2f12` | `f79999681866ecf0e5fe17d895170f3f6cae7361` |
| #211 | Draft | `85cc477688246900697f4cfb91c0c8f1f692934a` | `40d642d5470a7753b8211907c190367f742f2f12` |
| #210 | Draft | `c38b9665774d6b3754e572bed527737b5e179833` | `529d11a3571f6b1834b9baa49ef67eb08f043978` |
| #209 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `c38b9665774d6b3754e572bed527737b5e179833` |
| #208 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `85cc477688246900697f4cfb91c0c8f1f692934a` |

The stack topology shows #209 → #210 → #217 → #222 (WARC/PROV chain, with #217 merged into #210's unprotected branch), #208 → #211 → #214 (BAP chain), and #218 → #221 → #220 (release/enterprise chain) at this snapshot. Every active row above remains PR evidence; none is protected-main behavior.

### Required-check provider failure record

On 2026-08-23 the required Strix security scan failed closed on exact heads of #220 (`ed4cab16…`), #218 (`49e98fba…`), and #208 (`85cc4776…`) because its LLM provider/backend was unavailable (rate limit, token cap, connection, warm-up, or model-behavior failure); no vulnerability report artifact was produced, so the workflow correctly refused to convert an incomplete scan into passing security evidence. Failed jobs were re-dispatched on the unchanged exact heads on 2026-08-24 and again on 2026-08-26. This is a provider-infrastructure failure record, not a weakening of the fail-closed gate or a substitute for a completed authoritative scan.

On 2026-08-26 rerun outcomes were verified per run: completed reruns returned `success` on the heads of #46, #48, #156, #157, #159, #218, and #219; several earlier runs for #37, #43, and #149 were cancelled only because conflict-reconciliation pushes created newer heads with fresh scans; remaining reruns were still in flight at snapshot time. One rerun (#124) produced a real MEDIUM finding (vuln-0001) instead of provider noise; that finding was remediated on the branch head rather than suppressed, preserving the fail-closed contract.

#### #195/#198 WebDriver BiDi opening path status

Phase 1 is **in progress**, not shipped. #195 and #198 provide bounded WebSocket opening-path evidence on active branches; framed BiDi commands, authenticated browser-process provenance, semantic task execution, and protected-main integration remain open.

#### #149 VPN/profile intent status

PR #149 is a ready (non-draft) pull request whose conflict reconciliation and rustfmt correction landed on head `54f96008` on 2026-08-26; it still only describes bounded WireGuard/IKEv2 profile authority and does not create a tunnel, route, DNS state, authenticated gateway, or connectivity proof.

The current queue must be processed in dependency order. A green child branch cannot substitute for current checks and review on its prerequisite, synthetic merge, or eventual protected-main commit. PRs that only duplicate, supersede, or preserve stale branch topology should be closed with explicit replacement evidence rather than retained indefinitely; this loop exercised that policy by closing superseded #153 with replacement evidence.

### Review and merge authority

The active `CWL Central required workflows` ruleset (re-fetched for this snapshot) requires one approving review, resolved review threads, no last-push approval requirement, `merge`/`squash` merge methods, and seven configured required workflows (`close-empty-pr`, `opencode-review`, `pr-review-merge-scheduler`, `security-scan`, `strix`, `sast-semgrep`, and `noema-review`). The current collaborator inventory contains only `seonghobae` with administration and push permissions, creating a **reviewer-provisioning gap** for counted non-author approval.

This gap does not authorize self-approval, stale-head merges, administrative bypass, or weaker checks. Because the current GitHub ruleset independently requires a counted approval, the solo-maintainer hold does not satisfy the live merge gate: an eligible non-author collaborator must submit a formal `APPROVED` review on the current head. Until that reviewer-provisioning gap is repaired, protected-main merges stop even when exact-head checks, security gates, complete coverage, rustdoc/Clippy, threads, and AI-review evidence are otherwise complete. Before any merge decision, re-fetch the exact ruleset, collaborators, PR head/base, reviews, unresolved threads, and required checks; do not assume this dated observation remains current.

### Open issues and governance signals

This snapshot contains 11 open issues plus 2 governance signals, for the 13 rows below. Governance signals remain visible because they affect delivery authority but are not counted as product or operational issues.

| Issue or signal | Current gap or signal |
|---|---|
| #28 | First real Chromium Agent Task vertical slice; highest immediate Phase 1 buyer-visible gap |
| #27 | Complete Manifest V3 compatibility and extension-authority isolation matrix |
| #9 | Bounded HTTP/1.1 semantics over the authenticated TLS stream |
| #10 | Purpose-bound operational PII disclosure and trusted broker/storage lifecycle |
| #123 | Fleet incident: disable orphaned TLS, HTTP, and one-shot workflow identities |
| #187 | Manual-authority review of the coverage-diagnostics workflow delta |
| #212 | Governance: remove or independently authorize the PR #43 MV3 workflow mutation — **option (b) executed 2026-08-26** with owner-directed authorization recorded on the issue and the mutation restored on the reconciled branch; re-evaluate if the authorization record is contested |
| #215 | Governance: restore an enforceable protected-main policy that does not create a routine admin bypass |
| #199 | Schema-bound extraction with durable WARC/PROV replay, retention, deletion, and offline verification |
| #200 | Stable BAP/MCP runtime API with authenticated, idempotent, cancellable, resumable task lifecycle |
| #201 | Signed cross-platform Chromium distribution, installer/updater, patch SLA, rollback, SBOM, and provenance |
| #202 | Enterprise control and experience plane: operator UI, Keyverse-compatible identity, tenancy, approval, audit, SLO, Figma, and Storybook |
| #203 | Release-grade web-agent benchmark and commercial acceptance gate bound to exact signed artifacts |

Issue #206 (harden-runner custom detection initialization failure) was closed after its remediation landed on protected `main` between snapshots.

The five newly separated product-completion tracks are **durable WARC/PROV replay**, **stable BAP/MCP runtime API**, **signed cross-platform Chromium distribution**, **enterprise control and experience plane**, and the **commercial acceptance gate**. They are separate issues because each has a distinct authority, data, release, and buyer-acceptance boundary.

The hourly product-development loop is operational infrastructure, not proof that a browser product, issue, pull request, or release meets buyer acceptance.

## Buyer-visible and technical gap matrix

| Priority | Buyer-visible outcome | Protected-main status | Completion issue and acceptance evidence |
|---|---|---|---|
| P0 | A bounded task observes a real Chromium page, performs one typed action, verifies the post-condition, and emits provenance | **Open / Phase 1** | #28; repeated real Chromium E2E with isolated context, exact session/node authority, typed dispatch, post-condition, crash cleanup, and protected-main checks |
| P0 | Navigation consumes approved origin, resolution, route, TCP peer, TLS identity, bounded HTTP, redirect, MIME, and download policy | **Partial foundation** | #9 plus #28; real browser-network adapter proves the governed path is consumed end to end |
| P1 | Existing Chromium extensions remain compatible while Agent authority stays separate | **Partial active-PR evidence** | #27; exact supported-build/platform compatibility matrix, managed allow-list, native-host isolation, repeatability, and release binding |
| P1 | Authorized work can use necessary PII without ambient exposure | **Policy foundation; runtime open** | #10; opaque broker, exact field/purpose/destination/model policy, atomic use/revocation, retention/deletion, and value-free telemetry |
| P1 | Every released structured field is traceable to replayable source evidence | **Foundations only** | #199; durable WARC/PROV replay, integrity, retention, deletion, offline verification, extraction precision/recall, and 100% provenance completeness |
| P1 | External Agents integrate through a stable, authenticated product contract | **Partial active-PR MCP primitives** | #200; BAP 1.0, MCP 2026-07-28 adapter, idempotency, task cancellation/resume, checkpoint/reconciliation, and SDK conformance |
| P1 | Buyers can install, update, verify, and roll back a supported product | **Not shipped** | #201; signed Windows/macOS/Linux/headless artifacts, Chromium revision manifest, updater security, patch SLA, SBOM, SLSA provenance, and recovery |
| P1 | Enterprise teams can provision, approve, audit, operate, and recover the service | **Not shipped** | #202; Keyverse-compatible OIDC/SCIM, tenant isolation, policy/approval/evidence UI, SLO/incident controls, data residency, CSAP/SOC 2 evidence mapping, WCAG 2.2, Figma File ID, and Storybook |
| P0 | A release has reproducible proof of usefulness, safety, evidence completeness, and recovery | **No product-wide release gate** | #203; deterministic, compatibility, adversarial, recovery, and enterprise suites with statistical reporting and an exact-artifact commercial acceptance gate |
| P0 | Valid changes reach protected `main` without authority improvisation or unbounded stack growth | **Blocked / high integration debt** | Shrink the open-PR queue in dependency order, provision legitimate review authority, require exact-current evidence, and close duplicates/superseded branches |

## Commercial completion definition

OriginWeave is not complete merely because every low-level primitive exists in some open branch. A release candidate is commercially complete only when all of the following are true for the declared support profile:

1. #9, #10, #27, and #28 are integrated on protected `main` as a complete browser/network/action/evidence chain.
2. #199 provides replayable, retention-governed evidence for every released structured result.
3. #200 exposes a stable authenticated runtime API and task lifecycle without raw Chromium authority leakage.
4. #201 produces signed, updateable, rollback-capable release artifacts bound to Chromium, SBOM, and provenance.
5. #202 supplies tenant-safe enterprise administration, approvals, audit, SLOs, incident recovery, accessible Figma/Storybook-backed UX, and control evidence.
6. #203 accepts the exact signed artifacts through a reproducible benchmark; missing or inconclusive evidence cannot be promoted to success.
7. Production function, line, region, and branch coverage and public API documentation remain exactly complete for OriginWeave-owned code.
8. CHANGELOG, version, supported-platform matrix, security policy, runbooks, licensing, release notes, upgrade/rollback guidance, and procurement evidence match the exact release.
9. No required check, browser/platform lane, security case, benchmark case, or independent review is skipped, stale, inherited, or represented by status-only evidence.
10. The open PR queue is reduced to bounded active work rather than being the only place where the product exists.

## Historical next executable queue

1. Drain the merge gate in dependency order: for every ready root PR whose current head is check-green with resolved threads, obtain the current ruleset's counted `APPROVED` review from an eligible non-author collaborator; OpenCode approval or skip evidence does not substitute for that GitHub review. If no eligible approver exists, record the reviewer-provisioning gap and do not merge. Root candidates include #37, #40, #43, #45–#48, #51, #62–#65, #74, #82, #124, #149, #152, #156–#166, #173, #175, #208, #209, #211, #218, #219, #229, #237, #238, and #239 as their current checks land. Treat dependent children separately: only after a predecessor reaches protected `main`, retarget and independently revalidate its immediate child; preserve orders such as #218 → #221 → #220 rather than treating #208–#220 as a flat merge range.
2. Keep the organization review pipeline healthy: monitor the central Actions backlog recorded above; if OpenCode reviews stop landing on OriginWeave heads while the queue is idle, repair `ContextualWisdomLab/.github` dispatch/concurrency configuration rather than weakening any gate.
3. Finish the #9/#28 browser-network and Chromium vertical slice, including the #181–#205 WebSocket opening path and framed BiDi command/response stack, then semantic observation, policy, action, post-condition, and recovery boundaries on protected `main`.
4. Finish #27 and #10 as separate security tracks; neither should be hidden inside the first browser PR.
5. Implement #199, then #200, so durable evidence and stable task authority precede broad enterprise integrations.
6. Implement #201 before making release/support claims; exact CI browser evidence must be bound to the actual signed artifact.
7. Design #202 in Figma, record the Figma File ID in the ADR, implement reusable design tokens and Storybook components, then add identity/tenant/approval/audit/operations integration.
8. Make #203 the final release gate across the exact signed distribution, not a source branch or model narrative.
9. Only after the commercial acceptance gate passes, increment the version, finalize CHANGELOG/release notes, publish signed artifacts, and verify upgrade/rollback from the prior supported release.

## Evidence commands

The volatile counts above are reproducible by paginating the complete open-PR and open-issue inventories, excluding pull requests from the issue count, flattening every page, and then inspecting each PR's exact head, checks, reviews, and review threads:

```bash
set -euo pipefail
EVIDENCE_DIR="$(mktemp -d /tmp/originweave-evidence.XXXXXX)"
printf 'Evidence directory: %s\n' "$EVIDENCE_DIR" >&2

gh api --paginate --slurp 'repos/ContextualWisdomLab/OriginWeave/pulls?state=open&per_page=100' \
  > "$EVIDENCE_DIR/open-pr-pages.json"
jq '[.[][]]' "$EVIDENCE_DIR/open-pr-pages.json" \
  > "$EVIDENCE_DIR/open-prs.json"
jq '{
  open_pull_requests: length,
  non_draft: (map(select(.draft == false)) | length),
  draft: (map(select(.draft == true)) | length)
}' "$EVIDENCE_DIR/open-prs.json"

gh api --paginate --slurp 'repos/ContextualWisdomLab/OriginWeave/issues?state=open&per_page=100' \
  > "$EVIDENCE_DIR/open-issue-pages.json"
jq '[.[][]] | map(select(has("pull_request") | not)) | {
  open_non_pr_issues: length
}' "$EVIDENCE_DIR/open-issue-pages.json"

gh api 'repos/ContextualWisdomLab/OriginWeave/branches/main' \
  > "$EVIDENCE_DIR/main-branch.json"
gh api --paginate --slurp \
  'repos/ContextualWisdomLab/OriginWeave/rules/branches/main?per_page=100' \
  > "$EVIDENCE_DIR/main-branch-rule-pages.json"
jq '[.[][]]' "$EVIDENCE_DIR/main-branch-rule-pages.json" \
  > "$EVIDENCE_DIR/main-branch-rules.json"
gh api --paginate --slurp \
  'repos/ContextualWisdomLab/OriginWeave/collaborators?affiliation=all&per_page=100' \
  > "$EVIDENCE_DIR/collaborator-pages.json"
jq '[.[][]]' "$EVIDENCE_DIR/collaborator-pages.json" \
  > "$EVIDENCE_DIR/collaborators.json"

jq -r '.[].number' "$EVIDENCE_DIR/open-prs.json" | while read -r PR; do
  STABLE_HEAD=false
  for ATTEMPT in 1 2 3; do
    VERDICT_PATH="$EVIDENCE_DIR/pr-${PR}-merge-verdict.json"
    VERDICT_TMP="$EVIDENCE_DIR/pr-${PR}-merge-verdict.json.tmp"
    rm -f "$VERDICT_PATH" "$VERDICT_TMP" "$EVIDENCE_DIR/pr-${PR}-rechecked.json"
    PR_JSON="$EVIDENCE_DIR/pr-${PR}.json"
    gh api "repos/ContextualWisdomLab/OriginWeave/pulls/$PR" > "$PR_JSON"
    HEAD_SHA=$(jq -r '.head.sha' "$PR_JSON")
    BASE_SHA=$(jq -r '.base.sha' "$PR_JSON")

    gh api --paginate --slurp \
      "repos/ContextualWisdomLab/OriginWeave/commits/$HEAD_SHA/check-runs?per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-check-runs.json"
    gh api --paginate --slurp \
      "repos/ContextualWisdomLab/OriginWeave/commits/$HEAD_SHA/statuses?per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-statuses.json"
    gh api --paginate --slurp \
      "repos/ContextualWisdomLab/OriginWeave/pulls/$PR/reviews?per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-reviews.json"
    gh api --paginate --slurp \
      "repos/ContextualWisdomLab/OriginWeave/actions/runs?head_sha=$HEAD_SHA&per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-workflow-runs.json"
    gh api graphql --paginate --slurp \
      -F owner=ContextualWisdomLab \
      -F name=OriginWeave \
      -F number="$PR" \
      -f query='
query($owner: String!, $name: String!, $number: Int!, $endCursor: String) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      reviewThreads(first: 100, after: $endCursor) {
        nodes { id isResolved isOutdated }
        pageInfo { hasNextPage endCursor }
      }
    }
  }
}' > "$EVIDENCE_DIR/pr-${PR}-review-threads.json"

    jq -n \
      --arg head "$HEAD_SHA" \
      --slurpfile pr "$PR_JSON" \
      --slurpfile checks "$EVIDENCE_DIR/pr-${PR}-check-runs.json" \
      --slurpfile statuses "$EVIDENCE_DIR/pr-${PR}-statuses.json" \
      --slurpfile reviews "$EVIDENCE_DIR/pr-${PR}-reviews.json" \
      --slurpfile workflow_runs "$EVIDENCE_DIR/pr-${PR}-workflow-runs.json" \
      --slurpfile rules "$EVIDENCE_DIR/main-branch-rules.json" \
      --slurpfile collaborators "$EVIDENCE_DIR/collaborators.json" \
      --slurpfile threads "$EVIDENCE_DIR/pr-${PR}-review-threads.json" \
      --arg base "$BASE_SHA" \
      '(
        [
          $rules[][]?
          | select(.type == "pull_request")
          | .parameters
        ] | first // {}
      ) as $pull_request_parameters
      | (
          [
            $reviews[][][]?
            | {reviewer: .user.login, state, submitted_at, commit_id}
            | select(.submitted_at != null)
            | select(.reviewer != $pr[0].user.login)
            | select(.reviewer as $reviewer |
                any($collaborators[][]?;
                  .login == $reviewer and
                  (.permissions.push == true or
                   .permissions.maintain == true or
                   .permissions.admin == true)))
          ]
          | group_by(.reviewer)
          | map(sort_by(.submitted_at) | last)
          | map(select(.state == "APPROVED" and .commit_id == $head))
        ) as $current_approvals
      | ($pull_request_parameters.required_approving_review_count // 0) as $required_review_count
      | ($pull_request_parameters.require_last_push_approval // false) as $require_last_push_approval
      | {
          head_sha: $head,
          base_sha: $base,
          required_status_checks: {
            check_runs: [$checks[][].check_runs[]?],
            legacy_statuses: [$statuses[][][]?]
          },
          workflow_runs: [$workflow_runs[][].workflow_runs[]?],
          counted_approvals: ($current_approvals | length),
          required_approving_review_count: $required_review_count,
          require_last_push_approval: $require_last_push_approval,
          last_push_approval_authority: (
            if $require_last_push_approval == true
            then "github_rule_evaluation_required"
            else "not_required"
            end
          ),
          approval_gate_satisfied: (
            if $pull_request_parameters.require_last_push_approval == true then false
            else (($current_approvals | length) >= $required_review_count)
            end
          ),
          required_workflows: [
            $rules[][]?
            | select(.type == "workflows")
            | .parameters.workflows[]
          ],
          unresolved_threads: [
            $threads[][].data.repository.pullRequest.reviewThreads.nodes[]?
            | select(.isResolved == false and .isOutdated == false)
          ]
        }' > "$VERDICT_TMP"

    RECHECKED_PR_JSON="$EVIDENCE_DIR/pr-${PR}-rechecked.json"
    RECHECKED_HEAD_SHA=$(gh api "repos/ContextualWisdomLab/OriginWeave/pulls/$PR" \
      | tee "$RECHECKED_PR_JSON" \
      | jq -r '.head.sha')
    RECHECKED_BASE_SHA=$(jq -r '.base.sha' "$RECHECKED_PR_JSON")
    if [[ "$RECHECKED_HEAD_SHA" == "$HEAD_SHA" && "$RECHECKED_BASE_SHA" == "$BASE_SHA" ]]; then
      mv "$VERDICT_TMP" "$VERDICT_PATH"
      mv "$RECHECKED_PR_JSON" "$PR_JSON"
      STABLE_HEAD=true
      break
    fi
    rm -f "$VERDICT_TMP" "$RECHECKED_PR_JSON"
    printf 'Discarding moving head/base evidence for PR #%s (head %s -> %s, base %s -> %s) and retrying.\n' \
      "$PR" "$HEAD_SHA" "$RECHECKED_HEAD_SHA" "$BASE_SHA" "$RECHECKED_BASE_SHA" >&2
  done
  if [[ "$STABLE_HEAD" != true ]]; then
    rm -f "$EVIDENCE_DIR"/pr-${PR}-*.json
    printf 'Unable to collect stable exact-head/base evidence for PR #%s after 3 attempts.\n' "$PR" >&2
    exit 1
  fi
done
```

The branch-scoped rules response determines the active rules affecting `main`; each PR's exact `HEAD_SHA` then determines which check runs, legacy statuses, workflow runs, reviews, and unresolved threads are current. The saved merge verdict binds counted approvals to the latest review per eligible collaborator, excludes the PR author, and requires `APPROVED` on the exact head. It deliberately does **not** infer GitHub's actual last-push actor from commit author or committer metadata: when `require_last_push_approval` is active, this portable evidence procedure records `github_rule_evaluation_required` and keeps `approval_gate_satisfied` false until GitHub's authoritative rule evaluation is consulted. The saved PR JSON also preserves the exact base reference and branch ancestry input for the dependency graph. Evidence is retained only when both `RECHECKED_HEAD_SHA` and `RECHECKED_BASE_SHA` match the collected values; a moving head or base discards the temporary verdict, and three failed attempts leave no unstable merge verdict.

For standards and binding architecture, use [`doctoring.md`](doctoring.md), [`doctoring/browser-agent-protocols.md`](doctoring/browser-agent-protocols.md), [`PRD.md`](PRD.md), [`TRD.md`](TRD.md), [`product-roadmap.md`](product-roadmap.md), and linked ADR/UML/ERD/traceability records. Issues #199-#203 contain their own APA 7th standards and research traceability. This baseline intentionally records delivery state and never promotes planned adapters or active pull-request code to implemented behavior.
