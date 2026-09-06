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

The active WebDriver BiDi stack inherited a historical whole-tree replacement, `5c111d0db6c363f9d1786c21cc01c5c7398007bd` (`fix(stack): restore opening-write prerequisite tree`), that restored its transport prerequisite but also removed unrelated valid product/source/test/documentation assets. That deletion is a repair finding rather than grounds to close dependent PRs.

PR #195 is the earliest active owner point currently repairing that foundation. Its exact head is `89708cf5e474f7701513b84a1356a8ce1699bef5` on retained prerequisite #193 `6922dd98779e8f8aad132a3b1f563d7ba6e6d070`. Two ordinary forward commits restore protected product contracts while preserving later browser work:

- `29dd314501299a3ad8276e5d73189591ff6327a0` restores the BAP workspace member, MCP and release-acceptance contracts/tests, destination freshness/revalidation, policy MCP binding, resource error contracts, TLS revocation/trust, the Agent Task fixture, and this product/technical gap baseline without replacing the modular WebDriver BiDi core.
- `89708cf5e474f7701513b84a1356a8ce1699bef5` restores extraction schema, sensitive-handle lifecycle, RFC 3986 evidence-path admission, and their tests while retaining `BrowserProtocolValidationEvidence` and its browser-protocol regression.

The child PR #242 still points at the pre-repair #195 generation and is currently non-mergeable after the base branch advanced. Descendants must therefore adopt the repaired foundation content-aware and non-destructively; preserving an old child tree in a topology-only merge would reintroduce the deleted product assets. Each reconstructed exact head needs fresh checks. No predecessor GREEN transfers.

### Browser sandbox and realistic Chromium acceptance

PR #148 is exact `0135984f1bc1f68d89d7777f49c4999474105a12`. Its repository CI `33990522263` is terminal success with exact 100% reported production coverage (415 functions, 3,555 lines, 4,444 regions, 476 branches). Its real Manifest V3 Compatibility run `33990522248`, job `101371812631`, is terminal failure on pinned Chrome `150.0.7871.129` after all inherited `--no-sandbox` launch overrides were removed. Artifact `9977680352` reports 0/3 for ordinary MV3, ordinary Agent Task, forced-close Agent Task, and browser-crash Agent Task; the crash lane localizes to `failure_stage=session_create`, `failure_type=RuntimeError`, `reason_code=runtime_error`. Cleanup completion is not browser success.

Issue #212 is the canonical workflow-owner boundary for the missing sandbox-helper integration. PR #43 previously proved that root-owned mode-`4755` `chrome_sandbox` plus `CHROME_DEVEL_SANDBOX` can run the same Chrome generation sandboxed on that leaf generation, but its GREEN does not transfer to #148 or to the current protected workflow. The authorized owner must reconstruct the validated helper mechanics against the current protected MV3 workflow, preserve harden-runner/egress, immutable pins, Draft/closed lifecycle and evidence retention, then consumers must adopt it non-destructively and regenerate exact-head Linux browser evidence. Restoring `--no-sandbox`, reducing trials, or treating cleanup as success is not an acceptable repair.

### CI, review, and evidence control plane

Issue #279 remains the protected-main owner for exact-head documentation verification and the Ready-transition execution gap. Its current record shows workflow-free classifier PR #287 succeeding in native CI/Security/Semgrep while required CodeQL fails only after current-head scan dispatch at the central verdict handoff. That repeated CodeQL dispatch-to-verdict defect is owned by `ContextualWisdomLab/.github#712`; leaf branches must not duplicate CodeQL, weaken required checks, or convert queued/skipped/provider-incomplete evidence into GREEN.

Protected review/ruleset requirements remain independent from tests. Passing automation is not approval. Stale review state after a push is not current approval, and a Draft, conflicted, or stack-incomplete PR is not merge-ready merely because one repository workflow passed.

### Product and buyer gaps that remain open

The following capabilities are not treated as protected-main commercial completion merely because foundations or active PRs exist:

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

1. Finish #195 exact-head repository verification and repair any new RED at that exact head.
2. Reconcile the remaining inherited documentation differences content-aware; do not overwrite later WebDriver deltas with an older whole tree.
3. Reconstruct #242 and descendants from the repaired #195 foundation using ordinary forward/non-force adoption, then regenerate exact-head checks on every claimed integration point.
4. Complete #212's authorized current-generation Chromium sandbox-helper integration and rerun realistic pinned-Chromium evidence on the exact consumer head.
5. Resolve central required-verdict failures through their canonical owner (`ContextualWisdomLab/.github#712`) rather than leaf duplication or gate weakening.
6. Integrate dependency-first through normal protected-branch review/ruleset gates.
7. Produce and verify the first immutable OriginWeave release with signed artifacts, SBOM, provenance, reproducibility and rollback evidence.

## Evidence commands

The snapshot is reproducible from GitHub without treating local branch state as authority. Paginate list endpoints before deriving counts or per-PR evidence.

```bash
set -euo pipefail
repo=ContextualWisdomLab/OriginWeave

gh api "repos/$repo/branches/main"
gh api --paginate "repos/$repo/pulls?state=open&per_page=100" --slurp
gh api --paginate "repos/$repo/issues?state=open&per_page=100" --slurp
gh api "repos/$repo/releases?per_page=100"

gh api "repos/$repo/pulls/195"
gh api "repos/$repo/commits/89708cf5e474f7701513b84a1356a8ce1699bef5/check-runs?per_page=100"
gh api "repos/$repo/actions/runs?head_sha=89708cf5e474f7701513b84a1356a8ce1699bef5&per_page=100"

gh api "repos/$repo/pulls/148"
gh api "repos/$repo/issues/212"
gh api "repos/$repo/issues/279"
```

Re-fetch the head and base immediately before any merge/readiness decision. If either moved, previous check/review evidence becomes lineage only until the new exact head is verified.
