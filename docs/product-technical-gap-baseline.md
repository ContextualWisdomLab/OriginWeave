# Product and Technical Gap Baseline

This is a dated delivery baseline, not a substitute for the PRD, TRD, roadmap, architecture decisions, or live GitHub state. It keeps buyer-visible gaps, current issues, active pull-request evidence, and commercial completion tracks in one discoverable place. Protected `main` is the implementation boundary: code in an open pull request is not shipped behavior.

## Observed snapshot: 2026-08-20

### Protected-main truth

- Protected `main` was at `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` when this snapshot was refreshed.
- Phase 0 is documented as complete as a reusable safety-kernel foundation: typed policy contracts, destination classification, direct TCP peer verification, TLS service identity, evidence bounds, resource mitigation, document-node authority, and protected-main tests.
- Phase 1 is **in progress**, not shipped. The first real Chromium vertical slice still needs the active WebDriver BiDi transport stack to reach protected `main`, then compose isolated Chromium launch, session/context identity, semantic observation, typed action authorization, native browser input, post-condition proof, evidence, cancellation, crash recovery, and profile/process teardown.
- HTTP/1.1 bounds, downloads/MIME, proxy/PAC consumption, full browser-network integration, the sensitive-data broker runtime, durable WARC/PROV capture, persistent task/API surfaces, signed cross-platform distribution, enterprise administration, and release-grade buyer acceptance remain open.
- Active pull requests remain evidence, not shipped behavior. Successful checks on a feature or stacked branch do not prove that protected `main` contains the capability or that a child can merge before its prerequisite.

### Open pull requests

The live repository contained **145 open pull requests: 38 non-draft and 107 draft**. The volume and stack depth are themselves a product-delivery risk: review, exact-head checks, dependency order, and integration truth can drift faster than a buyer-visible vertical slice reaches protected `main`.

Representative active workstreams at this snapshot were:

| Workstream | Representative active PR evidence | Delivery boundary |
|---|---|---|
| Product baseline | #196 | Ready/non-draft documentation PR; this refreshed inventory and the completion issues below remain review-gated |
| WebDriver BiDi transport | #188 through #198 | #198, exact head `924f260cac885a8c66c81de1101c1ba183d00e74`, validates the RFC 6455 opening response on top of #195; the stack still does not by itself complete framed BiDi browser commands, authenticated browser-process provenance, semantic task execution, or protected-main shipment |
| MCP adapter | #168 and #170 | Typed MCP routing and conservative `tools/list` metadata are active-PR foundations; complete authenticated transport, durable task lifecycle, cancellation/resume, and browser execution remain open under #200 |
| Controlled Chromium and recovery | #65, #70-#73, #100, #105, #142-#153 and descendants | Real pinned-browser fixture, semantic location, resource, crash, and teardown evidence exists on active stacks; evidence does not transfer across heads or prerequisites |
| Manifest V3 and native messaging | #27 and its active extension/native-host stack, including #154 and #169 | Compatibility and Agent-authority isolation remain incomplete until exact release artifacts and platform matrices are proven |
| Sensitive-data and model route policy | #10 and its active policy stacks | Deterministic policy values exist, but trusted broker execution, retention/deletion, runtime isolation, and auditable product workflows remain open |
| VPN/profile intent | #149 | Bounded WireGuard/IKEv2 profile authority is active-PR evidence; it does not create a tunnel, route, DNS state, authenticated gateway, or connectivity proof |

Draft PR #198 is the current top WebDriver BiDi opening-response slice; its prerequisite #195 owns the bounded opening-request write. It remains draft evidence and cannot be treated as shipped behavior.

The current queue must be processed in dependency order. A green child branch cannot substitute for current checks and review on its prerequisite, synthetic merge, or eventual protected-main commit. PRs that only duplicate, supersede, or preserve stale branch topology should be closed with explicit replacement evidence rather than retained indefinitely.

### Review and merge authority

The active `CWL Central required workflows` ruleset requires one approving review, approval after the last push, resolved review threads, and configured required workflows. The previously observed collaborator inventory contained only `seonghobae` with administration and push permissions, creating a **reviewer-provisioning gap** for counted non-author approval.

This gap does not authorize self-approval, administrative bypass, stale-head merge, or weaker checks. Exact current-head checks, security gates, complete coverage, rustdoc/Clippy, thread resolution, and branch protection remain mandatory. Before any merge decision, re-fetch the exact ruleset, collaborators, PR head/base, reviews, unresolved threads, and required checks; do not assume this dated observation remains current.

### Open issues and operational signals

