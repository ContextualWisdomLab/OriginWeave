# Product and Technical Gap Baseline

This file is the current delivery baseline for OriginWeave. It records buyer-visible gaps and exact repository evidence; it does not replace the PRD, TRD, architecture, ADRs, threat model, test strategy, or live GitHub state. Protected `main` is the shipped implementation boundary. Open PRs, successful predecessor checks, synthetic mergeability, and command acknowledgements are not shipped behavior.

## Observed snapshot: 2026-09-06

### Protected-main truth

- Protected `main` is exact `87c4daa1830bac5a5228b6036752ad5633232085`. GitHub reports the commit signature as verified/valid.
- The repository currently has **125 open pull requests: 12 non-draft and 113 draft**.
- The repository currently has **13 open non-PR issues**.
- The GitHub Releases API currently returns an empty collection: **0 GitHub Releases**. No release-ready claim is valid until a protected exact head is integrated and an immutable release artifact, SBOM, provenance, rollback evidence, tag/package, and release are all verified.
- Protected-main code and tests remain authority for shipped behavior. A feature branch can be useful evidence without being a production capability.

### Foundation and stack integrity

The active WebDriver BiDi stack inherited historical whole-tree replacement `5c111d0db6c363f9d1786c21cc01c5c7398007bd` (`fix(stack): restore opening-write prerequisite tree`). It restored its transport prerequisite while also removing unrelated valid product/source/test/documentation assets. That deletion is a repair finding rather than grounds to close dependent PRs.

PR #195 is the earliest active owner point for the foundation recovery. The branch remains based on retained prerequisite #193 `6922dd98779e8f8aad132a3b1f563d7ba6e6d070`. Its recovery lineage includes:

- `29dd314501299a3ad8276e5d73189591ff6327a0`, which restored the BAP workspace member, MCP and release-acceptance contracts/tests, destination freshness/revalidation, policy MCP binding, resource error contracts, TLS revocation/trust, the Agent Task fixture, and this product/technical gap baseline without replacing the modular WebDriver BiDi core;
- `89708cf5e474f7701513b84a1356a8ce1699bef5`, which restored extraction schema, sensitive-handle lifecycle, RFC 3986 evidence-path admission, and their tests while retaining `BrowserProtocolValidationEvidence` and its browser-protocol regression; and
- later content-aware documentation recovery through `64114aab9e000f9cdc017f68e6c926e5abf28df3`, restoring architecture/index/ADR discoverability, product authority contracts, and the protected MCP product boundary without copying a whole protected tree over later WebDriver work.

The baseline deliberately does not embed PR #195's mutable live head as its own current identity. Any commit that updates this file would immediately make such a literal stale. Evidence commands therefore re-resolve the PR head from GitHub before fetching checks. `tests/test_gap_snapshot_inventory_consistency.py` enforces that rule instead of pinning a self-invalidating head SHA.

PR #242 still targets the pre-recovery #195 generation `48eb2d23009c1c804520dd5efcd0d4d072aacef1` and GitHub currently reports it non-mergeable. Descendants must adopt the repaired foundation content-aware and non-destructively; preserving an old child tree in a topology-only merge would reintroduce the deleted product assets. Each reconstructed exact head needs fresh checks. No predecessor GREEN transfers.

### Browser sandbox and realistic Chromium acceptance

PR #148 is exact `0135984f1bc1f68d89d7777f49c4999474105a12`. Its repository CI `33990522263` is terminal success with exact 100% reported production coverage (415 functions, 3,555 lines, 4,444 regions, 476 branches). Its real Manifest V3 Compatibility run `33990522248`, job `101371812631`, is terminal failure on pinned Chrome `150.0.7871.129` after all inherited `--no-sandbox` launch overrides were removed. Artifact `9977680352` reports 0/3 for ordinary MV3, ordinary Agent Task, forced-close Agent Task, and browser-crash Agent Task; the crash lane localizes to `failure_stage=session_create`, `failure_type=RuntimeError`, `reason_code=runtime_error`. Cleanup completion is not browser success.

Issue #212 is the canonical workflow-owner boundary for the missing sandbox-helper integration. PR #43 previously proved that root-owned mode-`4755` `chrome_sandbox` plus `CHROME_DEVEL_SANDBOX` can run the same Chrome generation sandboxed on that leaf generation, but its GREEN does not transfer to #148 or the current protected workflow. The authorized owner must reconstruct the validated helper mechanics against the current protected MV3 workflow, preserve harden-runner/egress, immutable pins, Draft/closed lifecycle and evidence retention, then consumers must adopt it non-destructively and regenerate exact-head Linux browser evidence. Restoring `--no-sandbox`, reducing trials, or treating cleanup as success is not an acceptable repair.

### WebDriver BiDi navigation stack

The repaired teardown/navigation chain #255 → #256 → #257 → #258 → #259 → #260 → #261 → #277 has terminal repository-native success on the already-restacked exact heads. That evidence validates those exact trees only; it does not cure foundation lineage, transfer central review/security evidence, or establish real-browser acceptance.

PR #263 adds a typed `session.unsubscribe` path for the exact opaque committed-navigation subscription receipt. Predecessor `37ae698c4a9e12d2fabf821ae5b910ea8a35ab8a` failed hosted CI because canonical rustfmt was not applied and one real `send()` frame-error arm was uncovered. Repair `3f22de94b63da83eaa8b5b1270912b21a3ecd006` changes only the unsubscribe failure integration test: it applies canonical formatting and exercises the real loopback RFC 6455 no-write `MalformedFrame` path caused by adjacent client masking-key reuse, proving no unsubscribe bytes reach the peer and only that unsubscribe correlation retires. Its fresh CI `34009256997` is queued at this snapshot; predecessor or partial evidence is not promoted.

