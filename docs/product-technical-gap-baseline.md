# Product and Technical Gap Baseline

This is a dated delivery baseline, not a substitute for the PRD, TRD, roadmap, architecture decisions, or live GitHub state. It keeps buyer-visible gaps, current issues, active pull-request evidence, and commercial completion tracks in one discoverable place. Protected `main` is the implementation boundary: code in an open pull request is not shipped behavior.

## Observed snapshot: 2026-08-26

### Protected-main truth

- Protected `main` is at `b05d5acca82b9d916ada2c8e82f59f92a89817e1` for this snapshot. Since the 2026-08-24 observation (`0841d2ab`), protected `main` absorbed #196 (dated gap baseline publication), #216 (RFC 3986 evidence-path syntax enforcement), #194 (branch-coverage nightly and toolchain tracking refresh), #168 (typed MCP stateless tool-routing foundations), and #151 (exact crash-root termination before crash credit).
- Phase 0 remains complete as a reusable safety-kernel foundation: typed policy contracts, destination classification, direct TCP peer verification, TLS service identity, evidence bounds, resource mitigation, document-node authority, and protected-main tests.
- Phase 1 is **in progress**, not shipped. The first real Chromium vertical slice still needs the active WebDriver BiDi transport stack to reach protected `main`, then compose isolated Chromium launch, session/context identity, semantic observation, typed action authorization, native browser input, post-condition proof, evidence, cancellation, crash recovery, and profile/process teardown.
- HTTP/1.1 bounds, downloads/MIME, proxy/PAC consumption, full browser-network integration, the sensitive-data broker runtime, durable WARC/PROV capture, persistent task/API surfaces, signed cross-platform distribution, enterprise administration, and release-grade buyer acceptance remain open.
- Active pull requests remain evidence, not shipped behavior. Successful checks on a feature or stacked branch do not prove that protected `main` contains the capability or that a child can merge before its prerequisite.

### Open pull requests

The live repository contained **153 open pull requests: 39 non-draft and 114 draft** when this snapshot re-paginated the complete open inventory, a net reduction from 158/44/114 on 2026-08-24 amid merged work, supersession closure of #153, and newly opened follow-on work. The volume and stack depth remain themselves a product-delivery risk: review, exact-head checks, dependency order, and integration truth can drift faster than a buyer-visible vertical slice reaches protected `main`.

