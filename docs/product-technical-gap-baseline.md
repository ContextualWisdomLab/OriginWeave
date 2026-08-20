# Product and Technical Gap Baseline

This is a dated delivery baseline, not a substitute for the PRD, TRD, roadmap, or architecture decisions. It keeps buyer-visible gaps and volatile repository evidence in one discoverable place. Protected `main` is the implementation boundary: code in an open pull request is not shipped behavior.

## Observed snapshot: 2026-08-20

### Protected-main truth

- Protected `main` and `origin/main` were both at `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` when this snapshot was prepared.
- Phase 0 is documented as complete as a reusable safety-kernel foundation: typed policy contracts, destination classification, direct TCP peer verification, TLS service identity, evidence bounds, resource mitigation, document-node authority, and their protected-main tests.
- Phase 1 is **in progress**, not shipped. The first real Chromium vertical slice still needs an ephemeral Chromium context, a versioned WebDriver BiDi/browser adapter, semantic observation and typed actions, post-condition evidence, crash recovery, and proof that Chromium consumed the governed resolution, route, TCP, TLS, and HTTP boundaries.
- HTTP/1.1 bounds, download/MIME limits, proxy/PAC execution, full browser-network integration, the sensitive-data broker runtime, durable WARC/PROV capture, and the complete Manifest V3 compatibility program remain planned or open as recorded in the PRD, TRD, and roadmap.

### Open pull requests

The live repository contained **100 open pull requests: 21 non-draft and 79 draft**. The non-draft set was:

| Pull request | Base | Delivery state at snapshot |
|---|---|---|
| #194, #175, #173, #168, #166, #164, #163, #161, #160, #159, #158, #157, #156, #152, #124 | `main` | Ready/non-draft inventory; current review and check state must be re-fetched before merge |
| #149 | `main` | WireGuard/IKEv2 profile authority; exact head `b2be2e7`, Rust contracts and Production coverage successful, remaining required workflows were queued |
| #153, #151, #150, #148, #147 | stacked | Non-draft teardown/crash-recovery work; base-branch ordering applies |

Draft PR #195 is the current WebDriver BiDi opening-write repair. Its exact head is `29a310c`; Rust contracts are successful and Production coverage was in progress after a coverage-branch repair. It remains draft evidence and cannot be treated as shipped behavior.

The snapshot also retained an older open failure on #90 (`8721787d`): Rust contracts were successful but Production coverage was failing. That PR is not a protected-main implementation claim. The current exact head and check runs must be re-fetched before any action.

The 79 draft PRs are intentionally excluded from the merge queue. Several open PRs are stacked, so a green check on a child branch cannot be treated as evidence that its change is mergeable onto protected `main`.

### Review and merge authority

The active `CWL Central required workflows` ruleset requires one approving review, approval after the last push, resolved review threads, and the configured required workflows. The live collaborator list contained only `seonghobae` with repository administration and push permissions. This is a **reviewer-provisioning gap**: no eligible independent collaborator was available for a counted non-author approval at snapshot time.

This gap does not authorize self-approval, administrative bypass, stale-head merge, or weakening checks. Exact current-head checks, security gates, documentation, coverage, rustdoc/Clippy, thread resolution, and branch protection remain mandatory. The solo-maintainer governance condition may place an otherwise impossible independent-review rule on hold only through the documented governance path; it does not turn an unverified PR into shipped behavior.

### Open issues and operational signals

| Issue | Current gap or signal |
|---|---|
| #28 | First real Chromium agent vertical slice; highest buyer-visible Phase 1 gap |
| #27 | Complete Manifest V3 compatibility and extension-authority isolation matrix |
| #9 | Bounded HTTP/1.1 semantics over the authenticated TLS stream |
| #10 | Purpose-bound operational PII disclosure and trusted broker/storage lifecycle |
| #123 | Fleet incident: disable orphaned TLS, HTTP, and one-shot workflow identities |
| #187 | Manual-authority review of the coverage-diagnostics workflow delta |

The hourly product-development loop exists as a bounded, review-separated workflow. Its existence is operational infrastructure, not evidence that the browser product or an hourly run has completed the Phase 1 buyer acceptance.

## Buyer-visible and technical gap matrix

| Priority | Buyer-visible outcome | Protected-main status | Next acceptance evidence |
|---|---|---|---|
| P0 | A bounded task can observe a real Chromium page, perform a typed action, verify the post-condition, and emit provenance | **Open / Phase 1**; issue #28 | Repeated real Chromium E2E with ephemeral context, BiDi/session translation, observation, typed action, post-condition, evidence, crash cleanup, and exact current protected checks |
| P1 | Navigation uses the approved destination, route, TCP peer, TLS identity, and bounded HTTP/download policy | **Partial foundation**; HTTP and browser consumption remain planned | Real browser-network adapter proves the governed path is consumed end to end, including redirects, bounds, MIME, and failure evidence |
| P1 | Existing Chromium extensions remain compatible while Agent authority stays separate | **Partial evidence / planned completion**; issue #27 | Pinned-Chromium install/update/service-worker/content/storage/DNR/download/native-messaging/enterprise-isolation matrix with repeatability |
| P1 | Enterprise operators can disclose only necessary sensitive fields through a trusted, auditable path | **Policy foundation implemented; runtime open**; issue #10 | Opaque-handle broker, purpose/field/region policy, atomic reservation/revocation, retention/deletion, audit, and redaction tests |
| P2 | A buyer can receive durable replayable capture and provenance | **Foundations only** | Bounded WARC/PROV persistence, retention, integrity, replay, and benchmark evidence |
| P0 | Changes can pass protected review and merge without authority improvisation | **Blocked by reviewer-provisioning gap** | Provision an eligible independent collaborator or record an explicit current governance decision; then re-fetch exact head, reviews, checks, and merge state |

## Next executable queue

1. Re-fetch every active PR's exact head, reviews, threads, required checks, and base before selecting a merge candidate; repair a current failure only after reproducing its root cause.
2. Advance issue #28 with the smallest failing real-browser acceptance test, beginning at ephemeral Chromium launch/session teardown and the BiDi adapter boundary.
3. Keep HTTP/1.1 and browser-network integration separate from the already-proven destination, direct TCP, and TLS kernels; do not claim safe navigation until Chromium consumption is observed.
4. Maintain the #27 extension matrix and #10 broker/runtime boundaries as independent acceptance tracks.
5. Resolve the reviewer-provisioning gap through legitimate repository governance before a non-author approval is required; never manufacture approval or bypass protection.

## Evidence commands

The volatile values above were obtained from the repository and GitHub APIs, without exposing credentials:

```text
gh api repos/ContextualWisdomLab/OriginWeave/pulls?state=open&per_page=100
gh api repos/ContextualWisdomLab/OriginWeave/commits/<exact-head-sha>/check-runs?per_page=100
gh api repos/ContextualWisdomLab/OriginWeave/rulesets/18156473
gh api repos/ContextualWisdomLab/OriginWeave/collaborators?affiliation=all&per_page=100
```

For standards and binding architecture, use [`doctoring.md`](doctoring.md), [`PRD.md`](PRD.md), [`TRD.md`](TRD.md), [`product-roadmap.md`](product-roadmap.md), and the linked ADR/UML/ERD/traceability graph. This baseline intentionally records delivery state and does not promote planned adapters or open pull-request code to implemented behavior.