### CI, review, and evidence control plane

Issue #279 remains the protected-main owner for exact-head documentation verification and the Ready-transition execution gap. The repeated CodeQL dispatch-to-verdict defect observed after successful current-head central scan dispatch is owned by `ContextualWisdomLab/.github#712`; leaf branches must not duplicate CodeQL, weaken required checks, or convert queued/skipped/provider-incomplete evidence into GREEN.

Protected review/ruleset requirements remain independent from tests. Passing automation is not approval. Stale review state after a push is not current approval, and a Draft, conflicted, or stack-incomplete PR is not merge-ready merely because one repository workflow passed.

### Product and buyer gaps that remain open

| Track | Current boundary | Completion evidence required |
|---|---|---|
| Governed browser vertical slice | WebDriver BiDi contracts and transport work are active; current stack requires foundation repair/restack | Real pinned Chromium session/navigation/semantic observation/policy-authorized interaction/post-condition/evidence/cleanup GREEN on the same exact head, then protected integration |
| Chromium sandbox | #148 fails closed at session creation without sandbox bypass; #212 owns workflow integration | Current-generation least-privilege helper adoption plus exact-head sandboxed Linux replay |
| Evidence/provenance | Redacted network/provenance, extraction schema, sensitive lifecycle, and browser-protocol validation contracts exist | Durable replay/retention/deletion and buyer-facing evidence lifecycle proven end-to-end |
| MCP/agent boundary | Typed stateless MCP/core authority contracts exist; MCP remains an adapter | Released API/adapter behavior that cannot become policy authority or bypass browser post-condition verification |
| Persistent task/API surface | Foundations exist | Tenant-scoped persistence, recovery, idempotency, operability, and API acceptance on protected code |
| Enterprise administration | Governance primitives exist | Buyer-visible policy/approval/audit administration with purpose-bound sensitive-data handling and accessibility verification |
| Distribution and release | No GitHub Release exists | Signed cross-platform artifacts, SBOM/provenance, reproducibility, rollback, package/tag and immutable release verification |
| CI evidence throughput | Exact-head verification exists but central verdict/queue issues remain | Reliable exact-head required workflows without gate weakening, skipped-result promotion, or stale evidence transfer |

### Bounded-context and ownership constraints

OriginWeave owns governed browser-domain truth: Browser Session, Navigation, Observation, Interaction Policy integration, Evidence, Extension/native-host boundary, and browser adapters. WebDriver BiDi, CDP and MCP are adapters, not policy authority. Wardnet, EgressWeave, Keyverse, contextual-orchestrator and Context Fabric remain canonical owners of their own domains; OriginWeave consumes only released/versioned contracts or ACLs and must not copy their source, use cross-service SQL, or depend on mutable sibling heads.

Deterministic browser policy/security decisions remain deterministic. Model-backed workflows must not substitute LLM judgement for browser authority. Command ACK is never sufficient for task success; the expected post-condition and evidence must be observed.

### Current repair order

1. Resolve PR #195's live head immediately before interpreting its CI/MV3 results; repair any new RED at that exact head and keep the content-aware recovery lineage intact.
2. Reconcile any remaining inherited documentation differences content-aware; do not overwrite later WebDriver deltas with an older whole tree.
3. Reconstruct #242 and descendants from the repaired #195 foundation using ordinary forward/non-force adoption, then regenerate exact-head checks on every claimed integration point.
4. Complete #212's authorized current-generation Chromium sandbox-helper integration and rerun realistic pinned-Chromium evidence on the exact consumer head.
5. Resolve central required-verdict failures through their canonical owner (`ContextualWisdomLab/.github#712`) rather than leaf duplication or gate weakening.
6. Integrate dependency-first through normal protected-branch review/ruleset gates.
7. Produce and verify the first immutable OriginWeave release with signed artifacts, SBOM, provenance, reproducibility and rollback evidence.

## Evidence commands

The snapshot is reproducible from GitHub without treating local branch state as authority. Paginate list endpoints before deriving counts or per-PR evidence. Mutable PR heads are resolved immediately before their evidence is queried; this avoids making the baseline stale merely by committing an update to the baseline itself.

```bash
set -euo pipefail
repo=ContextualWisdomLab/OriginWeave

gh api "repos/$repo/branches/main"
gh api --paginate "repos/$repo/pulls?state=open&per_page=100" --slurp
gh api --paginate "repos/$repo/issues?state=open&per_page=100" --slurp
gh api "repos/$repo/releases?per_page=100"

foundation_head="$(gh api "repos/$repo/pulls/195" --jq ".head.sha")"
gh api "repos/$repo/pulls/195"
gh api "repos/$repo/commits/$foundation_head/check-runs?per_page=100"
gh api "repos/$repo/actions/runs?head_sha=$foundation_head&per_page=100"

gh api "repos/$repo/pulls/242"
gh api "repos/$repo/pulls/263"
gh api "repos/$repo/pulls/148"
gh api "repos/$repo/issues/212"
gh api "repos/$repo/issues/279"
```

Re-fetch the head and base immediately before any merge/readiness decision. If either moved, previous check/review evidence becomes lineage only until the new exact head is verified.