| Issue | Current gap or signal |
|---|---|
| #28 | First real Chromium Agent Task vertical slice; highest immediate Phase 1 buyer-visible gap |
| #27 | Complete Manifest V3 compatibility and extension-authority isolation matrix |
| #9 | Bounded HTTP/1.1 semantics over the authenticated TLS stream |
| #10 | Purpose-bound operational PII disclosure and trusted broker/storage lifecycle |
| #123 | Fleet incident: disable orphaned TLS, HTTP, and one-shot workflow identities |
| #187 | Manual-authority review of the coverage-diagnostics workflow delta |
| #199 | Schema-bound extraction with durable WARC/PROV replay, retention, deletion, and offline verification |
| #200 | Stable BAP/MCP runtime API with authenticated, idempotent, cancellable, resumable task lifecycle |
| #201 | Signed cross-platform Chromium distribution, installer/updater, patch SLA, rollback, SBOM, and provenance |
| #202 | Enterprise control and experience plane: operator UI, Keyverse-compatible identity, tenancy, approval, audit, SLO, Figma, and Storybook |
| #203 | Release-grade web-agent benchmark and commercial acceptance gate bound to exact signed artifacts |

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
| P0 | Valid changes reach protected `main` without authority improvisation or unbounded stack growth | **Blocked / high integration debt** | Shrink the 145-PR queue in dependency order, provision legitimate review authority, require exact-current evidence, and close duplicates/superseded branches |

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

1. Re-fetch all 145 PRs and compute the dependency graph, exact heads/bases, reviews, unresolved threads, current required checks, duplicate/supersession relationships, and branch ancestry.
2. Integrate merge-ready root PRs first; restack and independently revalidate only the immediate children. Close obsolete alternatives instead of carrying parallel truth.
3. Finish the #9/#28 browser-network and Chromium vertical slice, including the #195/#198 WebSocket opening path and the remaining framed BiDi command/response, semantic observation, policy, action, post-condition, and recovery boundaries.
4. Finish #27 and #10 as separate security tracks; neither should be hidden inside the first browser PR.
5. Implement #199, then #200, so durable evidence and stable task authority precede broad enterprise integrations.
6. Implement #201 before making release/support claims; exact CI browser evidence must be bound to the actual signed artifact.
7. Design #202 in Figma, record the Figma File ID in the ADR, implement reusable design tokens and Storybook components, then add identity/tenant/approval/audit/operations integration.
8. Make #203 the final release gate across the exact signed distribution, not a source branch or model narrative.
9. Only after the commercial acceptance gate passes, increment the version, finalize CHANGELOG/release notes, publish signed artifacts, and verify upgrade/rollback from the prior supported release.

## Evidence commands

The volatile counts above are reproducible by paginating the complete open-PR inventory, flattening every page, and then inspecting each PR's exact head, checks, reviews, and review threads:

```bash
gh api --paginate --slurp 'repos/ContextualWisdomLab/OriginWeave/pulls?state=open&per_page=100' \
  > /tmp/originweave-open-pr-pages.json
jq '[.[][]]' /tmp/originweave-open-pr-pages.json \
  > /tmp/originweave-open-prs.json
jq '{
  open_pull_requests: length,
  non_draft: (map(select(.draft == false)) | length),
  draft: (map(select(.draft == true)) | length)
}' /tmp/originweave-open-prs.json

gh api repos/ContextualWisdomLab/OriginWeave/branches/main
gh api repos/ContextualWisdomLab/OriginWeave/rulesets/18156473
gh api 'repos/ContextualWisdomLab/OriginWeave/collaborators?affiliation=all&per_page=100'

jq -r '.[].number' /tmp/originweave-open-prs.json | while read -r PR; do
  PR_JSON="/tmp/originweave-pr-${PR}.json"
  gh api "repos/ContextualWisdomLab/OriginWeave/pulls/$PR" > "$PR_JSON"
  HEAD_SHA=$(jq -r '.head.sha' "$PR_JSON")

  gh api --paginate --slurp \
    "repos/ContextualWisdomLab/OriginWeave/commits/$HEAD_SHA/check-runs?per_page=100" \
    > "/tmp/originweave-pr-${PR}-check-runs.json"
  gh api --paginate --slurp \
    "repos/ContextualWisdomLab/OriginWeave/pulls/$PR/reviews?per_page=100" \
    > "/tmp/originweave-pr-${PR}-reviews.json"
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
}' > "/tmp/originweave-pr-${PR}-review-threads.json"
done
```

The ruleset response determines the required workflow names; each PR's exact `HEAD_SHA` then determines which check runs, reviews, and unresolved threads are current. The saved PR JSON also preserves the exact base reference and branch ancestry input for the dependency graph.

For standards and binding architecture, use [`doctoring.md`](doctoring.md), [`doctoring/browser-agent-protocols.md`](doctoring/browser-agent-protocols.md), [`PRD.md`](PRD.md), [`TRD.md`](TRD.md), [`product-roadmap.md`](product-roadmap.md), and linked ADR/UML/ERD/traceability records. Issues #199-#203 contain their own APA 7th standards and research traceability. This baseline intentionally records delivery state and never promotes planned adapters or active pull-request code to implemented behavior.
