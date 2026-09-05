# Product and Technical Gap Baseline

This is a dated delivery baseline, not a substitute for the PRD, TRD, roadmap, architecture decisions, or live GitHub state. It keeps buyer-visible gaps, current issues, active pull-request evidence, and commercial completion tracks in one discoverable place. Protected `main` is the implementation boundary: code in an open pull request is not shipped behavior.

## Current live delivery state

This volatile section is refreshed from live GitHub state and is authoritative only for the exact observations recorded here. The dated snapshot below remains historical evidence and is not promoted to current acceptance evidence. Live GitHub PR/base/head/check APIs are authoritative over PR bodies and prior maintenance prose; a body that still names an older head is stale evidence, not merge evidence.

Observed at (UTC): `2026-09-05T05:08:00Z`.

- Protected `main` is `87c4daa1830bac5a5228b6036752ad5633232085` through #286. PR #286 skips repository-native CI jobs for draft pull requests; a skipped job is not passing evidence.
- Full live search returns **125 open pull requests: 14 Ready/non-draft and 111 Draft; 13 open non-PR issues**. PR #290 is the newly opened Ready workflow-admission child of #245; queue movement is not protected-main delivery.
- Ready roots are #37 `28721edcc67bb5379ffb11a259d746396ac7ae03`, #50 `30d032b64c8eca669fa029a9d9915519cc467e99`, #166 `e84a1a2cc82b1c666218efd441da97849f47b8c2`, #219 `65e4315d80137badc0b55e1b9617015beb1db568`, #220 `e545b94e1de499b96b867694f80ac04ad247becd`, #229 `024f63690cf05cfe6f0d4a430f0e18ea8fd2c4d6`, #240 `24930a3a9ee79c0b712ee3df6589b0592eb6e18f`, #248 `7d6db16b2ead201fcec320854923f90d3ad0d8bc`, #272 `b1cae8ad1cbd8eb6992037c830aea30b9aa436b3`, #274 `802d0bdff7536d9ac253305d3e0237b4e4a1789e`, #285 `f455c2cd64b3dd3f027c91d396103792a205ddd0`, #287 `af83c40dd2990a03064a92ca75430a9cc400f098`, and #290 `ebeefcd534db4324498fdb18046ebc6255ddcdf2`. PR #238's moving exact head is intentionally omitted from its self-referential document; live PR metadata is authoritative. Their hosted exact-head checks remain non-terminal and none has an eligible exact-head approval, so none is merge-ready.
- PR #166 exact head `e84a1a2cc82b1c666218efd441da97849f47b8c2` and PR #220 exact head `e545b94e1de499b96b867694f80ac04ad247becd` retain formal `CHANGES_REQUESTED` decisions even though every current review thread is resolved. The decisions remain merge blockers until the governing reviewer state changes; thread resolution alone is not approval and is not grounds to dismiss a review.
- Active organization ruleset `18156473` (`CWL Central required workflows`) applies to the default branch. It requires one approving review, dismissal of stale approvals after pushes, resolved review threads, extra approval for unattributed changes, and **7 central required workflows**: `opencode-review`, `pr-review-merge-scheduler`, `security-scan`, `strix`, `sast-semgrep`, `noema-review`, and `codeql-pr`. Current `ContextualWisdomLab/.github` consolidates OSV and Scorecard PR scanning into the required `security-scan.yml`; standalone `osv-scanner-pr.yml` and `scorecard-pr.yml` are no longer ruleset entries. Administrative bypass capability is not authorization to use it.
- PR #284 head `61bcf88c960c6c437ccd29b3fbb73cd4325f9e5a` reached `main` through rule-suite `3948421709`, whose live result is **`result: bypass`** with actor `seonghobae`. The required non-author approval and workflows were not proven before integration, and the new main's repository-native post-merge checks remain non-terminal. This is a governance incident tracked by #215, not a policy-compliant merge or evidence that later checks can retroactively authorize it.
- PR #285 is Ready at exact head `f455c2cd64b3dd3f027c91d396103792a205ddd0` on current protected main. It addresses #284's three post-merge review findings without changing `.github/**` and adds a dedicated regression contract. All 154 repository contracts, the full Rust gates, and exact 100% local coverage pass. Its Ready transition materialized repository-native CI `33930234387`, which remains queued, but did not materialize fresh central required-workflow runs: the only exact-head Security Scan `33924016851`, SAST Semgrep `33924016903`, and CodeQL PR `33924016883` runs came from the earlier Draft event and were cancelled. Missing current lifecycle evidence and the absent eligible approval remain merge blockers; toggling Draft or creating a no-op commit is not an acceptable substitute.
- Issue #279 owns the remaining documentation-CI partitioning gap. PR #287 is Ready at exact head `af83c40dd2990a03064a92ca75430a9cc400f098` on current protected main; it is the workflow-independent classifier foundation, changes no `.github/**` path, and is the prerequisite for an authorized workflow owner. Its predecessor failed its own release-record contract; the current head binds Git rename/copy similarity to blob identity, and its focused and full local suites plus exact 100% Rust coverage pass. Hosted exact-head checks remain queued and no eligible approval exists. PR #282 is Draft at externally advanced exact head `b64e0708584beff3fb54acf226cb3e667773e473` on current protected main; its new hosted jobs remain queued or skipped under the Draft policy, so predecessor local evidence is not transferred.
- PR #290 is Ready at exact head `ebeefcd534db4324498fdb18046ebc6255ddcdf2` on #245 exact `a769f484e2c110e0523b3b28cd21573f43867562`. It scopes MV3 pull-request concurrency and draft admission, but its newly queued hosted Rust, pinned-Chromium, and coverage checks remain non-passing evidence until terminal.
- PR #283 is Draft at exact head `c904300a6a1bda83af24f84d586f1c5f6a6491aa`, non-force stacked on #282 exact `b54a5856d8201911f05d69622f0d5594a371adf0`. Its exact parent compare remains one prose-only doctoring canary; draft-hosted jobs are skipped, so no trigger-shape GREEN is claimed for #279. Runner admission is a separate condition.
- PR #288 is Draft at exact head `39e36256651f62940ec3ca6149067f0cfcb2285a` on current protected main. It stages #70's controlled Agent Task product, test, and documentation delta without any `.github/**` mutation; its workflow-owned sandbox-helper assertion was removed after it made the workflow-free branch's full Python contract suite fail, and all 172 Python contracts now pass locally. Hosted CI and MV3 jobs remain skipped while Draft, and #212 still owns authorized workflow activation plus sandboxed pinned-Chromium execution evidence.
- Issue #28 remains the P0 governed-browser integration target. PR #260 is Draft at exact head `3a651967c421f77088fe25e86a63faae295390b3` on #259 exact base `e1105ddf86f6c79443af8b4d306b9d34cb703c17`. PR #261 is Draft at exact head `323ac9e147691e9f6572711f5a748e13f1036624`, stacked on exact #260. Repair PR #277 is Draft at exact head `01038ba71fb276426cc67f90a91a3c431e194db5`, stacked on exact #261 and preserving typed `SessionSubscribe` correlation. Each restacked tree passed 141 Python contracts, the full Rust gates, and CI-equivalent pinned-nightly 100% production coverage. Fresh hosted checks remain non-terminal, so stale conflicting #262 is not superseded and remains dependency-blocked until #277 reaches terminal GREEN on the unchanged head.
- Stacked PRs #178 and #85 were repaired without force pushes at exact heads `640e594dbc0a64f251a0f28b8e80943f53337e40` and `c9adea680d6186f1842a280e248ce55da4f1305b`. Parent-relative deltas now preserve the current MV3 diagnostic and extension-authority contracts; local full suites and exact 100% coverage passed, while hosted exact-head acceptance remains independently required.
- PR #70 is Draft at exact head `77eb0f2ee71783e06171784b7173c0b4cd530e61`; it targets protected main but has not adopted current main `87c4daa1830bac5a5228b6036752ad5633232085`. Its focused Agent Task contract proves the pre-repair `--no-sandbox` launch as source RED and the branch removes that argument only from the Agent Task lane. Sandbox-enabled pinned-Chromium E2E and repository-wide GREEN remain unproven while its exact-head browser/CI/security runs are non-passing.
- DDD/MCP repair #272 is Ready at exact head `b1cae8ad1cbd8eb6992037c830aea30b9aa436b3` on current protected main. Its exact-head hosted checks remain queued and no eligible approval exists. Documentation child PR #273 is Draft at exact head `e5c8fcb66bf644dfa750bb1b40ba3d600cb7805a` on predecessor #272 head `fe124e447cad3f679e22337fb6fbdfd135ab3652`, so it must adopt the parent only after #272's current head completes its gates. Both remain active-PR evidence.
- PR #229 is Ready at exact head `024f63690cf05cfe6f0d4a430f0e18ea8fd2c4d6` on current protected main. Its 157 Python contracts, full Rust gates, and exact 100% local coverage pass, while hosted exact-head checks remain queued and no eligible approval exists. The presentation-identity work retains the boundary that Chromium application and page-observed effectiveness remain separate adapter/browser-E2E work.
- PR #281 remains Draft at exact head `adaca6427d68f550b39293a69b7c733430d1c385` on canonical MV3 parent #43 `6f3134d18d3118aab33d28048671dc71a5f47b77`. Its child delta is diagnostic-evidence contract/doctoring only; inherited #43 implementation redacts browser/page-derived HTTP and WebDriver protocol data, startup exception details, returned capability values, DOM-dataset values, and click post-condition text before CI/audit evidence. Exact-head CI and Manifest V3 Compatibility succeeded and review threads are resolved, but parent-first workflow-ownership verification remains open, so the child stays Draft.
- PR #43 is Draft at exact head `6f3134d18d3118aab33d28048671dc71a5f47b77`; it targets protected main but has not adopted current main `87c4daa1830bac5a5228b6036752ad5633232085`. The branch now installs and configures the pinned `chrome_sandbox`; exact-head hosted verification remains required, and restoring `--no-sandbox` is not acceptable.
- PR #274 remains the bounded GitHub Pages source/README/CHANGELOG public-surface delta at exact head `802d0bdff7536d9ac253305d3e0237b4e4a1789e`, with a repository regression contract for the exact badge target, pre-alpha status, active-PR non-promotion statement, and publication boundary. Source presence is not publication evidence; live HTTPS publication and navigation remain required after the authorized Pages configuration/deployment path.
- PR #37 externally advanced to Ready exact head `28721edcc67bb5379ffb11a259d746396ac7ae03` to add its Content-Length persistence release record. Its predecessor reader returns after the exact declared bytes, already-buffered surplus remains fail-closed, and valid self-delimited content does not require TLS EOF, but that local evidence is predecessor-bound. The new hosted Rust, coverage, and central workflow jobs are queued, and no eligible exact-head approval exists.
- PR #269 is Draft at exact head `7854394266d3f292e779193c01413a34f6798d7c`, stacked on #268 exact `8d4027e40b790d28d866051ba741db12927ec22c`. It adds only a fixed, product-owned `script.callFunction` text-value observation for the exact admitted current node; it performs no browser I/O and proves no post-condition. Its new documentation boundary contract and all 140 Python contracts pass locally, while hosted exact-head CI is queued and the parent-first dependency keeps it Draft.
- PR #270 is Draft at exact head `191a14535219ea8033777fa4c970efb281b62418`, now non-force synchronized onto #269 exact `7854394266d3f292e779193c01413a34f6798d7c`. Its parent-relative transport delta is unchanged; 140 Python contracts, full Rust gates, and exact 100% local production function/line/region/branch coverage pass. Dispatch still proves neither the correlated result nor text-entry success, and fresh hosted exact-head checks plus parent-first integration remain required.
- PR #271 is Draft at exact head `802ec806cdd4560eab48c484f435766ecabda353`, non-force synchronized onto #270 exact `191a14535219ea8033777fa4c970efb281b62418`. It admits only the exact typed observation response, treats unequal text as `PostconditionMismatch`, and retains no page-controlled or expected text in public evidence or diagnostics. Its test-first documentation contract, all 141 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered ancestor integration remain required.
- PR #195 is Draft at exact head `48eb2d23009c1c804520dd5efcd0d4d072aacef1`. Its loopback regressions hold the accepted peer through opening-write timeout cleanup and locally revoked-stream classification, removing macOS close races without weakening production failures. The original affected test passed 50 consecutive focused regression passes, followed by 139 Python contracts, full Rust gates, and exact 100% local production coverage; fresh hosted exact-head checks and prerequisite integration remain required.
- PR #242 is Draft at exact head `55fef0c3fae1724eddada53e52c4a0311f509aa3`, stacked on #195 exact `48eb2d23009c1c804520dd5efcd0d4d072aacef1`. Its zero-deadline regression holds the accepted peer until validation completes, so macOS cannot race socket teardown against the intended invalid-input result. Its 139 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #247 exact head `6407895f4db4bee640074cb9c9d3cbe8b0e9e13a` merged as `b87191bcb6a95dfd7e0ed234e600639a1093c43a` into unprotected parent `feat/webdriver-bidi-text-message-assembly`; this is stack integration, not protected-main delivery. PR #248 externally advanced to Ready exact head `7d6db16b2ead201fcec320854923f90d3ad0d8bc` after scoping its BiDi release contract to the owned record. The prior local full-suite evidence applies only to predecessor `de7754aaeb97ccb0fd47bcbe1c4d99c10eaf84eb`; new hosted checks are queued and must prove the current head independently.
- PR #249 is Draft at exact head `017d6e816f5a86544a63821b3ceaba94d5f17f44`, non-force synchronized onto #248 exact `7d6db16b2ead201fcec320854923f90d3ad0d8bc`. Its correlation-state regression no longer creates uncovered assertion-internal failure regions while still proving local preflight retirement and ambiguous-write retention. The current parent and child trees passed 141 Python contracts, the full Rust gates, and CI-equivalent pinned-nightly 100% production coverage; hosted exact-head checks are queued and ordered parent integration remains required.
- PRs #250 through #257 remain Draft and are again synchronized in order through exact heads #250 `0eab23d5e388c5c8b984c0021a58316680c9ba8b`, #251 `86e8ad76838f2a64aa7e0cd56ba1f931c8d0c3dc`, #252 `2015259529ada99af836989079cc85a15779a2d8`, #253 `0d72082e595c0e1fcc03d609ba337896ed14e2fc`, #254 `cbaf50dcc97753cc73135497ea8225e8b18de190`, #255 `a13de5f9321e72c1867974eb7a43230f031e58df`, #256 `9f2e6f29be46371762e3031a97c1cac04720694f`, and #257 `ea2b5b78868917219c46f1304558b92490a7f6fe`. Each updated tree passed 141 Python contracts, the full Rust gates, and CI-equivalent pinned-nightly 100% production coverage. #250 and #257 each had one known macOS socket-observation race; their exact focused retries and complete coverage reruns passed. Hosted exact-head checks and ordered parent integration remain required.
- PR #258 is Draft at exact head `f2ceabb3ea50b1959e936503c50cae12f3e6e480` on exact #257, and PR #259 is Draft at exact head `e1105ddf86f6c79443af8b4d306b9d34cb703c17` on exact #258. Both final trees passed 141 Python contracts, the full Rust gates, and CI-equivalent pinned-nightly 100% production coverage; fresh hosted exact-head checks and ordered parent integration remain required.
- PR #93 is Draft at exact head `0664f0452cb329cd692cce7f61f9001652abfda2`, based on #271 exact `802ec806cdd4560eab48c484f435766ecabda353` and synchronized with #242's complete opening-fixture cleanup. `SemanticNodeActionBinding` preserves semantic-node and business-origin identity, but does not authorize policy or execute input. Its 141 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered ancestor integration remain required.
- PR #95 is Draft at exact head `97aa0f2e340ee6fd920d0418f97af276b190554f`, stacked on #93 exact `0664f0452cb329cd692cce7f61f9001652abfda2`. Only `Decision::Allow` creates policy-authorized value; other decisions fail closed, and registry-owned current authority returns `NotAdmitted` after document advance removes the node. The slice performs no browser I/O or postcondition proof. Its 142 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #96 is Draft at exact head `b7ba8dd1433410cee43084a73e31816da841b2a2`, stacked on #95 exact `97aa0f2e340ee6fd920d0418f97af276b190554f`. Registry-owned node authority is revalidated before one adapter callback, and the callback is never invoked after admission is removed. Adapter completion remains separate from postcondition proof. Its 143 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #101 is Draft at exact head `bc810b121bb0303f55afa8777a23cc0f9748c1db`, stacked on #96 exact `b7ba8dd1433410cee43084a73e31816da841b2a2`. A known-disabled interactive action fails as `NodeNotEnabled`, while `ScrollIntoView` remains selectable; retained observation state is neither current Chromium proof nor dispatch authority. Its 144 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #102 is Draft at exact head `a123c55d4839dae1db7e6671f7d4d158c7cfd9db`, stacked on #101 exact `bc810b121bb0303f55afa8777a23cc0f9748c1db`. Fresh semantic comparison rejects another node as `ObservationAuthorityMismatch`, removed action support, and newly disabled interactive state, but does not obtain or authenticate the observation or dispatch input. Its 145 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- PR #103 is Draft at exact head `8b3416169346fa04b53c915b813d55ccf47d1876`, stacked on #102 exact `a123c55d4839dae1db7e6671f7d4d158c7cfd9db`. Same-call dispatch checks registry-owned browser authority first and fresh semantic state second; the callback is never invoked on either failure, and completion is not postcondition proof. Its 146 Python contracts, full Rust gates, and exact 100% local production coverage pass; hosted exact-head checks and ordered parent integration remain required.
- Issue #201's release/SBOM lane remains dependency ordered, and GitHub Releases is empty. OriginWeave remains pre-GA until exact protected-head versioning, signed artifacts, SBOM/provenance, reproducibility, rollback, compatibility, security, and commercial-acceptance evidence exist.
- The principal commercial gaps remain #27 Manifest V3/native-host isolation, #9 bounded HTTP/browser-network consumption, #10 purpose-bound protected-data runtime, #28 the first complete Chromium Agent Task vertical slice, #199 durable WARC/PROV and retention/replay, #200 stable BAP/MCP runtime API, #201 signed cross-platform distribution/update/rollback/SBOM/SLSA, #202 enterprise identity/tenant/policy/approval/audit/SLO operations, #203 exact-artifact commercial acceptance, #276 contextual-orchestrator migration, and #279 exact-head documentation verification without unnecessary Rust queue load.
- Current evidence procedure remains fail-closed: queued reviewer evidence is non-passing; paginate the full PR inventory, bind checks/reviews/threads/workflow runs to each unchanged exact head and independently resolved live base, consult the active ruleset, and discard queued, skipped, cancelled, absent, predecessor, synthetic, status-only, and model-only evidence as passing proof.

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

## Next executable queue

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