#### 2026-08-26 maintenance-loop record

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
| Presentation identity | #229 at `585a7d5545b13f18d76f79100ff4d47ac423e861` onto `b05d5acca82b9d916ada2c8e82f59f92a89817e1` | Ready/non-draft local privacy kernel; all observed exact-head checks except Strix passed, but the PR remains blocked and review-required, and no Chromium adapter or protected-main shipment is claimed |
| Enterprise approval authority | #220 | Ready/non-draft bounded maker-checker approval lifecycle on the exact `ApprovalScope`; all current-head checks green at snapshot, awaiting current-head review evidence |
| Release artifact identity | #218 and #219 | Ready/non-draft fail-closed benchmark release decision and canonical release manifest binding; Strix provider-failure reruns completed green on both heads |
| Schema-bound extraction and BAP lifecycle | #209 and #208 | Ready/non-draft schema-bound extraction contract and resumable task-lifecycle kernel; #209 Strix rerun green, #208 rerun re-dispatched after a further provider failure |
| WebDriver BiDi transport | #188 through #205 | Active stack whose top #205 merged into its prerequisite branch, not protected `main`; it exercises framed `locateNodes` exchange over a bounded WebSocket opening path, but authenticated browser-process provenance, semantic task execution, and protected-main shipment remain unproven |
| MCP adapter | (#168 merged) and #170 | Typed MCP routing foundations are protected-main behavior since 2026-08-24; conservative `tools/list` cache metadata remains active-PR evidence with a Strix rerun in flight |
| Workflow-registry audit | #124 | Real Strix finding vuln-0001 (Unicode homoglyph path confusion, MEDIUM) remediated on head `30cc458b` with regression contract tests; fresh exact-head checks and review re-running |
| Controlled Chromium and recovery | #65, #70-#73, #100, #105, #142-#152 and descendants | Real pinned-browser fixture, semantic location, resource, crash, and teardown evidence exists on active stacks; evidence does not transfer across heads or prerequisites |
| Durable WARC/PROV evidence | #210, #217 | Bounded WARC resource records and PROV JSON-LD binding are draft active-PR foundations; durable ownership, replay, retention/deletion, and browser side-effect reconciliation remain open |
| Manifest V3 and native messaging | #27, #43 governance remediation, and the extension/native-host stack including #154 and #169 | Compatibility and Agent-authority isolation remain incomplete until exact release artifacts and platform matrices are proven; #43's sandbox workflow mutation is now owner-authorized under issue #212 option (b) |
| Sensitive-data and model route policy | #10 and its active policy stacks | Deterministic policy values exist, but trusted broker execution, retention/deletion, runtime isolation, and auditable product workflows remain open |
| VPN/profile intent | #149 | Bounded WireGuard/IKEv2 profile authority reconciled with main (`54f96008`); it does not create a tunnel, route, DNS state, authenticated gateway, or connectivity proof |

PR #205 head `f427aa69151987d7e3369bd96d5739ea38d0f7ad` merged as `6c5ef5e2079d54c617183ecfa757e406f48f0aea` into stacked prerequisite branch `feat/webdriver-bidi-websocket-frame-transport` at base `c1bc7e78f3a9debf4f517fb6b5f11dd67be4ad92`. Its successful exact-head checks are stacked-branch integration evidence only; protected `main` remains `b05d5acca82b9d916ada2c8e82f59f92a89817e1`.

#### Current exact-head active PR evidence

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

#### Regression-anchor exact-head evidence: superseded 2026-08-24 rows

The following rows were current on 2026-08-24 and are retained only as regression anchors; every listed head has since been superseded or merged and must never be promoted to current-head evidence:

| PR | State | Exact base head | Exact head |
|---|---|---|---|
| #222 | Draft | `56fcfa56525e4f2e980e0ee05b6776d621bcddc5` | `1e2ce3d4071a1a75ee891bdcd71c506b3b50d4bc` |
| #221 | Draft | `8145d40f1b028a8f4dc7e7da47ac89bb9e5bb2c7` | `6f339df1e5b3ddb265f4ddd7b262d4de1e0b5e1f` |
| #220 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `ed4cab16cf88c76ce1c145a22d0a274ef2d57263` |
| #219 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `8145d40f1b028a8f4dc7e7da47ac89bb9e5bb2c7` |
| #218 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `49e98fba6974219b3bb0336c822b12667f1e1c03` |
| #217 | Draft | `529d11a3571f6b1834b9baa49ef67eb08f043978` | `56fcfa56525e4f2e980e0ee05b6776d621bcddc5` |
| #216 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `75130851a0f7ce528a7a36382eb026ac7942a0aa` |
| #214 | Draft | `40d642d5470a7753b8211907c190367f742f2f12` | `f79999681866ecf0e5fe17d895170f3f6cae7361` |
| #211 | Draft | `85cc477688246900697f4cfb91c0c8f1f692934a` | `40d642d5470a7753b8211907c190367f742f2f12` |
| #210 | Draft | `c38b9665774d6b3754e572bed527737b5e179833` | `529d11a3571f6b1834b9baa49ef67eb08f043978` |
| #209 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `c38b9665774d6b3754e572bed527737b5e179833` |
| #208 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | `85cc477688246900697f4cfb91c0c8f1f692934a` |

The stack topology shows #209 → #210 → #217 → #222 (WARC/PROV chain), #208 → #211 → #214 (BAP chain), #218 → #221 → #220 (release/enterprise chain) at this snapshot. Every row above remains active-PR evidence; none is protected-main behavior.

### Required-check provider failure record

On 2026-08-23 the required Strix security scan failed closed on exact heads of #220 (`ed4cab16…`), #218 (`49e98fba…`), and #208 (`85cc4776…`) because its LLM provider/backend was unavailable (rate limit, token cap, connection, warm-up, or model-behavior failure); no vulnerability report artifact was produced, so the workflow correctly refused to convert an incomplete scan into passing security evidence. Failed jobs were re-dispatched on the unchanged exact heads on 2026-08-24 and again on 2026-08-26. This is a provider-infrastructure failure record, not a weakening of the fail-closed gate or a substitute for a completed authoritative scan.

On 2026-08-26 rerun outcomes were verified per run: completed reruns returned `success` on the heads of #46, #48, #156, #157, #159, #218, and #219; several earlier runs for #37, #43, and #149 were cancelled only because conflict-reconciliation pushes created newer heads with fresh scans; remaining reruns were still in flight at snapshot time. One rerun (#124) produced a real MEDIUM finding (vuln-0001) instead of provider noise; that finding was remediated on the branch head rather than suppressed, preserving the fail-closed contract.

#### #195/#198 WebDriver BiDi opening path status

Phase 1 is **in progress**, not shipped. #195 and #198 provide bounded WebSocket opening-path evidence on active branches; framed BiDi commands, authenticated browser-process provenance, semantic task execution, and protected-main integration remain open.

#### #149 VPN/profile intent status

PR #149 is a ready (non-draft) pull request whose conflict reconciliation and rustfmt correction landed on head `54f96008` on 2026-08-26; it still only describes bounded WireGuard/IKEv2 profile authority and does not create a tunnel, route, DNS state, authenticated gateway, or connectivity proof.

The current queue must be processed in dependency order. A green child branch cannot substitute for current checks and review on its prerequisite, synthetic merge, or eventual protected-main commit. PRs that only duplicate, supersede, or preserve stale branch topology should be closed with explicit replacement evidence rather than retained indefinitely; this loop exercised that policy by closing superseded #153 with replacement evidence.

### Review and merge authority

The active `CWL Central required workflows` ruleset (re-fetched for this snapshot) requires one approving review, resolved review threads, no last-push approval requirement, `merge`/`squash` merge methods, and seven configured required workflows (`close-empty-pr`, `opencode-review`, `pr-review-merge-scheduler`, `security-scan`, `strix`, `sast-semgrep`, `noema-review`). The current collaborator inventory contains only `seonghobae` with administration and push permissions, creating a **reviewer-provisioning gap** for counted non-author approval.

This gap does not authorize self-approval, stale-head merges, administrative bypass, or weaker checks. Because the current GitHub ruleset independently requires a counted approval, the solo-maintainer hold does not satisfy the live merge gate: an eligible non-author collaborator must submit a formal `APPROVED` review on the current head. Until that reviewer-provisioning gap is repaired, protected-main merges stop even when exact-head checks, security gates, complete coverage, rustdoc/Clippy, threads, and AI-review evidence are otherwise complete. Before any merge decision, re-fetch the exact ruleset, collaborators, PR head/base, reviews, unresolved threads, and required checks; do not assume this dated observation remains current.

### Open issues and operational signals

| Issue | Current gap or signal |
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
| P0 | Valid changes reach protected `main` without authority improvisation or unbounded stack growth | **Blocked / high integration debt** | Shrink the 153-PR queue in dependency order, provision legitimate review authority, require exact-current evidence, and close duplicates/superseded branches |

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

1. Drain the merge gate in dependency order: for every ready root PR whose current head is check-green with resolved threads, obtain the current ruleset's counted `APPROVED` review from an eligible non-author collaborator; OpenCode approval or skip evidence does not substitute for that GitHub review. If no eligible approver exists, record the reviewer-provisioning gap and do not merge. Root candidates include #37, #40, #43, #45–#48, #51, #62–#65, #74, #82, #124, #149, #152, #156–#166, #170, #173, #175, #208, #209, #218, and #219 as their re-dispatched checks land. Treat dependent children separately: only after a predecessor reaches protected `main`, retarget and independently revalidate its immediate child; preserve orders such as #218 → #221 → #220 rather than treating #208–#220 as a flat merge range.
2. Keep the organization review pipeline healthy: monitor the central Actions backlog recorded above; if OpenCode reviews stop landing on OriginWeave heads while the queue is idle, repair `ContextualWisdomLab/.github` dispatch/concurrency configuration rather than weakening any gate.
3. Finish the #9/#28 browser-network and Chromium vertical slice, including the #181–#205 WebSocket opening path and framed BiDi command/response stack, then semantic observation, policy, action, post-condition, and recovery boundaries on protected `main`.
4. Finish #27 and #10 as separate security tracks; neither should be hidden inside the first browser PR.
5. Implement #199, then #200, so durable evidence and stable task authority precede broad enterprise integrations.
6. Implement #201 before making release/support claims; exact CI browser evidence must be bound to the actual signed artifact.
7. Design #202 in Figma, record the Figma File ID in the ADR, implement reusable design tokens and Storybook components, then add identity/tenant/approval/audit/operations integration.
8. Make #203 the final release gate across the exact signed distribution, not a source branch or model narrative.
9. Only after the commercial acceptance gate passes, increment the version, finalize CHANGELOG/release notes, publish signed artifacts, and verify upgrade/rollback from the prior supported release.

## Evidence commands

The volatile counts above are reproducible by paginating the complete open-PR inventory, flattening every page, and then inspecting each PR's exact head, checks, reviews, and review threads:

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
